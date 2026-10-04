use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use sm_front::{LogosAtom, LogosCompareOp, LogosCondition, NumericLiteral};
use ton618_core::SourceMark;

/// PB-03 (#1671, #1672): the Logos semantic type model. Source families stay
/// distinct: `quad` is never `QVec(1)`, `i32`/`u32` and `f64`/`fx` never
/// collapse. `Unknown` marks unresolved state and never proves anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SemanticType {
    I32,
    U32,
    F64,
    Fx,
    QVec(usize),
    Mask,
    Str,
    Bool,
    Quad,
    Unit,
    Unknown,
}

impl SemanticType {
    pub const fn name(self) -> &'static str {
        match self {
            SemanticType::I32 => "I32",
            SemanticType::U32 => "U32",
            SemanticType::F64 => "F64",
            SemanticType::Fx => "Fx",
            SemanticType::QVec(_) => "QVec",
            SemanticType::Mask => "Mask",
            SemanticType::Str => "Str",
            SemanticType::Bool => "Bool",
            SemanticType::Quad => "Quad",
            SemanticType::Unit => "Unit",
            SemanticType::Unknown => "Unknown",
        }
    }
}

impl core::fmt::Display for SemanticType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeId(pub u16);

/// FA-03-027 / #1696: interning more distinct types than `TypeId` can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeRegistryError {
    CapacityExhausted,
}

/// Canonical storage/debug identity for semantic types. PB-03 (#1705): its
/// IDs are not a semantic admission authority; nothing admits or rejects a
/// program by comparing them.
#[derive(Debug, Clone, Default)]
pub struct TypeRegistry {
    by_id: Vec<SemanticType>,
    ids: BTreeMap<SemanticType, TypeId>,
}

impl TypeRegistry {
    /// Distinct types a registry can hold: every `TypeId(u16)` value.
    pub const CAPACITY: usize = u16::MAX as usize + 1;

    pub fn new() -> Self {
        Self::default()
    }

    /// Same type → same ID. A new type past [`Self::CAPACITY`] fails without
    /// changing the registry (no wrapped/aliased ID).
    pub fn intern(&mut self, ty: SemanticType) -> Result<TypeId, TypeRegistryError> {
        if let Some(id) = self.ids.get(&ty) {
            return Ok(*id);
        }
        let raw =
            u16::try_from(self.by_id.len()).map_err(|_| TypeRegistryError::CapacityExhausted)?;
        let id = TypeId(raw);
        self.by_id.push(ty);
        self.ids.insert(ty, id);
        Ok(id)
    }

    pub fn get(&self, id: TypeId) -> Option<SemanticType> {
        self.by_id.get(id.0 as usize).copied()
    }

    pub fn equals_fast(&self, a: TypeId, b: TypeId) -> bool {
        a == b
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn pretty(&self, id: TypeId) -> &'static str {
        self.get(id).unwrap_or(SemanticType::Unknown).name()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    Global,
    Module,
    Entity,
    Law,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub ty: SemanticType,
    pub scope: ScopeKind,
}

#[derive(Debug, Clone)]
struct Scope {
    kind: ScopeKind,
    symbols: BTreeMap<String, Symbol>,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolError {
    pub message: String,
}

impl core::fmt::Display for SymbolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope {
                kind: ScopeKind::Global,
                symbols: BTreeMap::new(),
            }],
        }
    }

    pub fn push(&mut self, kind: ScopeKind) {
        self.scopes.push(Scope {
            kind,
            symbols: BTreeMap::new(),
        });
    }

    pub fn pop(&mut self) {
        if self.scopes.len() > 1 {
            let _ = self.scopes.pop();
        }
    }

    pub fn insert(&mut self, sym: Symbol) -> Result<(), SymbolError> {
        let cur = self.scopes.last_mut().expect("scope stack is never empty");
        if cur.symbols.contains_key(sym.name.as_str()) {
            return Err(SymbolError {
                message: format!("duplicate symbol '{}'", sym.name),
            });
        }
        cur.symbols.insert(sym.name.clone(), sym);
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(s) = scope.symbols.get(name) {
                return Some(s);
            }
        }
        None
    }

    pub fn scope_kind(&self) -> ScopeKind {
        self.scopes
            .last()
            .map(|s| s.kind)
            .unwrap_or(ScopeKind::Global)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ModuleProvider {
    fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String>;
    fn resolve_import(&self, importer_module_id: &str, spec: &str) -> Result<String, String>;
    /// SSF-09 #1580: how diagnostic text names the module `module_id`.
    ///
    /// The provider owns module ids and therefore their presentation. The
    /// default keeps the id itself (legacy behaviour). A provider whose ids
    /// are host paths may return a checkout-independent, project-scoped
    /// name instead, so diagnostics never embed the host's absolute paths.
    /// Must be injective over the ids of one check.
    fn display_module(&self, module_id: &str) -> String {
        String::from(module_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateInstr {
    GateRead { device_id: u16, port: u16 },
    GateWrite { device_id: u16, port: u16 },
    PulseEmit { signal: String },
}

#[derive(Debug, Clone)]
pub struct ImmutableIr<T>(Vec<T>);

impl<T> ImmutableIr<T> {
    pub fn from_vec(v: Vec<T>) -> Self {
        Self(v)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
}

pub struct LawScheduler;

impl LawScheduler {
    pub fn schedule_by_priority_desc<T, F>(items: &[T], mut priority: F) -> Vec<T>
    where
        T: Clone,
        F: FnMut(&T) -> u32,
    {
        let mut out = items.to_vec();
        out.sort_by(|a, b| priority(b).cmp(&priority(a)));
        out
    }
}

/// PB-03 (#1672): assignment requires the exact same known family; there is
/// no implicit `i32`/`u32` -> `fx` (or any other) coercion.
pub fn is_assignment_compatible(dst: SemanticType, src: SemanticType) -> bool {
    dst != SemanticType::Unknown && dst == src
}

pub fn collect_duplicates<'a, I>(items: I) -> BTreeSet<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen = BTreeSet::new();
    let mut dup = BTreeSet::new();
    for it in items {
        let key = it.to_string();
        if !seen.insert(key.clone()) {
            dup.insert(key);
        }
    }
    dup
}

/// FA-03-008 / #1677: a When condition is dead only when it is proven
/// `bool`-false: the literal `false`, or a comparison of two literals of one
/// family whose result is false. Quad-valued conditions (`N`, `F`, `T`, `S`,
/// evidence operators) are never claimed dead: `N` is unknown, not false, and
/// no Logos contract defines quad-valued When firing.
pub fn is_dead_when_condition(condition: &LogosCondition) -> bool {
    match condition {
        LogosCondition::Atom(LogosAtom::Bool(false)) => true,
        LogosCondition::Compare { lhs, op, rhs } => {
            let equal = match (lhs, rhs) {
                (LogosAtom::Bool(a), LogosAtom::Bool(b)) => a == b,
                (LogosAtom::Quad(a), LogosAtom::Quad(b)) => a == b,
                _ => return false,
            };
            match op {
                LogosCompareOp::Eq => !equal,
                LogosCompareOp::Ne => equal,
            }
        }
        _ => false,
    }
}

pub fn parse_law_local_decl(effect: &str) -> Option<String> {
    let e = effect.trim();
    let mut words = e.split_whitespace();
    if words.next()? != "let" {
        return None;
    }
    let name = words.next()?.trim_end_matches(':').trim_end_matches(":=");
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

pub fn is_law_name_style_ok(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_uppercase() && !name.contains('_')
}

/// Why a structured When condition has no admitted semantic type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionInferError {
    MismatchedTypes {
        left: SemanticType,
        right: SemanticType,
    },
    /// A name or `Entity.field` that does not resolve to a typed value.
    Unresolved(String),
    /// Operands an evidence/`!` operator does not accept.
    InvalidOperands(SemanticType),
    /// `Present(...)` of something other than a resolvable name or field.
    InvalidPresent,
    /// Tokens `sm-front` could not project onto the modeled condition surface.
    Unsupported,
}

/// FA-03-004 / #1673: `Unknown` never proves compatibility, not even with
/// itself. Families are exact (#1672): no `i32`/`u32`, `f64`/`fx` or
/// `quad`/`qvec` cross-compatibility.
pub fn is_compatible_cmp(left: SemanticType, right: SemanticType) -> bool {
    left != SemanticType::Unknown && right != SemanticType::Unknown && left == right
}

fn numeric_literal_type(lit: &NumericLiteral) -> SemanticType {
    match lit {
        NumericLiteral::I32(_) => SemanticType::I32,
        NumericLiteral::U32(_) => SemanticType::U32,
        NumericLiteral::F64(_) => SemanticType::F64,
        NumericLiteral::Fx(_) => SemanticType::Fx,
    }
}

/// The semantic type of one structured atom. Names/fields must resolve to a
/// known type; otherwise the atom is `Unresolved` (never `Unknown`-as-valid).
pub fn infer_atom_type_core<FS, FF>(
    atom: &LogosAtom,
    resolve_symbol: FS,
    resolve_field: FF,
) -> Result<SemanticType, ConditionInferError>
where
    FS: Fn(&str) -> Option<SemanticType>,
    FF: Fn(&str, &str) -> Option<SemanticType>,
{
    let resolved = |ty: Option<SemanticType>, what: String| match ty {
        Some(ty) if ty != SemanticType::Unknown => Ok(ty),
        _ => Err(ConditionInferError::Unresolved(what)),
    };
    match atom {
        LogosAtom::Bool(_) => Ok(SemanticType::Bool),
        LogosAtom::Quad(_) => Ok(SemanticType::Quad),
        LogosAtom::Number(lit) => Ok(numeric_literal_type(lit)),
        LogosAtom::Text(_) => Ok(SemanticType::Str),
        LogosAtom::Name(name) => resolved(resolve_symbol(name), name.clone()),
        LogosAtom::Field { entity, field } => {
            resolved(resolve_field(entity, field), format!("{entity}.{field}"))
        }
    }
}

/// FA-03-005/006/007 (#1674-#1676): the semantic type of a structured When
/// condition. Every result is proven from resolved operands; nothing is
/// inferred from text.
pub fn infer_when_condition_type_core<FS, FF>(
    condition: &LogosCondition,
    resolve_symbol: FS,
    resolve_field: FF,
) -> Result<SemanticType, ConditionInferError>
where
    FS: Fn(&str) -> Option<SemanticType>,
    FF: Fn(&str, &str) -> Option<SemanticType>,
{
    let atom = |a: &LogosAtom| infer_atom_type_core(a, &resolve_symbol, &resolve_field);
    match condition {
        LogosCondition::Atom(a) => atom(a),
        LogosCondition::Compare { lhs, rhs, .. } => {
            let (lt, rt) = (atom(lhs)?, atom(rhs)?);
            if !is_compatible_cmp(lt, rt) {
                return Err(ConditionInferError::MismatchedTypes {
                    left: lt,
                    right: rt,
                });
            }
            Ok(SemanticType::Bool)
        }
        LogosCondition::Present(target) => match target {
            LogosAtom::Name(_) | LogosAtom::Field { .. } => {
                atom(target).map(|_| SemanticType::Bool)
            }
            _ => Err(ConditionInferError::InvalidPresent),
        },
        LogosCondition::Not(a) => match atom(a)? {
            ty @ (SemanticType::Quad | SemanticType::Bool) => Ok(ty),
            other => Err(ConditionInferError::InvalidOperands(other)),
        },
        LogosCondition::Evidence { operands, .. } => {
            let mut result = None;
            for operand in operands {
                let ty = atom(operand)?;
                match (result, ty) {
                    (None, SemanticType::Quad | SemanticType::Bool) => result = Some(ty),
                    (Some(prev), _) if prev == ty => {}
                    _ => return Err(ConditionInferError::InvalidOperands(ty)),
                }
            }
            result.ok_or(ConditionInferError::Unsupported)
        }
        LogosCondition::Unsupported => Err(ConditionInferError::Unsupported),
    }
}

/// FA-03-011 / #1680: W0253 applies to parsed numeric literals only; `0` and
/// `1` stay exempt (pre-existing policy).
pub fn is_magic_number_atom(atom: &LogosAtom) -> bool {
    let value = match atom {
        LogosAtom::Number(NumericLiteral::I32(v)) => f64::from(*v),
        LogosAtom::Number(NumericLiteral::U32(v)) => f64::from(*v),
        LogosAtom::Number(NumericLiteral::F64(v) | NumericLiteral::Fx(v)) => *v,
        _ => return false,
    };
    value != 0.0 && value != 1.0
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhenValidationError {
    pub code: &'static str,
    pub message: String,
}

pub fn validate_when_non_empty_core(
    condition: &str,
    effect: &str,
) -> Result<(), WhenValidationError> {
    if condition.trim().is_empty() {
        return Err(WhenValidationError {
            code: "E0224",
            message: "empty When condition".to_string(),
        });
    }
    if effect.trim().is_empty() {
        return Err(WhenValidationError {
            code: "E0225",
            message: "empty When body".to_string(),
        });
    }
    Ok(())
}

pub fn is_large_law_core(when_count: usize) -> bool {
    when_count > 16
}

pub fn diagnostic_help_core(code: &str) -> Option<&'static str> {
    match code {
        "E0101" => Some("Align indentation to an existing block level."),
        "E0201" => Some("Check type compatibility and explicit conversions."),
        "E0215" => Some("Entity fields must start with 'state' or 'prop'."),
        "E0223" => Some("Rename local declaration to avoid shadowing inside Law."),
        "E0238" => Some("Break cyclic imports by introducing an acyclic module boundary."),
        "E0239" => Some("Check import path, file extension, and module parse validity."),
        "E0240" => Some("Re-export policy violation."),
        "E0241" => Some("Rename one of the imports or use distinct 'as' aliases."),
        "E0242" => Some("Rename with 'as' or export symbols selectively to avoid collisions."),
        "E0243" => Some("Break re-export chain cycle by exporting local symbol directly."),
        "E0244" => Some("Check selected symbol name or export it in dependency module."),
        "E0245" => Some("Use unique aliases inside import select list."),
        "W0240" => Some("Remove or revise the branch whose When condition is always false."),
        "W0241" => {
            Some("Consider replacing the literal-only fx.* call with its precomputed constant.")
        }
        "W0250" => Some("Use UpperCamelCase names for laws to keep style consistent."),
        "W0251" => Some("Split large laws into smaller focused laws."),
        "W0252" => Some("Remove unused fields or reference them from at least one law."),
        "W0253" => Some("Replace literal with a named constant for maintainability."),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LawHeaderPolicy {
    pub non_idiomatic_name: bool,
    pub large_law: bool,
}

pub fn evaluate_law_header_policy_core(law_name: &str, when_count: usize) -> LawHeaderPolicy {
    LawHeaderPolicy {
        non_idiomatic_name: !is_law_name_style_ok(law_name),
        large_law: is_large_law_core(when_count),
    }
}

pub fn is_valid_when_result_type_core(ty: SemanticType) -> bool {
    matches!(ty, SemanticType::Bool | SemanticType::Quad)
}

pub fn insert_scoped_name_core(
    scopes: &mut BTreeMap<String, BTreeSet<String>>,
    scope: &str,
    name: &str,
) -> bool {
    scopes
        .entry(scope.to_string())
        .or_default()
        .insert(name.to_string())
}

pub fn insert_name_core(names: &mut BTreeSet<String>, name: &str) -> bool {
    names.insert(name.to_string())
}

#[derive(Debug, Clone)]
pub struct ImportDirective {
    pub spec: String,
    pub alias: Option<String>,
    pub reexport: bool,
    pub select_items: Vec<(String, Option<String>)>,
    pub wildcard: bool,
    pub line: u32,
    pub col: u32,
    pub decl_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportResolutionPlan {
    pub selected_bindings: Vec<String>,
    pub namespace_aliases: Vec<String>,
    pub wildcard_imports: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    System,
    Entity,
    Law,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportOrigin {
    Local { module: String },
    Imported { module: String, symbol: String },
    ReExport { chain: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportItem {
    pub public_name: String,
    pub kind: ExportKind,
    pub origin: ExportOrigin,
    pub span: SourceMark,
    pub decl_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExportSet {
    pub items: Vec<ExportItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExportDecl {
    pub public_name: String,
    pub kind: ExportKind,
    pub span: SourceMark,
}

pub fn collect_local_exports_core(module_key: &str, locals: &[LocalExportDecl]) -> ExportSet {
    let mut items = Vec::new();
    for (idx, local) in locals.iter().enumerate() {
        items.push(ExportItem {
            public_name: local.public_name.clone(),
            kind: local.kind,
            origin: ExportOrigin::Local {
                module: module_key.to_string(),
            },
            span: local.span,
            decl_order: idx as u32,
        });
    }
    ExportSet { items }
}

pub fn parse_select_items(inside: &str) -> Vec<(String, Option<String>)> {
    let mut out = Vec::new();
    for raw in inside.split(',') {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((src, dst)) = part.split_once(" as ") {
            let s = src.trim().trim_matches('"');
            let d = dst.trim().trim_matches('"');
            if !s.is_empty() && !d.is_empty() {
                out.push((s.to_string(), Some(d.to_string())));
            }
            continue;
        }
        let name = part.trim().trim_matches('"');
        if !name.is_empty() {
            out.push((name.to_string(), None));
        }
    }
    out
}

pub fn default_import_alias(spec: &str) -> String {
    let last = spec.rsplit(['/', '\\']).next().unwrap_or(spec);
    if let Some((stem, _)) = last.rsplit_once('.') {
        if !stem.is_empty() {
            return stem.to_string();
        }
    }
    last.to_string()
}

/// Line-scanning convenience over [`parse_import_directive`]. The module
/// loader does not use it: it consumes the `Import` directives `sm-front`
/// preserved in `LogosProgram::imports` (PB-02 / #1645).
pub fn parse_import_directives(source: &str) -> Vec<ImportDirective> {
    let mut out = Vec::new();
    let mut decl_order = 0u32;
    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        if !trimmed.starts_with("Import") {
            continue;
        }
        let ws = line.len().saturating_sub(trimmed.len());
        if let Some(directive) =
            parse_import_directive(trimmed, (idx + 1) as u32, (ws + 1) as u32, decl_order)
        {
            out.push(directive);
            decl_order += 1;
        }
    }
    out
}

/// The one Logos import-directive parser: interprets a single directive whose
/// text starts at the `Import` keyword. `None` means the text carries no
/// usable import spec.
pub fn parse_import_directive(
    directive: &str,
    line: u32,
    col: u32,
    decl_order: u32,
) -> Option<ImportDirective> {
    let after_kw = directive.strip_prefix("Import")?;
    let mut rest = after_kw.trim();
    if rest.is_empty() {
        return None;
    }
    let mut reexport = false;
    if let Some(after_pub) = rest.strip_prefix("pub ") {
        reexport = true;
        rest = after_pub.trim_start();
    }

    let spec = if let Some(stripped) = rest.strip_prefix('"') {
        if let Some(end) = stripped.find('"') {
            stripped[..end].to_string()
        } else {
            stripped.to_string()
        }
    } else {
        rest.split_whitespace().next().unwrap_or("").to_string()
    };
    if spec.is_empty() {
        return None;
    }

    let mut alias = None;
    let mut tail = "";
    if let Some(stripped) = rest.strip_prefix('"') {
        if let Some(end) = stripped.find('"') {
            tail = stripped[end + 1..].trim_start();
        }
    } else if let Some(pos) = rest.find(char::is_whitespace) {
        tail = rest[pos..].trim_start();
    }
    let mut wildcard = false;
    let mut select_items: Vec<(String, Option<String>)> = Vec::new();
    if let Some(after_as) = tail.strip_prefix("as ") {
        let mut split = after_as.splitn(2, char::is_whitespace);
        let head = split.next().unwrap_or("").trim_matches('"');
        if !head.is_empty() {
            alias = Some(head.to_string());
        }
        tail = split.next().unwrap_or("").trim_start();
    }
    if let Some(after_star) = tail.strip_prefix('*') {
        wildcard = true;
        tail = after_star.trim_start();
    }
    if let Some(after_lbrace) = tail.strip_prefix('{') {
        if let Some(end) = after_lbrace.find('}') {
            let inside = &after_lbrace[..end];
            select_items = parse_select_items(inside);
        }
    }

    Some(ImportDirective {
        spec,
        alias,
        reexport,
        select_items,
        wildcard,
        line,
        col,
        decl_order,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPolicyError {
    pub code: &'static str,
    pub message: String,
    pub line: u32,
    pub col: u32,
}

pub fn validate_import_namespace_rules(
    imports: &[ImportDirective],
) -> Result<(), ImportPolicyError> {
    let mut aliases = BTreeSet::<String>::new();
    for import in imports {
        let alias = import
            .alias
            .clone()
            .unwrap_or_else(|| default_import_alias(&import.spec));
        if !aliases.insert(alias.clone()) {
            return Err(ImportPolicyError {
                code: "E0241",
                message: format!("duplicate import namespace alias '{}' in one module", alias),
                line: import.line,
                col: import.col,
            });
        }

        let mut seen_select_alias = BTreeSet::<String>::new();
        for (src, dst) in &import.select_items {
            let local = dst.clone().unwrap_or_else(|| src.clone());
            if !seen_select_alias.insert(local.clone()) {
                return Err(ImportPolicyError {
                    code: "E0245",
                    message: format!(
                        "duplicate selected import alias '{}' in one Import statement",
                        local
                    ),
                    line: import.line,
                    col: import.col,
                });
            }
        }

        if import.wildcard && !import.select_items.is_empty() {
            return Err(ImportPolicyError {
                code: "E0245",
                message: "cannot combine wildcard import '*' with explicit select list".to_string(),
                line: import.line,
                col: import.col,
            });
        }
    }
    Ok(())
}

pub fn validate_import_bindings_core(
    imports: &[ImportDirective],
    local_names: &BTreeSet<String>,
) -> Result<(), ImportPolicyError> {
    validate_import_namespace_rules(imports)?;

    let mut bound = BTreeSet::<String>::new();
    for import in imports {
        let alias = import
            .alias
            .clone()
            .unwrap_or_else(|| default_import_alias(&import.spec));
        if local_names.contains(&alias) {
            return Err(ImportPolicyError {
                code: "E0241",
                message: format!("import alias '{}' conflicts with local symbol", alias),
                line: import.line,
                col: import.col,
            });
        }
        if !bound.insert(alias.clone()) {
            return Err(ImportPolicyError {
                code: "E0241",
                message: format!("duplicate import binding alias '{}'", alias),
                line: import.line,
                col: import.col,
            });
        }
        for (src, dst) in &import.select_items {
            let local = dst.clone().unwrap_or_else(|| src.clone());
            if local_names.contains(&local) {
                return Err(ImportPolicyError {
                    code: "E0241",
                    message: format!("import alias '{}' conflicts with local symbol", local),
                    line: import.line,
                    col: import.col,
                });
            }
            if !bound.insert(local.clone()) {
                return Err(ImportPolicyError {
                    code: "E0241",
                    message: format!("duplicate import binding alias '{}'", local),
                    line: import.line,
                    col: import.col,
                });
            }
        }
    }
    Ok(())
}

pub fn build_import_resolution_plan_core(imports: &[ImportDirective]) -> ImportResolutionPlan {
    let mut ordered: Vec<&ImportDirective> = imports.iter().collect();
    ordered.sort_by_key(|import| import.decl_order);

    let mut plan = ImportResolutionPlan::default();
    for import in &ordered {
        for (src, dst) in &import.select_items {
            let (_, base_name) = parse_select_expected_kind(src);
            plan.selected_bindings
                .push(dst.clone().unwrap_or_else(|| base_name.to_string()));
        }
    }
    for import in &ordered {
        plan.namespace_aliases.push(
            import
                .alias
                .clone()
                .unwrap_or_else(|| default_import_alias(&import.spec)),
        );
        if import.wildcard {
            plan.wildcard_imports.push(import.spec.clone());
        }
    }
    plan
}

#[derive(Debug, Clone)]
pub struct SelectImportModule {
    pub module_key: String,
    pub source: String,
    pub imports: Vec<ImportDirective>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectImportPolicyError {
    pub code: &'static str,
    pub message: String,
    pub module_key: String,
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone)]
pub struct ExportBuildModule {
    pub module_key: String,
    pub source: String,
    pub local_exports: ExportSet,
    pub imports: Vec<ImportDirective>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportBuildError {
    pub code: &'static str,
    pub message: String,
    pub module_key: String,
    pub line: u32,
    pub col: u32,
}

pub fn validate_select_imports_core(
    modules: &[SelectImportModule],
    dep_lookup: &BTreeMap<(String, String), String>,
    export_symbols: &BTreeMap<String, BTreeSet<String>>,
    export_kinds: &BTreeMap<String, BTreeMap<String, ExportKind>>,
) -> Result<(), SelectImportPolicyError> {
    for m in modules {
        for import in &m.imports {
            if import.select_items.is_empty() {
                continue;
            }
            let dep_key = dep_lookup
                .get(&(m.module_key.clone(), import.spec.clone()))
                .ok_or_else(|| SelectImportPolicyError {
                    code: "E0239",
                    message: format!("import module not loaded for select '{}'", import.spec),
                    module_key: m.module_key.clone(),
                    line: import.line,
                    col: import.col,
                })?;
            let symbols = export_symbols
                .get(dep_key)
                .ok_or_else(|| SelectImportPolicyError {
                    code: "E0239",
                    message: format!("import module not loaded for select '{}'", import.spec),
                    module_key: m.module_key.clone(),
                    line: import.line,
                    col: import.col,
                })?;
            let kinds = export_kinds.get(dep_key);
            for (sym, _) in &import.select_items {
                let (expected_kind, base_name) = parse_select_expected_kind(sym);
                if !symbols.contains(base_name) {
                    return Err(SelectImportPolicyError {
                        code: "E0244",
                        message: format!(
                            "selected import symbol '{}' not found in '{}'",
                            base_name, dep_key
                        ),
                        module_key: m.module_key.clone(),
                        line: import.line,
                        col: import.col,
                    });
                }
                if let Some(expected) = expected_kind {
                    if let Some(actual) = kinds.and_then(|k| k.get(base_name)).copied() {
                        if actual != expected {
                            return Err(SelectImportPolicyError {
                                code: "E0245",
                                message: format!(
                                    "selected import symbol '{}' kind mismatch: expected {:?}, found {:?}",
                                    base_name, expected, actual
                                ),
                                module_key: m.module_key.clone(),
                                line: import.line,
                                col: import.col,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_select_expected_kind(sym: &str) -> (Option<ExportKind>, &str) {
    if let Some((lhs, rhs)) = sym.split_once(':') {
        let kind = match lhs.trim() {
            "System" => Some(ExportKind::System),
            "Entity" => Some(ExportKind::Entity),
            "Law" => Some(ExportKind::Law),
            _ => None,
        };
        return (kind, rhs.trim());
    }
    (None, sym)
}

pub fn build_export_sets_core(
    modules: &[ExportBuildModule],
    dep_lookup: &BTreeMap<(String, String), String>,
) -> Result<BTreeMap<String, ExportSet>, ExportBuildError> {
    let mut by_key = BTreeMap::<String, ExportBuildModule>::new();
    for m in modules {
        by_key.insert(m.module_key.clone(), m.clone());
    }
    let mut cache = BTreeMap::<String, ExportSet>::new();
    let mut stack = Vec::<String>::new();
    let keys: Vec<String> = by_key.keys().cloned().collect();
    for key in keys {
        build_export_set_for_core(&key, &by_key, dep_lookup, &mut cache, &mut stack)?;
    }
    Ok(cache)
}

fn build_export_set_for_core(
    module_key: &str,
    modules: &BTreeMap<String, ExportBuildModule>,
    dep_lookup: &BTreeMap<(String, String), String>,
    cache: &mut BTreeMap<String, ExportSet>,
    stack: &mut Vec<String>,
) -> Result<(), ExportBuildError> {
    if cache.contains_key(module_key) {
        return Ok(());
    }
    if let Some(pos) = stack.iter().position(|m| m == module_key) {
        let mut chain_parts: Vec<String> = stack[pos..].to_vec();
        chain_parts.push(module_key.to_string());
        return Err(ExportBuildError {
            code: "E0243",
            message: format!(
                "symbol re-export cycle detected: {}",
                chain_parts.join(" -> ")
            ),
            module_key: module_key.to_string(),
            line: 0,
            col: 0,
        });
    }
    let module = modules.get(module_key).ok_or_else(|| ExportBuildError {
        code: "E0239",
        message: format!("unknown module '{}'", module_key),
        module_key: module_key.to_string(),
        line: 0,
        col: 0,
    })?;

    stack.push(module_key.to_string());
    let mut set = module.local_exports.clone();
    let mut next_decl = set.items.len() as u32;
    for import in &module.imports {
        if !import.reexport {
            continue;
        }
        let dep = dep_lookup
            .get(&(module_key.to_string(), import.spec.clone()))
            .cloned()
            .ok_or_else(|| ExportBuildError {
                code: "E0239",
                message: format!("unknown module '{}'", import.spec),
                module_key: module_key.to_string(),
                line: import.line,
                col: import.col,
            })?;
        build_export_set_for_core(&dep, modules, dep_lookup, cache, stack)?;
        let dep_set = cache.get(&dep).cloned().unwrap_or_default();
        let selected: Vec<ExportItem> = if import.wildcard || import.select_items.is_empty() {
            dep_set.items
        } else {
            let mut out = Vec::new();
            for (src_name, alias) in &import.select_items {
                if let Some(found) = dep_set.items.iter().find(|it| it.public_name == *src_name) {
                    let mut f = found.clone();
                    if let Some(a) = alias {
                        f.public_name = a.clone();
                    }
                    out.push(f);
                }
            }
            out
        };
        for item in selected {
            push_export_item_core(
                &mut set,
                ExportItem {
                    public_name: item.public_name.clone(),
                    kind: item.kind,
                    origin: ExportOrigin::ReExport {
                        chain: vec![module_key.to_string(), dep.clone(), item.public_name],
                    },
                    span: SourceMark {
                        line: import.line,
                        col: import.col,
                        file_id: 0,
                    },
                    decl_order: next_decl.max(item.decl_order + import.decl_order + 1),
                },
                module_key,
                import.line,
                import.col,
            )?;
            next_decl += 1;
        }
    }

    set.items.sort_by(|a, b| a.decl_order.cmp(&b.decl_order));
    let _ = stack.pop();
    cache.insert(module_key.to_string(), set);
    Ok(())
}

fn push_export_item_core(
    set: &mut ExportSet,
    item: ExportItem,
    module_key: &str,
    line: u32,
    col: u32,
) -> Result<(), ExportBuildError> {
    if let Some(prev) = set
        .items
        .iter()
        .find(|x| x.public_name == item.public_name && x.kind == item.kind)
    {
        return Err(ExportBuildError {
            code: "E0242",
            message: format!(
                "re-export collision for '{}' between {:?} and {:?}",
                item.public_name, prev.origin, item.origin
            ),
            module_key: module_key.to_string(),
            line,
            col,
        });
    }
    set.items.push(item);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_registry_roundtrip() {
        let mut reg = TypeRegistry::new();
        let a = reg.intern(SemanticType::I32).expect("intern");
        let b = reg.intern(SemanticType::I32).expect("intern");
        assert_eq!(a, b);
        assert_eq!(reg.get(a), Some(SemanticType::I32));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn symbol_table_resolve_scoped() {
        let mut st = SymbolTable::new();
        st.insert(Symbol {
            name: "x".to_string(),
            ty: SemanticType::I32,
            scope: ScopeKind::Global,
        })
        .expect("insert");
        st.push(ScopeKind::Law);
        st.insert(Symbol {
            name: "y".to_string(),
            ty: SemanticType::Fx,
            scope: ScopeKind::Law,
        })
        .expect("insert");
        assert_eq!(st.resolve("x").map(|s| s.ty), Some(SemanticType::I32));
        assert_eq!(st.resolve("y").map(|s| s.ty), Some(SemanticType::Fx));
        st.pop();
        assert!(st.resolve("y").is_none());
    }

    #[test]
    fn import_policy_rejects_duplicate_select_alias() {
        let src = r#"
Import "dep.sm" { A as X, B as X }
"#;
        let imports = parse_import_directives(src);
        let err = validate_import_namespace_rules(&imports).expect_err("must fail");
        assert_eq!(err.code, "E0245");
    }

    #[test]
    fn import_bindings_reject_local_alias_collision() {
        let src = r#"
Import "dep.sm" as Foo
"#;
        let imports = parse_import_directives(src);
        let mut locals = BTreeSet::new();
        locals.insert("Foo".to_string());
        let err = validate_import_bindings_core(&imports, &locals).expect_err("must fail");
        assert_eq!(err.code, "E0241");
    }

    #[test]
    fn select_import_core_reports_missing_symbol() {
        let modules = vec![SelectImportModule {
            module_key: "root.sm".to_string(),
            source: "Import \"dep.sm\" { Missing }".to_string(),
            imports: parse_import_directives("Import \"dep.sm\" { Missing }"),
        }];
        let mut dep_lookup = BTreeMap::new();
        dep_lookup.insert(
            ("root.sm".to_string(), "dep.sm".to_string()),
            "dep.sm".to_string(),
        );
        let mut export_symbols = BTreeMap::new();
        let mut syms = BTreeSet::new();
        syms.insert("Present".to_string());
        export_symbols.insert("dep.sm".to_string(), syms);
        let mut export_kinds = BTreeMap::new();
        let mut ks = BTreeMap::new();
        ks.insert("Present".to_string(), ExportKind::Law);
        export_kinds.insert("dep.sm".to_string(), ks);
        let err =
            validate_select_imports_core(&modules, &dep_lookup, &export_symbols, &export_kinds)
                .expect_err("must fail");
        assert_eq!(err.code, "E0244");
    }

    #[test]
    fn select_import_kind_mismatch_reports_e0245() {
        let modules = vec![SelectImportModule {
            module_key: "root.sm".to_string(),
            source: "Import \"dep.sm\" { Entity:A }".to_string(),
            imports: parse_import_directives("Import \"dep.sm\" { Entity:A }"),
        }];
        let mut dep_lookup = BTreeMap::new();
        dep_lookup.insert(
            ("root.sm".to_string(), "dep.sm".to_string()),
            "dep.sm".to_string(),
        );
        let mut export_symbols = BTreeMap::new();
        let mut syms = BTreeSet::new();
        syms.insert("A".to_string());
        export_symbols.insert("dep.sm".to_string(), syms);
        let mut export_kinds = BTreeMap::new();
        let mut kinds = BTreeMap::new();
        kinds.insert("A".to_string(), ExportKind::Law);
        export_kinds.insert("dep.sm".to_string(), kinds);
        let err =
            validate_select_imports_core(&modules, &dep_lookup, &export_symbols, &export_kinds)
                .expect_err("must fail");
        assert_eq!(err.code, "E0245");
    }

    #[test]
    fn import_resolution_plan_keeps_selected_before_namespace_and_wildcard() {
        let src = r#"
Import "dep/wild.sm" *
Import "dep/select.sm" { Entity:Sensor, Law:Check as Guard }
Import "dep/core.sm" as Core
"#;
        let plan = build_import_resolution_plan_core(&parse_import_directives(src));
        assert_eq!(plan.selected_bindings, vec!["Sensor", "Guard"]);
        assert_eq!(plan.namespace_aliases, vec!["wild", "select", "Core"]);
        assert_eq!(plan.wildcard_imports, vec!["dep/wild.sm"]);
    }

    #[test]
    fn import_resolution_plan_keeps_wildcard_fallbacks_in_declaration_order() {
        let src = r#"
Import "dep/first.sm" *
Import "dep/second.sm" *
Import "dep/third.sm" *
"#;
        let plan = build_import_resolution_plan_core(&parse_import_directives(src));
        assert_eq!(
            plan.wildcard_imports,
            vec!["dep/first.sm", "dep/second.sm", "dep/third.sm"]
        );
    }

    #[test]
    fn export_set_keeps_locals_before_reexports_in_import_order() {
        let modules = vec![
            ExportBuildModule {
                module_key: "root.sm".to_string(),
                source: r#"
Import pub "b.sm"
Import pub "a.sm"
Law "Root" [priority 1]:
    When true -> System.recovery()
"#
                .to_string(),
                local_exports: collect_local_exports_core(
                    "root.sm",
                    &[LocalExportDecl {
                        public_name: "Root".to_string(),
                        kind: ExportKind::Law,
                        span: SourceMark::default(),
                    }],
                ),
                imports: parse_import_directives(
                    r#"
Import pub "b.sm"
Import pub "a.sm"
"#,
                ),
            },
            ExportBuildModule {
                module_key: "a.sm".to_string(),
                source: String::new(),
                local_exports: collect_local_exports_core(
                    "a.sm",
                    &[
                        LocalExportDecl {
                            public_name: "A1".to_string(),
                            kind: ExportKind::Law,
                            span: SourceMark::default(),
                        },
                        LocalExportDecl {
                            public_name: "A2".to_string(),
                            kind: ExportKind::Law,
                            span: SourceMark::default(),
                        },
                    ],
                ),
                imports: Vec::new(),
            },
            ExportBuildModule {
                module_key: "b.sm".to_string(),
                source: String::new(),
                local_exports: collect_local_exports_core(
                    "b.sm",
                    &[LocalExportDecl {
                        public_name: "B1".to_string(),
                        kind: ExportKind::Law,
                        span: SourceMark::default(),
                    }],
                ),
                imports: Vec::new(),
            },
        ];
        let mut dep_lookup = BTreeMap::new();
        dep_lookup.insert(
            ("root.sm".to_string(), "b.sm".to_string()),
            "b.sm".to_string(),
        );
        dep_lookup.insert(
            ("root.sm".to_string(), "a.sm".to_string()),
            "a.sm".to_string(),
        );

        let export_sets = build_export_sets_core(&modules, &dep_lookup).expect("export sets");
        let root = export_sets.get("root.sm").expect("root set");
        let names: Vec<&str> = root
            .items
            .iter()
            .map(|item| item.public_name.as_str())
            .collect();
        assert_eq!(names, vec!["Root", "B1", "A1", "A2"]);
    }

    #[test]
    fn name_style_policy() {
        assert!(is_law_name_style_ok("CheckSignal"));
        assert!(!is_law_name_style_ok("check_signal"));
    }

    #[test]
    fn parse_law_local_decl_smoke() {
        assert_eq!(
            parse_law_local_decl("let a := fx.add(1,2)"),
            Some("a".to_string())
        );
        assert_eq!(parse_law_local_decl("System.recovery()"), None);
    }

    #[test]
    fn dead_when_condition_smoke() {
        let field = LogosAtom::Field {
            entity: "Sensor".to_string(),
            field: "val".to_string(),
        };
        assert!(is_dead_when_condition(&LogosCondition::Atom(
            LogosAtom::Bool(false)
        )));
        assert!(is_dead_when_condition(&LogosCondition::Compare {
            lhs: LogosAtom::Quad(sm_front::QuadVal::T),
            op: LogosCompareOp::Eq,
            rhs: LogosAtom::Quad(sm_front::QuadVal::F),
        }));
        assert!(!is_dead_when_condition(&LogosCondition::Compare {
            lhs: field,
            op: LogosCompareOp::Eq,
            rhs: LogosAtom::Quad(sm_front::QuadVal::T),
        }));
    }

    #[test]
    fn infer_when_type_mismatch_reports_error() {
        let err = infer_when_condition_type_core(
            &LogosCondition::Compare {
                lhs: LogosAtom::Name("x".to_string()),
                op: LogosCompareOp::Eq,
                rhs: LogosAtom::Text("\"s\"".to_string()),
            },
            |name| (name == "x").then_some(SemanticType::I32),
            |_e, _f| None,
        )
        .expect_err("must fail");
        assert_eq!(
            err,
            ConditionInferError::MismatchedTypes {
                left: SemanticType::I32,
                right: SemanticType::Str
            }
        );
    }

    #[test]
    fn magic_number_detector_smoke() {
        assert!(is_magic_number_atom(&LogosAtom::Number(
            NumericLiteral::I32(2)
        )));
        assert!(!is_magic_number_atom(&LogosAtom::Number(
            NumericLiteral::I32(1)
        )));
        assert!(!is_magic_number_atom(&LogosAtom::Text("\"2\"".to_string())));
    }

    #[test]
    fn when_non_empty_validation_errors() {
        let e1 = validate_when_non_empty_core("", "x").expect_err("must fail");
        assert_eq!(e1.code, "E0224");
        let e2 = validate_when_non_empty_core("x", "   ").expect_err("must fail");
        assert_eq!(e2.code, "E0225");
    }

    #[test]
    fn large_law_policy_threshold() {
        assert!(!is_large_law_core(16));
        assert!(is_large_law_core(17));
    }

    #[test]
    fn diagnostic_help_known_code() {
        assert!(diagnostic_help_core("E0242").is_some());
        assert!(diagnostic_help_core("UNKNOWN").is_none());
    }

    #[test]
    fn law_header_policy_flags() {
        let p = evaluate_law_header_policy_core("check_signal", 17);
        assert!(p.non_idiomatic_name);
        assert!(p.large_law);
    }

    #[test]
    fn when_result_type_policy() {
        assert!(is_valid_when_result_type_core(SemanticType::Bool));
        assert!(is_valid_when_result_type_core(SemanticType::Quad));
        assert!(!is_valid_when_result_type_core(SemanticType::Fx));
    }

    #[test]
    fn scoped_insert_policy() {
        let mut scopes = BTreeMap::<String, BTreeSet<String>>::new();
        assert!(insert_scoped_name_core(&mut scopes, "Sensor", "LawA"));
        assert!(!insert_scoped_name_core(&mut scopes, "Sensor", "LawA"));
        let mut names = BTreeSet::<String>::new();
        assert!(insert_name_core(&mut names, "x"));
        assert!(!insert_name_core(&mut names, "x"));
    }
}
