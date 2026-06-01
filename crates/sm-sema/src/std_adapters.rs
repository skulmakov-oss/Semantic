use crate::alloc_core::{
    build_export_sets_core, collect_local_exports_core, diagnostic_help_core,
    evaluate_law_header_policy_core, fold_fx_const_call_core, has_magic_number_core,
    infer_law_entity_core, infer_when_condition_type_core, insert_name_core,
    insert_scoped_name_core, is_dead_when_condition, is_valid_when_result_type_core,
    parse_import_directives, parse_law_local_decl, track_entity_field_usage_core,
    validate_import_bindings_core,
    validate_import_namespace_rules as validate_import_namespace_rules_core,
    validate_select_imports_core, validate_when_non_empty_core, ExportBuildModule, ExportKind,
    ExportSet, ImportDirective, LawScheduler, LocalExportDecl, ScopeKind, SelectImportModule,
    SemanticType, Symbol, SymbolTable, TypeRegistry,
};
use crate::frontend::{
    parse_logos_program_with_profile, parse_program_with_profile, type_check_program, LogosEntity,
    LogosEntityFieldKind, LogosProgram, ParserProfile, SourceMark, Type,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use ton618_core::diagnostics::{
    append_help_line, format_diagnostic_header, render_context_with_caret,
};
use ton618_core::{Arena, SourceMap};

impl From<Type> for SemanticType {
    fn from(value: Type) -> Self {
        match value {
            Type::I32 => SemanticType::Int,
            Type::Fx | Type::F64 => SemanticType::Fx,
            Type::Quad => SemanticType::QVec(1),
            Type::QVec(n) => SemanticType::QVec(n),
            Type::Bool => SemanticType::Bool,
            Type::Text => SemanticType::Unknown,
            Type::Sequence(_) => SemanticType::Unknown,
            Type::Closure(_) => SemanticType::Unknown,
            Type::U32 => SemanticType::Int,
            Type::Unit => SemanticType::Unit,
            Type::Measured(base, _) => SemanticType::from((*base).clone()),
            Type::RangeI32 => SemanticType::Unknown,
            Type::Tuple(_) => SemanticType::Unknown,
            Type::Option(_) => SemanticType::Unknown,
            Type::Result(_, _) => SemanticType::Unknown,
            Type::Record(_) => SemanticType::Unknown,
            Type::Adt(_) => SemanticType::Unknown,
            Type::Map(_) => SemanticType::Unknown,
            // TypeVar is an owner-layer marker; semantic type is unknown until
            // monomorphisation substitutes the variable (M9.1 Wave 2).
            Type::TypeVar(_) => SemanticType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiagnostic {
    pub level: DiagLevel,
    pub code: &'static str,
    pub message: String,
    pub mark: SourceMark,
    pub rendered: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub diag: SemanticDiagnostic,
}

impl core::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.diag.rendered)
    }
}

impl std::error::Error for SemanticError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticReport {
    pub warnings: Vec<SemanticDiagnostic>,
    pub scheduled_laws: Vec<String>,
    pub arena_nodes: usize,
}

pub fn check_source(input: &str) -> Result<SemanticReport, SemanticError> {
    let profile = ParserProfile::foundation_default();
    check_source_with_profile(input, &profile)
}

pub fn check_source_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<SemanticReport, SemanticError> {
    if let Ok(logos) = parse_logos_program_with_profile(input, profile) {
        if logos.system.is_some() || !logos.entities.is_empty() || !logos.laws.is_empty() {
            return analyze_logos_program(&logos, input);
        }
    }
    let parsed = parse_program_with_profile(input, profile).map_err(|e| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0000",
            e.message,
            SourceMark::default(),
            input,
        ),
    })?;
    type_check_program(&parsed).map_err(|e| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0201",
            e.message,
            SourceMark::default(),
            input,
        ),
    })?;
    Ok(SemanticReport {
        warnings: Vec::new(),
        scheduled_laws: Vec::new(),
        arena_nodes: 0,
    })
}

pub fn check_file_with_provider(
    root: &Path,
    provider: &dyn crate::alloc_core::ModuleProvider,
) -> Result<SemanticReport, SemanticError> {
    let profile = ParserProfile::foundation_default();
    check_file_with_provider_and_profile(root, provider, &profile)
}

pub fn check_file_with_provider_and_profile(
    root: &Path,
    provider: &dyn crate::alloc_core::ModuleProvider,
    profile: &ParserProfile,
) -> Result<SemanticReport, SemanticError> {
    let mut visiting: Vec<VisitingImport> = Vec::new();
    let mut loaded: HashMap<PathBuf, (String, LogosProgram)> = HashMap::new();
    load_module_recursive(root, &mut visiting, &mut loaded, provider, profile, false)?;
    let export_sets = build_export_sets(&loaded, provider)?;
    validate_select_imports(&loaded, &export_sets, provider)?;

    let mut warnings = Vec::new();
    let mut scheduled_laws = Vec::new();
    let mut arena_nodes = 0usize;
    let mut module_paths: Vec<PathBuf> = loaded.keys().cloned().collect();
    module_paths.sort();
    for module_path in module_paths {
        let (src, logos) = loaded
            .get(&module_path)
            .expect("module key from loaded.keys()");
        let report = analyze_logos_program(logos, src).map_err(|mut e| {
            e.diag.message = format!("{}: {}", module_path.display(), e.diag.message);
            e.diag.rendered = format!("in module '{}'\n{}", module_path.display(), e.diag.rendered);
            e
        })?;
        warnings.extend(report.warnings);
        let module_key = path_contract_key(&module_path);
        for law in report.scheduled_laws {
            scheduled_laws.push(format!("{}::{}", module_key, law));
        }
        arena_nodes += report.arena_nodes;
    }
    Ok(SemanticReport {
        warnings,
        scheduled_laws,
        arena_nodes,
    })
}

fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Prefix(p) => out.push(p.as_os_str()),
            Component::RootDir => out.push(Path::new(std::path::MAIN_SEPARATOR_STR)),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            Component::Normal(s) => out.push(s),
        }
    }
    out
}

fn path_contract_key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Debug, Clone)]
struct VisitingImport {
    path: PathBuf,
    via_reexport: bool,
}

fn load_module_recursive(
    path: &Path,
    visiting: &mut Vec<VisitingImport>,
    loaded: &mut HashMap<PathBuf, (String, LogosProgram)>,
    provider: &dyn crate::alloc_core::ModuleProvider,
    profile: &ParserProfile,
    via_reexport: bool,
) -> Result<(), SemanticError> {
    let key = normalize_lexical(path);
    if loaded.contains_key(&key) {
        return Ok(());
    }
    if let Some(pos) = visiting.iter().position(|entry| entry.path == key) {
        let mut full_chain = visiting
            .iter()
            .map(|entry| path_contract_key(&entry.path))
            .collect::<Vec<_>>();
        full_chain.push(path_contract_key(path));
        let mut cycle_chain = visiting[pos..]
            .iter()
            .map(|entry| path_contract_key(&entry.path))
            .collect::<Vec<_>>();
        cycle_chain.push(path_contract_key(path));
        let reexport_only_cycle =
            via_reexport && visiting[(pos + 1)..].iter().all(|entry| entry.via_reexport);
        let (code, message) = if reexport_only_cycle {
            (
                "E0243",
                format!(
                    "symbol re-export cycle detected: {}",
                    cycle_chain.join(" -> ")
                ),
            )
        } else {
            (
                "E0238",
                format!("cyclic import detected: {}", full_chain.join(" -> ")),
            )
        };
        return Err(SemanticError {
            diag: render_diag(DiagLevel::Error, code, message, SourceMark::default(), ""),
        });
    }

    let module_id = path_contract_key(&key);
    let bytes = provider
        .read_module(&module_id)
        .map_err(|e| SemanticError {
            diag: render_diag(
                DiagLevel::Error,
                "E0239",
                format!("failed to read import '{}': {}", path.display(), e),
                SourceMark::default(),
                "",
            ),
        })?;
    let source = String::from_utf8(bytes).map_err(|_| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0239",
            format!("module '{}' is not valid utf-8", path.display()),
            SourceMark::default(),
            "",
        ),
    })?;
    let logos = parse_logos_program_with_profile(&source, profile).map_err(|e| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0239",
            format!("failed to parse module '{}': {}", path.display(), e.message),
            SourceMark::default(),
            &source,
        ),
    })?;

    visiting.push(VisitingImport {
        path: key.clone(),
        via_reexport,
    });
    let importer_module_id = path_contract_key(&key);
    let imports = parse_import_directives(&source);
    validate_import_namespace_rules(&imports, &logos, &source)?;
    for import in imports {
        let resolved = provider
            .resolve_import(&importer_module_id, &import.spec)
            .map_err(|e| SemanticError {
                diag: render_diag(
                    DiagLevel::Error,
                    "E0239",
                    format!("failed to resolve import '{}': {}", import.spec, e),
                    SourceMark::default(),
                    &source,
                ),
            })?;
        let import_path = normalize_lexical(Path::new(&resolved));
        load_module_recursive(
            &import_path,
            visiting,
            loaded,
            provider,
            profile,
            import.reexport,
        )?;
    }
    let _ = visiting.pop();
    loaded.insert(key, (source, logos));
    Ok(())
}

fn validate_import_namespace_rules(
    imports: &[ImportDirective],
    logos: &LogosProgram,
    source: &str,
) -> Result<(), SemanticError> {
    validate_import_namespace_rules_core(imports).map_err(|e| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            e.code,
            e.message,
            SourceMark {
                line: e.line,
                col: e.col,
                file_id: 0,
            },
            source,
        ),
    })?;
    let mut local_names = std::collections::BTreeSet::<String>::new();
    if let Some(system) = &logos.system {
        local_names.insert(system.name.clone());
    }
    for entity in &logos.entities {
        local_names.insert(entity.name.clone());
    }
    for law in &logos.laws {
        local_names.insert(law.name.clone());
    }
    validate_import_bindings_core(imports, &local_names).map_err(|e| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            e.code,
            e.message,
            SourceMark {
                line: e.line,
                col: e.col,
                file_id: 0,
            },
            source,
        ),
    })
}

fn validate_select_imports(
    loaded: &HashMap<PathBuf, (String, LogosProgram)>,
    export_sets: &HashMap<PathBuf, ExportSet>,
    provider: &dyn crate::alloc_core::ModuleProvider,
) -> Result<(), SemanticError> {
    let mut modules: Vec<PathBuf> = loaded.keys().cloned().collect();
    modules.sort();

    let mut core_modules = Vec::<SelectImportModule>::new();
    let mut dep_lookup = std::collections::BTreeMap::<(String, String), String>::new();
    let mut export_symbols =
        std::collections::BTreeMap::<String, std::collections::BTreeSet<String>>::new();
    let mut export_kinds =
        std::collections::BTreeMap::<String, std::collections::BTreeMap<String, ExportKind>>::new();
    let mut src_by_key = std::collections::BTreeMap::<String, String>::new();

    for (k, set) in export_sets {
        let key = path_contract_key(k);
        let mut syms = std::collections::BTreeSet::<String>::new();
        let mut kinds = std::collections::BTreeMap::<String, ExportKind>::new();
        for item in &set.items {
            syms.insert(item.public_name.clone());
            kinds.entry(item.public_name.clone()).or_insert(item.kind);
        }
        export_symbols.insert(key, syms);
        export_kinds.insert(path_contract_key(k), kinds);
    }

    for module in modules {
        let (src, _) = loaded.get(&module).expect("module key from loaded.keys()");
        let imports = parse_import_directives(src);
        let module_key = path_contract_key(&module);
        src_by_key.insert(module_key.clone(), src.clone());
        for import in &imports {
            let dep = provider
                .resolve_import(&module_key, &import.spec)
                .map(PathBuf::from)
                .map(|path| normalize_lexical(&path))
                .map_err(|e| SemanticError {
                    diag: render_diag(
                        DiagLevel::Error,
                        "E0239",
                        format!("failed to resolve import '{}': {}", import.spec, e),
                        SourceMark::default(),
                        src,
                    ),
                })?;
            dep_lookup.insert(
                (module_key.clone(), import.spec.clone()),
                path_contract_key(&dep),
            );
        }
        core_modules.push(SelectImportModule {
            module_key,
            source: src.clone(),
            imports,
        });
    }

    validate_select_imports_core(&core_modules, &dep_lookup, &export_symbols, &export_kinds)
        .map_err(|e| {
            let src = src_by_key
                .get(&e.module_key)
                .map(|s| s.as_str())
                .unwrap_or_default();
            SemanticError {
                diag: render_diag(
                    DiagLevel::Error,
                    e.code,
                    e.message,
                    SourceMark {
                        line: e.line,
                        col: e.col,
                        file_id: 0,
                    },
                    src,
                ),
            }
        })
}

fn build_export_sets(
    loaded: &HashMap<PathBuf, (String, LogosProgram)>,
    provider: &dyn crate::alloc_core::ModuleProvider,
) -> Result<HashMap<PathBuf, ExportSet>, SemanticError> {
    let mut modules = Vec::<ExportBuildModule>::new();
    let mut dep_lookup = std::collections::BTreeMap::<(String, String), String>::new();
    let mut keys: Vec<PathBuf> = loaded.keys().cloned().collect();
    keys.sort();
    for module in &keys {
        let (source, logos) = loaded.get(module).ok_or_else(|| SemanticError {
            diag: render_diag(
                DiagLevel::Error,
                "E0239",
                format!("unknown module '{}'", module.display()),
                SourceMark::default(),
                "",
            ),
        })?;
        let module_key = path_contract_key(module);
        let imports = parse_import_directives(source);
        for import in &imports {
            let dep = provider
                .resolve_import(&module_key, &import.spec)
                .map(PathBuf::from)
                .map(|path| normalize_lexical(&path))
                .map_err(|e| SemanticError {
                    diag: render_diag(
                        DiagLevel::Error,
                        "E0239",
                        format!("failed to resolve import '{}': {}", import.spec, e),
                        SourceMark::default(),
                        source,
                    ),
                })?;
            dep_lookup.insert(
                (module_key.clone(), import.spec.clone()),
                path_contract_key(&dep),
            );
        }
        modules.push(ExportBuildModule {
            module_key,
            source: source.clone(),
            local_exports: collect_local_exports(module, logos),
            imports,
        });
    }
    let core_sets = build_export_sets_core(&modules, &dep_lookup).map_err(|e| {
        let src = modules
            .iter()
            .find(|m| m.module_key == e.module_key)
            .map(|m| m.source.as_str())
            .unwrap_or_default();
        SemanticError {
            diag: render_diag(
                DiagLevel::Error,
                e.code,
                e.message,
                SourceMark {
                    line: e.line,
                    col: e.col,
                    file_id: 0,
                },
                src,
            ),
        }
    })?;
    let mut out = HashMap::<PathBuf, ExportSet>::new();
    for (key, set) in core_sets {
        out.insert(PathBuf::from(key), set);
    }
    Ok(out)
}

fn collect_local_exports(module: &Path, logos: &LogosProgram) -> ExportSet {
    let mut locals = Vec::<LocalExportDecl>::new();
    if let Some(system) = &logos.system {
        locals.push(LocalExportDecl {
            public_name: system.name.clone(),
            kind: ExportKind::System,
            span: system.mark,
        });
    }
    for entity in &logos.entities {
        locals.push(LocalExportDecl {
            public_name: entity.name.clone(),
            kind: ExportKind::Entity,
            span: entity.mark,
        });
    }
    for law in &logos.laws {
        locals.push(LocalExportDecl {
            public_name: law.name.clone(),
            kind: ExportKind::Law,
            span: law.mark,
        });
    }
    collect_local_exports_core(&module.display().to_string(), &locals)
}

pub fn analyze_logos_program(
    program: &LogosProgram,
    source: &str,
) -> Result<SemanticReport, SemanticError> {
    let mut symbols = SymbolTable::new();
    let mut type_registry = TypeRegistry::new();
    symbols.push(ScopeKind::Module);

    let mut entity_map: HashMap<String, &LogosEntity> = HashMap::new();
    for entity in &program.entities {
        if entity_map.insert(entity.name.clone(), entity).is_some() {
            return Err(SemanticError {
                diag: render_diag(
                    DiagLevel::Error,
                    "E0220",
                    format!("duplicate Entity '{}'", entity.name),
                    entity.mark,
                    source,
                ),
            });
        }
        symbols
            .insert(Symbol {
                name: entity.name.clone(),
                ty: SemanticType::QVec(1),
                scope: symbols.scope_kind(),
            })
            .map_err(|_| SemanticError {
                diag: render_diag(
                    DiagLevel::Error,
                    "E0220",
                    format!("duplicate Entity '{}'", entity.name),
                    entity.mark,
                    source,
                ),
            })?;
    }

    let mut law_names_by_entity: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut warnings = Vec::new();
    let mut arena = Arena::<String>::new();
    let mut entity_field_usage: HashMap<String, HashSet<String>> = HashMap::new();
    for entity in &program.entities {
        let mut fields = HashSet::new();
        for field in &entity.fields {
            fields.insert(field.name.clone());
        }
        entity_field_usage.insert(entity.name.clone(), fields);
    }

    for law in &program.laws {
        let law_policy = evaluate_law_header_policy_core(&law.name, law.whens.len());
        if law_policy.non_idiomatic_name {
            warnings.push(render_diag(
                DiagLevel::Warning,
                "W0250",
                format!(
                    "Law name '{}' is non-idiomatic; expected UpperCamelCase",
                    law.name
                ),
                law.mark,
                source,
            ));
        }
        if law_policy.large_law {
            warnings.push(render_diag(
                DiagLevel::Warning,
                "W0251",
                format!(
                    "Law '{}' is large ({} When clauses); consider splitting",
                    law.name,
                    law.whens.len()
                ),
                law.mark,
                source,
            ));
        }

        let owner_entity =
            infer_law_entity_core(law.whens.first().map(|w| w.condition.as_str()), |name| {
                entity_map.contains_key(name)
            })
            .unwrap_or_else(|| "_global".into());
        if !insert_scoped_name_core(&mut law_names_by_entity, &owner_entity, &law.name) {
            return Err(SemanticError {
                diag: render_diag(
                    DiagLevel::Error,
                    "E0221",
                    format!(
                        "duplicate Law '{}' inside Entity '{}'",
                        law.name, owner_entity
                    ),
                    law.mark,
                    source,
                ),
            });
        }

        if law.whens.is_empty() {
            return Err(SemanticError {
                diag: render_diag(
                    DiagLevel::Error,
                    "E0222",
                    format!("Law '{}' has empty body", law.name),
                    law.mark,
                    source,
                ),
            });
        }

        symbols.push(ScopeKind::Law);
        if let Some(ent) = entity_map.get(&owner_entity) {
            for field in &ent.fields {
                let _ = symbols.insert(Symbol {
                    name: field.name.clone(),
                    ty: SemanticType::from(field.ty.clone()),
                    scope: symbols.scope_kind(),
                });
            }
        }

        let mut law_locals = BTreeSet::new();
        for when in &law.whens {
            validate_when_non_empty_core(&when.condition, &when.effect).map_err(|e| {
                SemanticError {
                    diag: render_diag(DiagLevel::Error, e.code, e.message, when.mark, source),
                }
            })?;
            track_entity_field_usage_core(&when.condition, |ent, field| {
                if entity_map.contains_key(ent) {
                    if let Some(rem) = entity_field_usage.get_mut(ent) {
                        rem.remove(field);
                    }
                }
            });
            track_entity_field_usage_core(&when.effect, |ent, field| {
                if entity_map.contains_key(ent) {
                    if let Some(rem) = entity_field_usage.get_mut(ent) {
                        rem.remove(field);
                    }
                }
            });
            let ty = infer_when_condition_type_core(
                &when.condition,
                |name| symbols.resolve(name).map(|s| s.ty),
                |ent, field| {
                    entity_map.get(ent).and_then(|entity| {
                        entity
                            .fields
                            .iter()
                            .find(|x| x.name == field)
                            .map(|f| SemanticType::from(f.ty.clone()))
                    })
                },
            )
            .map_err(|e| match e {
                crate::alloc_core::ConditionInferError::MismatchedTypes { left, right } => {
                    SemanticError {
                        diag: render_diag(
                            DiagLevel::Error,
                            "E0201",
                            format!("Mismatched types. Expected {}, found {}", left, right),
                            when.mark,
                            source,
                        ),
                    }
                }
            })?;
            let ty_id = type_registry.intern(ty);
            let bool_id = type_registry.intern(SemanticType::Bool);
            let quad_id = type_registry.intern(SemanticType::Quad);
            if !is_valid_when_result_type_core(ty) {
                return Err(SemanticError {
                    diag: render_diag(
                        DiagLevel::Error,
                        "E0201",
                        format!(
                            "Mismatched types. Expected {} or {}, found {}",
                            type_registry.pretty(quad_id),
                            type_registry.pretty(bool_id),
                            type_registry.pretty(ty_id)
                        ),
                        when.mark,
                        source,
                    ),
                });
            }
            let _ = type_registry.equals_fast(ty_id, bool_id)
                || type_registry.equals_fast(ty_id, quad_id);

            if let Some(local) = parse_law_local_decl(&when.effect) {
                if !insert_name_core(&mut law_locals, &local) {
                    return Err(SemanticError {
                        diag: render_diag(
                            DiagLevel::Error,
                            "E0223",
                            format!("shadowing is forbidden inside Law: '{}'", local),
                            when.mark,
                            source,
                        ),
                    });
                }
            }

            if is_dead_when_condition(&when.condition) {
                warnings.push(render_diag(
                    DiagLevel::Warning,
                    "W0240",
                    format!(
                        "dead law branch detected in '{}': condition is always false",
                        law.name
                    ),
                    when.mark,
                    source,
                ));
            }
            if let Some(folded) = fold_fx_const_call_core(&when.effect) {
                warnings.push(render_diag(
                    DiagLevel::Warning,
                    "W0241",
                    format!(
                        "constant folding candidate in Law '{}': '{}' -> '{}'",
                        law.name,
                        when.effect.trim(),
                        folded
                    ),
                    when.mark,
                    source,
                ));
            }
            if has_magic_number_core(&when.condition) || has_magic_number_core(&when.effect) {
                warnings.push(render_diag(
                    DiagLevel::Warning,
                    "W0253",
                    format!(
                        "magic number detected in Law '{}'; consider named constant",
                        law.name
                    ),
                    when.mark,
                    source,
                ));
            }
            let _ = arena.alloc(format!("{}::{}", law.name, when.condition));
        }
        symbols.pop();
    }

    for entity in &program.entities {
        if let Some(rem) = entity_field_usage.get(&entity.name) {
            for field in &entity.fields {
                if rem.contains(&field.name) {
                    warnings.push(render_diag(
                        DiagLevel::Warning,
                        "W0252",
                        format!(
                            "unused {} '{}.{}'",
                            match field.kind {
                                LogosEntityFieldKind::State => "state",
                                LogosEntityFieldKind::Prop => "prop",
                            },
                            entity.name,
                            field.name
                        ),
                        field.mark,
                        source,
                    ));
                }
            }
        }
    }

    let scheduled = LawScheduler::schedule_by_priority_desc(&program.laws, |l| l.priority);
    Ok(SemanticReport {
        warnings,
        scheduled_laws: scheduled.into_iter().map(|l| l.name).collect(),
        arena_nodes: arena.len(),
    })
}

fn render_diag(
    level: DiagLevel,
    code: &'static str,
    message: String,
    mark: SourceMark,
    source: &str,
) -> SemanticDiagnostic {
    let mut sm = SourceMap::new();
    let file_id = sm.add_file("<input>", source);
    let mark = SourceMark { file_id, ..mark };
    let mut body = render_context_with_caret(sm.source(file_id).unwrap_or(""), mark, 2);
    if let Some(help) = diagnostic_help_core(code) {
        append_help_line(&mut body, help);
    }
    let header = format_diagnostic_header(
        to_core_diag_level(level),
        code,
        &message,
        mark.line.max(1),
        mark.col.max(1),
    );
    let rendered = format!("{header}\n{body}");
    SemanticDiagnostic {
        level,
        code,
        message,
        mark,
        rendered,
    }
}

fn to_core_diag_level(level: DiagLevel) -> ton618_core::DiagLevel {
    match level {
        DiagLevel::Error => ton618_core::DiagLevel::Error,
        DiagLevel::Warning => ton618_core::DiagLevel::Warning,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parse_logos_program;
    use std::collections::{BTreeMap, HashMap};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestFsModuleProvider;

    impl crate::alloc_core::ModuleProvider for TestFsModuleProvider {
        fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String> {
            fs::read(module_id).map_err(|e| e.to_string())
        }

        fn resolve_import(&self, importer_module_id: &str, spec: &str) -> Result<String, String> {
            Ok(resolve_test_import(importer_module_id, spec))
        }
    }

    fn check_file_fs(path: &std::path::Path) -> Result<SemanticReport, SemanticError> {
        let root = path.canonicalize().map_err(|e| SemanticError {
            diag: render_diag(
                DiagLevel::Error,
                "E0239",
                format!("failed to resolve root module '{}': {}", path.display(), e),
                SourceMark::default(),
                "",
            ),
        })?;
        let provider = TestFsModuleProvider;
        check_file_with_provider(&root, &provider)
    }

    struct MapProvider {
        modules: BTreeMap<String, Vec<u8>>,
    }

    impl crate::alloc_core::ModuleProvider for MapProvider {
        fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String> {
            self.modules
                .get(module_id)
                .cloned()
                .ok_or_else(|| format!("missing module '{}'", module_id))
        }

        fn resolve_import(&self, importer_module_id: &str, spec: &str) -> Result<String, String> {
            if let Some((alias, rest)) = spec.split_once("::") {
                let resolved = format!("/virtual/deps/{}/{}", alias, rest);
                return Ok(resolved);
            }
            Ok(resolve_test_import(importer_module_id, spec))
        }
    }

    fn resolve_test_import(importer_module_id: &str, spec: &str) -> String {
        let importer = std::path::Path::new(importer_module_id);
        let base = importer
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));
        let mut spec_path = PathBuf::from(spec);
        if spec_path.extension().is_none() {
            spec_path.set_extension("exo");
        }
        let joined = if spec_path.is_absolute() {
            spec_path
        } else {
            base.join(spec_path)
        };
        joined.to_string_lossy().replace('\\', "/")
    }

    #[test]
    fn compat_policy_int_fx() {
        assert!(crate::alloc_core::is_assignment_compatible(
            SemanticType::Fx,
            SemanticType::Int
        ));
        assert!(!crate::alloc_core::is_assignment_compatible(
            SemanticType::Int,
            SemanticType::Fx
        ));
    }

    #[test]
    fn duplicate_entity_is_error() {
        let src = r#"
Entity A:
    state x: quad
Entity A:
    prop y: bool
"#;
        let p = parse_logos_program(src).expect("logos parse");
        let err = analyze_logos_program(&p, src).expect_err("must fail");
        assert!(err.to_string().contains("E0220"));
    }

    #[test]
    fn dead_when_warns() {
        let src = r#"
Entity A:
    state x: quad
Law "L" [priority 1]:
    When N ->
        Pulse.emit("x")
"#;
        let p = parse_logos_program(src).expect("logos parse");
        let rep = analyze_logos_program(&p, src).expect("analyze");
        assert!(rep.warnings.iter().any(|w| w.code == "W0240"));
    }

    #[test]
    fn provider_pipeline_matches_direct_analyze_smoke() {
        let module = "/virtual/root.sm";
        let src = r#"
Law "CheckSignal" [priority 10]:
    When true ->
        System.recovery()
"#;
        let mut modules = BTreeMap::new();
        modules.insert(module.to_string(), src.as_bytes().to_vec());
        let provider = MapProvider { modules };

        let from_provider =
            check_file_with_provider(std::path::Path::new(module), &provider).expect("provider");
        let parsed = parse_logos_program(src).expect("parse");
        let direct = analyze_logos_program(&parsed, src).expect("direct");

        assert_eq!(from_provider.warnings.len(), direct.warnings.len());
        let mut provider_codes: Vec<&'static str> =
            from_provider.warnings.iter().map(|w| w.code).collect();
        let mut direct_codes: Vec<&'static str> = direct.warnings.iter().map(|w| w.code).collect();
        provider_codes.sort_unstable();
        direct_codes.sort_unstable();
        assert_eq!(provider_codes, direct_codes);

        assert!(from_provider
            .scheduled_laws
            .iter()
            .any(|name| name.ends_with("::CheckSignal")));
    }

    #[test]
    fn provider_pipeline_uses_provider_import_resolution_hook() {
        let root = "/virtual/root.sm";
        let dep = "/virtual/deps/math/core.sm";
        let src = "Import \"math::core.sm\"\nLaw \"L\" [priority 1]:\n    When true ->\n        System.recovery()\n";
        let dep_src = "Law \"Core\" [priority 1]:\n    When true ->\n        System.recovery()\n";
        let mut modules = BTreeMap::new();
        modules.insert(root.to_string(), src.as_bytes().to_vec());
        modules.insert(dep.to_string(), dep_src.as_bytes().to_vec());
        let provider = MapProvider { modules };

        let report =
            check_file_with_provider(std::path::Path::new(root), &provider).expect("provider");
        assert!(report
            .scheduled_laws
            .iter()
            .any(|name| name.contains("/virtual/deps/math/core.sm::Core")));
    }

    #[test]
    fn type_registry_is_canonical() {
        let mut reg = TypeRegistry::new();
        let a = reg.intern(SemanticType::Fx);
        let b = reg.intern(SemanticType::Fx);
        let c = reg.intern(SemanticType::QVec(32));
        assert!(reg.equals_fast(a, b));
        assert!(!reg.equals_fast(a, c));
        assert_eq!(reg.pretty(a), "Fx");
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn crystal_fold_warns_for_fx_add_constants() {
        let src = r#"
Law "L" [priority 1]:
    When true -> fx.add(1.0, 2.0)
"#;
        let p = parse_logos_program(src).expect("logos parse");
        let report = analyze_logos_program(&p, src).expect("semantics");
        assert!(report.warnings.iter().any(|w| w.code == "W0241"));
    }

    #[test]
    fn import_recursive_modules_check_ok() {
        let base = std::env::temp_dir().join(format!(
            "exo_import_ok_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        let root = base.join("root.sm");
        let a = base.join("a.sm");
        let b = base.join("b.sm");

        std::fs::write(
            &root,
            r#"
Import "a.sm"
Law "R" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &a,
            r#"
Import "b.sm"
Entity A:
    state x: quad
"#,
        )
        .expect("write a");
        std::fs::write(
            &b,
            r#"
Law "B" [priority 2]:
    When true -> System.recovery()
"#,
        )
        .expect("write b");

        let rep = check_file_fs(&root).expect("check file");
        assert!(!rep.scheduled_laws.is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn import_cycle_detected() {
        let base = std::env::temp_dir().join(format!(
            "exo_import_cycle_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        let root = base.join("root.sm");
        let a = base.join("a.sm");

        std::fs::write(
            &root,
            r#"
Import "a.sm"
Law "R" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &a,
            r#"
Import "root.sm"
Entity A:
    state x: quad
"#,
        )
        .expect("write a");

        let err = check_file_fs(&root).expect_err("must fail");
        assert!(err.to_string().contains("E0238"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn import_reexport_is_allowed_in_v02() {
        let base = std::env::temp_dir().join(format!(
            "exo_import_reexport_v02_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        let root = base.join("root.sm");
        let a = base.join("a.sm");

        std::fs::write(
            &root,
            r#"
Import pub "a.sm"
Law "R" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &a,
            r#"
Law "A" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write a");

        let rep = check_file_fs(&root).expect("must pass");
        assert!(!rep.scheduled_laws.is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn import_reexport_collision_is_rejected() {
        let base = std::env::temp_dir().join(format!(
            "exo_import_reexport_collision_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        let root = base.join("root.sm");
        let a = base.join("a.sm");

        std::fs::write(
            &root,
            r#"
Import pub "a.sm"
Law "A" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &a,
            r#"
Law "A" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write a");

        let err = check_file_fs(&root).expect_err("must fail");
        assert!(err.to_string().contains("E0242"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn symbol_cycle_detect_via_reexport_graph() {
        let a = PathBuf::from("/virtual/a.sm");
        let b = PathBuf::from("/virtual/b.sm");
        let mut loaded: HashMap<PathBuf, (String, LogosProgram)> = HashMap::new();
        loaded.insert(
            a.clone(),
            (
                "Import pub \"b.sm\"\nLaw \"A\" [priority 1]:\n    When true -> System.recovery()\n"
                    .to_string(),
                parse_logos_program(
                    "Law \"A\" [priority 1]:\n    When true -> System.recovery()\n",
                )
                .expect("logos a"),
            ),
        );
        loaded.insert(
            b.clone(),
            (
                "Import pub \"a.sm\"\nLaw \"B\" [priority 1]:\n    When true -> System.recovery()\n"
                    .to_string(),
                parse_logos_program(
                    "Law \"B\" [priority 1]:\n    When true -> System.recovery()\n",
                )
                .expect("logos b"),
            ),
        );
        let provider = MapProvider {
            modules: BTreeMap::new(),
        };
        let err = build_export_sets(&loaded, &provider).expect_err("must fail cycle");
        assert!(err.to_string().contains("E0243"));
        assert!(err
            .to_string()
            .contains("/virtual/a.sm -> /virtual/b.sm -> /virtual/a.sm"));
    }

    #[test]
    fn import_select_missing_symbol_is_error() {
        let base = std::env::temp_dir().join(format!(
            "exo_import_select_missing_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        let root = base.join("root.sm");
        let a = base.join("a.sm");

        std::fs::write(
            &root,
            r#"
Import "a.sm" { Missing }
Law "R" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &a,
            r#"
Law "A" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write a");

        let err = check_file_fs(&root).expect_err("must fail");
        assert!(err.to_string().contains("E0244"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn import_pub_select_alias_passes() {
        let base = std::env::temp_dir().join(format!(
            "exo_import_pub_select_alias_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        let root = base.join("root.sm");
        let a = base.join("a.sm");

        std::fs::write(
            &root,
            r#"
Import pub "a.sm" { A as B }
Law "R" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &a,
            r#"
Law "A" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write a");

        let rep = check_file_fs(&root).expect("must pass");
        assert!(!rep.scheduled_laws.is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn import_alias_collision_is_rejected() {
        let base = std::env::temp_dir().join(format!(
            "exo_import_alias_collision_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        let root = base.join("root.sm");
        let a = base.join("a.sm");
        let b = base.join("b.sm");

        std::fs::write(
            &root,
            r#"
Import "a.sm" as Core
Import "b.sm" as Core
Law "R" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &a,
            r#"
Law "A" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write a");
        std::fs::write(
            &b,
            r#"
Law "B" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write b");

        let err = check_file_fs(&root).expect_err("must fail");
        assert!(err.to_string().contains("E0241"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn import_namespace_isolation_allows_same_entity_names_in_different_modules() {
        let base = std::env::temp_dir().join(format!(
            "exo_import_ns_isolation_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        let root = base.join("root.sm");
        let a = base.join("a.sm");
        let b = base.join("b.sm");

        std::fs::write(
            &root,
            r#"
Import "a.sm"
Import "b.sm"
Law "R" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &a,
            r#"
Entity Sensor:
    state val: quad
"#,
        )
        .expect("write a");
        std::fs::write(
            &b,
            r#"
Entity Sensor:
    state val: quad
"#,
        )
        .expect("write b");

        let rep = check_file_fs(&root).expect("must pass");
        assert!(!rep.scheduled_laws.is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn diagnostics_include_help_and_wider_context() {
        let src = "line1\nline2\nline3\nline4\nline5\n";
        let d = render_diag(
            DiagLevel::Error,
            "E0201",
            "mismatch".to_string(),
            SourceMark {
                line: 3,
                col: 2,
                file_id: 0,
            },
            src,
        );
        assert!(d.rendered.contains("line1"));
        assert!(d.rendered.contains("line5"));
        assert!(d.rendered.contains("help: Check type compatibility"));
    }

    #[test]
    fn lint_warnings_for_style_large_and_unused_fields() {
        let src = r#"
Entity Sensor:
    state val: quad
    prop active: bool

Law "bad_name" [priority 1]:
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
    When true -> System.recovery()
"#;
        let p = parse_logos_program(src).expect("logos parse");
        let rep = analyze_logos_program(&p, src).expect("analyze");
        assert!(rep.warnings.iter().any(|w| w.code == "W0250"));
        assert!(rep.warnings.iter().any(|w| w.code == "W0251"));
        assert!(rep.warnings.iter().any(|w| w.code == "W0252"));
    }

    #[test]
    fn lint_warns_on_magic_numbers() {
        let src = r#"
Law "MagicLaw" [priority 1]:
    When true -> fx.add(2.5, 7.0)
"#;
        let p = parse_logos_program(src).expect("logos parse");
        let rep = analyze_logos_program(&p, src).expect("analyze");
        assert!(rep.warnings.iter().any(|w| w.code == "W0253"));
    }

    #[test]
    fn scheduler_keeps_declaration_order_on_equal_priority() {
        let src = r#"
Law "Zeta" [priority 7]:
    When true -> System.recovery()
Law "Alpha" [priority 7]:
    When true -> System.recovery()
"#;
        let p = parse_logos_program(src).expect("logos parse");
        let names: Vec<String> = LawScheduler::schedule_by_priority_desc(&p.laws, |l| l.priority)
            .into_iter()
            .map(|l| l.name)
            .collect();
        assert_eq!(names, vec!["Zeta".to_string(), "Alpha".to_string()]);
    }
}
