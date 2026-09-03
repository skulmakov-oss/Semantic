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
    ///
    /// SSF-08 Lane 1 (#1664): REQUIRED_BINDING -- fails closed if `name` is
    /// not a known binding, rather than silently no-op'ing. Zero production
    /// call sites exist anywhere in the workspace today (whole-root
    /// "consumed" is inert under Position A's frozen copy-by-value plain
    /// root-rebinding semantics, see PR #1879); this PR adds none.
    pub fn mark_consumed(&mut self, name: SymbolId) -> Result<(), crate::types::FrontendError> {
        self.require_binding_mut(name)?.consumed = true;
        Ok(())
    }

    /// Returns true if the variable has been moved and is no longer available.
    ///
    /// SSF-08 Lane 1 (#1664): REQUIRED_BINDING -- fails closed if `name` is
    /// not a known binding, rather than treating "missing" as "not
    /// consumed". Zero call sites exist anywhere in the workspace today.
    pub fn is_consumed(&self, name: SymbolId) -> Result<bool, crate::types::FrontendError> {
        Ok(self.require_binding(name)?.consumed)
    }

    /// SSF-08 Lane 1: `true` if `a` denotes the same path as, or a path
    /// leading to, `b` (i.e. `a` is a prefix of `b`'s element list).
    fn path_is_prefix(a: &crate::types::PatternPath, b: &crate::types::PatternPath) -> bool {
        if a.elems.len() > b.elems.len() {
            return false;
        }
        a.elems.iter().zip(&b.elems).all(|(x, y)| x == y)
    }

    /// SSF-08 Lane 1: shared normalisation core of `mark_path_state`, factored
    /// out so the branch/loop/match join (`join_ownership_from`) can build a
    /// joined `path_state` using the exact same compaction rules, rather than
    /// re-deriving them.
    ///
    /// Rule 1 — new path subsumes longer existing entries of the same state:
    ///   e.g. adding Moved(root) while Moved(root.0) exists → drop root.0.
    /// Rule 2 — if an existing entry already covers the new path (same state,
    ///   existing is a prefix of new path), the new entry is redundant.
    ///
    /// Deliberately does not merge across *different* states: `(p, Moved)`
    /// and `(p, Borrowed)` may both be present at once. `check_path_available`
    /// / `check_capture_allowed` treat that as "restricted under either
    /// reading", which is exactly the conservative behavior a branch join
    /// needs when one successor moved `p` and another only borrowed it.
    fn push_path_state_normalized(
        path_state: &mut Vec<(crate::types::PatternPath, crate::types::PathAvailability)>,
        path: crate::types::PatternPath,
        state: crate::types::PathAvailability,
    ) {
        path_state.retain(|(existing, existing_state)| {
            if *existing_state != state {
                return true;
            }
            !Self::path_is_prefix(&path, existing)
        });
        let redundant = path_state.iter().any(|(existing, existing_state)| {
            *existing_state == state && Self::path_is_prefix(existing, &path)
        });
        if !redundant {
            path_state.push((path, state));
        }
    }

    /// SSF-08 Lane 1: look up a binding that the caller has already
    /// established must exist (e.g. because an earlier, independently
    /// successful lookup on the same `name` already proved it, or because
    /// `name` is drawn from `self`'s own known binding set). Fails closed:
    /// unlike `binding`, absence here is treated as an internal ownership
    /// state invariant failure, not a legitimate "unknown variable" case.
    /// This is deliberately a *different* error from the source-level
    /// "unknown variable 'x'" diagnostic (see `Expr::Var` handling in
    /// `typecheck.rs`) — callers that have not already independently
    /// confirmed existence must not use this.
    fn require_binding(
        &self,
        name: SymbolId,
    ) -> Result<&ScopeBinding, crate::types::FrontendError> {
        self.binding(name)
            .ok_or_else(|| crate::types::FrontendError {
                pos: 0,
                message: format!(
                    "internal ownership state: required binding {} is missing",
                    name.0
                ),
            })
    }

    /// Mutable counterpart of `require_binding`. See its doc comment for the
    /// fail-closed contract.
    fn require_binding_mut(
        &mut self,
        name: SymbolId,
    ) -> Result<&mut ScopeBinding, crate::types::FrontendError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                return Ok(scope.get_mut(&name).expect("just checked contains_key"));
            }
        }
        Err(crate::types::FrontendError {
            pos: 0,
            message: format!(
                "internal ownership state: required binding {} is missing",
                name.0
            ),
        })
    }

    /// M9.7 / SSF-08 Lane 1: Record that `path` within variable `name` has
    /// been moved or borrowed. `name` must already be a known binding —
    /// callers that have not independently established this (e.g. a bare
    /// source-level read where "unknown variable" must remain the reported
    /// diagnostic) must not call this directly; see `require_binding_mut`.
    pub fn mark_path_state(
        &mut self,
        name: SymbolId,
        path: crate::types::PatternPath,
        state: crate::types::PathAvailability,
    ) -> Result<(), crate::types::FrontendError> {
        let binding = self.require_binding_mut(name)?;
        Self::push_path_state_normalized(&mut binding.path_state, path, state);
        Ok(())
    }

    /// M9.7: Check that accessing `access_path` within `name` is allowed.
    ///
    /// Rejects if any stored path overlaps `access_path` with state `Moved`.
    /// Conservative: borrows are not currently enforced as blocking reads.
    ///
    /// SSF-08 Lane 1 (#1664): REQUIRED_BINDING -- fails closed if `name` is
    /// not a known binding (via `require_binding`), rather than treating
    /// "missing" as "available". `name` must already be established to
    /// exist by the caller; the canonical source-level "unknown variable"
    /// diagnostic must come from that caller's own resolution, not from
    /// this ownership check -- see the call site in `infer_expr_type`
    /// (`crates/sm-front/src/typecheck.rs`), which only calls this once
    /// `env.get(name)` has already confirmed the binding exists.
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

        let binding = self.require_binding(name)?;
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
        Ok(())
    }

    /// M9.8 / SSF-08 Lane 1: Check that a new capture of `path` with `capture`
    /// mode is compatible with the existing path-state of variable `name`.
    /// `name` must already be a known binding (see `require_binding`) — this
    /// is a pattern-capture check, always run against a scrutinee whose own
    /// existence was already independently confirmed by successfully
    /// typechecking the scrutinee expression before any pattern plan is
    /// built.
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
        use crate::types::{CaptureMode, PathAvailability};

        fn paths_overlap(a: &crate::types::PatternPath, b: &crate::types::PatternPath) -> bool {
            ScopeEnv::path_is_prefix(a, b) || ScopeEnv::path_is_prefix(b, a)
        }

        let binding = self.require_binding(name)?;

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

    /// SSF-08 (#1664 completion): REQUIRED_BINDING -- fails closed if
    /// `name` is not a known binding, rather than treating "missing" as
    /// "not const". This is a semantic property query against a binding
    /// the caller has already established must exist -- every current
    /// call site (`crates/sm-front/src/typecheck.rs`'s assignment/const-
    /// initializer checks, and `crates/sm-ir/src/legacy_lowering.rs`'s
    /// closure-capture/tuple-assignment/ordinary-assignment lowering)
    /// resolves the binding via `env.get(name)` -- reporting the
    /// canonical source-level "unknown ..." diagnostic on absence -- and
    /// only queries constness once that has already succeeded, so this
    /// `Err` path represents an internal invariant failure (the identity
    /// disappeared after being proven to exist), never an ordinary source
    /// mistake. This was the last of #1664's seven APIs to be converted;
    /// it originally stayed fail-open because `crates/sm-ir` (Lane 2) had
    /// live consumers of the old bool-returning shape and Lane 1's scope
    /// was frontend-only -- SSF-08's #1664 completion slice explicitly
    /// authorized migrating those three call sites so this contract
    /// correction could land.
    pub fn is_const(&self, name: SymbolId) -> Result<bool, crate::types::FrontendError> {
        Ok(self.require_binding(name)?.is_const)
    }

    /// SSF-08 Lane 1 (#1664): REQUIRED_BINDING -- fails closed if `name` is
    /// not a known binding, rather than treating "missing" as "not
    /// mutable". Zero call sites exist anywhere in the workspace today, so
    /// unlike `is_const` this signature carries no external Lane 2
    /// dependency and can change freely.
    pub fn is_mutable(&self, name: SymbolId) -> Result<bool, crate::types::FrontendError> {
        Ok(self.require_binding(name)?.is_mutable)
    }

    fn binding(&self, name: SymbolId) -> Option<&ScopeBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(&name) {
                return Some(binding);
            }
        }
        None
    }

    /// SSF-08 Lane 1: the canonical ownership-state join. `self` is the
    /// pre-branch/pre-loop-body environment; each entry in `successors` is a
    /// state reached by fully checking one reachable alternative (an `if`
    /// arm, a `match`/`if-let` arm or default, one loop-body pass), starting
    /// from a clone of `self` with any of that alternative's own local
    /// (branch-local / loop-local) scope already popped — so every successor
    /// has exactly `self`'s own scope shape (same depth, same bindings at
    /// each depth), just with pre-existing bindings' `consumed`/`path_state`
    /// possibly more restricted.
    ///
    /// Conservative join law: a path may be claimed Available in the joined
    /// result only if it is Available in *every* successor. Equivalently: a
    /// restriction (`Moved`, `Borrowed`, or whole-binding `consumed`) that
    /// holds in *any* successor is retained. Uncertainty never restores
    /// availability — this is the invariant `#1656`-`#1664` violated by
    /// discarding successor state entirely rather than joining it.
    ///
    /// `successors` being empty is a no-op (nothing reachable to join in;
    /// `self` is left as the sole known state). A caller representing "this
    /// alternative may run zero times" (a bare `if` with no `else`, a
    /// `while`/`for` loop) must include an *unchanged clone of the pre-state*
    /// as one of the successors itself — this function does not synthesize
    /// that implicitly, so it stays correct for exhaustive constructs (a
    /// `match` whose arms are already proven exhaustive) that must not gain
    /// a spurious "or nothing happened" relaxation.
    ///
    /// Fails closed (`internal ownership state: ...`) if a successor's scope
    /// shape doesn't match `self`'s, or if a name known to `self` at some
    /// depth is missing from a successor at that same depth — both indicate
    /// an internal invariant failure (mismatched push/pop, or a successor
    /// built from something other than a clone of `self`), never a
    /// legitimate "this binding doesn't apply here" case.
    pub(crate) fn join_ownership_from(
        &mut self,
        successors: &[ScopeEnv],
    ) -> Result<(), crate::types::FrontendError> {
        if successors.is_empty() {
            return Ok(());
        }
        for successor in successors {
            if successor.scopes.len() != self.scopes.len() {
                return Err(crate::types::FrontendError {
                    pos: 0,
                    message: "internal ownership state: branch successor scope depth does not match predecessor".to_string(),
                });
            }
        }
        for depth in 0..self.scopes.len() {
            // `scopes` is a `BTreeMap`, so `.keys()` is already sorted and
            // this is a no-op today -- kept explicit (rather than relying
            // on the underlying collection type) so the (should-be-
            // unreachable, invariant-defense-only) "missing from a branch
            // successor" error below always names the same binding first
            // even if that collection type ever changes; no diagnostic
            // here may be iteration-order-dependent.
            let mut names: Vec<SymbolId> = self.scopes[depth].keys().copied().collect();
            names.sort_unstable();
            for name in names {
                let mut joined_consumed = false;
                let mut joined_path_state: Vec<(
                    crate::types::PatternPath,
                    crate::types::PathAvailability,
                )> = Vec::new();
                for successor in successors {
                    let Some(binding) = successor.scopes[depth].get(&name) else {
                        return Err(crate::types::FrontendError {
                            pos: 0,
                            message: format!(
                                "internal ownership state: required binding {} is missing from a branch successor",
                                name.0
                            ),
                        });
                    };
                    joined_consumed |= binding.consumed;
                    for (path, state) in &binding.path_state {
                        Self::push_path_state_normalized(
                            &mut joined_path_state,
                            path.clone(),
                            *state,
                        );
                    }
                }
                // `name` was collected from `self.scopes[depth]` above, so
                // this lookup cannot miss.
                let binding = self.scopes[depth]
                    .get_mut(&name)
                    .expect("name was just read from this same scope map");
                binding.consumed = joined_consumed;
                binding.path_state = joined_path_state;
            }
        }
        Ok(())
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
        // FA-02-002 / #1634: the first-wave generic contract admits at most
        // one type parameter per definition site. The parser deliberately
        // has no arity limit -- `parse_type_params_with_bounds` may
        // represent `<T, U, ...>` as raw AST (see
        // generic_function_two_type_params_are_parsed /
        // function_with_multiple_type_params_mixed_bounds_is_parsed in
        // parser.rs, which pin that parsing fidelity) -- so admission is
        // enforced here, at the same table-construction boundary that
        // already owns the reserved-name and duplicate-name checks above,
        // rather than truncating to the first parameter or silently
        // admitting the extras.
        if f.type_params.len() > 1 {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "function '{name}' declares {} type parameters; first-wave generic \
                     definitions admit at most one",
                    f.type_params.len()
                ),
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
        // FA-02-018 / #1650: the current nominal Type representation
        // (Type::Record(SymbolId)) has no applied type arguments, and no
        // source syntax exists to write one (parse_type's `Foo`/`Foo(Args)`
        // dispatch is hardcoded per builtin family -- Option/Result/
        // Sequence/Map/Closure -- with no general nominal-application rule
        // for a user-declared name; a bare declared name always parses as
        // an unparameterized Type::Record). A record declaring type
        // parameters therefore has no faithful concrete type identity any
        // use site could ever construct: canonicalize_declared_type (the
        // non-generic canonicalizer every record-literal/field-access call
        // site already uses) unconditionally rejects a declaration
        // TypeVar, so every construction attempt already fails today --
        // just not at the declaration boundary, and not with an honest
        // diagnostic. Reject the declaration itself, superseding #1634's
        // narrower ">1" arity check with this stricter zero-arity contract
        // (an admitted first-wave generic record is not merely
        // arity-bounded, it does not exist yet at all), mirroring the
        // zero-arity precedent #1635 established for traits.
        if !record.type_params.is_empty() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "generic record '{}' is not part of the current Stable Foundation \
                     nominal type contract because applied record type arguments are \
                     not representable",
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
        // FA-02-018 / #1650: identical rationale to build_record_table
        // above -- Type::Adt(SymbolId) has no applied type arguments, no
        // source application syntax exists, and canonicalize_declared_type
        // already unconditionally rejects a payload TypeVar at every
        // constructor call site, so a generic ADT's declaration is
        // admitted while every construction already fails. Supersedes
        // #1634's ">1" check with this stricter zero-arity contract.
        if !adt.type_params.is_empty() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "generic enum '{}' is not part of the current Stable Foundation \
                     nominal type contract because applied enum type arguments are \
                     not representable",
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
    // FA-02-037 / #1669: TraitTable is the owner-layer canonical trait
    // contract (the same architectural role FnSig/FnTable plays for
    // functions), not an unchecked cache of raw parsed declarations. Every
    // non-Self method-signature type must resolve against the same
    // canonical nominal authority every other declared-type position in the
    // frontend uses (see build_fn_table above), before this trait is
    // admitted -- independent of whether any impl ever exists. Built
    // locally (mirroring build_fn_table) rather than accepting
    // caller-supplied tables, so this invariant holds for every public
    // caller of build_trait_table, not only type_check_program's.
    let record_table = build_record_table(program)?;
    let adt_table = build_adt_table(program)?;
    // The sole admitted type variable in a trait method signature is the
    // trait-side `Self` placeholder (Type::TypeVar, interned once per
    // parsed trait). A trait's own declared type_params are never admitted
    // here -- generic traits are rejected outright below, before any
    // method signature is ever canonicalized (FA-02-003 / #1635), so this
    // list existing only as a defensive default: it must never be widened
    // to include a trait's own type_params, since that would make an
    // in-scope reference to them the one path that could smuggle a generic
    // trait past the declaration-level rejection below.
    let self_type_var = program.arena.symbol_to_id.get("Self").copied();
    let admitted_type_vars: Vec<SymbolId> = self_type_var.into_iter().collect();
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
        // FA-02-003 / #1635: TraitDecl.type_params is documented as "Empty
        // in first-wave canonical form" (types.rs), but nothing previously
        // enforced that -- a trait declaring type parameters, used or not,
        // silently reached TraitTable. Reject before any method-signature
        // work begins, independent of whether an impl exists or whether
        // the parameters are referenced, mirroring the sibling impl-side
        // enforcement in validate_trait_coherence (#1668) rather than
        // erasing the parameters and treating the trait as non-generic.
        if !t.type_params.is_empty() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "trait '{}' declares type parameters; generic traits are not supported",
                    resolve_symbol_name(&program.arena, t.name)?
                ),
            });
        }
        let methods = t
            .methods
            .iter()
            .map(|m| {
                let params = m
                    .params
                    .iter()
                    .map(|(name, ty)| {
                        let canonical = canonicalize_declared_type_generic(
                            ty,
                            &record_table,
                            &adt_table,
                            &program.arena,
                            &admitted_type_vars,
                        )?;
                        // FA-02-015 / #1647: TraitTable admission proves both
                        // canonical nominal identity AND that the canonical
                        // type is on the current executable-admitted
                        // surface -- never store first and validate later.
                        typecheck::ensure_executable_type_supported(
                            &canonical,
                            &program.arena,
                            &admitted_type_vars,
                            format!(
                                "parameter '{}' of trait method '{}'",
                                resolve_symbol_name(&program.arena, *name)?,
                                resolve_symbol_name(&program.arena, m.name)?
                            ),
                        )?;
                        Ok((*name, canonical))
                    })
                    .collect::<Result<Vec<_>, FrontendError>>()?;
                let ret = canonicalize_declared_type_generic(
                    &m.ret,
                    &record_table,
                    &adt_table,
                    &program.arena,
                    &admitted_type_vars,
                )?;
                typecheck::ensure_executable_type_supported(
                    &ret,
                    &program.arena,
                    &admitted_type_vars,
                    format!(
                        "return type of trait method '{}'",
                        resolve_symbol_name(&program.arena, m.name)?
                    ),
                )?;
                Ok(TraitMethodSig {
                    name: m.name,
                    params,
                    ret,
                })
            })
            .collect::<Result<Vec<_>, FrontendError>>()?;
        out.insert(
            t.name,
            TraitDecl {
                name: t.name,
                type_params: t.type_params.clone(),
                methods,
            },
        );
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedCallArgs {
    /// Final argument expression for each declared parameter slot, in
    /// parameter order (defaults substituted). Used for per-slot type
    /// checking and for the final argument-register order handed to the
    /// callee.
    pub slots: Vec<ExprId>,
    /// Parameter slot indices in the order their expressions must be
    /// *evaluated*: explicitly supplied arguments in source (left-to-right)
    /// order, followed by defaulted slots in parameter order.
    ///
    /// Named-argument slot assignment and source evaluation order are
    /// separate concerns: named arguments only ever reorder `slots`, never
    /// `eval_order`'s relative order of explicitly written argument
    /// expressions.
    pub eval_order: Vec<usize>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn validate_fn_sig_call_metadata(
    call_name: SymbolId,
    sig: &FnSig,
    arena: &AstArena,
) -> Result<(), FrontendError> {
    if let Some(param_names) = sig.param_names.as_ref() {
        if param_names.len() != sig.params.len() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "function '{}' has malformed signature: {} parameter name(s) declared for {} parameter(s)",
                    resolve_symbol_name(arena, call_name)?,
                    param_names.len(),
                    sig.params.len(),
                ),
            });
        }
    }
    if let Some(param_defaults) = sig.param_defaults.as_ref() {
        if param_defaults.len() != sig.params.len() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "function '{}' has malformed signature: {} parameter default(s) declared for {} parameter(s)",
                    resolve_symbol_name(arena, call_name)?,
                    param_defaults.len(),
                    sig.params.len(),
                ),
            });
        }
    }
    Ok(())
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub fn reorder_call_args(
    call_name: SymbolId,
    args: &[CallArg],
    sig: &FnSig,
    arena: &AstArena,
) -> Result<OrderedCallArgs, FrontendError> {
    validate_fn_sig_call_metadata(call_name, sig, arena)?;

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
        let mut eval_order = Vec::with_capacity(args.len());
        for (idx, arg) in args.iter().enumerate() {
            ordered[idx] = Some(arg.value);
            eval_order.push(idx);
        }
        return finalize_ordered_call_args(call_name, ordered, sig, arena, args.len(), eval_order);
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
    let mut eval_order = Vec::with_capacity(args.len());
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
            eval_order.push(param_index);
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
            eval_order.push(positional_index);
            positional_index += 1;
        }
    }

    finalize_ordered_call_args(call_name, ordered, sig, arena, args.len(), eval_order)
}

#[cfg(any(feature = "alloc", feature = "std"))]
fn finalize_ordered_call_args(
    call_name: SymbolId,
    mut ordered: Vec<Option<ExprId>>,
    sig: &FnSig,
    arena: &AstArena,
    provided_count: usize,
    mut eval_order: Vec<usize>,
) -> Result<OrderedCallArgs, FrontendError> {
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
            eval_order.push(idx);
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
    Ok(OrderedCallArgs {
        slots: ordered.into_iter().flatten().collect(),
        eval_order,
    })
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
    fn build_fn_table_admits_math_builtin_name_as_user_function() {
        // #1653/#1750 (umbrella #1617): user-defined math functions now
        // follow the same user-first resolution rule as every other name
        // (see crates/sm-vm/src/semcode_vm.rs's Opcode::Call dispatch) —
        // they are no longer rejected at the frontend boundary.
        for name in ["sin", "cos", "tan", "sqrt", "abs", "pow"] {
            let src = format!(
                r#"
fn {name}(x: f64) -> f64 {{
    return 42.0;
}}
fn main() {{
    return;
}}
"#
            );
            let program = match parse_rustlike(&src).expect("parse") {
                AstBundle::RustLike(p) => p,
                AstBundle::Logos(_) => panic!("expected rustlike bundle"),
            };
            build_fn_table(&program).unwrap_or_else(|err| {
                panic!("user function named '{name}' must be admitted: {err:?}")
            });
        }
    }

    #[test]
    fn build_fn_table_rejects_application_builtin_name_collision() {
        let src = r#"
fn stdout_write(text: text) {
    return;
}
fn main() {
    return;
}
"#;
        let program = match parse_rustlike(src).expect("parse") {
            AstBundle::RustLike(p) => p,
            AstBundle::Logos(_) => panic!("expected rustlike bundle"),
        };
        let err = build_fn_table(&program)
            .expect_err("user function named stdout_write must be rejected");
        assert!(
            err.message.contains("stdout_write") && err.message.contains("reserved"),
            "unexpected error: {}",
            err.message
        );
    }

    // FA-02-002 / #1634: first-wave generic-capable definitions admit at
    // most one type parameter. The parser deliberately keeps no arity limit
    // (see generic_function_two_type_params_are_parsed and
    // function_with_multiple_type_params_mixed_bounds_is_parsed in
    // parser.rs, both of which must remain green -- raw AST fidelity is
    // unaffected), so the boundary lives at the same table-construction
    // authority already used for the reserved/duplicate-name checks above.

    #[test]
    fn build_fn_table_admits_single_type_param_function() {
        // Positive control: unaffected by this slice's change.
        let program = parse_program(
            r#"
                fn id<T>(x: T) -> T {
                    return x;
                }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        build_fn_table(&program).expect("single type parameter must remain admitted");
    }

    #[test]
    fn build_fn_table_preserves_bound_on_single_type_param_function() {
        let program = parse_program(
            r#"
                trait Zeroable {
                    fn zero(v: ZeroInt) -> i32;
                }
                record ZeroInt { n: i32 }
                fn make_zero<T: Zeroable>(v: T) -> T {
                    return v;
                }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let table = build_fn_table(&program).expect("bounded single-param function must admit");
        let fn_id = *program
            .arena
            .symbol_to_id
            .get("make_zero")
            .expect("make_zero interned");
        let sig = table.get(&fn_id).expect("signature present");
        assert_eq!(sig.type_params.len(), 1);
        assert_eq!(sig.trait_bounds.len(), 1);
    }

    #[test]
    fn build_fn_table_rejects_function_with_two_type_params() {
        // Central regression, per #1634's own reproducer.
        let program = parse_program(
            r#"
                fn pair<T, U>(x: T, y: U) -> T {
                    return x;
                }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_fn_table(&program)
            .expect_err("a function declaring two type parameters must reject");
        assert!(
            err.message.contains("pair")
                && err.message.contains("2 type parameters")
                && err.message.contains("at most one"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_fn_table_rejects_bound_on_first_of_two_type_params() {
        // Mirrors function_with_multiple_type_params_mixed_bounds_is_parsed
        // in parser.rs (`<T: Eq, U>`) -- the parser still represents this
        // faithfully; canonical admission must still reject it, and must
        // not partially preserve the bounded parameter while discarding U.
        let program = parse_program(
            r#"
                trait Eq2 {
                    fn eq2(v: Self) -> i32;
                }
                fn apply<T: Eq2, U>(x: T, y: U) -> i32 {
                    return 0;
                }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_fn_table(&program).expect_err(
            "a function declaring two type parameters must reject even when only one is bounded",
        );
        assert!(
            err.message.contains("apply") && err.message.contains("at most one"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_fn_table_rejects_bound_on_second_of_two_type_params() {
        // `<T, U: Bound>` -- the opposite ordering.
        let program = parse_program(
            r#"
                trait Eq2 {
                    fn eq2(v: Self) -> i32;
                }
                fn apply<T, U: Eq2>(x: T, y: U) -> i32 {
                    return 0;
                }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_fn_table(&program)
            .expect_err("a function declaring two type parameters must reject regardless of which one is bounded");
        assert!(
            err.message.contains("apply") && err.message.contains("at most one"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_fn_table_rejects_duplicate_type_param_name_via_arity() {
        // (Duplicate-name classification, #1634 section 14): `<T, T>` is
        // arity 2 regardless of the repeated name, so it is rejected by
        // this same check. This is incidental arity coverage, not a
        // dedicated duplicate-name repair -- no separate uniqueness check
        // is added here.
        let program = parse_program(
            r#"
                fn f<T, T>(x: T) -> T {
                    return x;
                }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_fn_table(&program).expect_err("<T, T> must reject as a two-parameter list");
        assert!(
            err.message.contains("at most one"),
            "unexpected error: {}",
            err.message
        );
    }

    // FA-02-018 / #1650: generic Record/ADT declarations have no faithful
    // applied nominal type identity (Type::Record/Type::Adt carry no type
    // arguments, and no source application syntax exists), so first-wave
    // Stable Foundation nominal admission requires zero type parameters --
    // stricter than #1634's "at most one" arity bound, which remains the
    // current first-wave contract for generic functions; traits and impls
    // already require zero type parameters under their separate
    // owner-layer contracts (#1635, #1668). Raw parser fidelity is
    // preserved (see generic_record_type_params_are_parsed_and_stored /
    // generic_enum_type_params_are_parsed_and_stored in parser.rs, both
    // unmodified and still green): only canonical table-construction
    // admission is narrowed, mirroring the #1635 trait precedent.

    #[test]
    fn build_record_table_rejects_generic_record_with_single_type_param() {
        // Central #1650 regression: a single, otherwise first-wave-legal
        // type parameter is still rejected -- generic record admission is
        // zero-arity, not merely bounded by #1634's bare arity limit.
        let program = parse_program(
            r#"
                record Box<T> { value: T }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_record_table(&program)
            .expect_err("a generic record must not be admitted -- no applied type identity exists");
        assert!(
            err.message.contains("Box")
                && err
                    .message
                    .contains("not part of the current Stable Foundation")
                && err
                    .message
                    .contains("applied record type arguments are not representable"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_record_table_rejects_two_type_params() {
        let program = parse_program(
            r#"
                record Pair<T, U> { a: T, b: U }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_record_table(&program)
            .expect_err("a record declaring two type parameters must reject");
        assert!(
            err.message.contains("Pair")
                && err
                    .message
                    .contains("not part of the current Stable Foundation"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_adt_table_rejects_generic_adt_with_single_type_param() {
        // Central #1650 regression, ADT side.
        let program = parse_program(
            r#"
                enum Maybe<T> { Some(T), None }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_adt_table(&program)
            .expect_err("a generic enum must not be admitted -- no applied type identity exists");
        assert!(
            err.message.contains("Maybe")
                && err
                    .message
                    .contains("not part of the current Stable Foundation")
                && err
                    .message
                    .contains("applied enum type arguments are not representable"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_adt_table_rejects_two_type_params() {
        let program = parse_program(
            r#"
                enum Either<T, U> { Left(T), Right(U) }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_adt_table(&program)
            .expect_err("an enum declaring two type parameters must reject");
        assert!(
            err.message.contains("Either")
                && err
                    .message
                    .contains("not part of the current Stable Foundation"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_rejects_trait_with_two_type_params() {
        // #1634 classification evidence: already rejected by #1635's
        // stricter zero-arity trait contract, independent of this slice's
        // new max-one check -- not a #1634 repair target for traits.
        let program = parse_program(
            r#"
                trait Foo<X, Y> {
                    fn a(v: Self) -> i32;
                }
                fn main() { return; }
            "#,
        )
        .expect("parse");
        let err = build_trait_table(&program)
            .expect_err("a two-parameter generic trait must still reject via #1635's contract");
        assert!(
            err.message.contains("Foo") && err.message.contains("generic traits are not supported"),
            "unexpected error: {}",
            err.message
        );
    }

    // FA-02-037 / #1669: TraitTable is the owner-layer canonical trait
    // contract, not an unchecked cache of raw parsed declarations. Every
    // non-Self method-signature type must resolve against the same
    // canonical RecordTable/AdtTable authority build_fn_table already uses,
    // independent of whether any impl exists. See build_trait_table.

    #[test]
    fn build_trait_table_rejects_unknown_parameter_type() {
        // (A) No impl anywhere -- the trait contract must prove itself.
        let src = r#"
            trait Broken {
                fn f(x: MissingType) -> i32;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("unknown parameter type must reject with no impl present");
        assert!(
            err.message.contains("unknown nominal type 'MissingType'"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_rejects_unknown_return_type() {
        // (B)
        let src = r#"
            trait Broken {
                fn f() -> MissingType;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("unknown return type must reject with no impl present");
        assert!(
            err.message.contains("unknown nominal type 'MissingType'"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_rejects_nested_unknown_type() {
        // (C) At least one compound position containing the unknown name.
        let src = r#"
            trait Broken {
                fn f(x: Option(MissingType)) -> i32;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("unknown type nested inside Option must reject with no impl present");
        assert!(
            err.message.contains("unknown nominal type 'MissingType'"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_canonicalizes_record_signature_to_record_type() {
        // (D) The stored contract must be canonical, not the raw parsed
        // placeholder -- inspected directly, per #1669's requirement.
        let src = r#"
            record R { n: i32 }
            trait Contract {
                fn a(x: R) -> R;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let r_id = *program.arena.symbol_to_id.get("R").expect("R symbol");
        let table = build_trait_table(&program).expect("record signature should admit");
        let t_id = *program
            .arena
            .symbol_to_id
            .get("Contract")
            .expect("Contract symbol");
        let decl = &table[&t_id];
        assert_eq!(decl.methods[0].params[0].1, Type::Record(r_id));
        assert_eq!(decl.methods[0].ret, Type::Record(r_id));
    }

    #[test]
    fn build_trait_table_canonicalizes_adt_signature_to_adt_type() {
        // (E) Record-vs-Adt identity must be uniform with the merged
        // #1667/#1651 impl-side repair: an enum target must never be
        // stored as an unresolved/guessed Record.
        let src = r#"
            enum E { A, B }
            trait Contract {
                fn a(x: E) -> E;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let e_id = *program.arena.symbol_to_id.get("E").expect("E symbol");
        let table = build_trait_table(&program).expect("enum signature should admit");
        let t_id = *program
            .arena
            .symbol_to_id
            .get("Contract")
            .expect("Contract symbol");
        let decl = &table[&t_id];
        assert_eq!(decl.methods[0].params[0].1, Type::Adt(e_id));
        assert_eq!(decl.methods[0].ret, Type::Adt(e_id));
    }

    #[test]
    fn build_trait_table_admits_reserved_self_in_direct_and_nested_source_positions() {
        // (F) direct Self, tuple, Sequence, Option, Result(Self, ..),
        // Result(.., Self) -- all writable as explicit source type syntax.
        // Closure param/return positions are covered separately below via a
        // direct canonicalize_declared_type_generic call: this first-wave
        // surface has no source syntax for writing an explicit closure
        // type annotation.
        let cases = [
            "fn a(x: Self) -> Self;",
            "fn a(x: (Self, i32)) -> i32;",
            "fn a(x: Sequence(Self)) -> i32;",
            "fn a(x: Option(Self)) -> i32;",
            "fn a(x: Result(Self, i32)) -> i32;",
            "fn a(x: Result(i32, Self)) -> i32;",
        ];
        for method_sig in cases {
            let src = format!(
                r#"
                    trait Contract {{
                        {method_sig}
                    }}
                    fn main() {{
                        return;
                    }}
                "#
            );
            let program = parse_program(&src).expect("parse");
            build_trait_table(&program)
                .unwrap_or_else(|err| panic!("'{method_sig}' must admit Self: {err:?}"));
        }
    }

    #[test]
    fn canonicalize_declared_type_generic_admits_self_in_closure_positions() {
        // (F) closure parameter/return continuation of the above, using
        // build_trait_table's exact admission mechanism directly (no
        // source syntax exists for an explicit closure type annotation).
        let src = r#"
            trait Contract {
                fn a(x: Self) -> Self;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let self_id = *program
            .arena
            .symbol_to_id
            .get("Self")
            .expect("Self interned");
        let record_table = build_record_table(&program).expect("record table");
        let adt_table = build_adt_table(&program).expect("adt table");
        let admitted = [self_id];

        let closure_param_self = Type::Closure(ClosureType {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param: Box::new(Type::TypeVar(self_id)),
            ret: Box::new(Type::I32),
        });
        let resolved = canonicalize_declared_type_generic(
            &closure_param_self,
            &record_table,
            &adt_table,
            &program.arena,
            &admitted,
        )
        .expect("closure parameter containing Self must admit");
        assert_eq!(resolved, closure_param_self);

        let closure_ret_self = Type::Closure(ClosureType {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param: Box::new(Type::I32),
            ret: Box::new(Type::TypeVar(self_id)),
        });
        let resolved = canonicalize_declared_type_generic(
            &closure_ret_self,
            &record_table,
            &adt_table,
            &program.arena,
            &admitted,
        )
        .expect("closure return containing Self must admit");
        assert_eq!(resolved, closure_ret_self);
    }

    #[test]
    fn build_trait_table_does_not_treat_unrelated_nominal_type_as_self() {
        // (G) A record parameter that is not Self must canonicalize to its
        // own identity, never be conflated with the reserved placeholder.
        let src = r#"
            record R { n: i32 }
            trait Contract {
                fn a(x: R, y: Self) -> Self;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let r_id = *program.arena.symbol_to_id.get("R").expect("R symbol");
        let self_id = *program
            .arena
            .symbol_to_id
            .get("Self")
            .expect("Self interned");
        let table = build_trait_table(&program).expect("mixed signature should admit");
        let t_id = *program
            .arena
            .symbol_to_id
            .get("Contract")
            .expect("Contract symbol");
        let decl = &table[&t_id];
        assert_eq!(decl.methods[0].params[0].1, Type::Record(r_id));
        assert_eq!(decl.methods[0].params[1].1, Type::TypeVar(self_id));
        assert_ne!(r_id, self_id);
    }

    #[test]
    fn build_trait_table_rejects_ambiguous_nominal_identity_with_no_impl() {
        // (H) The resolver must fail closed on ambiguity itself, not rely
        // on validate_top_level_name_collisions (a later, unrelated pass)
        // to incidentally catch it -- and this must hold with zero impls,
        // proving trait admission is the independent authority.
        let src = r#"
            record Dup { n: i32 }
            enum Dup { A, B }
            trait Contract {
                fn a(x: Dup) -> i32;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("ambiguous record/enum identity in a trait signature must reject");
        assert!(
            err.message
                .contains("ambiguously declared as both record and enum"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_does_not_admit_a_trait_own_type_parameter_as_a_signature_type() {
        // Updated for FA-02-003 / #1635: this case -- a trait method
        // signature actually referencing the trait's own declared type
        // parameter -- is necessarily a non-empty-type_params trait, so it
        // is now rejected by the deliberate declaration-level generic-trait
        // check before method-signature canonicalization ever runs, not by
        // the incidental "type variable is not in scope" path this test
        // originally observed under #1858 (when build_trait_table did not
        // yet reject generic traits at all). The admitted_type_vars
        // TypeVar-scope path documented in build_trait_table remains a
        // defensive default for this same reason, not the active owner of
        // this rejection.
        let src = r#"
            trait Foo<TP> {
                fn bar(x: TP) -> TP;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("a trait declaring a type parameter must reject as a generic trait");
        assert!(
            err.message.contains("Foo") && err.message.contains("generic traits are not supported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_rejects_direct_qvec_signature() {
        // FA-02-015 / #1647, corrected: a post-review contract audit found
        // that #1669's "unsupported type must reject" requirement depends
        // on ensure_executable_type_supported actually being exhaustive.
        // build_trait_table now consumes that authoritative check after
        // canonicalizing, so a reserved, not-yet-promoted-to-executable
        // QVec signature must reject at trait admission, exactly like an
        // unknown nominal type does -- never silently stored.
        let src = r#"
            trait Marker {
                fn f(x: qvec[8]) -> i32;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("a reserved qvec trait signature must reject at admission");
        assert!(
            err.message.contains("qvec is a reserved type"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_rejects_nested_qvec_signature() {
        // Recursive enforcement: an unsupported family nested inside an
        // otherwise-admitted composite must also reject.
        let src = r#"
            trait Marker {
                fn f(x: Option(qvec[8])) -> i32;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("qvec nested inside Option must reject at admission");
        assert!(
            err.message.contains("qvec is a reserved type"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_rejects_generic_trait_with_unused_type_parameter() {
        // FA-02-003 / #1635: the central regression. `T` is never
        // referenced by any method signature, so this proves rejection is
        // driven by TraitDecl declaration admission itself (non-empty
        // type_params), not by incidental method-signature TypeVar-scope
        // validation -- and with zero impls anywhere, proving trait
        // declaration validity does not depend on an implementation
        // existing.
        let src = r#"
            trait GenericTrait<T> {
                fn value(self: Self) -> i32;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("a generic trait with an unused type parameter must reject");
        assert!(
            err.message.contains("GenericTrait")
                && err.message.contains("generic traits are not supported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_rejects_generic_trait_naming_the_offending_trait() {
        // (E) Stored-state invariant, proven at the whole-program level: a
        // successful TraitTable can never contain an entry whose
        // type_params is non-empty, because build_trait_table rejects the
        // owning trait before any entry is inserted -- even alongside an
        // otherwise-admitted non-generic trait declared first.
        let src = r#"
            trait Contract {
                fn a(x: Self) -> Self;
            }
            trait GenericTrait<T> {
                fn value(self: Self) -> i32;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let err = build_trait_table(&program)
            .expect_err("a generic trait anywhere in the program must reject the whole table");
        assert!(
            err.message.contains("GenericTrait")
                && err.message.contains("generic traits are not supported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn build_trait_table_admits_non_generic_first_wave_trait() {
        // (C) Positive control: an ordinary first-wave trait with an empty
        // type_params list must continue to admit successfully.
        let src = r#"
            trait Contract {
                fn value(self: Self) -> i32;
            }
            fn main() {
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let table = build_trait_table(&program).expect("non-generic first-wave trait must admit");
        let contract_id = *program
            .arena
            .symbol_to_id
            .get("Contract")
            .expect("Contract interned");
        let decl = table
            .get(&contract_id)
            .expect("Contract must be present in TraitTable");
        assert!(
            decl.type_params.is_empty(),
            "non-generic trait must be stored with empty type_params"
        );
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
