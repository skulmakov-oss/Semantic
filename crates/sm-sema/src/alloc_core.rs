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
    FF: Fn(Option<&str>, &str, &str) -> Option<SemanticType>,
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
        LogosAtom::Field { entity, field } => resolved(
            resolve_field(None, entity, field),
            format!("{entity}.{field}"),
        ),
        LogosAtom::QualifiedField {
            namespace,
            entity,
            field,
        } => resolved(
            resolve_field(Some(namespace), entity, field),
            format!("{namespace}.{entity}.{field}"),
        ),
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
    FF: Fn(Option<&str>, &str, &str) -> Option<SemanticType>,
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
            LogosAtom::Name(_) | LogosAtom::Field { .. } | LogosAtom::QualifiedField { .. } => {
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

/// PB-04 (#1683, #1685): one selected-import item, parsed exactly once.
/// `{ Entity:Foo as Bar }` selects the single export named `Foo`, asserts it
/// is an `Entity`, and binds it locally as `Bar`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedImport {
    pub public_name: String,
    pub expected_kind: Option<ExportKind>,
    pub local_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDirective {
    pub spec: String,
    pub alias: Option<String>,
    pub reexport: bool,
    pub select_items: Vec<SelectedImport>,
    pub wildcard: bool,
    pub line: u32,
    pub col: u32,
    pub decl_order: u32,
}

impl ImportDirective {
    /// The namespace alias every `Import` binds: explicit `as X` or the file stem.
    pub fn namespace_alias(&self) -> String {
        self.alias
            .clone()
            .unwrap_or_else(|| default_import_alias(&self.spec))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    System,
    Entity,
    Law,
}

/// PB-04 (#1703, #1706): export provenance. `Local` is a declaration of the
/// exporting module; `ReExport` carries the complete hop chain, ending with the
/// symbol name in the declaring module. (A plain, non-`pub` import exports
/// nothing, so there is no separate "imported" export state.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportOrigin {
    Local { module: String },
    ReExport { chain: Vec<String> },
}

/// PB-04: the one export item authority. `source_module` / `source_name` name
/// the declaration the item ultimately denotes (provider module id + declared
/// name), preserved across every re-export hop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportItem {
    pub public_name: String,
    pub kind: ExportKind,
    pub origin: ExportOrigin,
    pub source_module: String,
    pub source_name: String,
    pub span: SourceMark,
    pub decl_order: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExportSet {
    pub items: Vec<ExportItem>,
}

impl ExportSet {
    /// Flat namespace (#1686): at most one item per public name.
    pub fn get(&self, public_name: &str) -> Option<&ExportItem> {
        self.items
            .iter()
            .find(|item| item.public_name == public_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExportDecl {
    pub public_name: String,
    pub kind: ExportKind,
    pub span: SourceMark,
}

/// A module-graph, export or selection failure, bound to the provider module
/// id it was found in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleError {
    pub code: &'static str,
    pub message: String,
    pub module_id: String,
    pub line: u32,
    pub col: u32,
}

pub fn collect_local_exports_core(
    module_id: &str,
    module_display: &str,
    locals: &[LocalExportDecl],
) -> Result<ExportSet, ModuleError> {
    let mut set = ExportSet::default();
    for (idx, local) in locals.iter().enumerate() {
        // A same-kind duplicate is a declaration error with its own code
        // (E0220 Entity/System, E0221 Law), exactly as analysis reports it;
        // only a cross-kind clash is the flat-namespace E0242 (#1686).
        if let Some(prev) = set.get(&local.public_name) {
            if prev.kind == local.kind {
                let (code, message) = match local.kind {
                    ExportKind::Law => (
                        "E0221",
                        format!("duplicate Law '{}' in module", local.public_name),
                    ),
                    ExportKind::Entity => {
                        ("E0220", format!("duplicate Entity '{}'", local.public_name))
                    }
                    ExportKind::System => {
                        ("E0220", format!("duplicate System '{}'", local.public_name))
                    }
                };
                return Err(ModuleError {
                    code,
                    message,
                    module_id: module_id.to_string(),
                    line: local.span.line,
                    col: local.span.col,
                });
            }
        }
        push_export_item_core(
            &mut set,
            ExportItem {
                public_name: local.public_name.clone(),
                kind: local.kind,
                origin: ExportOrigin::Local {
                    module: module_display.to_string(),
                },
                source_module: module_id.to_string(),
                source_name: local.public_name.clone(),
                span: local.span,
                decl_order: idx as u32,
            },
            module_id,
            local.span.line,
            local.span.col,
        )?;
    }
    Ok(set)
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

/// The one Logos import-directive parser (semantic owner of import syntax
/// interpretation; `sm-front` preserves the directive text). Malformed text is
/// rejected (#1682), never repaired; an unknown kind qualifier is `E0245`
/// (#1683), never an unqualified selection.
pub fn parse_import_directive(
    directive: &str,
    line: u32,
    col: u32,
    decl_order: u32,
) -> Result<ImportDirective, ImportPolicyError> {
    let malformed = |what: &str| ImportPolicyError {
        code: "E0239",
        message: format!(
            "malformed Import directive ({what}): '{}'",
            directive.trim()
        ),
        line,
        col,
    };
    let after_kw = directive
        .strip_prefix("Import")
        .ok_or_else(|| malformed("missing Import keyword"))?;
    let mut rest = after_kw.trim();
    if rest.is_empty() {
        return Err(malformed("missing import path"));
    }
    let mut reexport = false;
    if let Some(after_pub) = rest.strip_prefix("pub ") {
        reexport = true;
        rest = after_pub.trim_start();
    }
    let (spec, mut tail) = if let Some(quoted) = rest.strip_prefix('"') {
        let end = quoted
            .find('"')
            .ok_or_else(|| malformed("unterminated import path"))?;
        (&quoted[..end], quoted[end + 1..].trim_start())
    } else {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        (&rest[..end], rest[end..].trim_start())
    };
    if spec.is_empty() {
        return Err(malformed("empty import path"));
    }
    let mut alias = None;
    if let Some(after_as) = tail.strip_prefix("as ") {
        let after_as = after_as.trim_start();
        let end = after_as.find(char::is_whitespace).unwrap_or(after_as.len());
        let name = after_as[..end].trim_matches('"');
        if name.is_empty() {
            return Err(malformed("missing alias after 'as'"));
        }
        alias = Some(name.to_string());
        tail = after_as[end..].trim_start();
    }
    let mut wildcard = false;
    if let Some(after_star) = tail.strip_prefix('*') {
        wildcard = true;
        tail = after_star.trim_start();
    }
    let mut select_items = Vec::new();
    if let Some(after_lbrace) = tail.strip_prefix('{') {
        let end = after_lbrace
            .find('}')
            .ok_or_else(|| malformed("missing '}' closing the select list"))?;
        for raw in after_lbrace[..end].split(',') {
            select_items.push(parse_selected_import(raw.trim(), line, col, &malformed)?);
        }
        tail = after_lbrace[end + 1..].trim_start();
    }
    if !(tail.is_empty() || tail.starts_with("//") || tail.starts_with('#')) {
        return Err(malformed("unexpected trailing text"));
    }
    Ok(ImportDirective {
        spec: spec.to_string(),
        alias,
        reexport,
        select_items,
        wildcard,
        line,
        col,
        decl_order,
    })
}

fn parse_selected_import(
    raw: &str,
    line: u32,
    col: u32,
    malformed: &dyn Fn(&str) -> ImportPolicyError,
) -> Result<SelectedImport, ImportPolicyError> {
    let (selector, local) = match raw.split_once(" as ") {
        Some((src, dst)) => (
            src.trim().trim_matches('"'),
            Some(dst.trim().trim_matches('"')),
        ),
        None => (raw.trim_matches('"'), None),
    };
    if selector.is_empty() || local == Some("") {
        return Err(malformed("empty selected item"));
    }
    let (expected_kind, public_name) = match selector.split_once(':') {
        Some((qualifier, name)) => {
            let kind = match qualifier.trim() {
                "System" => ExportKind::System,
                "Entity" => ExportKind::Entity,
                "Law" => ExportKind::Law,
                other => {
                    return Err(ImportPolicyError {
                        code: "E0245",
                        message: format!("unknown selected-import kind qualifier '{other}'"),
                        line,
                        col,
                    })
                }
            };
            (Some(kind), name.trim())
        }
        None => (None, selector),
    };
    if public_name.is_empty() {
        return Err(malformed("empty selected item"));
    }
    Ok(SelectedImport {
        public_name: public_name.to_string(),
        expected_kind,
        local_name: local.unwrap_or(public_name).to_string(),
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
        let alias = import.namespace_alias();
        if !aliases.insert(alias.clone()) {
            return Err(ImportPolicyError {
                code: "E0241",
                message: format!("duplicate import namespace alias '{}' in one module", alias),
                line: import.line,
                col: import.col,
            });
        }

        let mut seen_select_alias = BTreeSet::<String>::new();
        for selected in &import.select_items {
            if !seen_select_alias.insert(selected.local_name.clone()) {
                return Err(ImportPolicyError {
                    code: "E0245",
                    message: format!(
                        "duplicate selected import alias '{}' in one Import statement",
                        selected.local_name
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
        let alias = import.namespace_alias();
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
        for selected in &import.select_items {
            let local = &selected.local_name;
            if local_names.contains(local) {
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

/// PB-04 (#1692): one import declaration, resolved through the provider
/// exactly once; `target` is the provider-owned module id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImport {
    pub directive: ImportDirective,
    pub target: String,
}

/// One node of the frozen module graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNode {
    /// Provider-owned semantic identity.
    pub module_id: String,
    /// Provider presentation of `module_id` (diagnostics/provenance text only).
    pub display: String,
    pub local_exports: ExportSet,
    pub imports: Vec<ResolvedImport>,
}

/// PB-04: the one frozen, resolved module graph every module-semantic phase
/// (exports, selection, name binding, provenance) consumes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleGraph {
    nodes: BTreeMap<String, ModuleNode>,
}

impl ModuleGraph {
    /// Freezes `nodes`. A duplicate module id (#1688) or an edge whose target
    /// is not a node is a deterministic error.
    pub fn new(nodes: Vec<ModuleNode>) -> Result<Self, ModuleError> {
        let mut map = BTreeMap::new();
        for node in nodes {
            if let Some(prev) = map.get(&node.module_id) {
                let prev: &ModuleNode = prev;
                return Err(ModuleError {
                    code: "E0239",
                    message: format!("duplicate module id '{}'", prev.display),
                    module_id: node.module_id.clone(),
                    line: 0,
                    col: 0,
                });
            }
            map.insert(node.module_id.clone(), node);
        }
        for node in map.values() {
            for import in &node.imports {
                if !map.contains_key(&import.target) {
                    return Err(ModuleError {
                        code: "E0239",
                        message: format!(
                            "import '{}' resolved to module '{}' that is not in the module graph",
                            import.directive.spec, import.target
                        ),
                        module_id: node.module_id.clone(),
                        line: import.directive.line,
                        col: import.directive.col,
                    });
                }
            }
        }
        Ok(Self { nodes: map })
    }

    pub fn node(&self, module_id: &str) -> Option<&ModuleNode> {
        self.nodes.get(module_id)
    }

    /// Nodes in module-id order.
    pub fn nodes(&self) -> impl Iterator<Item = &ModuleNode> {
        self.nodes.values()
    }

    fn display_of(&self, module_id: &str) -> String {
        self.nodes
            .get(module_id)
            .map_or_else(|| module_id.to_string(), |n| n.display.clone())
    }
}

/// Export sets of every node, keyed by provider module id.
pub type ExportSets = BTreeMap<String, ExportSet>;

pub fn build_export_sets_core(graph: &ModuleGraph) -> Result<ExportSets, ModuleError> {
    let mut cache = ExportSets::new();
    let mut stack = Vec::<String>::new();
    for node in graph.nodes() {
        build_export_set_for_core(graph, &node.module_id, &mut cache, &mut stack)?;
    }
    Ok(cache)
}

fn build_export_set_for_core(
    graph: &ModuleGraph,
    module_id: &str,
    cache: &mut ExportSets,
    stack: &mut Vec<String>,
) -> Result<(), ModuleError> {
    if cache.contains_key(module_id) {
        return Ok(());
    }
    if let Some(pos) = stack.iter().position(|m| m == module_id) {
        let mut chain_parts: Vec<String> =
            stack[pos..].iter().map(|m| graph.display_of(m)).collect();
        chain_parts.push(graph.display_of(module_id));
        return Err(ModuleError {
            code: "E0243",
            message: format!(
                "symbol re-export cycle detected: {}",
                chain_parts.join(" -> ")
            ),
            module_id: module_id.to_string(),
            line: 0,
            col: 0,
        });
    }
    let node = graph.node(module_id).ok_or_else(|| ModuleError {
        code: "E0239",
        message: format!("unknown module '{}'", module_id),
        module_id: module_id.to_string(),
        line: 0,
        col: 0,
    })?;

    stack.push(module_id.to_string());
    let mut set = node.local_exports.clone();
    let mut next_decl = set.items.len() as u32;
    for import in &node.imports {
        let directive = &import.directive;
        if !directive.reexport {
            continue;
        }
        build_export_set_for_core(graph, &import.target, cache, stack)?;
        // #1689: the dependency was just built; its absence is an internal
        // invariant failure, never an empty export set.
        let dep_set = cache.get(&import.target).ok_or_else(|| ModuleError {
            code: "E0239",
            message: format!(
                "internal: export set for '{}' missing after construction",
                graph.display_of(&import.target)
            ),
            module_id: module_id.to_string(),
            line: directive.line,
            col: directive.col,
        })?;
        let selected = select_export_items(graph, node, import, dep_set)?;
        for (dep_item, public_name) in selected {
            let chain = match &dep_item.origin {
                // #1703: extend the existing chain, never truncate it.
                ExportOrigin::ReExport { chain } => {
                    let mut full = vec![node.display.clone()];
                    full.extend(chain.iter().cloned());
                    full
                }
                ExportOrigin::Local { .. } => vec![
                    node.display.clone(),
                    graph.display_of(&import.target),
                    dep_item.public_name.clone(),
                ],
            };
            push_export_item_core(
                &mut set,
                ExportItem {
                    public_name,
                    kind: dep_item.kind,
                    origin: ExportOrigin::ReExport { chain },
                    source_module: dep_item.source_module.clone(),
                    source_name: dep_item.source_name.clone(),
                    span: SourceMark {
                        line: directive.line,
                        col: directive.col,
                        file_id: 0,
                    },
                    decl_order: next_decl.max(dep_item.decl_order + directive.decl_order + 1),
                },
                module_id,
                directive.line,
                directive.col,
            )?;
            next_decl += 1;
        }
    }

    set.items.sort_by(|a, b| a.decl_order.cmp(&b.decl_order));
    let _ = stack.pop();
    cache.insert(module_id.to_string(), set);
    Ok(())
}

/// The items an import selects from its target's export set, with the public
/// name each is bound under. Selected items must exist (E0244) and satisfy
/// their kind assertion (E0245) on the unique export item (#1685, #1687).
fn select_export_items(
    graph: &ModuleGraph,
    node: &ModuleNode,
    import: &ResolvedImport,
    dep_set: &ExportSet,
) -> Result<Vec<(ExportItem, String)>, ModuleError> {
    let directive = &import.directive;
    if directive.select_items.is_empty() {
        return Ok(dep_set
            .items
            .iter()
            .map(|item| (item.clone(), item.public_name.clone()))
            .collect());
    }
    let mut out = Vec::new();
    for selected in &directive.select_items {
        let item = selected_item(graph, node, import, dep_set, selected)?;
        out.push((item.clone(), selected.local_name.clone()));
    }
    Ok(out)
}

fn selected_item<'a>(
    graph: &ModuleGraph,
    node: &ModuleNode,
    import: &ResolvedImport,
    dep_set: &'a ExportSet,
    selected: &SelectedImport,
) -> Result<&'a ExportItem, ModuleError> {
    let directive = &import.directive;
    let item = dep_set
        .get(&selected.public_name)
        .ok_or_else(|| ModuleError {
            code: "E0244",
            message: format!(
                "selected import symbol '{}' not found in '{}'",
                selected.public_name,
                graph.display_of(&import.target)
            ),
            module_id: node.module_id.clone(),
            line: directive.line,
            col: directive.col,
        })?;
    if let Some(expected) = selected.expected_kind {
        if item.kind != expected {
            return Err(ModuleError {
                code: "E0245",
                message: format!(
                    "selected import symbol '{}' kind mismatch: expected {:?}, found {:?}",
                    selected.public_name, expected, item.kind
                ),
                module_id: node.module_id.clone(),
                line: directive.line,
                col: directive.col,
            });
        }
    }
    Ok(item)
}

/// Every selected import of every module names an existing export item of the
/// declared kind. Consumes the frozen graph and export sets only.
pub fn validate_select_imports_core(
    graph: &ModuleGraph,
    export_sets: &ExportSets,
) -> Result<(), ModuleError> {
    for node in graph.nodes() {
        for import in &node.imports {
            if import.directive.select_items.is_empty() {
                continue;
            }
            let dep_set = export_sets.get(&import.target).ok_or_else(|| ModuleError {
                code: "E0239",
                message: format!(
                    "internal: no export set for imported module '{}'",
                    graph.display_of(&import.target)
                ),
                module_id: node.module_id.clone(),
                line: import.directive.line,
                col: import.directive.col,
            })?;
            for selected in &import.directive.select_items {
                selected_item(graph, node, import, dep_set, selected)?;
            }
        }
    }
    Ok(())
}

/// PB-04 (#1684): the documented lookup for an unqualified name that is not a
/// local symbol: explicit selected imports first, then wildcard imports in
/// declaration order (first match wins).
pub fn resolve_unqualified_import<'a>(
    node: &ModuleNode,
    export_sets: &'a ExportSets,
    name: &str,
) -> Option<&'a ExportItem> {
    let mut imports: Vec<&ResolvedImport> = node.imports.iter().collect();
    imports.sort_by_key(|import| import.directive.decl_order);
    for import in &imports {
        for selected in &import.directive.select_items {
            if selected.local_name == name {
                return export_sets.get(&import.target)?.get(&selected.public_name);
            }
        }
    }
    imports
        .iter()
        .filter(|import| import.directive.wildcard)
        .find_map(|import| export_sets.get(&import.target)?.get(name))
}

/// PB-04 (#1684): namespace-qualified access `X.Foo`: only through the import
/// whose namespace alias is `X`; never a fallback for unqualified names.
pub fn resolve_namespace_import<'a>(
    node: &ModuleNode,
    export_sets: &'a ExportSets,
    alias: &str,
    name: &str,
) -> Option<&'a ExportItem> {
    let import = node
        .imports
        .iter()
        .find(|import| import.directive.namespace_alias() == alias)?;
    export_sets.get(&import.target)?.get(name)
}

/// Flat export namespace (#1686): one public name, at most one item,
/// regardless of kind.
fn push_export_item_core(
    set: &mut ExportSet,
    item: ExportItem,
    module_id: &str,
    line: u32,
    col: u32,
) -> Result<(), ModuleError> {
    if let Some(prev) = set.get(&item.public_name) {
        return Err(ModuleError {
            code: "E0242",
            message: format!(
                "{} collision for '{}' between {:?} {:?} and {:?} {:?}",
                if matches!(item.origin, ExportOrigin::ReExport { .. }) {
                    "re-export"
                } else {
                    "export"
                },
                item.public_name,
                prev.kind,
                prev.origin,
                item.kind,
                item.origin
            ),
            module_id: module_id.to_string(),
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
            |_ns, _e, _f| None,
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
