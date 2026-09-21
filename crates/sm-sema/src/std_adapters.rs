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
    admit_logos_program_with_profile, admit_program_with_profile, lex,
    parse_logos_program_with_profile, resolve_surface_authority, type_check_program, FrontendError,
    FrontendErrorKind, LogosEntity, LogosEntityFieldKind, LogosProgram, ParserProfile, Program,
    SourceMark, SurfaceAuthority, Token, Type,
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
    /// Provider-graph module key attached only when a project/module
    /// aggregation path has authoritative provider context. This is
    /// transitional provenance for SSF-09/#1697, not the future canonical
    /// external FileIdentity.
    pub provider_module_id: Option<String>,
    /// Optional frontend-boundary provenance for a direct RustLike admission
    /// error. This is not a complete diagnostic taxonomy or external schema.
    pub frontend_error_kind: Option<FrontendErrorKind>,
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

fn describe_outcome<T>(outcome: &Result<T, FrontendError>) -> String {
    match outcome {
        Ok(_) => "accepted".to_string(),
        Err(e) => e.message.clone(),
    }
}

/// Decision E, "Evidence preservation": the message shape below is the
/// one sketched verbatim in the frozen decision text - not a new
/// diagnostic-carrier design (deferred, per the decision's own exit
/// gate).
fn ambiguous_surface_error<L, R>(
    input: &str,
    logos: &Result<L, FrontendError>,
    rustlike: &Result<R, FrontendError>,
) -> SemanticError {
    let message = format!(
        "AMBIGUOUS / CONFLICTING SOURCE SURFACE\n\nEvidence:\nLogos     -> {}\nRustLike  -> {}",
        describe_outcome(logos),
        describe_outcome(rustlike),
    );
    SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0000",
            message,
            SourceMark::default(),
            input,
        ),
    }
}

fn no_surface_claim_error(input: &str) -> SemanticError {
    SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0000",
            "NO SURFACE CLAIM: this input establishes no top-level evidence for either the Logos or RustLike grammar".to_string(),
            SourceMark::default(),
            input,
        ),
    }
}

/// #1933: type-checks an already-authority-classified RustLike `Program`,
/// without lexing, parsing, admitting, or resolving surface authority
/// itself. Exists so a caller that has already established (via
/// `resolve_surface_authority`) that RustLike owns a given source - and
/// has possibly composed an executable bundle from it - can type-check
/// the resulting `Program` without re-deriving admission from the
/// composed text, which would be a second, illegitimate Auto
/// classification of what is by then an internal RustLike composition
/// artifact rather than a fresh source candidate (see the #1933 addendum
/// in `docs/roadmap/stable_foundation/ssf09_diagnostic_authority_decision.md`).
///
/// `source` is diagnostic-context only - passed straight to `render_diag`
/// for caret-bearing error text - it is never lexed, parsed, or otherwise
/// interpreted, and it MUST be the exact text `program` was parsed from
/// (whether that is a raw root or a composed bundle) so the rendered
/// diagnostic points at real source. This function performs zero
/// lexing, zero parsing, zero `GrammarAdmission`, zero
/// `resolve_surface_authority`, and zero fallback - a type-check failure
/// here is unconditionally terminal, exactly as `check_source_with_profile`
/// already treats it.
///
/// `check_source_with_profile`'s own `RustLikeOwns(Ok(_))` arm calls this
/// function too, so the logic exists in exactly one place.
pub fn check_rustlike_program(
    program: &Program,
    source: &str,
) -> Result<SemanticReport, SemanticError> {
    type_check_program(program).map_err(|e| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0201",
            e.message,
            SourceMark::default(),
            source,
        ),
    })?;
    Ok(SemanticReport {
        warnings: Vec::new(),
        scheduled_laws: Vec::new(),
        arena_nodes: 0,
    })
}

pub fn check_source_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<SemanticReport, SemanticError> {
    // Owner-review correction (SSF09-E2 round 1, still valid under
    // Decision E): a lexer failure means admission evidence could not be
    // obtained at all - it does not prove the source is blank, and is
    // not a surface-admission outcome for either grammar. Preserve the
    // lexer's own deterministic failure instead of feeding either
    // `admit_*` function anything.
    let tokens = lex(input).map_err(|e| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0000",
            e.message,
            SourceMark::default(),
            input,
        ),
    })?;
    let logos = admit_logos_program_with_profile(input, &tokens, profile);
    let rustlike = admit_program_with_profile(input, &tokens, profile);
    match resolve_surface_authority(logos, rustlike) {
        SurfaceAuthority::LogosOwns(Ok(program)) => analyze_logos_program(&program, input),
        SurfaceAuthority::LogosOwns(Err(e)) => Err(SemanticError {
            diag: render_diag(
                DiagLevel::Error,
                "E0000",
                e.message,
                SourceMark::default(),
                input,
            ),
        }),
        SurfaceAuthority::RustLikeOwns(Ok(parsed)) => check_rustlike_program(&parsed, input),
        SurfaceAuthority::RustLikeOwns(Err(e)) => Err(rustlike_frontend_error(input, &tokens, e)),
        SurfaceAuthority::Ambiguous { logos, rustlike } => {
            Err(ambiguous_surface_error(input, &logos, &rustlike))
        }
        SurfaceAuthority::NoSurfaceClaim => Err(no_surface_claim_error(input)),
    }
}

fn rustlike_frontend_error(input: &str, tokens: &[Token], error: FrontendError) -> SemanticError {
    let kind = error.kind();
    let mut diag = render_diag(
        DiagLevel::Error,
        "E0000",
        error.message,
        frontend_error_mark(tokens, input, error.pos),
        input,
    );
    diag.frontend_error_kind = Some(kind);
    SemanticError { diag }
}

fn frontend_error_mark(tokens: &[Token], source: &str, pos: usize) -> SourceMark {
    if let Some(token) = tokens.iter().find(|token| token.pos == pos) {
        return token.mark;
    }

    source_mark_from_byte_offset(source, pos)
}

fn source_mark_from_byte_offset(source: &str, pos: usize) -> SourceMark {
    let mut mark = SourceMark {
        line: 1,
        col: 1,
        file_id: 0,
    };
    for byte in source.as_bytes().iter().take(pos.min(source.len())) {
        if *byte == b'\n' {
            mark.line += 1;
            mark.col = 1;
        } else {
            mark.col += 1;
        }
    }
    mark
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
        let mut report = analyze_logos_program(logos, src).map_err(|mut e| {
            e.diag.message = format!("{}: {}", module_path.display(), e.diag.message);
            e.diag.rendered = format!("in module '{}'\n{}", module_path.display(), e.diag.rendered);
            e
        })?;
        let module_key = path_contract_key(&module_path);
        for warning in &mut report.warnings {
            warning.provider_module_id = Some(module_key.clone());
        }
        warnings.extend(report.warnings);
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
            source_mark_from_byte_offset(&source, e.pos),
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
        provider_module_id: None,
        frontend_error_kind: None,
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
    use crate::frontend::{parse_logos_program, parse_program_with_profile};
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

    fn warning_fixture_source() -> &'static str {
        r#"Entity A:
    state x: quad
Law "L" [priority 1]:
    When N ->
        Pulse.emit("x")
"#
    }

    fn warning_project_root(imports: &str) -> String {
        format!(
            "{imports}Law \"Root\" [priority 1]:\n    When true ->\n        System.recovery()\n"
        )
    }

    #[test]
    fn provider_warning_attaches_root_module_key() {
        let root = "/virtual/root.sm";
        let mut modules = BTreeMap::new();
        modules.insert(
            root.to_string(),
            warning_fixture_source().as_bytes().to_vec(),
        );
        let provider = MapProvider { modules };

        let report =
            check_file_with_provider(std::path::Path::new(root), &provider).expect("provider");
        assert!(!report.warnings.is_empty());
        assert!(report
            .warnings
            .iter()
            .all(|warning| warning.provider_module_id.as_deref() == Some(root)));
    }

    #[test]
    fn provider_warning_attaches_imported_module_key() {
        let root = "/virtual/root.sm";
        let helper = "/virtual/deps/a/warn.sm";
        let root_src = warning_project_root("Import \"a::warn.sm\"\n");
        let mut modules = BTreeMap::new();
        modules.insert(root.to_string(), root_src.into_bytes());
        modules.insert(
            helper.to_string(),
            warning_fixture_source().as_bytes().to_vec(),
        );
        let provider = MapProvider { modules };

        let report =
            check_file_with_provider(std::path::Path::new(root), &provider).expect("provider");
        assert!(!report.warnings.is_empty());
        assert!(report
            .warnings
            .iter()
            .all(|warning| warning.provider_module_id.as_deref() == Some(helper)));
    }

    #[test]
    fn provider_warnings_distinguish_same_basename_modules() {
        let root = "/virtual/root.sm";
        let helper_a = "/virtual/deps/a/warn.sm";
        let helper_b = "/virtual/deps/b/warn.sm";
        let root_src = warning_project_root("Import \"a::warn.sm\"\nImport \"b::warn.sm\"\n");
        let mut modules = BTreeMap::new();
        modules.insert(root.to_string(), root_src.into_bytes());
        modules.insert(
            helper_a.to_string(),
            warning_fixture_source().as_bytes().to_vec(),
        );
        modules.insert(
            helper_b.to_string(),
            warning_fixture_source().as_bytes().to_vec(),
        );
        let provider = MapProvider { modules };

        let report =
            check_file_with_provider(std::path::Path::new(root), &provider).expect("provider");
        let mut w0240_modules = report
            .warnings
            .iter()
            .filter(|warning| warning.code == "W0240")
            .map(|warning| warning.provider_module_id.clone())
            .collect::<Vec<_>>();
        w0240_modules.sort();
        assert_eq!(
            w0240_modules,
            vec![Some(helper_a.to_string()), Some(helper_b.to_string())]
        );
    }

    #[test]
    fn provider_warning_order_is_deterministic() {
        let root = "/virtual/root.sm";
        let helper_a = "/virtual/deps/a/warn.sm";
        let helper_b = "/virtual/deps/b/warn.sm";
        let root_src = warning_project_root("Import \"b::warn.sm\"\nImport \"a::warn.sm\"\n");
        let mut modules = BTreeMap::new();
        modules.insert(root.to_string(), root_src.into_bytes());
        modules.insert(
            helper_a.to_string(),
            warning_fixture_source().as_bytes().to_vec(),
        );
        modules.insert(
            helper_b.to_string(),
            warning_fixture_source().as_bytes().to_vec(),
        );
        let provider = MapProvider { modules };

        let first =
            check_file_with_provider(std::path::Path::new(root), &provider).expect("provider");
        let second =
            check_file_with_provider(std::path::Path::new(root), &provider).expect("provider");

        let key = |report: &SemanticReport| {
            report
                .warnings
                .iter()
                .map(|warning| {
                    (
                        warning.provider_module_id.clone(),
                        warning.code,
                        warning.mark.line,
                        warning.mark.col,
                    )
                })
                .collect::<Vec<_>>()
        };
        let expected = vec![
            (Some(helper_a.to_string()), "W0240", 4, 5),
            (Some(helper_a.to_string()), "W0252", 2, 5),
            (Some(helper_b.to_string()), "W0240", 4, 5),
            (Some(helper_b.to_string()), "W0252", 2, 5),
        ];
        assert_eq!(key(&first), expected);
        assert_eq!(key(&second), expected);
    }

    #[test]
    fn direct_check_source_does_not_invent_provider_module_id() {
        let report = check_source(warning_fixture_source()).expect("direct check");
        assert!(!report.warnings.is_empty());
        assert!(report
            .warnings
            .iter()
            .all(|warning| warning.provider_module_id.is_none()));
    }

    #[test]
    fn provider_error_path_does_not_attach_warning_provenance() {
        let root = "/virtual/root.sm";
        let helper = "/virtual/a.sm";
        let root_src = warning_project_root("Import \"a.sm\"\n");
        let mut modules = BTreeMap::new();
        modules.insert(root.to_string(), root_src.into_bytes());
        modules.insert(helper.to_string(), b"Entity".to_vec());
        let provider = MapProvider { modules };

        let err = check_file_with_provider(std::path::Path::new(root), &provider)
            .expect_err("malformed imported module must fail");
        let expected_helper = normalize_lexical(std::path::Path::new(helper));
        let expected_helper = expected_helper.to_string_lossy();
        assert!(
            err.diag.message.contains(expected_helper.as_ref()),
            "expected imported-module error to contain module path; got: {}",
            err.diag.message
        );
        assert!(err.diag.provider_module_id.is_none());
    }

    const MALFORMED_MODULE_SRC: &str =
        "Entity Player:\n    state hp: quad\nLaw \"L\" [priority x]:\n    When true -> System.recovery()\n";
    const MALFORMED_MODULE_MARK: SourceMark = SourceMark {
        line: 3,
        col: 19,
        file_id: 0,
    };

    fn check_modules(
        root: &str,
        modules: &[(&str, &[u8])],
    ) -> Result<SemanticReport, SemanticError> {
        let modules = modules
            .iter()
            .map(|(id, bytes)| (id.to_string(), bytes.to_vec()))
            .collect();
        check_file_with_provider(std::path::Path::new(root), &MapProvider { modules })
    }

    fn assert_module_parse_error_at(
        err: &SemanticError,
        module: &str,
        source: &str,
        expected: SourceMark,
    ) {
        let profile = ParserProfile::foundation_default();
        let direct = parse_logos_program_with_profile(source, &profile)
            .expect_err("fixture must fail direct Logos parse");
        assert_eq!(err.diag.code, "E0239");
        assert_eq!(err.diag.mark, expected);
        assert_eq!(
            err.diag.mark,
            source_mark_from_byte_offset(source, direct.pos)
        );
        assert_ne!(err.diag.mark, SourceMark::default());
        let rest = err
            .diag
            .message
            .strip_prefix("failed to parse module '")
            .expect("wrapper prefix must be preserved");
        let (shown_module, parser_message) = rest
            .split_once("': ")
            .expect("wrapper must separate module identity from the parser message");
        assert_eq!(shown_module.replace('\\', "/"), module);
        assert_eq!(parser_message, direct.message);
        assert!(err
            .diag
            .rendered
            .contains(&format!("at line {}:{}", expected.line, expected.col)));
        assert_eq!(err.diag.provider_module_id, None);
        assert_eq!(err.diag.frontend_error_kind, None);
    }

    #[test]
    fn imported_module_parse_error_preserves_parser_position() {
        let root = "/virtual/root.sm";
        let child = "/virtual/child.sm";
        let root_src = warning_project_root("Import \"child.sm\"\n");
        let err = check_modules(
            root,
            &[
                (root, root_src.as_bytes()),
                (child, MALFORMED_MODULE_SRC.as_bytes()),
            ],
        )
        .expect_err("malformed imported module must fail");
        assert_module_parse_error_at(&err, child, MALFORMED_MODULE_SRC, MALFORMED_MODULE_MARK);
    }

    #[test]
    fn root_module_parse_error_preserves_parser_position() {
        let root = "/virtual/root.sm";
        let err = check_modules(root, &[(root, MALFORMED_MODULE_SRC.as_bytes())])
            .expect_err("malformed root module must fail");
        assert_module_parse_error_at(&err, root, MALFORMED_MODULE_SRC, MALFORMED_MODULE_MARK);
    }

    #[test]
    fn module_parse_error_aggregate_uses_first_error_position() {
        let root = "/virtual/root.sm";
        let src = "Entity P:\n    state hp: quad\nEntity Q\nEntity R\n";
        let err = check_modules(root, &[(root, src.as_bytes())])
            .expect_err("malformed root module must fail");
        assert!(err.diag.message.contains("multiple parser errors (2)"));
        let expected = SourceMark {
            line: 3,
            col: 9,
            file_id: 0,
        };
        assert_module_parse_error_at(&err, root, src, expected);
    }

    #[test]
    fn module_parse_error_at_genuine_byte_zero_maps_to_line_one_column_one() {
        let root = "/virtual/root.sm";
        let src = "Bogus\nEntity P:\n    state hp: quad\n";
        let profile = ParserProfile::foundation_default();
        let direct = parse_logos_program_with_profile(src, &profile).expect_err("must fail");
        assert_eq!(direct.pos, 0);
        assert!(direct.pos < src.len(), "offset zero must not be exhaustion");
        assert!(
            direct.message.contains("--> <input>:1:1"),
            "the parser located a real token at offset zero: {}",
            direct.message
        );
        let err = check_modules(root, &[(root, src.as_bytes())]).expect_err("must fail");
        let expected = SourceMark {
            line: 1,
            col: 1,
            file_id: 0,
        };
        assert_module_parse_error_at(&err, root, src, expected);
    }

    #[test]
    fn module_parse_error_at_token_exhaustion_maps_to_source_eof() {
        let root = "/virtual/root.sm";
        let cases = [
            (
                "Entity Player:\n",
                SourceMark {
                    line: 2,
                    col: 1,
                    file_id: 0,
                },
            ),
            (
                "Entity P:",
                SourceMark {
                    line: 1,
                    col: 10,
                    file_id: 0,
                },
            ),
        ];
        let profile = ParserProfile::foundation_default();
        for (src, expected) in cases {
            let direct = parse_logos_program_with_profile(src, &profile).expect_err("must fail");
            assert_eq!(
                direct.pos,
                src.len(),
                "exhausted parser must report EOF: {src:?}"
            );
            let err = check_modules(root, &[(root, src.as_bytes())]).expect_err("must fail");
            assert_module_parse_error_at(&err, root, src, expected);
        }
    }

    struct ResolveFailsProvider;

    impl crate::alloc_core::ModuleProvider for ResolveFailsProvider {
        fn read_module(&self, _module_id: &str) -> Result<Vec<u8>, String> {
            Ok(b"Import \"x.sm\"\nEntity A:\n    state a: quad\n".to_vec())
        }

        fn resolve_import(&self, _importer_module_id: &str, _spec: &str) -> Result<String, String> {
            Err("no such import".to_string())
        }
    }

    #[test]
    fn non_parse_e0239_failures_remain_positionless() {
        let root = "/virtual/root.sm";
        let root_src = warning_project_root("Import \"gone.sm\"\n");
        let read_fail = check_modules(root, &[(root, root_src.as_bytes())])
            .expect_err("missing import must fail to read");
        let not_utf8 = check_modules(root, &[(root, &[0xff, 0xfe, 0x00][..])])
            .expect_err("invalid utf-8 must fail");
        let resolve_fail =
            check_file_with_provider(std::path::Path::new(root), &ResolveFailsProvider)
                .expect_err("unresolvable import must fail");
        for (err, needle) in [
            (read_fail, "failed to read import"),
            (not_utf8, "not valid utf-8"),
            (resolve_fail, "failed to resolve import"),
        ] {
            assert_eq!(err.diag.code, "E0239");
            assert!(err.diag.message.contains(needle), "{}", err.diag.message);
            assert_eq!(err.diag.mark, SourceMark::default());
        }
    }

    #[test]
    fn provider_warning_preserves_code_message_and_mark() {
        let root = "/virtual/root.sm";
        let helper = "/virtual/deps/a/warn.sm";
        let root_src = warning_project_root("Import \"a::warn.sm\"\n");
        let mut modules = BTreeMap::new();
        modules.insert(root.to_string(), root_src.into_bytes());
        modules.insert(
            helper.to_string(),
            warning_fixture_source().as_bytes().to_vec(),
        );
        let provider = MapProvider { modules };

        let project =
            check_file_with_provider(std::path::Path::new(root), &provider).expect("provider");
        let direct = check_source(warning_fixture_source()).expect("direct");

        for code in ["W0240", "W0252"] {
            let project_warning = project
                .warnings
                .iter()
                .find(|warning| warning.code == code)
                .expect("project warning");
            let direct_warning = direct
                .warnings
                .iter()
                .find(|warning| warning.code == code)
                .expect("direct warning");
            assert_eq!(project_warning.code, direct_warning.code);
            assert_eq!(project_warning.message, direct_warning.message);
            assert_eq!(project_warning.mark, direct_warning.mark);
            assert_eq!(project_warning.provider_module_id.as_deref(), Some(helper));
            assert!(direct_warning.provider_module_id.is_none());
        }
    }

    #[test]
    fn provider_warning_rendered_output_is_unchanged() {
        let root = "/virtual/root.sm";
        let helper = "/virtual/deps/a/warn.sm";
        let root_src = warning_project_root("Import \"a::warn.sm\"\n");
        let mut modules = BTreeMap::new();
        modules.insert(root.to_string(), root_src.into_bytes());
        modules.insert(
            helper.to_string(),
            warning_fixture_source().as_bytes().to_vec(),
        );
        let provider = MapProvider { modules };

        let project =
            check_file_with_provider(std::path::Path::new(root), &provider).expect("provider");
        let direct = check_source(warning_fixture_source()).expect("direct");

        for code in ["W0240", "W0252"] {
            let project_warning = project
                .warnings
                .iter()
                .find(|warning| warning.code == code)
                .expect("project warning");
            let direct_warning = direct
                .warnings
                .iter()
                .find(|warning| warning.code == code)
                .expect("direct warning");
            assert_eq!(project_warning.rendered, direct_warning.rendered);
        }
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

    // ------------------------------------------------------------------
    // SSF-09 Decision E / #1670 Stage 2A: `check_source_with_profile`
    // regressions (Tier B - end-to-end, real source text).
    //
    // Decision F (2026-09-14): `resolve_surface_authority`/`SurfaceVerdict`
    // moved to the canonical `sm_front::{resolve_surface_authority,
    // SurfaceAuthority}` - this crate now only consumes it. The former
    // Tier A direct-unit-test suite (every cell of the frozen two-stage
    // table via synthetic `GrammarAdmission` values) moved with it to
    // `sm-front`'s own test module, where the canonical implementation
    // now lives.

    // T1 - authoritative malformed Logos: clear Logos-exclusive evidence
    // (Entity) followed by a genuine Logos syntax failure (empty When
    // condition, E0231). Must fail with the originating Logos error,
    // never reinterpreted as RustLike.
    #[test]
    fn malformed_logos_authoritative_failure_not_retried_as_rustlike() {
        let src = "Entity A:\n    state x: quad\nLaw \"L\" [priority 1]:\n    When -> System.recovery()\n";
        let profile = ParserProfile::foundation_default();
        let err = check_source_with_profile(src, &profile).expect_err("must fail");
        let rendered = err.to_string();
        assert!(
            rendered.contains("E0231") || rendered.contains("empty When condition"),
            "expected the originating Logos failure, got: {rendered}"
        );
        assert!(
            !rendered.contains("expected top-level"),
            "must not be reinterpreted as a RustLike parse failure: {rendered}"
        );
    }

    // Logos policy rejection: genuine Logos-exclusive evidence under a
    // profile with the Logos surface disabled. Exercises the
    // policy-vs-syntax finalization law's first branch
    // (`require_logos_surface` outranks everything). Uses the existing
    // `FeaturePolicy::allow_logos_surface` knob - no new profile knob.
    #[test]
    fn logos_policy_rejection_is_preserved_not_reinterpreted() {
        let src = "Entity A:\n    state x: quad\n";
        let mut profile = ParserProfile::foundation_default();
        profile.features.allow_logos_surface = false;
        let err = check_source_with_profile(src, &profile).expect_err("must fail");
        let rendered = err.to_string();
        assert!(
            rendered.contains("disabled by profile policy"),
            "expected the Logos policy rejection, got: {rendered}"
        );
        assert!(
            !rendered.contains("expected top-level"),
            "must not be reinterpreted as a RustLike parse failure: {rendered}"
        );
    }

    // T2 - malformed RustLike: must never be reinterpreted through Logos.
    #[test]
    fn malformed_rustlike_never_reinterpreted_through_logos() {
        let src = "fn main(\n";
        let profile = ParserProfile::foundation_default();
        let err = check_source_with_profile(src, &profile).expect_err("must fail");
        let rendered = err.to_string();
        assert!(
            !rendered.contains("expected Logos declaration"),
            "must not be reinterpreted as a Logos error: {rendered}"
        );
    }

    // T3 - exact FrontendError preservation for the authoritative Logos
    // failure: `check_source_with_profile`'s diagnostic message must be
    // byte-identical to the direct parser's own message, not a
    // regenerated or flattened one.
    #[test]
    fn logos_exclusive_failure_message_exactly_matches_direct_parse() {
        let src = "Entity A:\n    state x: quad\nLaw \"L\" [priority 1]:\n    When -> System.recovery()\n";
        let profile = ParserProfile::foundation_default();
        let direct = parse_logos_program_with_profile(src, &profile)
            .expect_err("direct logos parse must fail");
        let via_sema = check_source_with_profile(src, &profile).expect_err("must fail");
        assert_eq!(via_sema.diag.message, direct.message);
    }

    // T4 - exact FrontendError preservation for the authoritative
    // RustLike failure, mirroring T3.
    #[test]
    fn rustlike_exclusive_failure_message_exactly_matches_direct_parse() {
        let src = "fn main(\n";
        let profile = ParserProfile::foundation_default();
        let direct =
            parse_program_with_profile(src, &profile).expect_err("direct rustlike parse must fail");
        let via_sema = check_source_with_profile(src, &profile).expect_err("must fail");
        assert_eq!(via_sema.diag.message, direct.message);
    }

    #[test]
    fn rustlike_syntax_error_preserves_frontend_source_mark() {
        let src = "fn main() {\n    let value: i32 =\n}\n";
        let profile = ParserProfile::foundation_default();
        let direct =
            parse_program_with_profile(src, &profile).expect_err("direct RustLike parse must fail");
        let expected_mark = lex(src)
            .expect("fixture must lex")
            .iter()
            .find(|token| token.pos == direct.pos)
            .expect("parser failure must point at a token")
            .mark;
        assert_ne!(expected_mark, SourceMark::default());

        let via_sema = check_source_with_profile(src, &profile).expect_err("must fail");
        assert_eq!(via_sema.diag.code, "E0000");
        assert_eq!(via_sema.diag.message, direct.message);
        assert_eq!(via_sema.diag.mark, expected_mark);
        assert_eq!(via_sema.diag.provider_module_id, None);
        assert_eq!(
            via_sema.diag.frontend_error_kind,
            Some(FrontendErrorKind::Syntax)
        );
        assert!(via_sema.diag.rendered.contains("2:21"));
    }

    #[test]
    fn rustlike_policy_error_preserves_frontend_mark_and_kind() {
        let src = "fn main() {\n    let value: f64 = 1.5;\n    return;\n}\n";
        let profile = ParserProfile::default();
        let direct =
            parse_program_with_profile(src, &profile).expect_err("strict profile must reject f64");
        let expected_mark = lex(src)
            .expect("fixture must lex")
            .iter()
            .find(|token| token.pos == direct.pos)
            .expect("policy rejection must point at a token")
            .mark;
        assert_eq!(direct.kind(), FrontendErrorKind::PolicyViolation);
        assert_eq!(expected_mark.line, 2);
        assert!(expected_mark.col > 1);

        let via_sema = check_source_with_profile(src, &profile).expect_err("must fail");
        assert_eq!(via_sema.diag.code, "E0000");
        assert_eq!(via_sema.diag.message, direct.message);
        assert_eq!(via_sema.diag.mark, expected_mark);
        assert_eq!(
            via_sema.diag.frontend_error_kind,
            Some(FrontendErrorKind::PolicyViolation)
        );
        assert_eq!(via_sema.diag.provider_module_id, None);
        assert!(via_sema
            .diag
            .rendered
            .contains(&format!("{}:{}", expected_mark.line, expected_mark.col)));
    }

    #[test]
    fn frontend_error_mark_uses_token_marks_and_byte_offset_fallback() {
        let source = "x\n\tz";
        let tokens = lex(source).expect("fixture must lex");
        let token = tokens
            .iter()
            .find(|token| token.pos == 3)
            .expect("symbol token must start at byte offset 3");
        assert_eq!(frontend_error_mark(&tokens, source, 3), token.mark);
        assert_eq!(
            frontend_error_mark(&tokens, source, source.len()),
            SourceMark {
                line: 2,
                col: 3,
                file_id: 0,
            }
        );
    }

    #[test]
    fn unrelated_direct_diagnostic_has_no_frontend_error_kind() {
        let profile = ParserProfile::foundation_default();
        let err = check_source_with_profile("\n", &profile).expect_err("blank source must fail");
        assert_eq!(err.diag.frontend_error_kind, None);
    }

    #[test]
    fn rustlike_typecheck_path_remains_unannotated() {
        let src = "fn main() { let value: i32 = true; return; }";
        let profile = ParserProfile::foundation_default();
        let err = check_source_with_profile(src, &profile).expect_err("type check must fail");
        assert_eq!(err.diag.code, "E0201");
        assert_eq!(err.diag.mark, SourceMark::default());
        assert_eq!(err.diag.frontend_error_kind, None);
    }

    // T5 - confirmed shared Import collision: `Import "a.sm"` alone
    // parses to completion under both grammars. Must be rejected with a
    // deterministic ambiguity outcome, never silently resolved to
    // either side.
    #[test]
    fn quoted_import_alone_is_ambiguous() {
        let src = "Import \"a.sm\"\n";
        let profile = ParserProfile::foundation_default();
        let err = check_source_with_profile(src, &profile).expect_err(
            "Import \"a.sm\" is independently valid under both grammars and must not silently succeed",
        );
        let rendered = err.to_string().to_lowercase();
        assert!(
            rendered.contains("ambiguous") || rendered.contains("conflict"),
            "expected a deterministic ambiguity/conflict diagnosis, got: {}",
            err
        );
    }

    // T6 - shared vocabulary is not automatic ambiguity: `Import a.sm`
    // (no string literal) is accepted by Logos's permissive legacy
    // handling but rejected by RustLike's own `parse_import_decl`, which
    // requires a string literal specifically. Must resolve to unique
    // Logos ownership (accepted), never manufactured ambiguity from the
    // shared `Import` keyword alone.
    #[test]
    fn unquoted_legacy_import_is_accepted_as_logos() {
        let src = "Import a.sm\n";
        let profile = ParserProfile::foundation_default();
        assert!(
            check_source_with_profile(src, &profile).is_ok(),
            "an Import line RustLike's own parser rejects (no string literal) must be unique \
             Logos ownership, not ambiguity"
        );
    }

    // Bare Pulse/Profile: unconditional Logos-exclusive evidence: no
    // System/Entity/Law companion required, no RustLike fallback even
    // when the Logos surface is policy-disabled.
    #[test]
    fn bare_pulse_is_unconditional_logos_exclusive_evidence() {
        let profile = ParserProfile::foundation_default();
        let src = "Pulse \"x\"\n";
        assert!(
            check_source_with_profile(src, &profile).is_ok(),
            "a bare Pulse directive is unconditional Logos-exclusive evidence"
        );

        let mut policy_profile = ParserProfile::foundation_default();
        policy_profile.features.allow_logos_surface = false;
        let err = check_source_with_profile(src, &policy_profile)
            .expect_err("Logos surface disabled must fail, never silently pass to RustLike");
        let rendered = err.to_string();
        assert!(
            rendered.contains("disabled by profile policy"),
            "must fail via the Logos-authoritative policy path, got: {rendered}"
        );
        assert!(
            !rendered.contains("expected top-level"),
            "must not fall through to RustLike's own rejection of `Pulse`: {rendered}"
        );
    }

    // T7 - Logos-exclusive evidence alongside shared Import still
    // reaches unique Logos ownership: `Exclusive(Ok)` vs `Shared(Err)` ->
    // Logos owns, accepted.
    #[test]
    fn logos_exclusive_with_shared_import_is_accepted_as_logos() {
        let src = "Import \"a.sm\"\nEntity Player:\n    state hp: quad\n";
        let profile = ParserProfile::foundation_default();
        check_source_with_profile(src, &profile).expect(
            "Logos-exclusive evidence alongside shared import must still reach Logos, accepted",
        );
    }

    // T8 - symmetric case: RustLike-exclusive evidence alongside shared
    // Import still reaches unique RustLike ownership: `Shared(Err)` vs
    // `Exclusive(Ok)` -> RustLike owns, accepted. Also doubles as the
    // "ordinary RustLike program must keep working" regression.
    #[test]
    fn rustlike_exclusive_with_shared_import_is_accepted_as_rustlike() {
        let src = "Import \"a.sm\"\nfn main() {}\n";
        let profile = ParserProfile::foundation_default();
        check_source_with_profile(src, &profile).expect(
            "RustLike-exclusive evidence alongside shared import must still reach RustLike, accepted",
        );
    }

    #[test]
    fn ordinary_rustlike_program_is_still_admitted() {
        let src = "fn main() {\n    return;\n}\n";
        let profile = ParserProfile::foundation_default();
        assert!(
            check_source_with_profile(src, &profile).is_ok(),
            "an ordinary RustLike program must still be admitted"
        );
    }

    // T9 - Decision E's own worked "side effect" example: RustLike
    // commits at the string literal then fails on the malformed alias
    // (`Shared(Err)`); Logos has no further content requirement for
    // Import (`Shared(Ok)`). Stage 2: Ok+Err -> Logos owns, accepted.
    #[test]
    fn shared_ok_err_resolves_to_logos_via_malformed_alias() {
        let src = "Import \"a.sm\" as 123\n";
        let profile = ParserProfile::foundation_default();
        check_source_with_profile(src, &profile)
            .expect("Logos Shared(Ok) must win over RustLike Shared(Err) on this shape");
    }

    // T13 - no-evidence case: genuinely blank/comment-free input
    // establishes no top-level evidence for either grammar
    // (`NoClaim`/`NoClaim`). Deliberately different from the pre-#1670
    // behavior of silently succeeding as a trivially-empty RustLike
    // program - see the discovery evidence note.
    #[test]
    fn blank_input_is_no_surface_claim() {
        let profile = ParserProfile::foundation_default();
        let err = check_source_with_profile("\n\n   \n", &profile)
            .expect_err("genuinely blank input establishes no evidence for either grammar");
        assert!(err.to_string().contains("NO SURFACE CLAIM"), "got: {}", err);
    }

    // A lex failure is not a surface-admission outcome for either
    // grammar (Decision E) - it must short-circuit before either
    // `admit_*` function is ever called, and must never be reported as
    // an ambiguity between the two grammars.
    #[test]
    fn lex_failure_short_circuits_before_admission() {
        let src = "Import \"unterminated\n";
        let profile = ParserProfile::foundation_default();
        let err = check_source_with_profile(src, &profile)
            .expect_err("unterminated string literal must fail to lex");
        assert!(
            !err.to_string().to_lowercase().contains("ambiguous"),
            "a lex failure must not be reported as cross-grammar ambiguity: {}",
            err
        );
    }
}
