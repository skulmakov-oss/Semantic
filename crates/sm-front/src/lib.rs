#![allow(
    clippy::type_complexity,
    clippy::let_and_return,
    clippy::only_used_in_recursion,
    clippy::doc_overindented_list_items,
    clippy::collapsible_if,
    clippy::needless_lifetimes,
    clippy::empty_line_after_doc_comments
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::collections::BTreeMap;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::format;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::vec;
#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::vec::Vec;

#[cfg(any(feature = "alloc", feature = "std"))]
pub mod hello_parser;
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod hello_sema;
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod types;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use sm_profile::{CompatibilityMode, ParserProfile};
#[cfg(any(feature = "alloc", feature = "std"))]
pub use types::{
    AdtCtorExpr,
    AdtDecl,
    AdtVariant,
    AstArena,
    BinaryOp,
    BlockExpr,
    CallArg,
    ClosureCapturePolicy,
    ClosureLiteral,
    ClosureType,
    ClosureValueFamily,
    Expr,
    ExprId,
    FrontendError,
    FrontendErrorKind,
    Function,
    IfExpr,
    ImplDecl,
    IterableLoopDesugaring,
    LogosEntity,
    LogosEntityField,
    LogosEntityFieldKind,
    LogosLaw,
    LogosProgram,
    LogosSystem,
    LogosWhen,
    LoopExpr,
    MapType,
    MatchArm,
    MatchExpr,
    MatchExprArm,
    // M9.7
    PathAvailability,
    PatternPath,
    Program,
    QuadVal,
    RecordDecl,
    RecordField,
    RecordFieldExpr,
    RecordInitField,
    RecordLiteralExpr,
    RecordUpdateExpr,
    SchemaDecl,
    SchemaField,
    SchemaRole,
    SchemaShape,
    SchemaVariant,
    SchemaVersion,
    SequenceCollectionFamily,
    SequenceIndexExpr,
    SequenceLiteral,
    SequenceType,
    Stmt,
    StmtId,
    SymbolId,
    TextLiteral,
    TextLiteralFamily,
    Token,
    TokenKind,
    TraitBound,
    TraitDecl,
    TraitMethodSig,
    TuplePatternItem,
    Type,
    UnaryOp,
    ValidationCheck,
    ValidationFieldPlan,
    ValidationPlan,
    ValidationShapePlan,
    ValidationVariantPlan,
};

#[cfg(any(feature = "alloc", feature = "std"))]
pub mod lexer;
#[cfg(any(feature = "alloc", feature = "std"))]
pub mod parser;
#[cfg(any(feature = "alloc", feature = "std"))]
mod typecheck;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use typecheck::{
    derive_validation_plan_table, type_check_function, type_check_function_with_table,
    type_check_program,
};

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnSig {
    /// Generic type parameter names declared on this function.
    ///
    /// Non-empty signals a generic function. Call-site type-checking performs
    /// substitution map inference from arguments before checking param types.
    pub type_params: Vec<SymbolId>,
    /// Trait bounds on the type parameters: `<T: TraitName>` constraints.
    ///
    /// Admitted at the owner layer (Wave 1). Bound checking at call sites
    /// and impl resolution are deferred to Wave 3.
    pub trait_bounds: Vec<TraitBound>,
    pub params: Vec<Type>,
    pub param_names: Option<Vec<SymbolId>>,
    pub param_defaults: Option<Vec<Option<ExprId>>>,
    pub ret: Type,
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub type FnTable = BTreeMap<SymbolId, FnSig>;

#[cfg(any(feature = "alloc", feature = "std"))]
pub type RecordTable = BTreeMap<SymbolId, RecordDecl>;

#[cfg(any(feature = "alloc", feature = "std"))]
pub type AdtTable = BTreeMap<SymbolId, AdtDecl>;

const APPLICATION_BUILTIN_NAMES: &[&str] = &[
    "args_read",
    "stdin_read_text",
    "stdout_write",
    "stderr_write",
    "path_inspect",
    "fs_read_text",
    "fs_write_text",
    "time_duration_ms",
];

#[cfg(any(feature = "alloc", feature = "std"))]
pub type SchemaTable = BTreeMap<SymbolId, SchemaDecl>;

#[cfg(any(feature = "alloc", feature = "std"))]
pub type ValidationPlanTable = BTreeMap<SymbolId, ValidationPlan>;

#[cfg(any(feature = "alloc", feature = "std"))]
/// Trait definitions indexed by trait name.
///
/// Admitted at the owner layer (Wave 1). Build function is deferred to Wave 2
/// when parser admission lands.
pub type TraitTable = BTreeMap<SymbolId, TraitDecl>;

#[cfg(any(feature = "alloc", feature = "std"))]
/// All impl blocks in the program, ordered by declaration.
///
/// Not keyed by a single SymbolId because the coherence key is
/// (trait_name, for_type). Admitted at the owner layer (Wave 1).
/// Build function and coherence checks are deferred to Wave 2/3.
pub type ImplTable = Vec<ImplDecl>;

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBinding {
    pub ty: Type,
    pub is_const: bool,
    pub is_mutable: bool,
    /// M9.5 Wave C: true after the binding's value has been moved out (whole-variable).
    pub consumed: bool,
    /// M9.7: per-path availability for partial-move tracking.
    /// Empty means the whole variable is fully available.
    pub path_state: Vec<(crate::types::PatternPath, crate::types::PathAvailability)>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeEnv {
    scopes: Vec<BTreeMap<SymbolId, ScopeBinding>>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl ScopeEnv {
    pub fn new() -> Self {
        Self {
            scopes: vec![BTreeMap::new()],
        }
    }

    pub fn with_params(params: &[(SymbolId, Type)]) -> Self {
        let mut env = Self::new();
        for (name, ty) in params {
            env.insert(*name, ty.clone());
        }
        env
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            let _ = self.scopes.pop();
        }
    }

    pub fn insert(&mut self, name: SymbolId, ty: Type) {
        self.insert_binding(
            name,
            ScopeBinding {
                ty,
                is_const: false,
                is_mutable: false,
                consumed: false,
                path_state: Vec::new(),
            },
        );
    }

    pub fn insert_mut(&mut self, name: SymbolId, ty: Type) {
        self.insert_binding(
            name,
            ScopeBinding {
                ty,
                is_const: false,
                is_mutable: true,
                consumed: false,
                path_state: Vec::new(),
            },
        );
    }

    pub fn insert_const(&mut self, name: SymbolId, ty: Type) {
        self.insert_binding(
            name,
            ScopeBinding {
                ty,
                is_const: true,
                is_mutable: false,
                consumed: false,
                path_state: Vec::new(),
            },
        );
    }

    /// Mark a variable as consumed (moved out). Subsequent reads will be rejected.
    pub fn mark_consumed(&mut self, name: SymbolId) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(&name) {
                binding.consumed = true;
                return;
            }
        }
    }

    /// Returns true if the variable has been moved and is no longer available.
    pub fn is_consumed(&self, name: SymbolId) -> bool {
        self.binding(name).map(|b| b.consumed).unwrap_or(false)
    }

    /// M9.7: Record that `path` within variable `name` has been moved or borrowed.
    pub fn mark_path_state(
        &mut self,
        name: SymbolId,
        path: crate::types::PatternPath,
        state: crate::types::PathAvailability,
    ) {
        use crate::types::PatternPath;

        fn path_is_prefix(a: &PatternPath, b: &PatternPath) -> bool {
            if a.elems.len() > b.elems.len() {
                return false;
            }
            a.elems.iter().zip(&b.elems).all(|(x, y)| x == y)
        }

        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(&name) {
                // M9.9 Wave C: normalise path-state to keep it compact.
                //
                // Rule 1 — new path subsumes longer existing entries of the same state:
                //   e.g. adding Moved(root) while Moved(root.0) exists → drop root.0.
                binding.path_state.retain(|(existing, existing_state)| {
                    if *existing_state != state {
                        return true;
                    }
                    !path_is_prefix(&path, existing)
                });
                // Rule 2 — if an existing entry already covers the new path (same state,
                //   existing is a prefix of new path), the new entry is redundant.
                let redundant = binding.path_state.iter().any(|(existing, existing_state)| {
                    *existing_state == state && path_is_prefix(existing, &path)
                });
                if !redundant {
                    binding.path_state.push((path, state));
                }
                return;
            }
        }
    }

    /// M9.7: Check that accessing `access_path` within `name` is allowed.
    ///
    /// Rejects if any stored path overlaps `access_path` with state `Moved`.
    /// Conservative: borrows are not currently enforced as blocking reads.
    pub fn check_path_available(
        &self,
        name: SymbolId,
        access_path: &crate::types::PatternPath,
    ) -> Result<(), crate::types::FrontendError> {
        use crate::types::{PathAvailability, PatternPath};

        fn path_is_prefix(a: &PatternPath, b: &PatternPath) -> bool {
            if a.elems.len() > b.elems.len() {
                return false;
            }
            a.elems.iter().zip(&b.elems).all(|(x, y)| x == y)
        }
        fn paths_overlap(a: &PatternPath, b: &PatternPath) -> bool {
            path_is_prefix(a, b) || path_is_prefix(b, a)
        }

        if let Some(binding) = self.binding(name) {
            // Whole-variable consumed takes priority.
            if binding.consumed {
                return Err(crate::types::FrontendError {
                    pos: 0,
                    message: format!("use of moved value '{}'", name.0),
                });
            }
            for (stored_path, avail) in &binding.path_state {
                if paths_overlap(stored_path, access_path) {
                    if *avail == PathAvailability::Moved {
                        // M9.9 Wave D: more precise diagnostic.
                        // Distinguish "accessing moved path" from "accessing parent of moved child".
                        let msg = if path_is_prefix(stored_path, access_path) {
                            // stored = root.0, access = root.0 or root.0.x → moved path
                            format!(
                                "use of moved value: path was moved earlier (moved path {:?})",
                                stored_path.elems
                            )
                        } else {
                            // stored = root.0, access = root → whole-var after partial move
                            format!(
                                "use of partially moved value: cannot use whole variable because \
                                 child path {:?} was moved",
                                stored_path.elems
                            )
                        };
                        return Err(crate::types::FrontendError {
                            pos: 0,
                            message: msg,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// M9.8: Check that a new capture of `path` with `capture` mode is compatible
    /// with the existing path-state of variable `name`.
    ///
    /// Rules:
    ///   prior Borrowed + new Move   → error ("cannot move from borrowed value")
    ///   prior Moved   + new Borrow  → error ("cannot borrow from moved value")
    ///   prior Moved   + new Move    → error ("cannot move from moved value")
    ///   prior Borrowed + new Borrow → ok
    ///   prior Available + anything  → ok
    pub fn check_capture_allowed(
        &self,
        name: SymbolId,
        path: &crate::types::PatternPath,
        capture: crate::types::CaptureMode,
    ) -> Result<(), crate::types::FrontendError> {
        use crate::types::{CaptureMode, PathAvailability, PatternPath};

        fn path_is_prefix(a: &PatternPath, b: &PatternPath) -> bool {
            if a.elems.len() > b.elems.len() {
                return false;
            }
            a.elems.iter().zip(&b.elems).all(|(x, y)| x == y)
        }
        fn paths_overlap(a: &PatternPath, b: &PatternPath) -> bool {
            path_is_prefix(a, b) || path_is_prefix(b, a)
        }

        let Some(binding) = self.binding(name) else {
            return Ok(());
        };

        if binding.consumed {
            return Err(crate::types::FrontendError {
                pos: 0,
                message: format!("cannot capture moved value '{}'", name.0),
            });
        }

        for (stored_path, stored_state) in &binding.path_state {
            if !paths_overlap(stored_path, path) {
                continue;
            }
            let msg: Option<&str> = match (stored_state, capture) {
                (PathAvailability::Borrowed, CaptureMode::Move) => {
                    Some("cannot move from borrowed path")
                }
                (PathAvailability::Moved, CaptureMode::Borrow) => {
                    Some("cannot borrow from moved path")
                }
                (PathAvailability::Moved, CaptureMode::Move) => {
                    Some("cannot move from already-moved path")
                }
                _ => None,
            };
            if let Some(m) = msg {
                return Err(crate::types::FrontendError {
                    pos: 0,
                    message: m.to_string(),
                });
            }
        }
        Ok(())
    }

    fn insert_binding(&mut self, name: SymbolId, binding: ScopeBinding) {
        if let Some(last) = self.scopes.last_mut() {
            last.insert(name, binding);
        }
    }

    pub fn get(&self, name: SymbolId) -> Option<Type> {
        self.binding(name).map(|binding| binding.ty.clone())
    }

    pub fn is_const(&self, name: SymbolId) -> bool {
        self.binding(name)
            .map(|binding| binding.is_const)
            .unwrap_or(false)
    }

    pub fn is_mutable(&self, name: SymbolId) -> bool {
        self.binding(name)
            .map(|binding| binding.is_mutable)
            .unwrap_or(false)
    }

    fn binding(&self, name: SymbolId) -> Option<&ScopeBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(&name) {
                return Some(binding);
            }
        }
        None
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl Default for ScopeEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn build_fn_table(program: &Program) -> Result<FnTable, FrontendError> {
    let record_table = build_record_table(program)?;
    let adt_table = build_adt_table(program)?;
    let mut out = BTreeMap::new();
    for f in &program.functions {
        let name = resolve_symbol_name(&program.arena, f.name)?;
        if APPLICATION_BUILTIN_NAMES.contains(&name) {
            return Err(FrontendError {
                pos: 0,
                message: format!("function name '{name}' is reserved for the application boundary"),
            });
        }
        if out.contains_key(&f.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!("duplicate function '{name}'"),
            });
        }
        out.insert(
            f.name,
            FnSig {
                type_params: f.type_params.clone(),
                trait_bounds: f.trait_bounds.clone(),
                params: f
                    .params
                    .iter()
                    .map(|(_, t)| {
                        canonicalize_declared_type_generic(
                            t,
                            &record_table,
                            &adt_table,
                            &program.arena,
                            &f.type_params,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                param_names: Some(f.params.iter().map(|(name, _)| *name).collect()),
                param_defaults: Some(f.param_defaults.clone()),
                ret: canonicalize_declared_type_generic(
                    &f.ret,
                    &record_table,
                    &adt_table,
                    &program.arena,
                    &f.type_params,
                )?,
            },
        );
    }
    Ok(out)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn build_record_table(program: &Program) -> Result<RecordTable, FrontendError> {
    let mut out = BTreeMap::new();
    for record in &program.records {
        if out.contains_key(&record.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "duplicate record '{}'",
                    resolve_symbol_name(&program.arena, record.name)?
                ),
            });
        }
        out.insert(record.name, record.clone());
    }
    Ok(out)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn build_adt_table(program: &Program) -> Result<AdtTable, FrontendError> {
    let mut out = BTreeMap::new();
    for adt in &program.adts {
        if out.contains_key(&adt.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "duplicate enum '{}'",
                    resolve_symbol_name(&program.arena, adt.name)?
                ),
            });
        }
        out.insert(adt.name, adt.clone());
    }
    Ok(out)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn build_trait_table(program: &Program) -> Result<TraitTable, FrontendError> {
    let mut out = BTreeMap::new();
    for t in &program.traits {
        if out.contains_key(&t.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "duplicate trait '{}'",
                    resolve_symbol_name(&program.arena, t.name)?
                ),
            });
        }
        out.insert(t.name, t.clone());
    }
    Ok(out)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn build_schema_table(program: &Program) -> Result<SchemaTable, FrontendError> {
    let mut out = BTreeMap::new();
    for schema in &program.schemas {
        if out.contains_key(&schema.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "duplicate schema '{}'",
                    resolve_symbol_name(&program.arena, schema.name)?
                ),
            });
        }
        out.insert(schema.name, schema.clone());
    }
    Ok(out)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn canonicalize_declared_type(
    ty: &Type,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
) -> Result<Type, FrontendError> {
    match ty {
        Type::Tuple(items) => Ok(Type::Tuple(
            items
                .iter()
                .map(|item| canonicalize_declared_type(item, record_table, adt_table, arena))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Type::Sequence(sequence) => Ok(Type::Sequence(SequenceType {
            family: sequence.family,
            item: Box::new(canonicalize_declared_type(
                sequence.item.as_ref(),
                record_table,
                adt_table,
                arena,
            )?),
        })),
        Type::Map(map) => Ok(Type::Map(MapType {
            key: Box::new(canonicalize_declared_type(
                map.key.as_ref(),
                record_table,
                adt_table,
                arena,
            )?),
            val: Box::new(canonicalize_declared_type(
                map.val.as_ref(),
                record_table,
                adt_table,
                arena,
            )?),
        })),
        Type::Measured(base, unit) => {
            let canonical_base = canonicalize_declared_type(base, record_table, adt_table, arena)?;
            if !canonical_base.is_core_numeric_scalar() {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unit annotation '{}' is allowed only on i32, u32, f64, or fx in v0",
                        resolve_symbol_name(arena, *unit)?
                    ),
                });
            }
            Ok(Type::Measured(Box::new(canonical_base), *unit))
        }
        Type::Option(item) => Ok(Type::Option(Box::new(canonicalize_declared_type(
            item,
            record_table,
            adt_table,
            arena,
        )?))),
        Type::Result(ok_ty, err_ty) => Ok(Type::Result(
            Box::new(canonicalize_declared_type(
                ok_ty,
                record_table,
                adt_table,
                arena,
            )?),
            Box::new(canonicalize_declared_type(
                err_ty,
                record_table,
                adt_table,
                arena,
            )?),
        )),
        Type::Record(name) => {
            let is_record = record_table.contains_key(name);
            let is_adt = adt_table.contains_key(name);
            match (is_record, is_adt) {
                (true, false) => Ok(Type::Record(*name)),
                (false, true) => Ok(Type::Adt(*name)),
                (true, true) => Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "top-level name '{}' is ambiguously declared as both record and enum",
                        resolve_symbol_name(arena, *name)?
                    ),
                }),
                (false, false) => Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unknown nominal type '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                }),
            }
        }
        Type::Adt(name) => {
            if adt_table.contains_key(name) {
                Ok(Type::Adt(*name))
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!("unknown enum type '{}'", resolve_symbol_name(arena, *name)?),
                })
            }
        }
        Type::TypeVar(name) => Err(FrontendError::policy_violation(
            0,
            format!(
                "type variable '{}' is not admitted in the executable type-check path yet; \
                 generic monomorphisation is deferred to M9.1 Wave 2",
                resolve_symbol_name(arena, *name).unwrap_or("<unknown>")
            ),
        )),
        _ => Ok(ty.clone()),
    }
}

/// Variant of `canonicalize_declared_type` that permits `TypeVar` when the
/// variable is listed in `type_params`.
///
/// Used during `build_fn_table` so that generic function signatures can be
/// stored with TypeVar placeholders without triggering the policy_violation gap.
/// Monomorphisation (substituting concrete types at call sites) is done at
/// Wave 3 call-site type-check time.
#[cfg(any(feature = "alloc", feature = "std"))]
pub fn canonicalize_declared_type_generic(
    ty: &Type,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
    type_params: &[SymbolId],
) -> Result<Type, FrontendError> {
    match ty {
        Type::Tuple(items) => Ok(Type::Tuple(
            items
                .iter()
                .map(|item| {
                    canonicalize_declared_type_generic(
                        item,
                        record_table,
                        adt_table,
                        arena,
                        type_params,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Type::Sequence(sequence) => Ok(Type::Sequence(SequenceType {
            family: sequence.family,
            item: Box::new(canonicalize_declared_type_generic(
                sequence.item.as_ref(),
                record_table,
                adt_table,
                arena,
                type_params,
            )?),
        })),
        Type::Map(map) => Ok(Type::Map(MapType {
            key: Box::new(canonicalize_declared_type_generic(
                map.key.as_ref(),
                record_table,
                adt_table,
                arena,
                type_params,
            )?),
            val: Box::new(canonicalize_declared_type_generic(
                map.val.as_ref(),
                record_table,
                adt_table,
                arena,
                type_params,
            )?),
        })),
        Type::Measured(base, unit) => {
            let canonical_base = canonicalize_declared_type_generic(
                base,
                record_table,
                adt_table,
                arena,
                type_params,
            )?;
            if !canonical_base.is_core_numeric_scalar() {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unit annotation '{}' is allowed only on i32, u32, f64, or fx in v0",
                        resolve_symbol_name(arena, *unit)?
                    ),
                });
            }
            Ok(Type::Measured(Box::new(canonical_base), *unit))
        }
        Type::Option(item) => Ok(Type::Option(Box::new(canonicalize_declared_type_generic(
            item,
            record_table,
            adt_table,
            arena,
            type_params,
        )?))),
        Type::Result(ok_ty, err_ty) => Ok(Type::Result(
            Box::new(canonicalize_declared_type_generic(
                ok_ty,
                record_table,
                adt_table,
                arena,
                type_params,
            )?),
            Box::new(canonicalize_declared_type_generic(
                err_ty,
                record_table,
                adt_table,
                arena,
                type_params,
            )?),
        )),
        Type::Closure(closure) => Ok(Type::Closure(crate::types::ClosureType {
            family: closure.family,
            capture: closure.capture,
            param: Box::new(canonicalize_declared_type_generic(
                &closure.param,
                record_table,
                adt_table,
                arena,
                type_params,
            )?),
            ret: Box::new(canonicalize_declared_type_generic(
                &closure.ret,
                record_table,
                adt_table,
                arena,
                type_params,
            )?),
        })),
        Type::Record(name) => {
            let is_record = record_table.contains_key(name);
            let is_adt = adt_table.contains_key(name);
            match (is_record, is_adt) {
                (true, false) => Ok(Type::Record(*name)),
                (false, true) => Ok(Type::Adt(*name)),
                (true, true) => Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "top-level name '{}' is ambiguously declared as both record and enum",
                        resolve_symbol_name(arena, *name)?
                    ),
                }),
                (false, false) => Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unknown nominal type '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                }),
            }
        }
        Type::Adt(name) => {
            if adt_table.contains_key(name) {
                Ok(Type::Adt(*name))
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!("unknown enum type '{}'", resolve_symbol_name(arena, *name)?),
                })
            }
        }
        Type::TypeVar(name) => {
            if type_params.contains(name) {
                Ok(Type::TypeVar(*name))
            } else {
                Err(FrontendError::policy_violation(
                    0,
                    format!(
                        "type variable '{}' is not in scope; \
                         it was not declared as a type parameter of this declaration",
                        resolve_symbol_name(arena, *name).unwrap_or("<unknown>")
                    ),
                ))
            }
        }
        _ => Ok(ty.clone()),
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn builtin_sig(name: &str) -> Option<FnSig> {
    match name {
        "sin" | "cos" | "tan" | "sqrt" | "abs" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::F64],
            param_names: None,
            param_defaults: None,
            ret: Type::F64,
        }),
        "pow" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::F64, Type::F64],
            param_names: None,
            param_defaults: None,
            ret: Type::F64,
        }),
        "args_read" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::U32],
            param_names: None,
            param_defaults: None,
            ret: Type::Text,
        }),
        "stdin_read_text" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: Vec::new(),
            param_names: None,
            param_defaults: None,
            ret: Type::Text,
        }),
        "stdout_write" | "stderr_write" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::Text],
            param_names: None,
            param_defaults: None,
            ret: Type::Unit,
        }),
        "path_inspect" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::Text],
            param_names: None,
            param_defaults: None,
            ret: Type::Bool,
        }),
        "fs_read_text" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::Text],
            param_names: None,
            param_defaults: None,
            ret: Type::Text,
        }),
        "fs_write_text" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::Text, Type::Text],
            param_names: None,
            param_defaults: None,
            ret: Type::Unit,
        }),
        "time_duration_ms" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: Vec::new(),
            param_names: None,
            param_defaults: None,
            ret: Type::U32,
        }),
        "qtruth_and" | "qtruth_or" | "qtruth_impl" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::Quad, Type::Quad],
            param_names: None,
            param_defaults: None,
            ret: Type::Quad,
        }),
        "qtruth_not" => Some(FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: vec![Type::Quad],
            param_names: None,
            param_defaults: None,
            ret: Type::Quad,
        }),
        _ => None,
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn reorder_call_args(
    call_name: SymbolId,
    args: &[CallArg],
    sig: &FnSig,
    arena: &AstArena,
) -> Result<Vec<ExprId>, FrontendError> {
    let has_named = args.iter().any(|arg| arg.name.is_some());
    if !has_named {
        if args.len() > sig.params.len() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "function '{}' expects {} args, got {}",
                    resolve_symbol_name(arena, call_name)?,
                    sig.params.len(),
                    args.len()
                ),
            });
        }
        let mut ordered = vec![None; sig.params.len()];
        for (idx, arg) in args.iter().enumerate() {
            ordered[idx] = Some(arg.value);
        }
        return finalize_ordered_call_args(call_name, ordered, sig, arena, args.len());
    }

    let Some(param_names) = sig.param_names.as_ref() else {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "named arguments are not supported for builtin '{}'",
                resolve_symbol_name(arena, call_name)?
            ),
        });
    };

    let mut ordered = vec![None; sig.params.len()];
    let mut positional_index = 0usize;
    let mut named_seen = false;
    for arg in args {
        if let Some(arg_name) = arg.name {
            named_seen = true;
            let Some(param_index) = param_names.iter().position(|name| *name == arg_name) else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "function '{}' has no parameter named '{}'",
                        resolve_symbol_name(arena, call_name)?,
                        resolve_symbol_name(arena, arg_name)?
                    ),
                });
            };
            if ordered[param_index].is_some() {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "duplicate named argument '{}' in call to '{}'",
                        resolve_symbol_name(arena, arg_name)?,
                        resolve_symbol_name(arena, call_name)?
                    ),
                });
            }
            ordered[param_index] = Some(arg.value);
        } else {
            if named_seen {
                return Err(FrontendError {
                    pos: 0,
                    message: "positional arguments cannot follow named arguments".to_string(),
                });
            }
            if positional_index >= ordered.len() {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "function '{}' expects {} args, got {}",
                        resolve_symbol_name(arena, call_name)?,
                        sig.params.len(),
                        args.len()
                    ),
                });
            }
            ordered[positional_index] = Some(arg.value);
            positional_index += 1;
        }
    }

    finalize_ordered_call_args(call_name, ordered, sig, arena, args.len())
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn finalize_ordered_call_args(
    call_name: SymbolId,
    mut ordered: Vec<Option<ExprId>>,
    sig: &FnSig,
    arena: &AstArena,
    provided_count: usize,
) -> Result<Vec<ExprId>, FrontendError> {
    let param_names = sig.param_names.as_ref();
    let param_defaults = sig.param_defaults.as_ref();
    for idx in 0..ordered.len() {
        if ordered[idx].is_some() {
            continue;
        }
        let default_expr = param_defaults
            .and_then(|defaults| defaults.get(idx))
            .copied()
            .flatten();
        if let Some(default_expr) = default_expr {
            ordered[idx] = Some(default_expr);
            continue;
        }
        if let Some(param_names) = param_names {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "function '{}' is missing argument for parameter '{}'",
                    resolve_symbol_name(arena, call_name)?,
                    resolve_symbol_name(arena, param_names[idx])?
                ),
            });
        }
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "function '{}' expects {} args, got {}",
                resolve_symbol_name(arena, call_name)?,
                sig.params.len(),
                provided_count
            ),
        });
    }
    Ok(ordered.into_iter().flatten().collect())
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn resolve_symbol_name<'a>(
    arena: &'a AstArena,
    id: SymbolId,
) -> Result<&'a str, FrontendError> {
    arena.try_symbol_name(id).ok_or(FrontendError {
        pos: 0,
        message: format!("invalid symbol id {}", id.0),
    })
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, PartialEq)]
pub enum AstBundle {
    RustLike(Program),
    Logos(LogosProgram),
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy)]
pub struct CompilePolicyView<'a> {
    pub profile: &'a ParserProfile,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<'a> CompilePolicyView<'a> {
    pub const fn new(profile: &'a ParserProfile) -> Self {
        Self { profile }
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_rustlike(input: &str) -> Result<AstBundle, FrontendError> {
    let profile = ParserProfile::foundation_default();
    parser::parse_rustlike_with_profile(input, &profile).map(AstBundle::RustLike)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_rustlike_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<AstBundle, FrontendError> {
    parser::parse_rustlike_with_profile(input, profile).map(AstBundle::RustLike)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_logos(input: &str) -> Result<AstBundle, FrontendError> {
    let profile = ParserProfile::foundation_default();
    parser::parse_logos_with_profile(input, &profile).map(AstBundle::Logos)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_logos_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<AstBundle, FrontendError> {
    parser::parse_logos_with_profile(input, profile).map(AstBundle::Logos)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_program(input: &str) -> Result<Program, FrontendError> {
    let profile = ParserProfile::foundation_default();
    parser::parse_rustlike_with_profile(input, &profile)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_program_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<Program, FrontendError> {
    parser::parse_rustlike_with_profile(input, profile)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_logos_program(input: &str) -> Result<LogosProgram, FrontendError> {
    let profile = ParserProfile::foundation_default();
    parser::parse_logos_with_profile(input, &profile)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn parse_logos_program_with_profile(
    input: &str,
    profile: &ParserProfile,
) -> Result<LogosProgram, FrontendError> {
    parser::parse_logos_with_profile(input, profile)
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn lex(input: &str) -> Result<Vec<Token>, FrontendError> {
    lexer::lex_tokens(input)
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompileProfile {
    Auto,
    RustLike,
    Logos,
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptLevel {
    O0,
    O1,
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn parse_rustlike_bundle() {
        let src = "fn main() { return; }";
        let ast = parse_rustlike(src).expect("parse");
        match ast {
            AstBundle::RustLike(p) => {
                assert!(p.adts.is_empty());
                assert!(p.records.is_empty());
                assert!(p.schemas.is_empty());
                assert_eq!(p.functions.len(), 1);
            }
            AstBundle::Logos(_) => panic!("expected rustlike bundle"),
        }
    }

    #[test]
    fn parse_logos_bundle() {
        let src = r#"
Law "L" [priority 1]:
    When true -> System.recovery()
"#;
        let ast = parse_logos(src).expect("parse");
        match ast {
            AstBundle::Logos(p) => assert_eq!(p.laws.len(), 1),
            AstBundle::RustLike(_) => panic!("expected logos bundle"),
        }
    }

    #[test]
    fn lex_via_frontend_crate() {
        let toks = lexer::lex_tokens("fn main() { return; }").expect("lex");
        assert!(!toks.is_empty());
    }

    #[test]
    fn build_schema_table_retains_schema_version_metadata() {
        let program = parse_program(
            r#"
api schema Telemetry version(3) {
    enabled: bool,
}

fn main() {
    return;
}
"#,
        )
        .expect("schema with version should parse");

        let table = build_schema_table(&program).expect("schema table should build");
        let schema = table
            .values()
            .next()
            .expect("canonical schema table must contain schema");
        assert_eq!(schema.role, Some(SchemaRole::Api));
        assert_eq!(schema.version, Some(SchemaVersion { value: 3 }));
    }
}
