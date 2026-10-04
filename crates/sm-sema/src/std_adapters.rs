use crate::alloc_core::ConditionInferError;
use crate::alloc_core::{
    build_export_sets_core, collect_local_exports_core, diagnostic_help_core,
    evaluate_law_header_policy_core, infer_when_condition_type_core, insert_name_core,
    is_dead_when_condition, is_magic_number_atom, is_valid_when_result_type_core,
    parse_import_directive, parse_law_local_decl, validate_import_bindings_core,
    validate_import_namespace_rules as validate_import_namespace_rules_core,
    validate_select_imports_core, validate_when_non_empty_core, ExportBuildModule, ExportKind,
    ExportSet, ImportDirective, LawScheduler, LocalExportDecl, ScopeKind, SelectImportModule,
    SemanticType, Symbol, SymbolTable,
};
use crate::frontend::{
    admit_logos_program_with_profile, admit_program_with_profile, lex,
    parse_logos_program_with_profile, resolve_surface_authority, type_check_program, FrontendError,
    FrontendErrorKind, LogosEntity, LogosEntityFieldKind, LogosProgram, ParserProfile, Program,
    SourceMark, SurfaceAuthority, Token, Type,
};
use sm_diagnostic::{
    Diagnostic as CanonicalDiagnostic, DiagnosticCause, DiagnosticCode, DiagnosticFamily,
    DiagnosticMessage, DiagnosticSeverity, SourceContext, SourceId, SourceRange,
};
use sm_front::diagnostic_authority::{
    token_anchor_range_at_mark, FrontendDiagnostic, FrontendRelated, FrontendStage,
    FRONTEND_AMBIGUOUS_SURFACE_CODE, FRONTEND_NO_SURFACE_CLAIM_CODE,
};
use sm_front::lexer::lex_tokens_with_authority;
use sm_front::LogosAtom;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use ton618_core::diagnostics::{
    append_help_line, format_diagnostic_header, render_context_with_caret,
};
use ton618_core::{Arena, SourceMap};

impl From<Type> for SemanticType {
    fn from(value: Type) -> Self {
        match value {
            // PB-03 (#1671, #1672): exact source families.
            Type::I32 => SemanticType::I32,
            Type::F64 => SemanticType::F64,
            Type::Fx => SemanticType::Fx,
            Type::Quad => SemanticType::Quad,
            Type::QVec(n) => SemanticType::QVec(n),
            Type::Bool => SemanticType::Bool,
            Type::Text => SemanticType::Unknown,
            Type::Sequence(_) => SemanticType::Unknown,
            Type::Closure(_) => SemanticType::Unknown,
            Type::U32 => SemanticType::U32,
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
    /// SSF-09 C2: everything the producer-owned canonical projection needs
    /// beyond the legacy fields, boxed so the error type stays small.
    pub canonical: Box<CanonicalAttachment>,
}

/// SSF-09 C2: canonical-projection data attached to a [`SemanticDiagnostic`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalAttachment {
    /// Originating canonical stage. `Frontend` when this
    /// diagnostic relays an `sm-front` failure (lexer, parser, surface
    /// resolution, type checker) under the frontend's own code; `Semantic`
    /// for every diagnostic `sm-sema` itself originates.
    pub family: DiagnosticFamily,
    /// Genuine UTF-8 byte range `[start, end)` within the source
    /// text this diagnostic was produced from, or `None` when no producer
    /// authority proves one. Never derived from a default/zero mark.
    pub range: Option<Range<usize>>,
    /// The presentation-free message for the canonical carrier
    /// when `message` still carries transitional legacy presentation (a
    /// relayed Logos parser block, or a module path). `None` means
    /// `message` is already presentation-free.
    pub canonical_message: Option<String>,
    /// Further recovered frontend errors of a relayed parse
    /// failure, in producer order (same source as this diagnostic).
    pub related: Vec<FrontendRelated>,
    /// The structured frontend failure this diagnostic wraps
    /// (for example `E0239` wrapping the imported module's parse error).
    pub frontend_cause: Option<Box<FrontendDiagnostic>>,
}

impl CanonicalAttachment {
    /// A `sm-sema`-originated diagnostic with no proven range and a
    /// presentation-free legacy message.
    pub fn semantic(range: Option<Range<usize>>) -> Self {
        Self {
            family: DiagnosticFamily::Semantic,
            range,
            canonical_message: None,
            related: Vec::new(),
            frontend_cause: None,
        }
    }
}

impl SemanticDiagnostic {
    /// SSF-09 C2: producer-owned lossless projection into the canonical
    /// carrier. Code, severity and family are relayed unchanged; the range
    /// is attached only when both a `source` token and a proven range exist.
    /// Returns `None` only if the code or message violates the carrier's
    /// non-empty invariants - never a substituted placeholder.
    pub fn to_canonical(&self, source: Option<SourceId>) -> Option<CanonicalDiagnostic> {
        let code = DiagnosticCode::try_from_static(self.code).ok()?;
        let message = DiagnosticMessage::new(
            self.canonical
                .canonical_message
                .clone()
                .unwrap_or_else(|| self.message.clone()),
        )
        .ok()?;
        let severity = match self.level {
            DiagLevel::Error => DiagnosticSeverity::Error,
            DiagLevel::Warning => DiagnosticSeverity::Warning,
        };
        let mut diagnostic =
            CanonicalDiagnostic::new(code, severity, self.canonical.family, message);
        diagnostic.source_context = source.map(|source| SourceContext {
            source,
            range: self
                .canonical
                .range
                .clone()
                .and_then(|r| SourceRange::try_from_bounds(r.start, r.end)),
        });
        if !self.canonical.related.is_empty() {
            // Relayed frontend failure: its recovered errors project exactly
            // as the frontend's own adapter projects them.
            let relay = FrontendDiagnostic {
                stage: FrontendStage::Parse,
                code: self.code,
                message: self.message.clone(),
                range: None,
                related: self.canonical.related.clone(),
            };
            let projected = relay.to_canonical(source)?;
            diagnostic.related_locations = projected.related_locations;
            diagnostic.notes = projected.notes;
        }
        if let Some(cause) = &self.canonical.frontend_cause {
            diagnostic.cause = Some(Box::new(DiagnosticCause::Diagnostic(Box::new(
                cause.to_canonical(source)?,
            ))));
        }
        Some(diagnostic)
    }
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
        diag: render_frontend_diag(
            FrontendDiagnostic::surface_resolution(FRONTEND_AMBIGUOUS_SURFACE_CODE, message),
            SourceMark::default(),
            input,
        ),
    }
}

fn no_surface_claim_error(input: &str) -> SemanticError {
    SemanticError {
        diag: render_frontend_diag(
            FrontendDiagnostic::surface_resolution(
                FRONTEND_NO_SURFACE_CLAIM_CODE,
                "NO SURFACE CLAIM: this input establishes no top-level evidence for either the Logos or RustLike grammar".to_string(),
            ),
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
        diag: render_frontend_diag(
            FrontendDiagnostic::from_type_check_error(&e),
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
    let tokens = lex_tokens_with_authority(input).map_err(|failure| {
        let mark = source_mark_from_byte_offset(input, failure.error.pos);
        SemanticError {
            diag: render_frontend_diag(FrontendDiagnostic::from_lex_failure(&failure), mark, input),
        }
    })?;
    let logos = admit_logos_program_with_profile(input, &tokens, profile);
    let rustlike = admit_program_with_profile(input, &tokens, profile);
    match resolve_surface_authority(logos, rustlike) {
        SurfaceAuthority::LogosOwns(Ok(program)) => analyze_logos_program(&program, input),
        SurfaceAuthority::LogosOwns(Err(e)) => Err(SemanticError {
            diag: render_frontend_diag_with_legacy(
                FrontendDiagnostic::from_parse_error(input, &tokens, &e),
                Some(e.message.clone()),
                frontend_error_mark(&tokens, input, e.pos),
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
    let mut diag = render_frontend_diag_with_legacy(
        FrontendDiagnostic::from_parse_error(input, tokens, &error),
        Some(error.message.clone()),
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
        let module_key = path_contract_key(&module_path);
        let mut report = analyze_logos_program(logos, src).map_err(|mut e| {
            // SSF-09 C2: the host-path prefix below is legacy presentation;
            // the canonical message stays the analyzer's own text and the
            // module is bound structurally through `provider_module_id`.
            if e.diag.canonical.canonical_message.is_none() {
                e.diag.canonical.canonical_message = Some(e.diag.message.clone());
            }
            e.diag.message = format!("{}: {}", module_path.display(), e.diag.message);
            e.diag.rendered = format!("in module '{}'\n{}", module_path.display(), e.diag.rendered);
            // SSF-09 C2: the failing module is known structurally; attach it
            // as provenance instead of leaving file attribution to the text.
            e.diag.provider_module_id = Some(module_key.clone());
            e
        })?;
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

/// SSF-09 #1580: the provider-owned name of the module at `path` in
/// diagnostic text. Legacy providers keep the id, and then the historical
/// `Path::display` spelling is preserved byte for byte.
fn shown_module(provider: &dyn crate::alloc_core::ModuleProvider, path: &Path) -> String {
    let id = path_contract_key(path);
    let display = provider.display_module(&id);
    if display == id {
        path.display().to_string()
    } else {
        display
    }
}

/// SSF-09 #1580: the provider-owned module key used inside the select/export
/// core checks (whose messages name modules by key).
fn display_key(provider: &dyn crate::alloc_core::ModuleProvider, path: &Path) -> String {
    provider.display_module(&path_contract_key(path))
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
            .map(|entry| display_key(provider, &entry.path))
            .collect::<Vec<_>>();
        full_chain.push(display_key(provider, path));
        let mut cycle_chain = visiting[pos..]
            .iter()
            .map(|entry| display_key(provider, &entry.path))
            .collect::<Vec<_>>();
        cycle_chain.push(display_key(provider, path));
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
                format!(
                    "failed to read import '{}': {}",
                    shown_module(provider, path),
                    e
                ),
                SourceMark::default(),
                "",
            ),
        })?;
    let source = String::from_utf8(bytes).map_err(|_| SemanticError {
        diag: render_diag(
            DiagLevel::Error,
            "E0239",
            format!(
                "module '{}' is not valid utf-8",
                shown_module(provider, path)
            ),
            SourceMark::default(),
            "",
        ),
    })?;
    let logos = parse_logos_program_with_profile(&source, profile).map_err(|e| {
        let mut diag = render_diag(
            DiagLevel::Error,
            "E0239",
            format!(
                "failed to parse module '{}': {}",
                shown_module(provider, path),
                e.message
            ),
            source_mark_from_byte_offset(&source, e.pos),
            &source,
        );
        diag.provider_module_id = Some(module_id.clone());
        // SSF-09 C2: canonically this is sm-sema's module-load failure,
        // bound to the module's own source, wrapping the frontend's
        // structured parse error as its cause - no host path and no
        // rendered caret block in the canonical message.
        diag.canonical.canonical_message = Some("failed to parse module".to_string());
        if let Ok(tokens) = lex(&source) {
            diag.canonical.frontend_cause = Some(Box::new(FrontendDiagnostic::from_parse_error(
                &source, &tokens, &e,
            )));
        }
        SemanticError { diag }
    })?;

    visiting.push(VisitingImport {
        path: key.clone(),
        via_reexport,
    });
    let importer_module_id = path_contract_key(&key);
    // PB-02 / #1645: consume exactly the Import directives sm-front
    // preserved, never a second scan of the raw source.
    let imports = preserved_import_directives(&logos, &source).map_err(|mut e| {
        e.diag.provider_module_id = Some(importer_module_id.clone());
        e
    })?;
    validate_import_namespace_rules(&imports, &logos, &source).map_err(|mut e| {
        e.diag.provider_module_id = Some(importer_module_id.clone());
        e
    })?;
    for import in imports {
        let resolved = provider
            .resolve_import(&importer_module_id, &import.spec)
            .map_err(|e| {
                let mut diag = render_diag(
                    DiagLevel::Error,
                    "E0239",
                    format!("failed to resolve import '{}': {}", import.spec, e),
                    SourceMark::default(),
                    &source,
                );
                diag.provider_module_id = Some(importer_module_id.clone());
                SemanticError { diag }
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

/// Interprets each `LogosProgram::imports` node exactly once, in order. A
/// preserved directive that carries no usable import spec is a deterministic
/// error, never a silent skip.
fn preserved_import_directives(
    logos: &LogosProgram,
    source: &str,
) -> Result<Vec<ImportDirective>, SemanticError> {
    logos
        .imports
        .iter()
        .enumerate()
        .map(|(order, import)| {
            parse_import_directive(
                &import.directive,
                import.mark.line,
                import.mark.col,
                order as u32,
            )
            .ok_or_else(|| SemanticError {
                diag: render_diag(
                    DiagLevel::Error,
                    "E0239",
                    format!("malformed Import directive '{}'", import.directive),
                    import.mark,
                    source,
                ),
            })
        })
        .collect()
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

    // SSF-09 #1580: the core check names modules by key in its messages,
    // so it runs over provider display keys; `id_by_key` maps a key back to
    // the module id for structural provenance.
    let mut id_by_key = std::collections::BTreeMap::<String, String>::new();
    for (k, set) in export_sets {
        let key = display_key(provider, k);
        let mut syms = std::collections::BTreeSet::<String>::new();
        let mut kinds = std::collections::BTreeMap::<String, ExportKind>::new();
        for item in &set.items {
            syms.insert(item.public_name.clone());
            kinds.entry(item.public_name.clone()).or_insert(item.kind);
        }
        export_symbols.insert(key.clone(), syms);
        export_kinds.insert(key, kinds);
    }

    for module in modules {
        let (src, logos) = loaded.get(&module).expect("module key from loaded.keys()");
        let imports = preserved_import_directives(logos, src)?;
        let module_id = path_contract_key(&module);
        let module_key = display_key(provider, &module);
        id_by_key.insert(module_key.clone(), module_id.clone());
        src_by_key.insert(module_key.clone(), src.clone());
        for import in &imports {
            let dep = provider
                .resolve_import(&module_id, &import.spec)
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
                display_key(provider, &dep),
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
            let mut diag = render_diag(
                DiagLevel::Error,
                e.code,
                e.message,
                SourceMark {
                    line: e.line,
                    col: e.col,
                    file_id: 0,
                },
                src,
            );
            // SSF-09 C2: the failing import site is in this module.
            diag.provider_module_id = id_by_key.get(&e.module_key).cloned();
            SemanticError { diag }
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
    // SSF-09 #1580: the core names modules by provider display key; this
    // maps each key back to its module path.
    let mut path_by_key = std::collections::BTreeMap::<String, PathBuf>::new();
    for module in &keys {
        let (source, logos) = loaded.get(module).ok_or_else(|| SemanticError {
            diag: render_diag(
                DiagLevel::Error,
                "E0239",
                format!("unknown module '{}'", shown_module(provider, module)),
                SourceMark::default(),
                "",
            ),
        })?;
        let module_id = path_contract_key(module);
        let module_key = display_key(provider, module);
        path_by_key.insert(module_key.clone(), module.clone());
        let imports = preserved_import_directives(logos, source)?;
        for import in &imports {
            let dep = provider
                .resolve_import(&module_id, &import.spec)
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
                display_key(provider, &dep),
            );
        }
        modules.push(ExportBuildModule {
            local_exports: collect_local_exports(&shown_module(provider, module), logos),
            module_key,
            source: source.clone(),
            imports,
        });
    }
    let core_sets = build_export_sets_core(&modules, &dep_lookup).map_err(|e| {
        let src = modules
            .iter()
            .find(|m| m.module_key == e.module_key)
            .map(|m| m.source.as_str())
            .unwrap_or_default();
        let mut diag = render_diag(
            DiagLevel::Error,
            e.code,
            e.message,
            SourceMark {
                line: e.line,
                col: e.col,
                file_id: 0,
            },
            src,
        );
        // SSF-09 C2: bind the failure to the module it was found in.
        diag.provider_module_id = path_by_key.get(&e.module_key).map(|p| path_contract_key(p));
        SemanticError { diag }
    })?;
    let mut out = HashMap::<PathBuf, ExportSet>::new();
    for (key, set) in core_sets {
        let path = path_by_key
            .get(&key)
            .cloned()
            .unwrap_or_else(|| PathBuf::from(&key));
        out.insert(path, set);
    }
    Ok(out)
}

fn collect_local_exports(module_origin: &str, logos: &LogosProgram) -> ExportSet {
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
    collect_local_exports_core(module_origin, &locals)
}

pub fn analyze_logos_program(
    program: &LogosProgram,
    source: &str,
) -> Result<SemanticReport, SemanticError> {
    let mut symbols = SymbolTable::new();
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
        // An Entity name is a declaration, not a value: it never types an
        // operand (PB-03 / #1671).
        symbols
            .insert(Symbol {
                name: entity.name.clone(),
                ty: SemanticType::Unknown,
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
        // FA-03-026 / #1695: an Entity field set is a namespace; a collision
        // is an error, never a silently surviving symbol.
        let mut field_names = BTreeSet::new();
        for field in &entity.fields {
            if !field_names.insert(field.name.as_str()) {
                return Err(SemanticError {
                    diag: render_diag(
                        DiagLevel::Error,
                        "E0220",
                        format!("duplicate field '{}.{}'", entity.name, field.name),
                        field.mark,
                        source,
                    ),
                });
            }
        }
    }

    // FA-03-024/025 (#1693, #1694): Law names are module-level. Logos syntax
    // declares no Law owner, so none is inferred and there is no `_global`.
    let mut law_names = BTreeSet::new();
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
    let resolve_field = |ent: &str, field: &str| {
        entity_map.get(ent).and_then(|entity| {
            entity
                .fields
                .iter()
                .find(|x| x.name == field)
                .map(|f| SemanticType::from(f.ty.clone()))
        })
    };

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

        if !insert_name_core(&mut law_names, &law.name) {
            return Err(SemanticError {
                diag: render_diag(
                    DiagLevel::Error,
                    "E0221",
                    format!("duplicate Law '{}' in module", law.name),
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

        let mut law_locals = BTreeSet::new();
        for when in &law.whens {
            validate_when_non_empty_core(&when.condition, &when.effect).map_err(|e| {
                SemanticError {
                    diag: render_diag(DiagLevel::Error, e.code, e.message, when.mark, source),
                }
            })?;
            // FA-03-010 / #1679: only parsed `Entity.field` atoms that resolve
            // count as a use; text inside string literals is never a field.
            for atom in when.condition_atoms.iter().chain(&when.effect_atoms) {
                if let LogosAtom::Field { entity, field } = atom {
                    if resolve_field(entity, field).is_some() {
                        if let Some(rem) = entity_field_usage.get_mut(entity) {
                            rem.remove(field);
                        }
                    }
                }
            }
            let ty = infer_when_condition_type_core(
                &when.structure,
                |name| symbols.resolve(name).map(|s| s.ty),
                resolve_field,
            )
            .map_err(|e| {
                let message = match e {
                    ConditionInferError::MismatchedTypes { left, right } => {
                        format!("Mismatched types. Expected {}, found {}", left, right)
                    }
                    ConditionInferError::Unresolved(name) => {
                        format!("unresolved name '{}' in When condition", name)
                    }
                    ConditionInferError::InvalidOperands(ty) => {
                        format!("When condition operator cannot take {} operands", ty)
                    }
                    ConditionInferError::InvalidPresent => {
                        "Present(...) requires a name or Entity.field".to_string()
                    }
                    ConditionInferError::Unsupported => {
                        format!("unsupported When condition '{}'", when.condition.trim())
                    }
                };
                SemanticError {
                    diag: render_diag(DiagLevel::Error, "E0201", message, when.mark, source),
                }
            })?;
            if !is_valid_when_result_type_core(ty) {
                return Err(SemanticError {
                    diag: render_diag(
                        DiagLevel::Error,
                        "E0201",
                        format!(
                            "Mismatched types. Expected {} or {}, found {}",
                            SemanticType::Quad,
                            SemanticType::Bool,
                            ty
                        ),
                        when.mark,
                        source,
                    ),
                });
            }

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

            if is_dead_when_condition(&when.structure) {
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
            // FA-03-009 / #1678: no W0241 fx constant folding; no fx evaluator
            // is available to sm-sema, and host f64 is not fx semantics.
            if when
                .condition_atoms
                .iter()
                .chain(&when.effect_atoms)
                .any(is_magic_number_atom)
            {
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

/// SSF-09 C2: sm-sema's own range authority. A diagnostic mark is a legacy
/// `(line, col)` point; it only proves a byte range when it is the mark of a
/// lexical token of `source` itself (the tokens the analyzed AST was built
/// from), in which case the range is that token's exact extent. A default
/// mark, an empty source, or a mark between tokens yields `None`.
fn mark_token_range(source: &str, mark: SourceMark) -> Option<Range<usize>> {
    if source.is_empty() || mark.line == 0 || mark.col == 0 {
        return None;
    }
    let tokens = lex(source).ok()?;
    token_anchor_range_at_mark(source, &tokens, mark.line, mark.col)
}

/// SSF-09 C2: renders an `sm-front` failure under the frontend's own code,
/// family and range. The human `rendered` text keeps the legacy caret
/// presentation anchored at `mark`.
fn render_frontend_diag(
    frontend: FrontendDiagnostic,
    mark: SourceMark,
    source: &str,
) -> SemanticDiagnostic {
    render_frontend_diag_with_legacy(frontend, None, mark, source)
}

/// As [`render_frontend_diag`], keeping `legacy_message` (a relayed
/// parser's transitional rendered text) as the legacy `message` while the
/// frontend's bare message becomes the canonical message.
fn render_frontend_diag_with_legacy(
    frontend: FrontendDiagnostic,
    legacy_message: Option<String>,
    mark: SourceMark,
    source: &str,
) -> SemanticDiagnostic {
    let canonical_message = legacy_message
        .as_ref()
        .filter(|legacy| **legacy != frontend.message)
        .map(|_| frontend.message.clone());
    let mut diag = render_diag(
        DiagLevel::Error,
        frontend.code,
        legacy_message.unwrap_or(frontend.message),
        mark,
        source,
    );
    diag.canonical.family = DiagnosticFamily::Frontend;
    diag.canonical.range = frontend.range;
    diag.canonical.related = frontend.related;
    diag.canonical.canonical_message = canonical_message;
    diag
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
    let range = mark_token_range(source, mark);
    SemanticDiagnostic {
        level,
        code,
        message,
        mark,
        rendered,
        provider_module_id: None,
        frontend_error_kind: None,
        canonical: Box::new(CanonicalAttachment::semantic(range)),
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
    fn compat_policy_is_family_exact() {
        // PB-03 / #1672: no implicit cross-family numeric coercion.
        for (dst, src) in [
            (SemanticType::Fx, SemanticType::I32),
            (SemanticType::I32, SemanticType::Fx),
            (SemanticType::U32, SemanticType::I32),
            (SemanticType::Fx, SemanticType::F64),
            (SemanticType::Unknown, SemanticType::Unknown),
        ] {
            assert!(!crate::alloc_core::is_assignment_compatible(dst, src));
        }
        assert!(crate::alloc_core::is_assignment_compatible(
            SemanticType::Fx,
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
    When false ->
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
        // PB-03 / #1677: `When N` is not dead (N is unknown); `false` is.
        r#"Entity A:
    state x: quad
Law "L" [priority 1]:
    When false ->
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
        // SSF-09 #1580 AC2: the failing module is structurally known here,
        // so the error now carries that module's provider identity (it is
        // still never the *warning* provenance of another module).
        // DEFECT-SSF12-003: provider_module_id is the canonical forward-slash
        // module identifier, independent of host OS path separators.
        assert_eq!(err.diag.provider_module_id.as_deref(), Some(helper));
        assert!(
            !err.diag
                .provider_module_id
                .as_deref()
                .unwrap_or("")
                .contains('\\'),
            "provider_module_id must not leak host backslashes"
        );
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
        // SSF-09 #1580 AC2: module parse errors carry the failing module's
        // provider identity instead of leaving attribution to message text.
        assert_eq!(
            err.diag
                .provider_module_id
                .as_deref()
                .map(|m| m.replace('\\', "/")),
            Some(module.to_string())
        );
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
        let mut reg = crate::alloc_core::TypeRegistry::new();
        let a = reg.intern(SemanticType::Fx).expect("intern");
        let b = reg.intern(SemanticType::Fx).expect("intern");
        let c = reg.intern(SemanticType::QVec(32)).expect("intern");
        assert!(reg.equals_fast(a, b));
        assert!(!reg.equals_fast(a, c));
        assert_eq!(reg.pretty(a), "Fx");
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn no_host_f64_fx_constant_fold_warning() {
        // PB-03 / #1678: W0241 is no longer computed with host f64.
        let src = r#"
Law "L" [priority 1]:
    When true -> fx.add(0.1, 0.2)
"#;
        let p = parse_logos_program(src).expect("logos parse");
        let report = analyze_logos_program(&p, src).expect("semantics");
        assert!(!report.warnings.iter().any(|w| w.code == "W0241"));
        assert!(!report
            .warnings
            .iter()
            .any(|w| w.message.contains("0.30000000000000004")));
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
                // PB-02 / #1645: parsed from the same source; imports are
                // read only from the preserved LogosProgram nodes.
                parse_logos_program(
                    "Import pub \"b.sm\"\nLaw \"A\" [priority 1]:\n    When true -> System.recovery()\n",
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
                    "Import pub \"a.sm\"\nLaw \"B\" [priority 1]:\n    When true -> System.recovery()\n",
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
        // SSF-09 C1A: the frontend's own syntax code replaces retired E0000.
        assert_eq!(via_sema.diag.code, "E0005");
        assert_eq!(via_sema.diag.canonical.family, DiagnosticFamily::Frontend);
        assert_eq!(via_sema.diag.message, direct.message);
        assert_eq!(via_sema.diag.mark, expected_mark);
        let range = via_sema
            .diag
            .canonical
            .range
            .clone()
            .expect("anchored at a token");
        assert_eq!(range.start, direct.pos);
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
        let profile = ParserProfile::core();
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
        // SSF-09 C1A: the frontend's own policy-violation code.
        assert_eq!(via_sema.diag.code, "E0006");
        assert_eq!(via_sema.diag.canonical.family, DiagnosticFamily::Frontend);
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

    // #1943: the corrected `FrontendError.pos` flows through the #1698
    // exact-token mapping without any sema change.
    fn assert_numeric_diag_maps_to_token(src: &str, bad: &str) {
        let profile = ParserProfile::foundation_default();
        let direct = parse_program_with_profile(src, &profile).expect_err("must fail to parse");
        let expected_pos = src.find(bad).expect("bad spelling must occur");
        assert_eq!(direct.pos, expected_pos);
        let expected_mark = lex(src)
            .expect("fixture must lex")
            .iter()
            .find(|token| token.pos == expected_pos)
            .expect("numeric token must exist at its byte offset")
            .mark;
        assert_ne!(
            expected_mark.line, 1,
            "fixture keeps the literal off line 1"
        );

        let via_sema = check_source_with_profile(src, &profile).expect_err("must fail");
        assert_eq!(via_sema.diag.code, "E0005");
        assert_eq!(
            via_sema.diag.canonical.range.clone().map(|r| r.start),
            Some(expected_pos)
        );
        assert_eq!(via_sema.diag.message, direct.message);
        assert_eq!(via_sema.diag.mark, expected_mark);
        assert_eq!(
            via_sema.diag.frontend_error_kind,
            Some(FrontendErrorKind::Syntax)
        );
        assert!(via_sema
            .diag
            .rendered
            .contains(&format!("{}:{}", expected_mark.line, expected_mark.col)));
    }

    #[test]
    fn numeric_literal_errors_map_to_the_numeric_token_not_line_one() {
        for (src, bad) in [
            (
                "fn main() {\n    let q: i32 = 1;\n    let a: i32 = 99999999999;\n    return;\n}\n",
                "99999999999",
            ),
            (
                "fn main() {\n    let q: i32 = 1;\n    let a: u32 = 1.5u32;\n    return;\n}\n",
                "1.5u32",
            ),
            (
                "fn main() {\n    let q: i32 = 1;\n    let a: f64 = 0x10f64;\n    return;\n}\n",
                "0x10f64",
            ),
        ] {
            assert_numeric_diag_maps_to_token(src, bad);
        }
    }

    #[test]
    fn range_pattern_end_bound_error_maps_to_the_end_token() {
        let src = "fn main() {\n    let q: i32 = 1;\n    match x { 1..0x => { return; } _ => { return; } }\n    return;\n}\n";
        assert_numeric_diag_maps_to_token(src, "0x");
    }

    #[test]
    fn numeric_error_mapping_holds_under_crlf_and_utf8_prefix() {
        let base =
            "fn main() {\n    let q: i32 = 1;\n    let a: i32 = 99999999999;\n    return;\n}\n";
        assert_numeric_diag_maps_to_token(&base.replace('\n', "\r\n"), "99999999999");
        assert_numeric_diag_maps_to_token(&format!("// \u{e9}\u{4e2d}\n{base}"), "99999999999");
    }

    #[test]
    fn genuine_byte_zero_rustlike_error_still_maps_to_line_one_column_one() {
        // A RustLike-owned error whose real failing token is at byte zero:
        // the strict profile rejects the leading `schema` token itself.
        let src = "schema S { a: i32 }\nfn main() { return; }\n";
        let profile = ParserProfile::core();
        let direct = parse_program_with_profile(src, &profile).expect_err("must fail");
        assert_eq!(direct.pos, 0);
        let via_sema = check_source_with_profile(src, &profile).expect_err("must fail");
        assert_eq!((via_sema.diag.mark.line, via_sema.diag.mark.col), (1, 1));
        assert_eq!(via_sema.diag.message, direct.message);
        assert_eq!(
            via_sema.diag.frontend_error_kind,
            Some(FrontendErrorKind::PolicyViolation)
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

#[cfg(test)]
mod pb02_logos_import_tests {
    use super::*;
    use sm_front::parse_logos_program;

    fn key(d: &[ImportDirective]) -> Vec<(String, Option<String>, bool, u32)> {
        d.iter()
            .map(|d| (d.spec.clone(), d.alias.clone(), d.reexport, d.decl_order))
            .collect()
    }

    // #1645: an accepted Import is preserved, never discarded.
    #[test]
    fn accepted_logos_imports_are_preserved_losslessly() {
        let src = "Import \"dep.sm\" as D\nImport a.sm\nImport \"b.sm\" as 123\nSystem A():\n";
        let logos = parse_logos_program(src).expect("parse");
        let directives: Vec<&str> = logos.imports.iter().map(|i| i.directive.as_str()).collect();
        assert_eq!(
            directives,
            [
                "Import \"dep.sm\" as D",
                "Import a.sm",
                "Import \"b.sm\" as 123"
            ]
        );
        for import in &logos.imports {
            assert_eq!(&src[import.span.clone()], import.directive);
            assert_eq!(import.mark.col, 1);
        }
        assert_eq!(
            logos
                .imports
                .iter()
                .map(|i| i.mark.line)
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
    }

    // #1645: parse -> preserved imports -> sema consumes exactly those, in
    // order; unrelated raw source text can never create a new import.
    #[test]
    fn sema_consumes_exactly_the_preserved_imports() {
        let src = "Import \"dep.sm\" as D\nImport pub \"re.sm\"\nSystem A():\n";
        let logos = parse_logos_program(src).expect("parse");
        let got = preserved_import_directives(&logos, src).expect("consume");
        assert_eq!(got.len(), logos.imports.len());
        assert_eq!(
            got.iter()
                .map(|d| (
                    d.spec.as_str(),
                    d.alias.as_deref(),
                    d.reexport,
                    d.decl_order
                ))
                .collect::<Vec<_>>(),
            [("dep.sm", Some("D"), false, 0), ("re.sm", None, true, 1)]
        );

        let tampered = format!("{src}Import \"evil.sm\"\n    Import \"evil2.sm\"\n");
        let again = preserved_import_directives(&logos, &tampered).expect("consume");
        assert_eq!(
            key(&again),
            key(&got),
            "raw source text outside preserved nodes must not add imports"
        );
        assert!(again.iter().all(|d| !d.spec.starts_with("evil")));

        let no_imports = parse_logos_program("System A():\n").expect("parse");
        assert!(preserved_import_directives(&no_imports, &tampered)
            .unwrap()
            .is_empty());
    }

    // A preserved directive that carries no import spec fails closed in the
    // semantic owner instead of being silently skipped.
    #[test]
    fn unusable_preserved_directive_is_rejected_not_skipped() {
        let logos = parse_logos_program("Import\nSystem A():\n").expect("parse");
        assert_eq!(logos.imports.len(), 1, "accepted Import must be preserved");
        let err = preserved_import_directives(&logos, "Import\nSystem A():\n")
            .expect_err("empty directive");
        assert_eq!(err.diag.code, "E0239");
        assert!(
            err.to_string().contains("malformed Import directive"),
            "{err}"
        );
    }
}

#[cfg(test)]
mod pb03_semantic_core_tests {
    use super::*;
    use crate::alloc_core::{is_compatible_cmp, TypeId, TypeRegistry, TypeRegistryError};
    use sm_front::parse_logos_program;

    fn analyze(src: &str) -> Result<SemanticReport, SemanticError> {
        analyze_logos_program(&parse_logos_program(src).expect("parse"), src)
    }

    fn codes(src: &str) -> Vec<&'static str> {
        analyze(src)
            .expect("analyze")
            .warnings
            .iter()
            .map(|w| w.code)
            .collect()
    }

    fn rejected(src: &str) -> String {
        analyze(src).expect_err(src).to_string()
    }

    /// Parse may reject some malformed text itself; either way it must never
    /// be semantically admitted.
    fn never_admitted(src: &str) -> bool {
        match parse_logos_program(src) {
            Ok(p) => analyze_logos_program(&p, src).is_err(),
            Err(_) => true,
        }
    }

    const QUAD_SENSOR: &str = "Entity Sensor:\n    state val: quad\n";

    fn law(cond: &str) -> String {
        format!("{QUAD_SENSOR}Law \"L\":\n    When {cond} -> System.recovery()\n")
    }

    // #1671
    #[test]
    fn quad_field_is_quad_not_qvec1() {
        assert_eq!(SemanticType::from(Type::Quad), SemanticType::Quad);
        assert_ne!(SemanticType::from(Type::Quad), SemanticType::QVec(1));
        assert_eq!(SemanticType::from(Type::QVec(1)), SemanticType::QVec(1));
        for q in ["N", "F", "T", "S"] {
            analyze(&law(&format!("Sensor.val == {q}")))
                .unwrap_or_else(|e| panic!("Sensor.val == {q}: {e}"));
        }
        let qvec = "Entity Sensor:\n    state val: qvec[1]\nLaw \"L\":\n    When Sensor.val == T -> System.recovery()\n";
        assert!(rejected(qvec).contains("Mismatched types"));
    }

    // #1672
    #[test]
    fn source_numeric_families_stay_distinct() {
        for (src_ty, sem) in [
            (Type::I32, SemanticType::I32),
            (Type::U32, SemanticType::U32),
            (Type::F64, SemanticType::F64),
            (Type::Fx, SemanticType::Fx),
            (Type::Bool, SemanticType::Bool),
            (Type::Unit, SemanticType::Unit),
        ] {
            assert_eq!(SemanticType::from(src_ty), sem);
        }
        let fields =
            "Entity E:\n    state a: i32\n    state b: u32\n    state c: f64\n    state d: fx\n";
        for (l, r, ok) in [
            ("E.a", "E.a", true),
            ("E.b", "E.b", true),
            ("E.c", "E.c", true),
            ("E.d", "E.d", true),
            ("E.a", "E.b", false),
            ("E.c", "E.d", false),
            ("E.a", "E.d", false),
            ("E.a", "5", true),
            ("E.b", "5", false),
            ("E.b", "5u32", true),
            ("E.d", "1.5fx", true),
            ("E.d", "1.5", false),
        ] {
            let src = format!("{fields}Law \"L\":\n    When {l} == {r} -> System.recovery()\n");
            assert_eq!(analyze(&src).is_ok(), ok, "{l} == {r}");
        }
    }

    // #1673
    #[test]
    fn unknown_never_proves_compatibility() {
        assert!(!is_compatible_cmp(
            SemanticType::Unknown,
            SemanticType::Unknown
        ));
        assert!(!is_compatible_cmp(
            SemanticType::Unknown,
            SemanticType::Quad
        ));
        assert!(!is_compatible_cmp(
            SemanticType::Quad,
            SemanticType::Unknown
        ));
        for cond in [
            "MissingA == MissingB",
            "MissingA == T",
            "T == MissingB",
            "Sensor.nope == T",
            "Nope.val == T",
            "Sensor == T",
        ] {
            assert!(rejected(&law(cond)).contains("unresolved name"), "{cond}");
        }
    }

    // #1674
    #[test]
    fn evidence_operators_require_structure_and_resolved_quad_operands() {
        let ok = format!(
            "{QUAD_SENSOR}Entity B:\n    state v: quad\nLaw \"L\":\n    When Sensor.val && B.v -> System.recovery()\n    When T || F || S -> System.recovery()\n    When !Sensor.val -> System.recovery()\n"
        );
        analyze(&ok).expect("structured evidence ops");
        for cond in [
            "nope && junk",
            "T && \"a\"",
            "T && Sensor.missing",
            "T && F || S",
            "T &&",
            "\"a && b\"",
            "\"x | y\"",
            "T && true",
        ] {
            assert!(never_admitted(&law(cond)), "{cond}");
        }
    }

    // #1675
    #[test]
    fn present_requires_structural_resolved_target() {
        analyze(&law("Present(Sensor.val)")).expect("Present(field)");
        for cond in [
            "Present(Sensor.nope)",
            "Present(Unknown)",
            "Present(\"Sensor.val\")",
            "Present(T)",
            "\"Present(x)\"",
            "Present(",
            "fooPresent(x)",
            "Present(Sensor.val) == T",
        ] {
            assert!(never_admitted(&law(cond)), "{cond}");
        }
    }

    // #1676
    #[test]
    fn malformed_numeric_text_never_gets_a_numeric_type() {
        for num in [".", "1..2", "1.2.3", "1.2.3fx"] {
            let src = format!(
                "Entity E:\n    state d: fx\nLaw \"L\":\n    When E.d == {num} -> System.recovery()\n"
            );
            assert!(never_admitted(&src), "{num}");
        }
    }

    // #1677
    #[test]
    fn quad_n_is_not_always_false() {
        for (cond, dead) in [
            ("N", false),
            ("F", false),
            ("T", false),
            ("S", false),
            ("N == N", false),
            ("true", false),
            ("false", true),
            ("T == F", true),
            ("T != T", true),
        ] {
            let src = format!("Law \"L\":\n    When {cond} -> System.recovery()\n");
            assert_eq!(codes(&src).contains(&"W0240"), dead, "When {cond}");
        }
    }

    // #1679
    #[test]
    fn field_usage_counts_only_parsed_resolved_references() {
        assert!(!codes(&law("Sensor.val == T")).contains(&"W0252"));
        for effect in ["Log.emit(\"Sensor.val\")", "Log.emit(\"Sensor . val\")"] {
            let src = format!("{QUAD_SENSOR}Law \"L\":\n    When true -> {effect}\n");
            assert!(
                codes(&src).contains(&"W0252"),
                "{effect} must not count as use"
            );
        }
    }

    // #1680
    #[test]
    fn magic_number_lint_sees_only_numeric_literals() {
        let real =
            "Entity E:\n    state a: i32\nLaw \"L\":\n    When E.a == 42 -> System.recovery()\n";
        assert!(codes(real).contains(&"W0253"));
        for effect in [
            "Log.emit(\"42\")",
            "Log.emit(\"version2\")",
            "Sensor2.go()",
            "identifier_123()",
        ] {
            let src = format!("Law \"L\":\n    When true -> {effect}\n");
            assert!(!codes(&src).contains(&"W0253"), "{effect}");
        }
        let exempt =
            "Entity E:\n    state a: i32\nLaw \"L\":\n    When E.a == 1 -> System.recovery()\n";
        assert!(!codes(exempt).contains(&"W0253"));
    }

    // #1693 / #1694
    #[test]
    fn law_names_are_module_level_and_never_owner_inferred() {
        let dup = "Entity A:\n    state v: bool\nEntity B:\n    state v: bool\nLaw \"Check\":\n    When A.v == true -> System.recovery()\nLaw \"Check\":\n    When B.v == true -> System.recovery()\n";
        assert!(rejected(dup).contains("duplicate Law 'Check' in module"));
        for src in [
            "Entity A:\n    state v: bool\nLaw \"L\":\n    When A.v == true -> System.recovery()\n    When v == true -> System.recovery()\n",
            "Entity A:\n    state v: bool\nLaw \"L\":\n    When v == true -> System.recovery()\n    When A.v == true -> System.recovery()\n",
        ] {
            assert!(rejected(src).contains("unresolved name 'v'"), "{src}");
        }
        let unknown_first = "Law \"L\":\n    When Ghost.v == true -> System.recovery()\n";
        assert!(rejected(unknown_first).contains("unresolved name 'Ghost.v'"));
    }

    // #1695: public analyze_logos_program input with colliding fields.
    #[test]
    fn hand_built_duplicate_entity_field_is_rejected() {
        let src =
            "Entity A:\n    state v: bool\nLaw \"L\":\n    When A.v == true -> System.recovery()\n";
        let mut program = parse_logos_program(src).expect("parse");
        let mut dup = program.entities[0].fields[0].clone();
        dup.ty = Type::Quad;
        program.entities[0].fields.push(dup);
        let err = analyze_logos_program(&program, src).expect_err("collision");
        assert!(err.to_string().contains("duplicate field 'A.v'"), "{err}");
    }

    // #1696
    #[test]
    fn type_registry_capacity_is_enforced_without_aliasing() {
        let mut reg = TypeRegistry::new();
        let first = reg.intern(SemanticType::Bool).expect("first");
        for n in 0..TypeRegistry::CAPACITY - 1 {
            reg.intern(SemanticType::QVec(n)).expect("within capacity");
        }
        assert_eq!(reg.len(), TypeRegistry::CAPACITY);
        let last_ty = SemanticType::QVec(TypeRegistry::CAPACITY - 2);
        let last = reg.intern(last_ty).expect("existing");
        assert_eq!(last, TypeId(u16::MAX));
        assert_eq!(
            reg.intern(SemanticType::Quad),
            Err(TypeRegistryError::CapacityExhausted)
        );
        assert_eq!(
            reg.len(),
            TypeRegistry::CAPACITY,
            "failed intern changes nothing"
        );
        assert_eq!(reg.get(first), Some(SemanticType::Bool));
        assert_eq!(reg.intern(SemanticType::Bool), Ok(first));
        assert_eq!(reg.get(last), Some(last_ty));
    }

    // #1705: registry identity is not admission authority.
    #[test]
    fn analysis_does_not_consult_type_registry() {
        let src = include_str!("std_adapters.rs");
        let start = src.find("pub fn analyze_logos_program(").unwrap();
        let end = start + src[start..].find("\n}").unwrap();
        let body = &src[start..end];
        assert!(!body.contains("TypeRegistry") && !body.contains("equals_fast"));
    }
}
