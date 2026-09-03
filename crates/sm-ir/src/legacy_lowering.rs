use super::*;
use crate::semcode_decode::MAX_SIGNATURE_PARAMETERS_PER_FUNCTION;
use crate::semcode_format::{
    header_spec_from_magic, write_f64_le, write_i32_le, write_u16_le, write_u32_le,
    CallableValueFamily, Opcode, MAGIC0, MAGIC1, MAGIC10, MAGIC11, MAGIC12, MAGIC13, MAGIC14,
    MAGIC15, MAGIC16, MAGIC17, MAGIC18, MAGIC19, MAGIC2, MAGIC3, MAGIC4, MAGIC5, MAGIC6, MAGIC7,
    MAGIC8, MAGIC9, OWNERSHIP_EVENT_KIND_BORROW, OWNERSHIP_EVENT_KIND_WRITE,
    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL, OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX,
    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX, OWNERSHIP_SECTION_TAG, SEMCODE_SIGNATURE_MIN_REVISION,
    SIGNATURE_SECTION_TAG,
};
use sm_front::types::{
    AdtCtorExpr, ClosureCapturePolicy, ClosureLiteral, ClosureType, ClosureValueFamily,
    MatchPattern, NumericLiteral, RecordPatternItem, RecordPatternTarget, SequenceCollectionFamily,
    SequenceType,
};
use sm_front::{CallArg, LoopExpr, TuplePatternItem};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub enum IrInstr {
    Label {
        name: String,
    },
    LoadQ {
        dst: u16,
        val: QuadVal,
    },
    LoadBool {
        dst: u16,
        val: bool,
    },
    LoadI32 {
        dst: u16,
        val: i32,
    },
    LoadU32 {
        dst: u16,
        val: u32,
    },
    LoadF64 {
        dst: u16,
        val: f64,
    },
    LoadFx {
        dst: u16,
        val: i32,
    },
    LoadText {
        dst: u16,
        val: String,
    },
    ConcatText {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    MakeSequence {
        dst: u16,
        items: Vec<u16>,
    },
    SequenceLen {
        dst: u16,
        src: u16,
    },
    SequenceIsEmpty {
        dst: u16,
        src: u16,
    },
    SequenceContains {
        dst: u16,
        seq: u16,
        val: u16,
    },
    SequencePush {
        dst: u16,
        seq: u16,
        val: u16,
    },
    SequencePrepend {
        dst: u16,
        seq: u16,
        val: u16,
    },
    SequencePop {
        dst: u16,
        src: u16,
    },
    MapEmpty {
        dst: u16,
    },
    MapContains {
        dst: u16,
        map: u16,
        key: u16,
    },
    MapGet {
        dst: u16,
        map: u16,
        key: u16,
        default_val: u16,
    },
    MapSet {
        dst: u16,
        map: u16,
        key: u16,
        val: u16,
    },
    RngSeed {
        dst: u16,
        seed: u16,
    },
    RngNextI32 {
        dst: u16,
        lo: u16,
        hi: u16,
    },
    MakeClosure {
        dst: u16,
        name: String,
        captures: Vec<u16>,
    },
    SequenceGet {
        dst: u16,
        src: u16,
        index: u16,
    },
    ClosureCall {
        dst: Option<u16>,
        closure: u16,
        arg: u16,
    },
    AddFx {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    SubFx {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    MulFx {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    DivFx {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    MakeTuple {
        dst: u16,
        items: Vec<u16>,
    },
    MakeRecord {
        dst: u16,
        name: String,
        items: Vec<u16>,
    },
    MakeAdt {
        dst: u16,
        adt_name: String,
        variant_name: String,
        tag: u16,
        items: Vec<u16>,
    },
    AdtTag {
        dst: u16,
        src: u16,
        adt_name: String,
    },
    AdtGet {
        dst: u16,
        src: u16,
        adt_name: String,
        index: u16,
    },
    RecordGet {
        dst: u16,
        src: u16,
        record_name: String,
        index: u16,
    },
    TupleGet {
        dst: u16,
        src: u16,
        index: u16,
    },
    LoadVar {
        dst: u16,
        name: String,
    },
    StoreVar {
        name: String,
        src: u16,
    },
    QAnd {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    QOr {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    QNot {
        dst: u16,
        src: u16,
    },
    QImpl {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    QTruthAnd {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    QTruthOr {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    QTruthNot {
        dst: u16,
        src: u16,
    },
    QTruthImpl {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    BoolAnd {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    BoolOr {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    BoolNot {
        dst: u16,
        src: u16,
    },
    CmpEq {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    CmpNe {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    CmpI32Lt {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    CmpI32Le {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    AddI32 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    SubI32 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    MulI32 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    DivI32 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    ModI32 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    AddF64 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    SubF64 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    MulF64 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    DivF64 {
        dst: u16,
        lhs: u16,
        rhs: u16,
    },
    Jmp {
        label: String,
    },
    JmpIf {
        cond: u16,
        label: String,
    },
    Assert {
        cond: u16,
    },
    Call {
        dst: Option<u16>,
        name: String,
        args: Vec<u16>,
    },
    GateRead {
        dst: u16,
        device_id: u16,
        port: u16,
    },
    GateWrite {
        device_id: u16,
        port: u16,
        src: u16,
    },
    PulseEmit {
        signal: String,
    },
    StateQuery {
        dst: u16,
        key: String,
    },
    StateUpdate {
        key: String,
        src: u16,
    },
    EventPost {
        signal: String,
    },
    ClockRead {
        dst: u16,
    },
    Ret {
        src: Option<u16>,
    },
}

/// Canonical execution-layer access path for ownership transport.
///
/// This is intentionally IR-owned and append-only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessPath {
    pub root: SymbolId,
    pub components: Vec<PathComponent>,
}

impl AccessPath {
    pub fn new(root: SymbolId) -> Self {
        Self {
            root,
            components: Vec::new(),
        }
    }

    pub fn tuple_index(&self, index: u16) -> Self {
        let mut components = self.components.clone();
        components.push(PathComponent::TupleIndex(index));
        Self {
            root: self.root,
            components,
        }
    }

    pub fn sequence_index_static(&self, index: u32) -> Self {
        let mut components = self.components.clone();
        components.push(PathComponent::SequenceIndexStatic(index));
        Self {
            root: self.root,
            components,
        }
    }

    pub fn field(&self, name: SymbolId) -> Self {
        let mut components = self.components.clone();
        components.push(PathComponent::Field(name));
        Self {
            root: self.root,
            components,
        }
    }

    pub fn adt_payload(&self, variant: SymbolId, index: u16) -> Self {
        let mut components = self.components.clone();
        components.push(PathComponent::AdtPayload { variant, index });
        Self {
            root: self.root,
            components,
        }
    }
}

#[derive(Clone)]
enum SequenceOwnershipPath {
    Exact(AccessPath),
    DynamicFallback(AccessPath),
}

impl SequenceOwnershipPath {
    fn as_path(&self) -> &AccessPath {
        match self {
            Self::Exact(path) | Self::DynamicFallback(path) => path,
        }
    }

    fn is_dynamic_fallback(&self) -> bool {
        matches!(self, Self::DynamicFallback(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathComponent {
    TupleIndex(u16),
    SequenceIndexStatic(u32),
    Field(SymbolId),
    AdtPayload { variant: SymbolId, index: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipPathEventKind {
    Borrow,
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipPathEvent {
    pub kind: OwnershipPathEventKind,
    pub path: AccessPath,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrFunction {
    pub name: String,
    pub instrs: Vec<IrInstr>,
    pub ownership_events: Vec<OwnershipPathEvent>,
    /// The canonical callable-signature record for this function (#1773 /
    /// FA-09-005): one executable runtime family per parameter, in
    /// declaration order. This is the source of truth `emit_semcode_function`
    /// writes into each function's `SIG0` wire section - see
    /// `callable_family_for_type` for the source-`Type` to
    /// `CallableValueFamily` mapping this is derived from.
    pub params: Vec<CallableValueFamily>,
}

/// Maps a canonicalized source `Type` to the executable runtime family a
/// value of that type actually has (#1773 / FA-09-005 approved architecture
/// decision). This is the callable-signature type-family checkpoint made
/// executable: every variant is handled explicitly (no wildcard arm), so a
/// future `Type` variant added without updating this function is a
/// compile-time error, not a silent gap.
///
/// `Type::QVec` is the one currently-legal source parameter type with no
/// sound executable family - real, parser-writable, typechecks, and can
/// lower when unused, but `sm-vm::Value` has no corresponding variant and no
/// lowering path anywhere ever constructs one (see the #1773 architecture
/// checkpoint and its QVec follow-up, both posted to issue #1773). Per the
/// owner's decision, this is a deterministic *emission/compilation* failure,
/// not a wire-format tag and not deferred to VM execution.
///
/// `Type::RangeI32` and `Type::TypeVar` can never reach this function as a
/// resolved, written parameter type (see the same checkpoint), so their
/// arms exist only for exhaustiveness and return a descriptive error rather
/// than `unreachable!()`, matching this codebase's "no fallback" discipline
/// if that invariant is ever violated.
fn callable_family_for_type(ty: &Type) -> Result<CallableValueFamily, FrontendError> {
    match ty {
        Type::Quad => Ok(CallableValueFamily::Quad),
        Type::Bool => Ok(CallableValueFamily::Bool),
        Type::Text => Ok(CallableValueFamily::Text),
        Type::Sequence(_) => Ok(CallableValueFamily::Sequence),
        Type::Map(_) => Ok(CallableValueFamily::Map),
        Type::Closure(_) => Ok(CallableValueFamily::Closure),
        Type::I32 => Ok(CallableValueFamily::I32),
        Type::U32 => Ok(CallableValueFamily::U32),
        Type::Fx => Ok(CallableValueFamily::Fx),
        Type::F64 => Ok(CallableValueFamily::F64),
        // #1773: a unit annotation is compile-time-only (parser restricts it
        // to i32/u32/f64/fx bases; `erase_units()` strips it recursively;
        // `legacy_lowering` has zero `Type::Measured`-specific handling
        // anywhere else) - the runtime family is the erased base's family.
        Type::Measured(base, _) => callable_family_for_type(base),
        Type::Tuple(_) => Ok(CallableValueFamily::Tuple),
        // #1773: both lower via `IrInstr::AdtTag`, the same instruction
        // `Type::Adt` uses (legacy_lowering.rs match arms grouping
        // `Type::Adt(_) | Type::Option(_) | Type::Result(_, _)` together) -
        // the runtime representation genuinely is `Value::Adt` for all three.
        Type::Option(_) => Ok(CallableValueFamily::Adt),
        Type::Result(_, _) => Ok(CallableValueFamily::Adt),
        Type::Record(_) => Ok(CallableValueFamily::Record),
        Type::Adt(_) => Ok(CallableValueFamily::Adt),
        Type::Unit => Ok(CallableValueFamily::Unit),
        Type::QVec(_) => Err(FrontendError {
            pos: 0,
            message:
                "'qvec' has no executable runtime value representation and cannot be used as a \
                 callable parameter type (#1773 architecture decision: qvec is real, \
                 parser-writable, typechecking syntax with no corresponding sm-vm::Value variant \
                 and no lowering path that ever constructs one)"
                    .to_string(),
        }),
        Type::RangeI32 => Err(FrontendError {
            pos: 0,
            message: "internal error: 'RangeI32' is a range-expression-only type and should be \
                       unreachable as a resolved callable parameter type"
                .to_string(),
        }),
        Type::TypeVar(_) => Err(FrontendError {
            pos: 0,
            message: "internal error: an unresolved type-inference variable should be \
                       unreachable as a resolved callable parameter type"
                .to_string(),
        }),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImmutableIrProgram {
    funcs: Vec<IrFunction>,
}

impl ImmutableIrProgram {
    pub fn from_vec(funcs: Vec<IrFunction>) -> Self {
        Self { funcs }
    }

    pub fn functions(&self) -> &[IrFunction] {
        &self.funcs
    }
}

#[derive(Debug, Clone, PartialEq)]
struct LoweredFunctionBundle {
    primary: IrFunction,
    lifted: Vec<IrFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogosIrLaw {
    pub name: String,
    pub priority: u32,
    pub when_count: usize,
}

const FX_SCALE: i32 = 1_000;

fn iterable_for_gap_message() -> &'static str {
    "iterable 'for x in collection' currently requires built-in Sequence(type), i32 range, or a direct record `Iterable` impl shaped as `fn next(self: Self, index: i32) -> Option(Item)`"
}

fn encode_fx_literal(value: f64) -> Result<i32, FrontendError> {
    let scaled = value * FX_SCALE as f64;
    if !scaled.is_finite() {
        return Err(FrontendError {
            pos: 0,
            message: "fx literal is not finite".to_string(),
        });
    }
    let rounded = scaled.round();
    if rounded < i32::MIN as f64 || rounded > i32::MAX as f64 {
        return Err(FrontendError {
            pos: 0,
            message: "fx literal is out of range for the v1 fixed-point carrier".to_string(),
        });
    }
    Ok(rounded as i32)
}

fn try_encode_fx_literal_expr(
    expr_id: ExprId,
    arena: &AstArena,
) -> Result<Option<i32>, FrontendError> {
    match arena.expr(expr_id) {
        Expr::NumericLiteral(literal) => match literal {
            NumericLiteral::I32(value) => value
                .checked_mul(FX_SCALE)
                .ok_or(FrontendError {
                    pos: 0,
                    message: "fx literal is out of range for the v1 fixed-point carrier"
                        .to_string(),
                })
                .map(Some),
            NumericLiteral::U32(value) => {
                let value = i32::try_from(*value).map_err(|_| FrontendError {
                    pos: 0,
                    message: "fx literal is out of range for the v1 fixed-point carrier"
                        .to_string(),
                })?;
                value
                    .checked_mul(FX_SCALE)
                    .ok_or(FrontendError {
                        pos: 0,
                        message: "fx literal is out of range for the v1 fixed-point carrier"
                            .to_string(),
                    })
                    .map(Some)
            }
            NumericLiteral::F64(value) | NumericLiteral::Fx(value) => {
                encode_fx_literal(*value).map(Some)
            }
        },
        Expr::Unary(UnaryOp::Pos, inner) => try_encode_fx_literal_expr(*inner, arena),
        Expr::Unary(UnaryOp::Neg, inner) => {
            let Some(value) = try_encode_fx_literal_expr(*inner, arena)? else {
                return Ok(None);
            };
            value
                .checked_neg()
                .ok_or(FrontendError {
                    pos: 0,
                    message: "fx literal is out of range for the v1 fixed-point carrier"
                        .to_string(),
                })
                .map(Some)
        }
        _ => Ok(None),
    }
}

fn is_builtin_assert_name(
    name: SymbolId,
    arena: &AstArena,
    fn_table: &FnTable,
) -> Result<bool, FrontendError> {
    Ok(!fn_table.contains_key(&name) && resolve_symbol_name(arena, name)? == "assert")
}

pub fn lower_logos_laws_to_ir(program: &LogosProgram) -> Vec<LogosIrLaw> {
    let mut laws = program.laws.clone();
    laws.sort_by_key(|law| core::cmp::Reverse(law.priority));
    laws.into_iter()
        .map(|law| LogosIrLaw {
            name: law.name,
            priority: law.priority,
            when_count: law.whens.len(),
        })
        .collect()
}

pub fn lower_expr_to_ir(
    expr: ExprId,
    arena: &AstArena,
    var_types: &HashMap<SymbolId, Type>,
    fn_table: &FnTable,
) -> Result<Vec<IrInstr>, FrontendError> {
    let mut out = Vec::new();
    let mut next = 0u16;
    let mut env = ScopeEnv::new();
    let mut loop_stack = Vec::new();
    let mut closure_state = ClosureLoweringState {
        parent_fn_name: "__expr".to_string(),
        next_closure_id: 0,
        lifted_funcs: Vec::new(),
    };
    let empty_records = RecordTable::new();
    let empty_adts = AdtTable::new();
    // #1724 (FA-04-018): mirrors `ownership_events` below - no owning
    // `IrFunction`, so a local instance is not the same defect class as a
    // same-function nested path losing its parent's state. `var_types`
    // entries are pre-existing frame locals supplied by the caller under
    // their raw spelling (`bind_raw`), not fresh declarations this
    // function lowers itself - there is nothing to shadow-mangle here.
    let mut lowered_locals = LoweredLocalEnv::new();
    for (name, ty) in var_types {
        env.insert(*name, ty.clone());
        lowered_locals.bind_raw(arena, *name)?;
    }
    // #1709: this entry point has no owning `IrFunction` (it lowers a bare
    // expression, not a compiled function - see the `lifted_funcs` rejection
    // below), so there is no retained sink for these events to preserve
    // into; a local, discarded `Vec` here is not the same defect class as a
    // same-function nested lowering path silently dropping its parent
    // function's events.
    let mut ownership_events = Vec::new();
    let _ = lower_expr(
        expr,
        arena,
        &mut next,
        &mut out,
        &env,
        &mut loop_stack,
        fn_table,
        &empty_records,
        &empty_adts,
        Type::Unit,
        &mut closure_state,
        &mut ownership_events,
        &mut lowered_locals,
    )?;
    if !closure_state.lifted_funcs.is_empty() {
        return Err(FrontendError {
            pos: 0,
            message: "lower_expr_to_ir does not emit lifted closure helpers; use compile_program_to_ir for first-class closures".to_string(),
        });
    }
    Ok(out)
}

/// FA-04-011 / #1717: sm-front's #1634/#1648/#1649 frontend generic-function
/// contract (arity <= 1 per definition site, recursive call-site type
/// inference/substitution, canonical `FnSig` authority) has no counterpart
/// at the IR/SemCode executable boundary -- `crates/sm-ir/src/passes/`
/// contains only `StructuralCleanup`/`CrystalFold`, no specialization or
/// monomorphisation pass. Before this check, whether a generic function's
/// declaration reached IR lowering at all depended entirely on incidental
/// structure: `canonicalize_declared_type` (the non-generic canonicalizer
/// used below, unaware of `func.type_params`) recursively rejects a
/// `TypeVar` wherever one appears directly or nested in `params`/`ret`, so
/// `fn id<T>(x: T) -> T` and `fn keep<T>(x: Option(T)) -> Option(T)`
/// happened to fail -- with a stale, misleading "deferred to M9.1 Wave 2"
/// diagnostic that falsely implies future support is coming, not that the
/// current architecture has decided against runtime generic dispatch. A
/// function whose declared type parameter never appears in its own
/// signature (e.g. `fn marker<T>(x: i32) -> i32`) had no `TypeVar`
/// anywhere for that incidental path to catch, and lowered as an ordinary
/// `IrFunction` with `type_params` silently discarded -- genuine type
/// erasure, not partial monomorphisation support, and strictly worse than
/// the direct/nested cases' accidental rejection because it is
/// *inconsistent*: whether a generic declaration survives to execute
/// depended on where its own type parameter happened to be written, not on
/// whether the declaration is generic at all.
///
/// This is the single, deliberate generic-execution admission boundary --
/// every declared type parameter is rejected here, used or not, before any
/// canonicalization or lowering work begins. `callable_family_for_type`'s
/// own `TypeVar` arm remains a late, defense-in-depth safety net for a
/// resolved parameter type that should be structurally impossible by the
/// time it is reached (see its doc comment); it is not weakened or relied
/// upon as the primary admission authority.
fn ensure_function_is_ir_concrete(func: &Function, arena: &AstArena) -> Result<(), FrontendError> {
    if !func.type_params.is_empty() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "generic function '{}' is admitted by the frontend but is not \
                 executable in the current IR contract because concrete IR \
                 monomorphisation is not implemented",
                resolve_symbol_name(arena, func.name)?
            ),
        });
    }
    Ok(())
}

fn lower_function_to_ir_with_tables(
    func: &Function,
    arena: &AstArena,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    impl_list: &[sm_front::ImplDecl],
) -> Result<LoweredFunctionBundle, FrontendError> {
    ensure_function_is_ir_concrete(func, arena)?;
    let parent_fn_name = resolve_symbol_name(arena, func.name)?.to_string();
    let ensures_result_symbol = find_contract_result_symbol(&func.ensures, arena)?;
    let invariants_result_symbol = find_contract_result_symbol(&func.invariants, arena)?;
    let mut ctx = LoweringCtx::new(
        parent_fn_name.clone(),
        func.ensures.clone(),
        ensures_result_symbol,
        func.invariants.clone(),
        invariants_result_symbol,
        impl_list,
    );
    let canonical_params = func
        .params
        .iter()
        .map(|(name, ty)| {
            Ok((
                *name,
                canonicalize_declared_type(ty, record_table, adt_table, arena)?,
            ))
        })
        .collect::<Result<Vec<_>, FrontendError>>()?;
    // #1773 (FA-09-005): derived from the same `canonical_params` the
    // register/StoreVar lowering below already uses - this is the exact
    // lowering boundary that previously discarded `func.params` entirely.
    let signature_params = canonical_params
        .iter()
        .map(|(_, ty)| callable_family_for_type(ty))
        .collect::<Result<Vec<_>, FrontendError>>()?;
    let canonical_ret = canonicalize_declared_type(&func.ret, record_table, adt_table, arena)?;
    let mut env = ScopeEnv::with_params(&canonical_params);
    ctx.next_reg = u16::try_from(func.params.len()).map_err(|_| FrontendError {
        pos: 0,
        message: "too many function parameters for register space".to_string(),
    })?;
    for (idx, (name, _)) in func.params.iter().enumerate() {
        ctx.instrs.push(IrInstr::StoreVar {
            name: ctx.lowered_locals.bind(arena, *name)?,
            src: idx as u16,
        });
    }
    for condition in &func.requires {
        let (cond_reg, cond_ty) = lower_expr(
            *condition,
            arena,
            &mut ctx.next_reg,
            &mut ctx.instrs,
            &env,
            &mut ctx.loop_stack,
            fn_table,
            record_table,
            adt_table,
            canonical_ret.clone(),
            &mut ctx.closure_state,
            &mut ctx.ownership_events,
            &mut ctx.lowered_locals,
        )?;
        if cond_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "requires clause condition must be bool in lowering, got {:?}",
                    cond_ty
                ),
            });
        }
        ctx.instrs.push(IrInstr::Assert { cond: cond_reg });
    }
    lower_invariant_clauses(
        &ctx.invariants,
        ctx.invariants_result_symbol,
        None,
        ContractInvariantPhase::Entry,
        arena,
        &mut ctx.next_reg,
        &mut ctx.instrs,
        &env,
        &mut ctx.loop_stack,
        fn_table,
        record_table,
        adt_table,
        func.ret.clone(),
        &mut ctx.closure_state,
        &mut ctx.ownership_events,
        &mut ctx.lowered_locals,
    )?;
    for stmt in &func.body {
        lower_stmt(
            *stmt,
            arena,
            &mut ctx,
            &mut env,
            canonical_ret.clone(),
            fn_table,
            record_table,
            adt_table,
        )?;
    }

    if !ctx.ends_with_ret() {
        if func.ret == Type::Unit {
            lower_ensures_clauses(
                &ctx.ensures,
                ctx.ensures_result_symbol,
                None,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                &env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                func.ret.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            lower_invariant_clauses(
                &ctx.invariants,
                ctx.invariants_result_symbol,
                None,
                ContractInvariantPhase::Exit,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                &env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                func.ret.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            ctx.instrs.push(IrInstr::Ret { src: None });
        } else {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "function '{}' may exit without returning {:?}",
                    resolve_symbol_name(arena, func.name)?,
                    func.ret
                ),
            });
        }
    }

    Ok(LoweredFunctionBundle {
        primary: IrFunction {
            name: parent_fn_name,
            instrs: ctx.instrs,
            ownership_events: ctx.ownership_events,
            params: signature_params,
        },
        lifted: ctx.closure_state.lifted_funcs,
    })
}

fn impl_method_function_name(
    arena: &AstArena,
    imp: &sm_front::ImplDecl,
    method: &Function,
) -> Result<String, FrontendError> {
    Ok(format!(
        "__impl::{}::{}::{}",
        resolve_symbol_name(arena, imp.trait_name)?,
        resolve_symbol_name(arena, imp.for_type)?,
        resolve_symbol_name(arena, method.name)?,
    ))
}

fn resolve_explicit_iterable_loop_contract(
    iterable_ty: &Type,
    trait_name: SymbolId,
    arena: &AstArena,
    impl_list: &[sm_front::ImplDecl],
) -> Result<Option<(Type, String)>, FrontendError> {
    let nominal = match iterable_ty {
        Type::Record(name) => *name,
        _ => return Ok(None),
    };
    for imp in impl_list {
        if imp.for_type != nominal || imp.trait_name != trait_name {
            continue;
        }
        let method = imp
            .methods
            .iter()
            .find(|method| resolve_symbol_name(arena, method.name).ok() == Some("next"))
            .ok_or(FrontendError {
                pos: 0,
                message: iterable_for_gap_message().to_string(),
            })?;
        if method.params.len() != 2
            || method.params[0].1 != Type::Record(nominal)
            || method.params[1].1 != Type::I32
        {
            return Err(FrontendError {
                pos: 0,
                message: iterable_for_gap_message().to_string(),
            });
        }
        let Type::Option(item_ty) = &method.ret else {
            return Err(FrontendError {
                pos: 0,
                message: iterable_for_gap_message().to_string(),
            });
        };
        return Ok(Some((
            item_ty.as_ref().clone(),
            impl_method_function_name(arena, imp, method)?,
        )));
    }
    Ok(None)
}

pub fn lower_function_to_ir(
    func: &Function,
    arena: &AstArena,
    fn_table: &FnTable,
) -> Result<IrFunction, FrontendError> {
    type_check_function_with_table(func, arena, fn_table)?;
    let empty_records = RecordTable::new();
    let empty_adts = AdtTable::new();
    let lowered =
        lower_function_to_ir_with_tables(func, arena, fn_table, &empty_records, &empty_adts, &[])?;
    if !lowered.lifted.is_empty() {
        return Err(FrontendError {
            pos: 0,
            message: "lower_function_to_ir does not emit lifted closure helpers; use compile_program_to_ir for first-class closures".to_string(),
        });
    }
    Ok(lowered.primary)
}

pub fn compile_program_to_ir(input: &str) -> Result<Vec<IrFunction>, FrontendError> {
    let profile = ParserProfile::foundation_default();
    compile_program_to_ir_with_options_and_profile(
        input,
        CompileProfile::RustLike,
        OptLevel::O0,
        &profile,
    )
}

pub fn compile_program_to_immutable_ir(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
) -> Result<ImmutableIrProgram, FrontendError> {
    let parser_profile = ParserProfile::foundation_default();
    Ok(ImmutableIrProgram::from_vec(
        compile_program_to_ir_with_options_and_profile(input, profile, opt, &parser_profile)?,
    ))
}

pub fn compile_program_to_ir_with_options(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
) -> Result<Vec<IrFunction>, FrontendError> {
    let parser_profile = ParserProfile::foundation_default();
    compile_program_to_ir_with_options_and_profile(input, profile, opt, &parser_profile)
}

pub fn compile_program_to_ir_with_profile(
    input: &str,
    parser_profile: &ParserProfile,
) -> Result<Vec<IrFunction>, FrontendError> {
    compile_program_to_ir_with_options_and_profile(
        input,
        CompileProfile::RustLike,
        OptLevel::O0,
        parser_profile,
    )
}

pub fn compile_program_to_ir_with_options_and_profile(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
    parser_profile: &ParserProfile,
) -> Result<Vec<IrFunction>, FrontendError> {
    match profile {
        CompileProfile::RustLike if !cfg!(feature = "profile-rust") => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "RustLike profile is disabled at compile time (enable feature 'profile-rust')"
                        .to_string(),
            });
        }
        CompileProfile::Logos if !cfg!(feature = "profile-logos") => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "Logos profile is disabled at compile time (enable feature 'profile-logos')"
                        .to_string(),
            });
        }
        _ => {}
    }
    let logos_detected = parse_logos_program_with_profile(input, parser_profile)
        .map(|p| p.system.is_some() || !p.entities.is_empty() || !p.laws.is_empty())
        .unwrap_or(false);
    if (matches!(profile, CompileProfile::Logos)
        || (matches!(profile, CompileProfile::Auto) && logos_detected))
        && cfg!(feature = "profile-logos")
    {
        return Err(FrontendError {
            pos: 0,
            message: "Logos input lowers to LogosIrLaw stream; SemCode function IR requires RustLike frontend".to_string(),
        });
    }
    if matches!(profile, CompileProfile::Auto) && logos_detected && !cfg!(feature = "profile-logos")
    {
        return Err(FrontendError {
            pos: 0,
            message: "Logos input detected, but Logos profile is disabled at compile time"
                .to_string(),
        });
    }
    if !cfg!(feature = "profile-rust") {
        return Err(FrontendError {
            pos: 0,
            message: "RustLike lowering is disabled at compile time".to_string(),
        });
    }
    let mut program = parse_program_with_profile(input, parser_profile)?;
    let fn_table = build_fn_table(&program)?;
    let record_table = build_record_table(&program)?;
    let adt_table = build_adt_table(&program)?;
    type_check_program(&program)?;
    let mut out = Vec::new();
    for f in &program.functions {
        let lowered = lower_function_to_ir_with_tables(
            f,
            &program.arena,
            &fn_table,
            &record_table,
            &adt_table,
            &program.impls,
        )?;
        out.push(lowered.primary);
        out.extend(lowered.lifted);
    }
    let impls = program.impls.clone();
    for imp in &impls {
        for method in &imp.methods {
            let mut lowered_method = method.clone();
            let lowered_name = impl_method_function_name(&program.arena, imp, method)?;
            lowered_method.name = program.arena.intern_symbol(&lowered_name);
            let lowered = lower_function_to_ir_with_tables(
                &lowered_method,
                &program.arena,
                &fn_table,
                &record_table,
                &adt_table,
                &program.impls,
            )?;
            out.push(lowered.primary);
            out.extend(lowered.lifted);
        }
    }
    if matches!(opt, OptLevel::O1) {
        let _ = crate::passes::run_default_opt_passes(&mut out);
    }
    Ok(out)
}

pub fn compile_program_to_ir_optimized(input: &str) -> Result<Vec<IrFunction>, FrontendError> {
    let profile = ParserProfile::foundation_default();
    compile_program_to_ir_with_options_and_profile(
        input,
        CompileProfile::RustLike,
        OptLevel::O1,
        &profile,
    )
}

pub fn validate_ir(f: &IrFunction) -> Result<(), FrontendError> {
    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut has_ret = false;

    for (idx, instr) in f.instrs.iter().enumerate() {
        if let IrInstr::Label { name } = instr {
            if labels.insert(name.clone(), idx).is_some() {
                return Err(FrontendError {
                    pos: idx,
                    message: format!("duplicate label '{}' in '{}'", name, f.name),
                });
            }
        }
        if matches!(instr, IrInstr::Ret { .. }) {
            has_ret = true;
        }
    }

    if !has_ret {
        return Err(FrontendError {
            pos: 0,
            message: format!("function '{}' has no RET", f.name),
        });
    }

    for (idx, instr) in f.instrs.iter().enumerate() {
        match instr {
            IrInstr::Jmp { label } | IrInstr::JmpIf { label, .. }
                if !labels.contains_key(label) =>
            {
                return Err(FrontendError {
                    pos: idx,
                    message: format!("jump to unknown label '{}' in '{}'", label, f.name),
                });
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn compile_program_to_semcode(input: &str) -> Result<Vec<u8>, FrontendError> {
    compile_program_to_semcode_with_options(input, CompileProfile::RustLike, OptLevel::O0)
}

pub fn compile_program_to_semcode_with_options(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
) -> Result<Vec<u8>, FrontendError> {
    compile_program_to_semcode_with_options_debug(input, profile, opt, false)
}

pub fn compile_program_to_semcode_with_options_debug(
    input: &str,
    profile: CompileProfile,
    opt: OptLevel,
    debug_symbols: bool,
) -> Result<Vec<u8>, FrontendError> {
    if debug_symbols && !cfg!(feature = "debug-symbols") {
        return Err(FrontendError {
            pos: 0,
            message: "debug symbols are disabled at compile time (enable feature 'debug-symbols')"
                .to_string(),
        });
    }
    let ir = compile_program_to_immutable_ir(input, profile, opt)?;
    for f in ir.functions() {
        validate_ir(f)?;
    }
    emit_semcode(ir.functions(), debug_symbols)
}

pub fn emit_ir_to_semcode(
    funcs: &[IrFunction],
    debug_symbols: bool,
) -> Result<Vec<u8>, FrontendError> {
    emit_semcode(funcs, debug_symbols)
}

fn emit_semcode(funcs: &[IrFunction], debug_symbols: bool) -> Result<Vec<u8>, FrontendError> {
    let mut out = Vec::new();
    // require_ownership_section: whenever the chosen header includes CAP_OWNERSHIP_PATHS,
    // every function must have an OWN0 section (even if empty) so the verifier check passes.
    let opcode_driven_magic: [u8; 8];
    let opcode_driven_require_ownership_section;
    if has_v18_qtruth_instr(funcs) {
        opcode_driven_magic = MAGIC18;
        opcode_driven_require_ownership_section = true;
    } else if has_v17_application_instr(funcs) {
        opcode_driven_magic = MAGIC17;
        opcode_driven_require_ownership_section = true;
    } else if has_v16_stdout_instr(funcs) {
        opcode_driven_magic = MAGIC16;
        opcode_driven_require_ownership_section = true;
    } else if has_v15_prng_instr(funcs) {
        opcode_driven_magic = MAGIC15;
        opcode_driven_require_ownership_section = true;
    } else if has_v14_map_instr(funcs) {
        opcode_driven_magic = MAGIC14;
        opcode_driven_require_ownership_section = true;
    } else if has_v13_sequence_iter_instr(funcs) {
        opcode_driven_magic = MAGIC13;
        opcode_driven_require_ownership_section = true;
    } else if has_v12_record_field_ownership_events(funcs) {
        opcode_driven_magic = MAGIC12;
        opcode_driven_require_ownership_section = true;
    } else if has_v11_ownership_events(funcs) {
        opcode_driven_magic = MAGIC11;
        opcode_driven_require_ownership_section = true;
    } else if has_v10_closure_instr(funcs) {
        opcode_driven_magic = MAGIC10;
        opcode_driven_require_ownership_section = false;
    } else if has_v9_sequence_instr(funcs) {
        opcode_driven_magic = MAGIC9;
        opcode_driven_require_ownership_section = false;
    } else if has_v8_text_instr(funcs) {
        opcode_driven_magic = MAGIC8;
        opcode_driven_require_ownership_section = false;
    } else if has_v7_clock_read_instr(funcs) {
        opcode_driven_magic = MAGIC7;
        opcode_driven_require_ownership_section = false;
    } else if has_v6_event_post_instr(funcs) {
        opcode_driven_magic = MAGIC6;
        opcode_driven_require_ownership_section = false;
    } else if has_v5_state_update_instr(funcs) {
        opcode_driven_magic = MAGIC5;
        opcode_driven_require_ownership_section = false;
    } else if has_v4_state_query_instr(funcs) {
        opcode_driven_magic = MAGIC4;
        opcode_driven_require_ownership_section = false;
    } else if has_v3_fx_math_instr(funcs) {
        opcode_driven_magic = MAGIC3;
        opcode_driven_require_ownership_section = false;
    } else if has_v2_fx_instr(funcs) {
        opcode_driven_magic = MAGIC2;
        opcode_driven_require_ownership_section = false;
    } else if has_v1_math_instr(funcs) {
        opcode_driven_magic = MAGIC1;
        opcode_driven_require_ownership_section = false;
    } else {
        opcode_driven_magic = MAGIC0;
        opcode_driven_require_ownership_section = false;
    }
    let opcode_driven_header =
        header_spec_from_magic(&opcode_driven_magic).expect("known header just chosen");
    // #1773 (FA-09-005): every function's envelope now carries a canonical
    // callable-signature record unconditionally (see `emit_semcode_function`
    // below), which only a header at or above `SEMCODE_SIGNATURE_MIN_REVISION`
    // can structurally carry - so that revision is now the emitter's floor,
    // regardless of which opcodes a program happens to use. The opcode-driven
    // chain above is preserved unchanged beneath that floor so a future
    // opcode needing an even newer revision still promotes correctly on top
    // of it (mirrors the #1732 precedent: a new header revision closing a
    // version-identity gap, not a capability gap).
    let (chosen_magic, require_ownership_section) =
        if opcode_driven_header.rev < SEMCODE_SIGNATURE_MIN_REVISION {
            (MAGIC19, true)
        } else {
            (opcode_driven_magic, opcode_driven_require_ownership_section)
        };
    out.extend_from_slice(&chosen_magic);
    // #1732 (FA-05-002): the `has_vN_*_instr` chain above is a hand-written
    // promotion decision, independent of `Opcode::minimum_semcode_revision`
    // (sm-format's actual admission authority). This is the mechanical
    // safety net closing that gap: every function's actual emitted opcode
    // bytes are checked against the chosen header's revision below, so a
    // future opcode assigned a non-baseline minimum revision without a
    // matching promotion branch here hard-fails at emission time - loudly,
    // at the point of the bug - instead of silently shipping an artifact
    // its own verifier would reject.
    let chosen_header = header_spec_from_magic(&chosen_magic).expect("known header just chosen");
    let mut max_opcode_revision_used: u16 = 1;
    for f in funcs {
        let name_bytes = f.name.as_bytes();
        write_u16_le(
            &mut out,
            u16::try_from(name_bytes.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "function name too long".to_string(),
            })?,
        );
        out.extend_from_slice(name_bytes);
        let (code, func_max_revision) = emit_semcode_function(
            f,
            debug_symbols,
            require_ownership_section,
            chosen_header.rev,
        )?;
        max_opcode_revision_used = max_opcode_revision_used.max(func_max_revision);
        write_u32_le(
            &mut out,
            u32::try_from(code.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "function code too large".to_string(),
            })?,
        );
        out.extend_from_slice(&code);
    }
    if max_opcode_revision_used > chosen_header.rev {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "internal error: emitted opcode requires minimum SemCode revision {}, \
                 but header selection chose revision {} ('{}') - a header-selection \
                 predicate is missing for an opcode with a non-baseline \
                 Opcode::minimum_semcode_revision()",
                max_opcode_revision_used,
                chosen_header.rev,
                String::from_utf8_lossy(&chosen_header.magic)
            ),
        });
    }
    Ok(out)
}

/// #1732 (FA-05-002) review follow-up: reads back the opcode byte
/// `emit_instr` just wrote at `opcode_byte_pos` and returns its
/// `Opcode::minimum_semcode_revision()`. Fails closed - returns
/// `Err(FrontendError)` - rather than silently skipping, if the byte is
/// missing or unrecognized. Under the current `emit_instr`, every non-Label
/// `IrInstr` variant always writes a real `Opcode::X.byte()` as its first
/// byte, so both error paths are unreachable through the public API today;
/// they exist so a future `emit_instr` change that broke that invariant
/// hard-fails immediately instead of letting the mechanical revision guard
/// silently under-report the required header revision.
fn opcode_minimum_revision_at(code: &[u8], opcode_byte_pos: usize) -> Result<u16, FrontendError> {
    let raw_opcode = *code.get(opcode_byte_pos).ok_or_else(|| FrontendError {
        pos: 0,
        message: "internal error: emit_instr produced no opcode byte for a non-Label instruction"
            .to_string(),
    })?;
    let opcode = Opcode::from_byte(raw_opcode).map_err(|_| FrontendError {
        pos: 0,
        message: format!(
            "internal error: emit_instr wrote an unrecognized opcode byte 0x{raw_opcode:02x} \
             that Opcode::from_byte cannot decode"
        ),
    })?;
    Ok(opcode.minimum_semcode_revision())
}

fn emit_semcode_function(
    f: &IrFunction,
    debug_symbols: bool,
    require_ownership_section: bool,
    chosen_header_rev: u16,
) -> Result<(Vec<u8>, u16), FrontendError> {
    let mut interner = StringInterner::new();
    for instr in &f.instrs {
        match instr {
            IrInstr::LoadText { val, .. } => {
                let _ = interner.id(val)?;
            }
            IrInstr::MakeSequence { .. }
            | IrInstr::SequenceGet { .. }
            | IrInstr::SequenceLen { .. }
            | IrInstr::SequenceIsEmpty { .. }
            | IrInstr::SequenceContains { .. }
            | IrInstr::SequencePush { .. }
            | IrInstr::SequencePrepend { .. }
            | IrInstr::SequencePop { .. }
            | IrInstr::MapEmpty { .. }
            | IrInstr::MapContains { .. }
            | IrInstr::MapGet { .. }
            | IrInstr::MapSet { .. }
            | IrInstr::RngSeed { .. }
            | IrInstr::RngNextI32 { .. }
            | IrInstr::ConcatText { .. } => {}
            IrInstr::MakeClosure { name, .. } => {
                let _ = interner.id(name)?;
            }
            IrInstr::ClosureCall { .. } => {}
            IrInstr::LoadVar { name, .. } => {
                let _ = interner.id(name)?;
            }
            IrInstr::StoreVar { name, .. } => {
                let _ = interner.id(name)?;
            }
            IrInstr::MakeRecord { name, .. } => {
                let _ = interner.id(name)?;
            }
            IrInstr::MakeAdt {
                adt_name,
                variant_name,
                ..
            } => {
                let _ = interner.id(adt_name)?;
                let _ = interner.id(variant_name)?;
            }
            IrInstr::RecordGet { record_name, .. } => {
                let _ = interner.id(record_name)?;
            }
            IrInstr::AdtTag { adt_name, .. } | IrInstr::AdtGet { adt_name, .. } => {
                let _ = interner.id(adt_name)?;
            }
            IrInstr::Call { name, .. } => {
                let _ = interner.id(name)?;
            }
            IrInstr::PulseEmit { signal } => {
                let _ = interner.id(signal)?;
            }
            IrInstr::StateQuery { key, .. } => {
                let _ = interner.id(key)?;
            }
            IrInstr::StateUpdate { key, .. } => {
                let _ = interner.id(key)?;
            }
            IrInstr::EventPost { signal } => {
                let _ = interner.id(signal)?;
            }
            IrInstr::ClockRead { .. } => {}
            _ => {}
        }
    }

    let mut label_pc: HashMap<String, u32> = HashMap::new();
    let mut pc: u32 = 0;
    for instr in &f.instrs {
        match instr {
            IrInstr::Label { name } => {
                label_pc.insert(name.clone(), pc);
            }
            _ => {
                pc = pc
                    .checked_add(encoded_size(instr).ok_or(FrontendError {
                        pos: 0,
                        message: "label has no encoded size".to_string(),
                    })? as u32)
                    .ok_or(FrontendError {
                        pos: 0,
                        message: "bytecode size overflow".to_string(),
                    })?;
            }
        }
    }

    let mut instr_stream = Vec::new();
    let mut dbg = Vec::<(u32, u32, u16)>::new();
    // #1732 (FA-05-002): tracks the maximum Opcode::minimum_semcode_revision()
    // actually emitted, read back from the real opcode byte emit_instr just
    // wrote - not a second hand-maintained opcode/feature table - so the
    // emit_semcode caller can mechanically detect (and hard-fail on) any
    // future opcode whose minimum-revision authority is updated without a
    // matching header-selection promotion, instead of silently emitting an
    // artifact its own verifier would reject.
    let mut max_opcode_revision: u16 = 1;
    for instr in &f.instrs {
        if matches!(instr, IrInstr::Label { .. }) {
            continue;
        }
        let pc = u32::try_from(instr_stream.len()).map_err(|_| FrontendError {
            pos: 0,
            message: "instruction stream too large".to_string(),
        })?;
        let opcode_byte_pos = instr_stream.len();
        emit_instr(instr, &label_pc, &interner, &mut instr_stream)?;
        let instr_min_revision = opcode_minimum_revision_at(&instr_stream, opcode_byte_pos)?;
        max_opcode_revision = max_opcode_revision.max(instr_min_revision);
        if debug_symbols {
            let line = u32::try_from(dbg.len() + 1).map_err(|_| FrontendError {
                pos: 0,
                message: "debug table too large".to_string(),
            })?;
            dbg.push((pc, line, 1));
        }
    }

    let mut code = Vec::new();
    interner.emit_table(&mut code)?;
    if debug_symbols {
        code.extend_from_slice(b"DBG0");
        write_u16_le(
            &mut code,
            u16::try_from(dbg.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "too many debug symbols".to_string(),
            })?,
        );
        for (pc, line, col) in dbg {
            write_u32_le(&mut code, pc);
            write_u32_le(&mut code, line);
            write_u16_le(&mut code, col);
        }
    }
    emit_ownership_events(&f.ownership_events, require_ownership_section, &mut code)?;
    // #1773 (FA-09-005): unconditional, never a per-function/emission-time
    // choice like `debug_symbols` - every function gets a SIG0 section once
    // the chosen header can carry one, so `sm-format`'s decoder can expect
    // it deterministically from the header revision alone (see
    // `parse_string_table_debug_and_ownership`'s doc comment on why it is
    // not content-sniffed).
    if chosen_header_rev >= SEMCODE_SIGNATURE_MIN_REVISION {
        // #1773 review follow-up: bounded by the decoder's own
        // MAX_SIGNATURE_PARAMETERS_PER_FUNCTION, not just the wire field's
        // u16 width - without this, a function with 4,097..=65,535
        // parameters would emit successfully (fits in a u16) but produce
        // bytes `decode_semcode_envelope` unconditionally rejects,
        // discovered only downstream at every verified execution route.
        if f.params.len() > MAX_SIGNATURE_PARAMETERS_PER_FUNCTION {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "too many callable-signature parameters: {} (max {})",
                    f.params.len(),
                    MAX_SIGNATURE_PARAMETERS_PER_FUNCTION
                ),
            });
        }
        code.extend_from_slice(&SIGNATURE_SECTION_TAG);
        write_u16_le(
            &mut code,
            u16::try_from(f.params.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "too many callable-signature parameters".to_string(),
            })?,
        );
        for family in &f.params {
            code.push(family.byte());
        }
    }
    code.extend_from_slice(&instr_stream);
    Ok((code, max_opcode_revision))
}

fn encoded_size(instr: &IrInstr) -> Option<usize> {
    let s = match instr {
        IrInstr::Label { .. } => return None,
        IrInstr::LoadQ { .. } => 1 + 2 + 1,
        IrInstr::LoadBool { .. } => 1 + 2 + 1,
        IrInstr::LoadI32 { .. } => 1 + 2 + 4,
        IrInstr::LoadU32 { .. } => 1 + 2 + 4,
        IrInstr::LoadF64 { .. } => 1 + 2 + 8,
        IrInstr::LoadFx { .. } => 1 + 2 + 4,
        IrInstr::LoadText { .. } => 1 + 2 + 2,
        IrInstr::ConcatText { .. } => 1 + 2 + 2 + 2,
        IrInstr::MakeSequence { items, .. } => 1 + 2 + 2 + (items.len() * 2),
        IrInstr::SequenceLen { .. } => 1 + 2 + 2,
        IrInstr::SequenceIsEmpty { .. } => 1 + 2 + 2,
        IrInstr::SequenceContains { .. } => 1 + 2 + 2 + 2,
        IrInstr::SequencePush { .. } => 1 + 2 + 2 + 2,
        IrInstr::SequencePrepend { .. } => 1 + 2 + 2 + 2,
        IrInstr::SequencePop { .. } => 1 + 2 + 2,
        IrInstr::MapEmpty { .. } => 1 + 2,
        IrInstr::MapContains { .. } => 1 + 2 + 2 + 2,
        IrInstr::MapGet { .. } => 1 + 2 + 2 + 2 + 2,
        IrInstr::MapSet { .. } => 1 + 2 + 2 + 2 + 2,
        IrInstr::RngSeed { .. } => 1 + 2 + 2,
        IrInstr::RngNextI32 { .. } => 1 + 2 + 2 + 2,
        IrInstr::MakeClosure { captures, .. } => 1 + 2 + 2 + 2 + (captures.len() * 2),
        IrInstr::SequenceGet { .. } => 1 + 2 + 2 + 2,
        IrInstr::ClosureCall { .. } => 1 + 1 + 2 + 2 + 2,
        IrInstr::MakeTuple { items, .. } => 1 + 2 + 2 + (items.len() * 2),
        IrInstr::MakeRecord { items, .. } => 1 + 2 + 2 + 2 + (items.len() * 2),
        IrInstr::MakeAdt { items, .. } => 1 + 2 + 2 + 2 + 2 + 2 + (items.len() * 2),
        IrInstr::AdtTag { .. } => 1 + 2 + 2 + 2,
        IrInstr::AdtGet { .. } => 1 + 2 + 2 + 2 + 2,
        IrInstr::RecordGet { .. } => 1 + 2 + 2 + 2 + 2,
        IrInstr::TupleGet { .. } => 1 + 2 + 2 + 2,
        IrInstr::LoadVar { .. } => 1 + 2 + 2,
        IrInstr::StoreVar { .. } => 1 + 2 + 2,
        IrInstr::QAnd { .. }
        | IrInstr::QOr { .. }
        | IrInstr::QImpl { .. }
        | IrInstr::QTruthAnd { .. }
        | IrInstr::QTruthOr { .. }
        | IrInstr::QTruthImpl { .. }
        | IrInstr::BoolAnd { .. }
        | IrInstr::BoolOr { .. }
        | IrInstr::CmpEq { .. }
        | IrInstr::CmpNe { .. }
        | IrInstr::CmpI32Lt { .. }
        | IrInstr::CmpI32Le { .. }
        | IrInstr::AddI32 { .. }
        | IrInstr::SubI32 { .. }
        | IrInstr::MulI32 { .. }
        | IrInstr::DivI32 { .. }
        | IrInstr::ModI32 { .. }
        | IrInstr::AddF64 { .. }
        | IrInstr::SubF64 { .. }
        | IrInstr::MulF64 { .. }
        | IrInstr::DivF64 { .. }
        | IrInstr::AddFx { .. }
        | IrInstr::SubFx { .. }
        | IrInstr::MulFx { .. }
        | IrInstr::DivFx { .. } => 1 + 2 + 2 + 2,
        IrInstr::QNot { .. } | IrInstr::QTruthNot { .. } | IrInstr::BoolNot { .. } => 1 + 2 + 2,
        IrInstr::Jmp { .. } => 1 + 4,
        IrInstr::JmpIf { .. } => 1 + 2 + 4,
        IrInstr::Assert { .. } => 1 + 2,
        IrInstr::Call { args, .. } => 1 + 1 + 2 + 2 + 2 + (args.len() * 2),
        IrInstr::GateRead { .. } => 1 + 2 + 2 + 2,
        IrInstr::GateWrite { .. } => 1 + 2 + 2 + 2,
        IrInstr::PulseEmit { .. } => 1 + 2,
        IrInstr::StateQuery { .. } => 1 + 2 + 2,
        IrInstr::StateUpdate { .. } => 1 + 2 + 2,
        IrInstr::EventPost { .. } => 1 + 2,
        IrInstr::ClockRead { .. } => 1 + 2,
        IrInstr::Ret { src: Some(_) } => 1 + 1 + 2,
        IrInstr::Ret { src: None } => 1 + 1,
    };
    Some(s)
}

fn emit_instr(
    instr: &IrInstr,
    label_pc: &HashMap<String, u32>,
    interner: &StringInterner,
    out: &mut Vec<u8>,
) -> Result<(), FrontendError> {
    match instr {
        IrInstr::Label { .. } => {}
        IrInstr::LoadQ { dst, val } => {
            out.push(Opcode::LoadQ.byte());
            write_u16_le(out, *dst);
            out.push(match val {
                QuadVal::N => 0,
                QuadVal::F => 1,
                QuadVal::T => 2,
                QuadVal::S => 3,
            });
        }
        IrInstr::LoadBool { dst, val } => {
            out.push(Opcode::LoadBool.byte());
            write_u16_le(out, *dst);
            out.push(if *val { 1 } else { 0 });
        }
        IrInstr::LoadI32 { dst, val } => {
            out.push(Opcode::LoadI32.byte());
            write_u16_le(out, *dst);
            write_i32_le(out, *val);
        }
        IrInstr::LoadU32 { dst, val } => {
            out.push(Opcode::LoadU32.byte());
            write_u16_le(out, *dst);
            write_u32_le(out, *val);
        }
        IrInstr::LoadF64 { dst, val } => {
            out.push(Opcode::LoadF64.byte());
            write_u16_le(out, *dst);
            write_f64_le(out, *val);
        }
        IrInstr::LoadFx { dst, val } => {
            out.push(Opcode::LoadFx.byte());
            write_u16_le(out, *dst);
            write_i32_le(out, *val);
        }
        IrInstr::LoadText { dst, val } => {
            out.push(Opcode::LoadText.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, interner.lookup(val)?);
        }
        IrInstr::ConcatText { dst, lhs, rhs } => {
            out.push(Opcode::ConcatText.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *lhs);
            write_u16_le(out, *rhs);
        }
        IrInstr::MakeSequence { dst, items } => {
            out.push(Opcode::MakeSequence.byte());
            write_u16_le(out, *dst);
            let count = u16::try_from(items.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "sequence literal has too many items".to_string(),
            })?;
            write_u16_le(out, count);
            for item in items {
                write_u16_le(out, *item);
            }
        }
        IrInstr::MakeClosure {
            dst,
            name,
            captures,
        } => {
            out.push(Opcode::MakeClosure.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, interner.lookup(name)?);
            let count = u16::try_from(captures.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "closure literal captures exceed v0 limit".to_string(),
            })?;
            write_u16_le(out, count);
            for capture in captures {
                write_u16_le(out, *capture);
            }
        }
        IrInstr::SequenceLen { dst, src } => {
            out.push(Opcode::SequenceLen.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *src);
        }
        IrInstr::SequenceIsEmpty { dst, src } => {
            out.push(Opcode::SequenceIsEmpty.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *src);
        }
        IrInstr::SequenceContains { dst, seq, val } => {
            out.push(Opcode::SequenceContains.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *seq);
            write_u16_le(out, *val);
        }
        IrInstr::SequencePush { dst, seq, val } => {
            out.push(Opcode::SequencePush.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *seq);
            write_u16_le(out, *val);
        }
        IrInstr::SequencePrepend { dst, seq, val } => {
            out.push(Opcode::SequencePrepend.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *seq);
            write_u16_le(out, *val);
        }
        IrInstr::SequencePop { dst, src } => {
            out.push(Opcode::SequencePop.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *src);
        }
        IrInstr::MapEmpty { dst } => {
            out.push(Opcode::MapEmpty.byte());
            write_u16_le(out, *dst);
        }
        IrInstr::MapContains { dst, map, key } => {
            out.push(Opcode::MapContains.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *map);
            write_u16_le(out, *key);
        }
        IrInstr::MapGet {
            dst,
            map,
            key,
            default_val,
        } => {
            out.push(Opcode::MapGet.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *map);
            write_u16_le(out, *key);
            write_u16_le(out, *default_val);
        }
        IrInstr::MapSet { dst, map, key, val } => {
            out.push(Opcode::MapSet.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *map);
            write_u16_le(out, *key);
            write_u16_le(out, *val);
        }
        IrInstr::RngSeed { dst, seed } => {
            out.push(Opcode::RngSeed.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *seed);
        }
        IrInstr::RngNextI32 { dst, lo, hi } => {
            out.push(Opcode::RngNextI32.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *lo);
            write_u16_le(out, *hi);
        }
        IrInstr::SequenceGet { dst, src, index } => {
            out.push(Opcode::SequenceGet.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *src);
            write_u16_le(out, *index);
        }
        IrInstr::ClosureCall { dst, closure, arg } => {
            out.push(Opcode::ClosureCall.byte());
            match dst {
                Some(reg) => {
                    out.push(1);
                    write_u16_le(out, *reg);
                }
                None => {
                    out.push(0);
                    write_u16_le(out, 0);
                }
            }
            write_u16_le(out, *closure);
            write_u16_le(out, *arg);
        }
        IrInstr::MakeTuple { dst, items } => {
            out.push(Opcode::MakeTuple.byte());
            write_u16_le(out, *dst);
            let count = u16::try_from(items.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "tuple literal has too many elements".to_string(),
            })?;
            write_u16_le(out, count);
            for item in items {
                write_u16_le(out, *item);
            }
        }
        IrInstr::MakeRecord { dst, name, items } => {
            out.push(Opcode::MakeRecord.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, interner.lookup(name)?);
            let count = u16::try_from(items.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "record literal has too many fields".to_string(),
            })?;
            write_u16_le(out, count);
            for item in items {
                write_u16_le(out, *item);
            }
        }
        IrInstr::MakeAdt {
            dst,
            adt_name,
            variant_name,
            tag,
            items,
        } => {
            out.push(Opcode::MakeAdt.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, interner.lookup(adt_name)?);
            write_u16_le(out, interner.lookup(variant_name)?);
            write_u16_le(out, *tag);
            let count = u16::try_from(items.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "enum constructor has too many payload items".to_string(),
            })?;
            write_u16_le(out, count);
            for item in items {
                write_u16_le(out, *item);
            }
        }
        IrInstr::AdtTag { dst, src, adt_name } => {
            out.push(Opcode::AdtTag.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *src);
            write_u16_le(out, interner.lookup(adt_name)?);
        }
        IrInstr::AdtGet {
            dst,
            src,
            adt_name,
            index,
        } => {
            out.push(Opcode::AdtGet.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *src);
            write_u16_le(out, interner.lookup(adt_name)?);
            write_u16_le(out, *index);
        }
        IrInstr::RecordGet {
            dst,
            src,
            record_name,
            index,
        } => {
            out.push(Opcode::RecordGet.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *src);
            write_u16_le(out, interner.lookup(record_name)?);
            write_u16_le(out, *index);
        }
        IrInstr::TupleGet { dst, src, index } => {
            out.push(Opcode::TupleGet.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *src);
            write_u16_le(out, *index);
        }
        IrInstr::LoadVar { dst, name } => {
            out.push(Opcode::LoadVar.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, interner.lookup(name)?);
        }
        IrInstr::StoreVar { name, src } => {
            out.push(Opcode::StoreVar.byte());
            write_u16_le(out, interner.lookup(name)?);
            write_u16_le(out, *src);
        }
        IrInstr::QAnd { dst, lhs, rhs } => emit_3reg(Opcode::QAnd, *dst, *lhs, *rhs, out),
        IrInstr::QOr { dst, lhs, rhs } => emit_3reg(Opcode::QOr, *dst, *lhs, *rhs, out),
        IrInstr::QNot { dst, src } => emit_2reg(Opcode::QNot, *dst, *src, out),
        IrInstr::QImpl { dst, lhs, rhs } => emit_3reg(Opcode::QImpl, *dst, *lhs, *rhs, out),
        IrInstr::QTruthAnd { dst, lhs, rhs } => emit_3reg(Opcode::QTruthAnd, *dst, *lhs, *rhs, out),
        IrInstr::QTruthOr { dst, lhs, rhs } => emit_3reg(Opcode::QTruthOr, *dst, *lhs, *rhs, out),
        IrInstr::QTruthNot { dst, src } => emit_2reg(Opcode::QTruthNot, *dst, *src, out),
        IrInstr::QTruthImpl { dst, lhs, rhs } => {
            emit_3reg(Opcode::QTruthImpl, *dst, *lhs, *rhs, out)
        }
        IrInstr::BoolAnd { dst, lhs, rhs } => emit_3reg(Opcode::BoolAnd, *dst, *lhs, *rhs, out),
        IrInstr::BoolOr { dst, lhs, rhs } => emit_3reg(Opcode::BoolOr, *dst, *lhs, *rhs, out),
        IrInstr::BoolNot { dst, src } => emit_2reg(Opcode::BoolNot, *dst, *src, out),
        IrInstr::CmpEq { dst, lhs, rhs } => emit_3reg(Opcode::CmpEq, *dst, *lhs, *rhs, out),
        IrInstr::CmpNe { dst, lhs, rhs } => emit_3reg(Opcode::CmpNe, *dst, *lhs, *rhs, out),
        IrInstr::CmpI32Lt { dst, lhs, rhs } => emit_3reg(Opcode::CmpI32Lt, *dst, *lhs, *rhs, out),
        IrInstr::CmpI32Le { dst, lhs, rhs } => emit_3reg(Opcode::CmpI32Le, *dst, *lhs, *rhs, out),
        IrInstr::AddI32 { dst, lhs, rhs } => emit_3reg(Opcode::AddI32, *dst, *lhs, *rhs, out),
        IrInstr::SubI32 { dst, lhs, rhs } => emit_3reg(Opcode::SubI32, *dst, *lhs, *rhs, out),
        IrInstr::MulI32 { dst, lhs, rhs } => emit_3reg(Opcode::MulI32, *dst, *lhs, *rhs, out),
        IrInstr::DivI32 { dst, lhs, rhs } => emit_3reg(Opcode::DivI32, *dst, *lhs, *rhs, out),
        IrInstr::ModI32 { dst, lhs, rhs } => emit_3reg(Opcode::ModI32, *dst, *lhs, *rhs, out),
        IrInstr::AddF64 { dst, lhs, rhs } => emit_3reg(Opcode::AddF64, *dst, *lhs, *rhs, out),
        IrInstr::SubF64 { dst, lhs, rhs } => emit_3reg(Opcode::SubF64, *dst, *lhs, *rhs, out),
        IrInstr::MulF64 { dst, lhs, rhs } => emit_3reg(Opcode::MulF64, *dst, *lhs, *rhs, out),
        IrInstr::DivF64 { dst, lhs, rhs } => emit_3reg(Opcode::DivF64, *dst, *lhs, *rhs, out),
        IrInstr::AddFx { dst, lhs, rhs } => emit_3reg(Opcode::AddFx, *dst, *lhs, *rhs, out),
        IrInstr::SubFx { dst, lhs, rhs } => emit_3reg(Opcode::SubFx, *dst, *lhs, *rhs, out),
        IrInstr::MulFx { dst, lhs, rhs } => emit_3reg(Opcode::MulFx, *dst, *lhs, *rhs, out),
        IrInstr::DivFx { dst, lhs, rhs } => emit_3reg(Opcode::DivFx, *dst, *lhs, *rhs, out),
        IrInstr::Jmp { label } => {
            out.push(Opcode::Jmp.byte());
            let addr = *label_pc.get(label).ok_or(FrontendError {
                pos: 0,
                message: format!("unknown label '{}'", label),
            })?;
            write_u32_le(out, addr);
        }
        IrInstr::JmpIf { cond, label } => {
            out.push(Opcode::JmpIf.byte());
            write_u16_le(out, *cond);
            let addr = *label_pc.get(label).ok_or(FrontendError {
                pos: 0,
                message: format!("unknown label '{}'", label),
            })?;
            write_u32_le(out, addr);
        }
        IrInstr::Assert { cond } => {
            out.push(Opcode::Assert.byte());
            write_u16_le(out, *cond);
        }
        IrInstr::Call { dst, name, args } => {
            out.push(Opcode::Call.byte());
            match dst {
                Some(r) => {
                    out.push(1);
                    write_u16_le(out, *r);
                }
                None => {
                    out.push(0);
                    write_u16_le(out, 0);
                }
            }
            write_u16_le(out, interner.lookup(name)?);
            write_u16_le(
                out,
                u16::try_from(args.len()).map_err(|_| FrontendError {
                    pos: 0,
                    message: "too many call args".to_string(),
                })?,
            );
            for a in args {
                write_u16_le(out, *a);
            }
        }
        IrInstr::GateRead {
            dst,
            device_id,
            port,
        } => {
            out.push(Opcode::GateRead.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, *device_id);
            write_u16_le(out, *port);
        }
        IrInstr::GateWrite {
            device_id,
            port,
            src,
        } => {
            out.push(Opcode::GateWrite.byte());
            write_u16_le(out, *device_id);
            write_u16_le(out, *port);
            write_u16_le(out, *src);
        }
        IrInstr::PulseEmit { signal } => {
            out.push(Opcode::PulseEmit.byte());
            write_u16_le(out, interner.lookup(signal)?);
        }
        IrInstr::StateQuery { dst, key } => {
            out.push(Opcode::StateQuery.byte());
            write_u16_le(out, *dst);
            write_u16_le(out, interner.lookup(key)?);
        }
        IrInstr::StateUpdate { key, src } => {
            out.push(Opcode::StateUpdate.byte());
            write_u16_le(out, interner.lookup(key)?);
            write_u16_le(out, *src);
        }
        IrInstr::EventPost { signal } => {
            out.push(Opcode::EventPost.byte());
            write_u16_le(out, interner.lookup(signal)?);
        }
        IrInstr::ClockRead { dst } => {
            out.push(Opcode::ClockRead.byte());
            write_u16_le(out, *dst);
        }
        IrInstr::Ret { src } => {
            out.push(Opcode::Ret.byte());
            match src {
                Some(r) => {
                    out.push(1);
                    write_u16_le(out, *r);
                }
                None => {
                    out.push(0);
                }
            }
        }
    }
    Ok(())
}

fn emit_3reg(op: Opcode, dst: u16, lhs: u16, rhs: u16, out: &mut Vec<u8>) {
    out.push(op.byte());
    write_u16_le(out, dst);
    write_u16_le(out, lhs);
    write_u16_le(out, rhs);
}

fn emit_2reg(op: Opcode, dst: u16, src: u16, out: &mut Vec<u8>) {
    out.push(op.byte());
    write_u16_le(out, dst);
    write_u16_le(out, src);
}

fn has_v1_math_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| {
            matches!(
                i,
                IrInstr::LoadF64 { .. }
                    | IrInstr::AddF64 { .. }
                    | IrInstr::SubF64 { .. }
                    | IrInstr::MulF64 { .. }
                    | IrInstr::DivF64 { .. }
            )
        })
    })
}

fn has_v2_fx_instr(funcs: &[IrFunction]) -> bool {
    funcs
        .iter()
        .any(|f| f.instrs.iter().any(|i| matches!(i, IrInstr::LoadFx { .. })))
}

fn has_v3_fx_math_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| {
            matches!(
                i,
                IrInstr::AddFx { .. }
                    | IrInstr::SubFx { .. }
                    | IrInstr::MulFx { .. }
                    | IrInstr::DivFx { .. }
            )
        })
    })
}

fn has_v4_state_query_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::StateQuery { .. }))
    })
}

fn has_v5_state_update_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::StateUpdate { .. }))
    })
}

fn has_v6_event_post_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::EventPost { .. }))
    })
}

fn has_v7_clock_read_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::ClockRead { .. }))
    })
}

fn has_v8_text_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| match i {
            IrInstr::LoadText { .. } | IrInstr::ConcatText { .. } => true,
            IrInstr::Call { name, .. } => name == "to_text",
            _ => false,
        })
    })
}

fn has_v16_stdout_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::Call { name, .. } if name == "print"))
    })
}

fn has_v17_application_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| {
            matches!(
                i,
                IrInstr::Call { name, .. }
                    if matches!(
                        name.as_str(),
                        "args_read"
                            | "stdin_read_text"
                            | "stdout_write"
                            | "stderr_write"
                            | "path_inspect"
                            | "fs_read_text"
                            | "fs_write_text"
                            | "time_duration_ms"
                    )
            )
        })
    })
}

/// #1732 (FA-05-002): QTruth is the only IR-level trigger whose emitted
/// opcodes (`Opcode::QTruthAnd`/`QTruthOr`/`QTruthNot`/`QTruthImpl`) are
/// currently assigned a non-baseline `Opcode::minimum_semcode_revision()`
/// (see sm-format). This predicate exists to promote the header at the IR
/// level, mirroring every other `has_vN_*_instr` promotion in this file;
/// the actual minimum revision number itself lives in exactly one place,
/// sm-format's `Opcode::minimum_semcode_revision`, which `sm-verify` uses
/// independently to enforce admission - this function only decides
/// *whether* to promote, not *what revision* QTruth requires.
fn has_v18_qtruth_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| {
            matches!(
                i,
                IrInstr::QTruthAnd { .. }
                    | IrInstr::QTruthOr { .. }
                    | IrInstr::QTruthNot { .. }
                    | IrInstr::QTruthImpl { .. }
            )
        })
    })
}

fn has_v9_sequence_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| {
            matches!(
                i,
                IrInstr::MakeSequence { .. } | IrInstr::SequenceGet { .. }
            )
        })
    })
}

fn has_v13_sequence_iter_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| {
            matches!(
                i,
                IrInstr::SequenceLen { .. }
                    | IrInstr::SequenceIsEmpty { .. }
                    | IrInstr::SequenceContains { .. }
                    | IrInstr::SequencePush { .. }
                    | IrInstr::SequencePrepend { .. }
                    | IrInstr::SequencePop { .. }
            )
        })
    })
}

fn has_v14_map_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs.iter().any(|i| {
            matches!(
                i,
                IrInstr::MapEmpty { .. }
                    | IrInstr::MapContains { .. }
                    | IrInstr::MapGet { .. }
                    | IrInstr::MapSet { .. }
            )
        })
    })
}

fn has_v15_prng_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::RngSeed { .. } | IrInstr::RngNextI32 { .. }))
    })
}

fn has_v10_closure_instr(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| {
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::MakeClosure { .. } | IrInstr::ClosureCall { .. }))
    })
}

fn has_v11_ownership_events(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|f| !f.ownership_events.is_empty())
}

fn has_v12_record_field_ownership_events(funcs: &[IrFunction]) -> bool {
    funcs.iter().any(|function| {
        function.ownership_events.iter().any(|event| {
            event
                .path
                .components
                .iter()
                .any(|component| matches!(component, PathComponent::Field(_)))
        })
    })
}

fn emit_ownership_events(
    ownership_events: &[OwnershipPathEvent],
    require_section: bool,
    out: &mut Vec<u8>,
) -> Result<(), FrontendError> {
    if ownership_events.is_empty() {
        if require_section {
            // Header claims CAP_OWNERSHIP_PATHS; emit an empty OWN0 section so
            // the verifier check "at least one function has OWN0" passes.
            out.extend_from_slice(&OWNERSHIP_SECTION_TAG);
            write_u16_le(out, 0);
        }
        return Ok(());
    }

    out.extend_from_slice(&OWNERSHIP_SECTION_TAG);
    write_u16_le(
        out,
        u16::try_from(ownership_events.len()).map_err(|_| FrontendError {
            pos: 0,
            message: "too many ownership path events".to_string(),
        })?,
    );
    for event in ownership_events {
        out.push(match event.kind {
            OwnershipPathEventKind::Borrow => OWNERSHIP_EVENT_KIND_BORROW,
            OwnershipPathEventKind::Write => OWNERSHIP_EVENT_KIND_WRITE,
        });
        write_u32_le(out, event.path.root.0);
        write_u16_le(
            out,
            u16::try_from(event.path.components.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "ownership path is too deep".to_string(),
            })?,
        );
        for component in &event.path.components {
            match component {
                PathComponent::TupleIndex(index) => {
                    out.push(OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX);
                    write_u16_le(out, *index);
                }
                PathComponent::SequenceIndexStatic(index) => {
                    out.push(OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX);
                    write_u32_le(out, *index);
                }
                PathComponent::Field(name) => {
                    out.push(OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL);
                    write_u32_le(out, name.0);
                }
                PathComponent::AdtPayload { variant, index } => {
                    out.push(sm_format::semcode_format::OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD);
                    write_u32_le(out, variant.0);
                    write_u16_le(out, *index);
                }
            }
        }
    }
    Ok(())
}

fn is_numeric_literal_like_expr(expr_id: ExprId, arena: &AstArena) -> bool {
    match arena.expr(expr_id) {
        Expr::NumericLiteral(_) => true,
        Expr::Unary(UnaryOp::Pos | UnaryOp::Neg, inner) => {
            is_numeric_literal_like_expr(*inner, arena)
        }
        _ => false,
    }
}

fn erased_expected(expected: Option<&Type>) -> Option<Type> {
    expected.map(Type::erase_units)
}

fn lift_lowered_type(
    expected: Option<&Type>,
    actual: &Type,
    expr_id: ExprId,
    arena: &AstArena,
) -> Type {
    match expected {
        Some(expected_ty)
            if matches!(expected_ty.measured_parts(), Some((base, _)) if base == actual)
                && is_numeric_literal_like_expr(expr_id, arena) =>
        {
            expected_ty.clone()
        }
        _ => actual.clone(),
    }
}

#[derive(Debug, Default)]
struct StringInterner {
    ids: HashMap<String, u16>,
    by_id: Vec<String>,
}

impl StringInterner {
    fn new() -> Self {
        Self::default()
    }

    fn id(&mut self, s: &str) -> Result<u16, FrontendError> {
        if let Some(id) = self.ids.get(s) {
            return Ok(*id);
        }
        let id = u16::try_from(self.by_id.len()).map_err(|_| FrontendError {
            pos: 0,
            message: "string table overflow".to_string(),
        })?;
        self.ids.insert(s.to_string(), id);
        self.by_id.push(s.to_string());
        Ok(id)
    }

    fn lookup(&self, s: &str) -> Result<u16, FrontendError> {
        self.ids.get(s).copied().ok_or(FrontendError {
            pos: 0,
            message: format!("string '{}' not interned", s),
        })
    }

    fn emit_table(&self, out: &mut Vec<u8>) -> Result<(), FrontendError> {
        write_u16_le(
            out,
            u16::try_from(self.by_id.len()).map_err(|_| FrontendError {
                pos: 0,
                message: "string table too large".to_string(),
            })?,
        );
        for s in &self.by_id {
            let b = s.as_bytes();
            write_u16_le(
                out,
                u16::try_from(b.len()).map_err(|_| FrontendError {
                    pos: 0,
                    message: "string too long".to_string(),
                })?,
            );
            out.extend_from_slice(b);
        }
        Ok(())
    }
}

fn next_closure_function_name(closure_state: &mut ClosureLoweringState) -> String {
    let id = closure_state.next_closure_id;
    closure_state.next_closure_id += 1;
    format!(
        "__closure_{}_{}",
        closure_state.parent_fn_name.replace("::", "_"),
        id
    )
}

fn lower_closure_literal_expr(
    closure: &ClosureLiteral,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<&Type>,
    closure_state: &mut ClosureLoweringState,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(u16, Type), FrontendError> {
    let Some(Type::Closure(expected_closure)) = expected else {
        return Err(FrontendError {
            pos: 0,
            message:
                "canonical lowering for first-class closures requires contextual Closure(T -> U) type in M8.4 Wave 3"
                    .to_string(),
        });
    };
    if expected_closure.family != ClosureValueFamily::UnaryDirect
        || expected_closure.capture != ClosureCapturePolicy::Immutable
    {
        return Err(FrontendError {
            pos: 0,
            message:
                "canonical lowering currently admits only the UnaryDirect immutable closure family in M8.4 Wave 3"
                    .to_string(),
        });
    }

    let helper_name = next_closure_function_name(closure_state);
    let mut lifted_env = ScopeEnv::new();
    let mut lifted_instrs = Vec::new();
    let mut local_next = u16::try_from(closure.captures.len() + 1).map_err(|_| FrontendError {
        pos: 0,
        message: "closure parameter/capture count exceeds register space".to_string(),
    })?;
    let mut local_loop_stack = Vec::new();
    // #1709: the lifted helper is a new `IrFunction`, hence a new
    // function-owned ownership-event sink (§25/§26) - it must NOT share the
    // parent's `ownership_events` sink (that would leak the closure body's
    // own Borrow/Write events into the parent frame's event stream, which
    // later has to line up with the parent's own instruction/frame
    // boundaries) and must NOT be discarded as `Vec::new()` at construction
    // (that was the original #1709 defect for this call site).
    let mut lifted_ownership_events: Vec<OwnershipPathEvent> = Vec::new();
    // #1724 (FA-04-018): same reasoning as `lifted_ownership_events` above,
    // for lexical binding identity instead of ownership events - the
    // lifted helper is a new `IrFunction`, hence its own fresh runtime-local
    // namespace. A capture's parent-side key and its child-side key are
    // deliberately different authorities/values (§11): the child must not
    // resolve captures against the parent's frame.
    let mut lifted_lowered_locals = LoweredLocalEnv::new();
    // #1773 (FA-09-005): the lifted helper's real invocation convention is
    // captures first (r0..captures.len()-1), then the closure's own param
    // (r{captures.len()}) - built here from the exact same `capture_ty`/
    // `expected_closure.param` this function already uses to bind those
    // registers below, so the signature can never drift from the actual
    // register layout.
    let mut lifted_signature_params = Vec::with_capacity(closure.captures.len() + 1);

    for (index, capture) in closure.captures.iter().enumerate() {
        let capture_ty = env.get(*capture).ok_or(FrontendError {
            pos: 0,
            message: format!(
                "unknown captured value '{}' during closure lowering",
                resolve_symbol_name(arena, *capture)?
            ),
        })?;
        lifted_signature_params.push(callable_family_for_type(&capture_ty)?);
        if env.is_const(*capture)? {
            lifted_env.insert_const(*capture, capture_ty.clone());
        } else {
            lifted_env.insert(*capture, capture_ty.clone());
        }
        lifted_instrs.push(IrInstr::StoreVar {
            name: lifted_lowered_locals.bind(arena, *capture)?,
            src: u16::try_from(index).map_err(|_| FrontendError {
                pos: 0,
                message: "closure capture index exceeds v0 limit".to_string(),
            })?,
        });
    }

    let param_reg = u16::try_from(closure.captures.len()).map_err(|_| FrontendError {
        pos: 0,
        message: "closure parameter index exceeds v0 limit".to_string(),
    })?;
    lifted_signature_params.push(callable_family_for_type(expected_closure.param.as_ref())?);
    lifted_env.insert(closure.param, expected_closure.param.as_ref().clone());
    lifted_instrs.push(IrInstr::StoreVar {
        name: lifted_lowered_locals.bind(arena, closure.param)?,
        src: param_reg,
    });

    // #1709 corrective: `append_record_update_write_events_from_expr`
    // treats `Expr::Closure` as a deliberate leaf, so no enclosing
    // statement-level scan ever reaches into a closure body - the body is
    // its own function's root, and needs its own root-level prescan the
    // same way a top-level `lower_stmt` arm prescans its own statement's
    // expression before lowering it. Writes into the child's own sink, not
    // the parent's - the closure body is a new function boundary.
    append_record_update_write_events_from_expr(closure.body, arena, &mut lifted_ownership_events);
    let (body_reg, body_ty) = lower_expr_with_expected(
        closure.body,
        arena,
        &mut local_next,
        &mut lifted_instrs,
        &lifted_env,
        &mut local_loop_stack,
        fn_table,
        record_table,
        adt_table,
        Some(expected_closure.ret.as_ref().clone()),
        expected_closure.ret.as_ref().clone(),
        closure_state,
        &mut lifted_ownership_events,
        &mut lifted_lowered_locals,
    )?;
    if body_ty != expected_closure.ret.as_ref().clone() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "lifted closure body type mismatch during lowering: expected {:?}, got {:?}",
                expected_closure.ret, body_ty
            ),
        });
    }
    lifted_instrs.push(IrInstr::Ret {
        src: Some(body_reg),
    });
    closure_state.lifted_funcs.push(IrFunction {
        name: helper_name.clone(),
        instrs: lifted_instrs,
        ownership_events: lifted_ownership_events,
        params: lifted_signature_params,
    });

    let mut capture_regs = Vec::with_capacity(closure.captures.len());
    for capture in &closure.captures {
        let capture_reg = alloc(next);
        out.push(IrInstr::LoadVar {
            dst: capture_reg,
            name: lowered_locals.resolve(arena, *capture)?,
        });
        capture_regs.push(capture_reg);
    }
    let dst = alloc(next);
    out.push(IrInstr::MakeClosure {
        dst,
        name: helper_name,
        captures: capture_regs,
    });
    Ok((dst, Type::Closure(expected_closure.clone())))
}

fn lower_direct_closure_call_expr(
    name: SymbolId,
    args: &[CallArg],
    closure_ty: &ClosureType,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(u16, Type), FrontendError> {
    if closure_ty.family != ClosureValueFamily::UnaryDirect
        || closure_ty.capture != ClosureCapturePolicy::Immutable
    {
        return Err(FrontendError {
            pos: 0,
            message:
                "direct invocation lowering currently admits only the UnaryDirect immutable closure family in M8.4 Wave 3"
                    .to_string(),
        });
    }
    if args.len() != 1 || args.iter().any(|arg| arg.name.is_some()) {
        return Err(FrontendError {
            pos: 0,
            message:
                "direct invocation of first-class closure values currently requires exactly one positional argument in M8.4 Wave 3"
                    .to_string(),
        });
    }
    if closure_ty.ret.as_ref() == &Type::Unit {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "unit-returning direct closure call '{}' cannot be used as expression value",
                resolve_symbol_name(arena, name)?
            ),
        });
    }

    let closure_reg = alloc(next);
    out.push(IrInstr::LoadVar {
        dst: closure_reg,
        name: lowered_locals.resolve(arena, name)?,
    });
    let (arg_reg, arg_ty) = lower_expr_with_expected(
        args[0].value,
        arena,
        next,
        out,
        env,
        loop_stack,
        fn_table,
        record_table,
        adt_table,
        Some(closure_ty.param.as_ref().clone()),
        ret_ty,
        closure_state,
        ownership_events,
        lowered_locals,
    )?;
    if arg_ty != closure_ty.param.as_ref().clone() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "closure argument for '{}' has type {:?}, expected {:?}",
                resolve_symbol_name(arena, name)?,
                arg_ty,
                closure_ty.param
            ),
        });
    }
    let dst = alloc(next);
    out.push(IrInstr::ClosureCall {
        dst: Some(dst),
        closure: closure_reg,
        arg: arg_reg,
    });
    Ok((dst, closure_ty.ret.as_ref().clone()))
}

fn lower_direct_closure_call_stmt(
    name: SymbolId,
    args: &[CallArg],
    closure_ty: &ClosureType,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(), FrontendError> {
    if closure_ty.family != ClosureValueFamily::UnaryDirect
        || closure_ty.capture != ClosureCapturePolicy::Immutable
    {
        return Err(FrontendError {
            pos: 0,
            message:
                "direct invocation lowering currently admits only the UnaryDirect immutable closure family in M8.4 Wave 3"
                    .to_string(),
        });
    }
    if args.len() != 1 || args.iter().any(|arg| arg.name.is_some()) {
        return Err(FrontendError {
            pos: 0,
            message:
                "direct invocation of first-class closure values currently requires exactly one positional argument in M8.4 Wave 3"
                    .to_string(),
        });
    }
    let closure_reg = alloc(next);
    out.push(IrInstr::LoadVar {
        dst: closure_reg,
        name: lowered_locals.resolve(arena, name)?,
    });
    let (arg_reg, arg_ty) = lower_expr_with_expected(
        args[0].value,
        arena,
        next,
        out,
        env,
        loop_stack,
        fn_table,
        record_table,
        adt_table,
        Some(closure_ty.param.as_ref().clone()),
        ret_ty,
        closure_state,
        ownership_events,
        lowered_locals,
    )?;
    if arg_ty != closure_ty.param.as_ref().clone() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "closure argument for '{}' has type {:?}, expected {:?}",
                resolve_symbol_name(arena, name)?,
                arg_ty,
                closure_ty.param
            ),
        });
    }
    let dst = if closure_ty.ret.as_ref() == &Type::Unit {
        None
    } else {
        Some(alloc(next))
    };
    out.push(IrInstr::ClosureCall {
        dst,
        closure: closure_reg,
        arg: arg_reg,
    });
    Ok(())
}

fn lower_expr(
    expr_id: ExprId,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(u16, Type), FrontendError> {
    lower_expr_with_expected(
        expr_id,
        arena,
        next,
        out,
        env,
        loop_stack,
        fn_table,
        record_table,
        adt_table,
        None,
        ret_ty,
        closure_state,
        ownership_events,
        lowered_locals,
    )
}

fn lower_expr_with_expected(
    expr_id: ExprId,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<Type>,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(u16, Type), FrontendError> {
    match arena.expr(expr_id) {
        Expr::QuadLiteral(v) => {
            let r = alloc(next);
            out.push(IrInstr::LoadQ { dst: r, val: *v });
            Ok((r, Type::Quad))
        }
        Expr::BoolLiteral(v) => {
            let r = alloc(next);
            out.push(IrInstr::LoadBool { dst: r, val: *v });
            Ok((r, Type::Bool))
        }
        Expr::TextLiteral(lit) => {
            let r = alloc(next);
            out.push(IrInstr::LoadText {
                dst: r,
                val: lit.spelling.clone(),
            });
            Ok((r, Type::Text))
        }
        Expr::SequenceLiteral(sequence) => {
            let expected_item_ty = match expected.as_ref() {
                Some(Type::Sequence(sequence_ty)) => Some(sequence_ty.item.as_ref().clone()),
                _ => None,
            };
            if sequence.items.is_empty() && expected_item_ty.is_none() {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "empty ordered sequence literal currently requires contextual Sequence(type) in M8.3 Wave 2"
                            .to_string(),
                });
            }
            let mut item_regs = Vec::with_capacity(sequence.items.len());
            let mut item_ty = expected_item_ty;
            for item in &sequence.items {
                let (reg, actual_ty) = lower_expr_with_expected(
                    *item,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    item_ty.clone(),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                if let Some(expected_item_ty) = item_ty.as_ref() {
                    if *expected_item_ty != actual_ty {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "ordered sequence literal item type mismatch during lowering: expected {:?}, got {:?}",
                                expected_item_ty, actual_ty
                            ),
                        });
                    }
                } else {
                    item_ty = Some(actual_ty.clone());
                }
                item_regs.push(reg);
            }
            let item_ty = item_ty.ok_or(FrontendError {
                pos: 0,
                message: "ordered sequence literal lowering requires at least one item or contextual Sequence(type)".to_string(),
            })?;
            let dst = alloc(next);
            out.push(IrInstr::MakeSequence {
                dst,
                items: item_regs,
            });
            Ok((
                dst,
                Type::Sequence(SequenceType {
                    family: SequenceCollectionFamily::OrderedSequence,
                    item: Box::new(item_ty),
                }),
            ))
        }
        Expr::Closure(closure) => lower_closure_literal_expr(
            closure,
            arena,
            next,
            out,
            env,
            fn_table,
            record_table,
            adt_table,
            expected.as_ref(),
            closure_state,
            lowered_locals,
        ),
        Expr::Range(range_expr) => {
            let (start_reg, start_ty) = lower_expr_with_expected(
                range_expr.start,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::I32),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if start_ty != Type::I32 {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "range literal currently requires i32 bounds, got {:?}",
                        start_ty
                    ),
                });
            }
            let (end_reg, end_ty) = lower_expr_with_expected(
                range_expr.end,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::I32),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if end_ty != Type::I32 {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "range literal currently requires i32 bounds, got {:?}",
                        end_ty
                    ),
                });
            }
            let inclusive_reg = alloc(next);
            out.push(IrInstr::LoadBool {
                dst: inclusive_reg,
                val: range_expr.inclusive,
            });
            let dst = alloc(next);
            out.push(IrInstr::MakeTuple {
                dst,
                items: vec![start_reg, end_reg, inclusive_reg],
            });
            Ok((dst, Type::RangeI32))
        }
        Expr::Tuple(items) => {
            let expected_items = match expected.as_ref() {
                Some(Type::Tuple(types)) => Some(types),
                _ => None,
            };
            if let Some(types) = expected_items {
                if types.len() != items.len() {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "tuple arity mismatch in lowering: expected {}, got {}",
                            types.len(),
                            items.len()
                        ),
                    });
                }
            }
            let mut regs = Vec::with_capacity(items.len());
            let mut tys = Vec::with_capacity(items.len());
            for (index, item) in items.iter().enumerate() {
                let item_expected = expected_items.and_then(|types| types.get(index)).cloned();
                let (reg, ty) = lower_expr_with_expected(
                    *item,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    item_expected,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                regs.push(reg);
                tys.push(ty);
            }
            let dst = alloc(next);
            out.push(IrInstr::MakeTuple { dst, items: regs });
            Ok((dst, Type::Tuple(tys)))
        }
        Expr::RecordLiteral(record_literal) => {
            let record = record_table
                .get(&record_literal.name)
                .ok_or(FrontendError {
                    pos: 0,
                    message: format!(
                        "unknown record type '{}' in record literal lowering",
                        resolve_symbol_name(arena, record_literal.name)?
                    ),
                })?;
            let mut lowered_fields = HashMap::new();
            for field in &record_literal.fields {
                let expected_field_ty = record
                    .fields
                    .iter()
                    .find(|decl_field| decl_field.name == field.name)
                    .map(|decl_field| decl_field.ty.clone())
                    .ok_or(FrontendError {
                        pos: 0,
                        message: format!(
                            "record literal '{}' has no field named '{}' during lowering",
                            resolve_symbol_name(arena, record_literal.name)?,
                            resolve_symbol_name(arena, field.name)?
                        ),
                    })?;
                let (reg, _) = lower_expr_with_expected(
                    field.value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(expected_field_ty),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                lowered_fields.insert(field.name, reg);
            }
            let mut ordered_regs = Vec::with_capacity(record.fields.len());
            for decl_field in &record.fields {
                let reg = lowered_fields
                    .get(&decl_field.name)
                    .copied()
                    .ok_or(FrontendError {
                        pos: 0,
                        message: format!(
                            "record literal '{}' is missing field '{}' during lowering",
                            resolve_symbol_name(arena, record_literal.name)?,
                            resolve_symbol_name(arena, decl_field.name)?
                        ),
                    })?;
                ordered_regs.push(reg);
            }
            let dst = alloc(next);
            out.push(IrInstr::MakeRecord {
                dst,
                name: resolve_symbol_name(arena, record_literal.name)?.to_string(),
                items: ordered_regs,
            });
            Ok((dst, Type::Record(record_literal.name)))
        }
        Expr::RecordField(field_expr) => {
            let (src, base_ty) = lower_expr(
                field_expr.base,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let Type::Record(record_name) = base_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "record field access lowering requires record base before '.{}', got {:?}",
                        resolve_symbol_name(arena, field_expr.field)?,
                        base_ty
                    ),
                });
            };
            let record = record_table.get(&record_name).ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "unknown record type '{}' in field access lowering",
                    resolve_symbol_name(arena, record_name)?
                ),
            })?;
            let (index, field) = record
                .fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == field_expr.field)
                .ok_or(FrontendError {
                    pos: 0,
                    message: format!(
                        "record type '{}' has no field named '{}' during lowering",
                        resolve_symbol_name(arena, record_name)?,
                        resolve_symbol_name(arena, field_expr.field)?
                    ),
                })?;
            let dst = alloc(next);
            out.push(IrInstr::RecordGet {
                dst,
                src,
                record_name: resolve_symbol_name(arena, record_name)?.to_string(),
                index: u16::try_from(index).map_err(|_| FrontendError {
                    pos: 0,
                    message: "record field slot index exceeds v0 limit".to_string(),
                })?,
            });
            Ok((dst, field.ty.clone()))
        }
        Expr::SequenceIndex(index_expr) => {
            let (src, base_ty) = lower_expr(
                index_expr.base,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let Type::Sequence(sequence_ty) = base_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "sequence indexing lowering requires Sequence(type) base before '[...]', got {:?}",
                        base_ty
                    ),
                });
            };
            let (index_reg, index_ty) = lower_expr_with_expected(
                index_expr.index,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::I32),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if index_ty != Type::I32 {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "sequence indexing currently requires i32 index during lowering, got {:?}",
                        index_ty
                    ),
                });
            }
            let dst = alloc(next);
            out.push(IrInstr::SequenceGet {
                dst,
                src,
                index: index_reg,
            });
            Ok((dst, sequence_ty.item.as_ref().clone()))
        }
        Expr::RecordUpdate(update_expr) => {
            let (base_reg, base_ty) = lower_expr(
                update_expr.base,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let Type::Record(record_name) = base_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "record copy-with lowering requires record base before 'with', got {:?}",
                        base_ty
                    ),
                });
            };
            let record = record_table.get(&record_name).ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "unknown record type '{}' in record copy-with lowering",
                    resolve_symbol_name(arena, record_name)?
                ),
            })?;
            if update_expr.fields.is_empty() {
                return Err(FrontendError {
                    pos: 0,
                    message: "record copy-with requires at least one explicit override field"
                        .to_string(),
                });
            }
            let mut lowered_overrides = HashMap::new();
            for field in &update_expr.fields {
                let expected_field_ty = record
                    .fields
                    .iter()
                    .find(|decl_field| decl_field.name == field.name)
                    .map(|decl_field| decl_field.ty.clone())
                    .ok_or(FrontendError {
                        pos: 0,
                        message: format!(
                            "record copy-with '{}' has no field named '{}' during lowering",
                            resolve_symbol_name(arena, record_name)?,
                            resolve_symbol_name(arena, field.name)?
                        ),
                    })?;
                let (reg, _) = lower_expr_with_expected(
                    field.value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(expected_field_ty),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                if lowered_overrides.insert(field.name, reg).is_some() {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "record copy-with '{}' cannot repeat field '{}' during lowering",
                            resolve_symbol_name(arena, record_name)?,
                            resolve_symbol_name(arena, field.name)?
                        ),
                    });
                }
            }
            let mut ordered_regs = Vec::with_capacity(record.fields.len());
            for (index, decl_field) in record.fields.iter().enumerate() {
                if let Some(override_reg) = lowered_overrides.get(&decl_field.name).copied() {
                    ordered_regs.push(override_reg);
                    continue;
                }
                let reg = alloc(next);
                out.push(IrInstr::RecordGet {
                    dst: reg,
                    src: base_reg,
                    record_name: resolve_symbol_name(arena, record_name)?.to_string(),
                    index: u16::try_from(index).map_err(|_| FrontendError {
                        pos: 0,
                        message: "record copy-with slot index exceeds v0 limit".to_string(),
                    })?,
                });
                ordered_regs.push(reg);
            }
            let dst = alloc(next);
            out.push(IrInstr::MakeRecord {
                dst,
                name: resolve_symbol_name(arena, record_name)?.to_string(),
                items: ordered_regs,
            });
            Ok((dst, Type::Record(record_name)))
        }
        Expr::AdtCtor(ctor_expr) => lower_adt_ctor_expr(
            ctor_expr,
            arena,
            next,
            out,
            env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            expected,
            ret_ty,
            closure_state,
            ownership_events,
            lowered_locals,
        ),
        Expr::NumericLiteral(NumericLiteral::I32(n)) => {
            let r = alloc(next);
            let expected_erased = erased_expected(expected.as_ref());
            if expected_erased == Some(Type::Fx) {
                let val = try_encode_fx_literal_expr(expr_id, arena)?.ok_or(FrontendError {
                    pos: 0,
                    message: "expected fx literal".to_string(),
                })?;
                out.push(IrInstr::LoadFx { dst: r, val });
                Ok((
                    r,
                    lift_lowered_type(expected.as_ref(), &Type::Fx, expr_id, arena),
                ))
            } else {
                let val = i32::try_from(*n).map_err(|_| FrontendError {
                    pos: 0,
                    message: format!("numeric literal {} does not fit in i32", n),
                })?;
                out.push(IrInstr::LoadI32 { dst: r, val });
                Ok((
                    r,
                    lift_lowered_type(expected.as_ref(), &Type::I32, expr_id, arena),
                ))
            }
        }
        Expr::NumericLiteral(NumericLiteral::U32(n)) => {
            let r = alloc(next);
            let expected_erased = erased_expected(expected.as_ref());
            if expected_erased == Some(Type::Fx) {
                let val = try_encode_fx_literal_expr(expr_id, arena)?.ok_or(FrontendError {
                    pos: 0,
                    message: "expected fx literal".to_string(),
                })?;
                out.push(IrInstr::LoadFx { dst: r, val });
                Ok((
                    r,
                    lift_lowered_type(expected.as_ref(), &Type::Fx, expr_id, arena),
                ))
            } else {
                out.push(IrInstr::LoadU32 { dst: r, val: *n });
                Ok((
                    r,
                    lift_lowered_type(expected.as_ref(), &Type::U32, expr_id, arena),
                ))
            }
        }
        Expr::NumericLiteral(NumericLiteral::F64(n)) => {
            let r = alloc(next);
            let expected_erased = erased_expected(expected.as_ref());
            if expected_erased == Some(Type::Fx) {
                out.push(IrInstr::LoadFx {
                    dst: r,
                    val: encode_fx_literal(*n)?,
                });
                Ok((
                    r,
                    lift_lowered_type(expected.as_ref(), &Type::Fx, expr_id, arena),
                ))
            } else {
                out.push(IrInstr::LoadF64 { dst: r, val: *n });
                Ok((
                    r,
                    lift_lowered_type(expected.as_ref(), &Type::F64, expr_id, arena),
                ))
            }
        }
        Expr::NumericLiteral(NumericLiteral::Fx(n)) => {
            let r = alloc(next);
            out.push(IrInstr::LoadFx {
                dst: r,
                val: encode_fx_literal(*n)?,
            });
            Ok((
                r,
                lift_lowered_type(expected.as_ref(), &Type::Fx, expr_id, arena),
            ))
        }
        Expr::Var(name) => {
            let ty = env.get(*name).ok_or(FrontendError {
                pos: 0,
                message: format!("unknown variable '{}'", resolve_symbol_name(arena, *name)?),
            })?;
            let r = alloc(next);
            out.push(IrInstr::LoadVar {
                dst: r,
                name: lowered_locals.resolve(arena, *name)?,
            });
            Ok((r, ty))
        }
        Expr::Block(block) => lower_value_block_expr(
            block,
            arena,
            next,
            out,
            env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            expected,
            ret_ty,
            closure_state,
            ownership_events,
            lowered_locals,
        ),
        Expr::If(if_expr) => {
            let (cond_reg, cond_ty) = lower_expr(
                if_expr.condition,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if expression condition must be bool".to_string(),
                });
            }

            let id = alloc_if_expr_id(next);
            let then_label = format!("if_expr_{}_then", id);
            let else_label = format!("if_expr_{}_else", id);
            let end_label = format!("if_expr_{}_end", id);
            let result_name = format!("__if_expr_{}_result", id);

            out.push(IrInstr::JmpIf {
                cond: cond_reg,
                label: then_label.clone(),
            });
            out.push(IrInstr::Jmp {
                label: else_label.clone(),
            });

            out.push(IrInstr::Label { name: then_label });
            let (then_reg, then_ty) = lower_value_block_expr(
                &if_expr.then_block,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                expected.clone(),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            out.push(IrInstr::StoreVar {
                name: result_name.clone(),
                src: then_reg,
            });
            out.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            out.push(IrInstr::Label { name: else_label });
            let (else_reg, else_ty) = lower_value_block_expr(
                &if_expr.else_block,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                expected.clone(),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if then_ty != else_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "if expression branch type mismatch in lowering: then {:?}, else {:?}",
                        then_ty, else_ty
                    ),
                });
            }
            out.push(IrInstr::StoreVar {
                name: result_name.clone(),
                src: else_reg,
            });
            out.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            out.push(IrInstr::Label { name: end_label });
            let dst = alloc(next);
            out.push(IrInstr::LoadVar {
                dst,
                name: result_name,
            });
            Ok((dst, then_ty))
        }
        Expr::Loop(loop_expr) => lower_loop_expr(
            loop_expr,
            arena,
            next,
            out,
            env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            expected,
            ret_ty,
            closure_state,
            ownership_events,
            lowered_locals,
        ),
        Expr::Match(match_expr) => lower_match_expr(
            match_expr,
            arena,
            next,
            out,
            env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            expected,
            ret_ty,
            closure_state,
            ownership_events,
            lowered_locals,
        ),
        Expr::Call(name, args) => {
            if is_builtin_assert_name(*name, arena, fn_table)? {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "assert builtin is statement-only and cannot be used as expression value"
                            .to_string(),
                });
            }
            // builtin len(sequence) -> i32
            if resolve_symbol_name(arena, *name)? == "len" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'len' takes exactly one positional argument".to_string(),
                    });
                }
                let (src, arg_ty) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    None,
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                return match &arg_ty {
                    Type::Sequence(_) => {
                        let dst = alloc(next);
                        out.push(IrInstr::SequenceLen { dst, src });
                        Ok((dst, Type::I32))
                    }
                    _ => Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'len' expects a Sequence argument, got {:?}",
                            arg_ty
                        ),
                    }),
                };
            }
            // builtin is_empty(sequence) -> bool
            if resolve_symbol_name(arena, *name)? == "is_empty" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'is_empty' takes exactly one positional argument"
                            .to_string(),
                    });
                }
                let (src, arg_ty) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    None,
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                return match &arg_ty {
                    Type::Sequence(_) => {
                        let dst = alloc(next);
                        out.push(IrInstr::SequenceIsEmpty { dst, src });
                        Ok((dst, Type::Bool))
                    }
                    _ => Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'is_empty' expects a Sequence argument, got {:?}",
                            arg_ty
                        ),
                    }),
                };
            }
            // builtin push / prepend (sequence, value) -> Sequence(T)  [persistent]
            let name_str = resolve_symbol_name(arena, *name)?;
            if name_str == "push" || name_str == "prepend" {
                if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin '{name_str}' takes exactly two positional arguments"
                        ),
                    });
                }
                let (seq, seq_ty) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    None,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let Type::Sequence(seq_type) = &seq_ty else {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin '{name_str}' first argument must be a Sequence, got {:?}",
                            seq_ty
                        ),
                    });
                };
                let elem_ty = seq_type.item.as_ref().clone();
                let (val, val_ty) = lower_expr_with_expected(
                    args[1].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(elem_ty.clone()),
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                if val_ty != elem_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin '{name_str}' second argument type {:?} does not match \
                             sequence element type {:?}",
                            val_ty, elem_ty
                        ),
                    });
                }
                let dst = alloc(next);
                if name_str == "push" {
                    out.push(IrInstr::SequencePush { dst, seq, val });
                } else {
                    out.push(IrInstr::SequencePrepend { dst, seq, val });
                }
                return Ok((dst, seq_ty));
            }
            // builtin contains(sequence, value) -> bool
            if resolve_symbol_name(arena, *name)? == "contains" {
                if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'contains' takes exactly two positional arguments"
                            .to_string(),
                    });
                }
                let (seq, seq_ty) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    None,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let Type::Sequence(seq_type) = &seq_ty else {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'contains' first argument must be a Sequence, got {:?}",
                            seq_ty
                        ),
                    });
                };
                let elem_ty = seq_type.item.as_ref().clone();
                let (val, val_ty) = lower_expr_with_expected(
                    args[1].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(elem_ty.clone()),
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                if val_ty != elem_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'contains' value type {:?} does not match element type {:?}",
                            val_ty, elem_ty
                        ),
                    });
                }
                let dst = alloc(next);
                out.push(IrInstr::SequenceContains { dst, seq, val });
                return Ok((dst, Type::Bool));
            }
            // builtin pop(sequence) -> Sequence(T)
            if resolve_symbol_name(arena, *name)? == "pop" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'pop' takes exactly one positional argument".to_string(),
                    });
                }
                let (src, arg_ty) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    None,
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                return match &arg_ty {
                    Type::Sequence(_) => {
                        let dst = alloc(next);
                        let seq_ty = arg_ty.clone();
                        out.push(IrInstr::SequencePop { dst, src });
                        Ok((dst, seq_ty))
                    }
                    _ => Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'pop' expects a Sequence argument, got {:?}",
                            arg_ty
                        ),
                    }),
                };
            }
            // builtin map_empty() — contextual type required
            if resolve_symbol_name(arena, *name)? == "map_empty" {
                if !args.is_empty() {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'map_empty' takes no arguments".to_string(),
                    });
                }
                // map_empty requires a contextual Map(K,V) type from the let annotation.
                // We must NOT fall back to a sentinel — if expected is missing or not a Map,
                // typecheck should have already rejected the program before lowering runs.
                let map_ty = match expected {
                    Some(ref t @ Type::Map(_)) => t.clone(),
                    Some(ref other) => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "map_empty() requires a Map(K, V) contextual type, got {:?}",
                                other
                            ),
                        })
                    }
                    None => {
                        return Err(FrontendError {
                            pos: 0,
                            message: "map_empty() requires a contextual Map(K, V) type; \
                                 use 'let q: Map(K, V) = map_empty()'"
                                .to_string(),
                        })
                    }
                };
                let dst = alloc(next);
                out.push(IrInstr::MapEmpty { dst });
                return Ok((dst, map_ty));
            }
            // builtin map_contains(Map(K, V), K) -> bool
            if resolve_symbol_name(arena, *name)? == "map_contains" {
                if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'map_contains' takes exactly two positional arguments"
                            .to_string(),
                    });
                }
                let (map_reg, map_ty) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    None,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let Type::Map(ref map_type) = map_ty else {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'map_contains' first argument must be Map, got {:?}",
                            map_ty
                        ),
                    });
                };
                let key_ty = map_type.key.as_ref().clone();
                let (key_reg, _) = lower_expr_with_expected(
                    args[1].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(key_ty),
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let dst = alloc(next);
                out.push(IrInstr::MapContains {
                    dst,
                    map: map_reg,
                    key: key_reg,
                });
                return Ok((dst, Type::Bool));
            }
            // builtin map_get(Map(K, V), K, V) -> V
            if resolve_symbol_name(arena, *name)? == "map_get" {
                if args.len() != 3 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'map_get' takes exactly three positional arguments"
                            .to_string(),
                    });
                }
                let (map_reg, map_ty) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    None,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let Type::Map(ref map_type) = map_ty else {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'map_get' first argument must be Map, got {:?}",
                            map_ty
                        ),
                    });
                };
                let key_ty = map_type.key.as_ref().clone();
                let val_ty = map_type.val.as_ref().clone();
                let (key_reg, _) = lower_expr_with_expected(
                    args[1].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(key_ty),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let (default_reg, _) = lower_expr_with_expected(
                    args[2].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(val_ty.clone()),
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let dst = alloc(next);
                out.push(IrInstr::MapGet {
                    dst,
                    map: map_reg,
                    key: key_reg,
                    default_val: default_reg,
                });
                return Ok((dst, val_ty));
            }
            // builtin map_set(Map(K, V), K, V) -> Map(K, V)
            if resolve_symbol_name(arena, *name)? == "map_set" {
                if args.len() != 3 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'map_set' takes exactly three positional arguments"
                            .to_string(),
                    });
                }
                let (map_reg, map_ty) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    None,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let Type::Map(ref map_type) = map_ty else {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'map_set' first argument must be Map, got {:?}",
                            map_ty
                        ),
                    });
                };
                let key_ty = map_type.key.as_ref().clone();
                let val_ty = map_type.val.as_ref().clone();
                let (key_reg, _) = lower_expr_with_expected(
                    args[1].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(key_ty),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let (val_reg, _) = lower_expr_with_expected(
                    args[2].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(val_ty),
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let dst = alloc(next);
                let ret_map_ty = map_ty.clone();
                out.push(IrInstr::MapSet {
                    dst,
                    map: map_reg,
                    key: key_reg,
                    val: val_reg,
                });
                return Ok((dst, ret_map_ty));
            }
            // builtin print(msg: text) -> ()
            // builtin random_seed(seed: i32) -> ()
            // builtin to_text(value: text|bool|i32|u32|quad) -> text
            if resolve_symbol_name(arena, *name)? == "print" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "builtin 'print' takes exactly one positional argument (msg: text)"
                                .to_string(),
                    });
                }
                let (arg_reg, _) = lower_expr(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let dst = alloc(next);
                out.push(IrInstr::Call {
                    dst: Some(dst),
                    name: "print".to_string(),
                    args: vec![arg_reg],
                });
                return Ok((dst, Type::Unit));
            }
            if resolve_symbol_name(arena, *name)? == "to_text" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'to_text' takes exactly one positional argument"
                            .to_string(),
                    });
                }
                let (arg_reg, _) = lower_expr(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let dst = alloc(next);
                out.push(IrInstr::Call {
                    dst: Some(dst),
                    name: "to_text".to_string(),
                    args: vec![arg_reg],
                });
                return Ok((dst, Type::Text));
            }
            let name_str = resolve_symbol_name(arena, *name)?;
            if matches!(
                name_str,
                "qtruth_and" | "qtruth_or" | "qtruth_not" | "qtruth_impl"
            ) {
                let expected_arity = if name_str == "qtruth_not" { 1 } else { 2 };
                if args.len() != expected_arity || args.iter().any(|arg| arg.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin '{name_str}' takes exactly {expected_arity} positional argument{}",
                            if expected_arity == 1 { "" } else { "s" }
                        ),
                    });
                }
                let mut regs = Vec::with_capacity(expected_arity);
                for arg in args {
                    let (reg, arg_ty) = lower_expr_with_expected(
                        arg.value,
                        arena,
                        next,
                        out,
                        env,
                        loop_stack,
                        fn_table,
                        record_table,
                        adt_table,
                        Some(Type::Quad),
                        ret_ty.clone(),
                        closure_state,
                        ownership_events,
                        lowered_locals,
                    )?;
                    if arg_ty != Type::Quad {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "builtin '{name_str}' expects quad arguments, got {:?}",
                                arg_ty
                            ),
                        });
                    }
                    regs.push(reg);
                }
                let dst = alloc(next);
                match name_str {
                    "qtruth_and" => out.push(IrInstr::QTruthAnd {
                        dst,
                        lhs: regs[0],
                        rhs: regs[1],
                    }),
                    "qtruth_or" => out.push(IrInstr::QTruthOr {
                        dst,
                        lhs: regs[0],
                        rhs: regs[1],
                    }),
                    "qtruth_not" => out.push(IrInstr::QTruthNot { dst, src: regs[0] }),
                    "qtruth_impl" => out.push(IrInstr::QTruthImpl {
                        dst,
                        lhs: regs[0],
                        rhs: regs[1],
                    }),
                    _ => unreachable!("qtruth intrinsic was checked above"),
                }
                return Ok((dst, Type::Quad));
            }
            if resolve_symbol_name(arena, *name)? == "random_seed" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'random_seed' takes exactly one positional argument (seed: i32)"
                            .to_string(),
                    });
                }
                let (seed_reg, _) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(Type::I32),
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let dst = alloc(next);
                out.push(IrInstr::RngSeed {
                    dst,
                    seed: seed_reg,
                });
                return Ok((dst, Type::Unit));
            }
            // builtin random_next_i32(lo: i32, hi: i32) -> i32
            if resolve_symbol_name(arena, *name)? == "random_next_i32" {
                if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "builtin 'random_next_i32' takes exactly two positional arguments (lo: i32, hi: i32)"
                                .to_string(),
                    });
                }
                let (lo_reg, _) = lower_expr_with_expected(
                    args[0].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(Type::I32),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let (hi_reg, _) = lower_expr_with_expected(
                    args[1].value,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(Type::I32),
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let dst = alloc(next);
                out.push(IrInstr::RngNextI32 {
                    dst,
                    lo: lo_reg,
                    hi: hi_reg,
                });
                return Ok((dst, Type::I32));
            }
            let sig = if let Some(s) = fn_table.get(name) {
                s.clone()
            } else if let Some(s) = builtin_sig(resolve_symbol_name(arena, *name)?) {
                s
            } else if let Some(Type::Closure(closure_ty)) = env.get(*name) {
                return lower_direct_closure_call_expr(
                    *name,
                    args,
                    &closure_ty,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                );
            } else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("unknown function '{}'", resolve_symbol_name(arena, *name)?),
                });
            };
            let ordered_args = reorder_call_args(*name, args, &sig, arena)?;
            // Evaluate argument expressions in source order (eval_order), but
            // place each resulting register into its declared parameter slot
            // so the call's register list stays in parameter order. Named
            // arguments only ever reorder slot assignment, never evaluation
            // order — see FA-04-016 / #1722.
            let mut regs: Vec<Option<u16>> = vec![None; ordered_args.slots.len()];
            for &slot in &ordered_args.eval_order {
                let arg = ordered_args.slots[slot];
                let expected_arg_ty = sig.params[slot].clone();
                let (r, t) = lower_expr_with_expected(
                    arg,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(expected_arg_ty.clone()),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                if t != expected_arg_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "arg {} for '{}' has type {:?}, expected {:?}",
                            slot,
                            resolve_symbol_name(arena, *name)?,
                            t,
                            expected_arg_ty
                        ),
                    });
                }
                regs[slot] = Some(r);
            }
            let regs: Vec<u16> = regs.into_iter().flatten().collect();
            if sig.ret == Type::Unit {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unit-returning call '{}' cannot be used as expression value",
                        resolve_symbol_name(arena, *name)?
                    ),
                });
            }
            let r = alloc(next);
            out.push(IrInstr::Call {
                dst: Some(r),
                name: resolve_symbol_name(arena, *name)?.to_string(),
                args: regs,
            });
            Ok((r, sig.ret.clone()))
        }
        Expr::Unary(op, inner) => {
            let expected_erased = erased_expected(expected.as_ref());
            if expected_erased == Some(Type::Fx) {
                if let Some(value) = try_encode_fx_literal_expr(expr_id, arena)? {
                    let dst = alloc(next);
                    out.push(IrInstr::LoadFx { dst, val: value });
                    return Ok((
                        dst,
                        lift_lowered_type(expected.as_ref(), &Type::Fx, expr_id, arena),
                    ));
                }
            }
            let (src, ty) = lower_expr_with_expected(
                *inner,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                expected,
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            match op {
                UnaryOp::Not => {
                    let dst = alloc(next);
                    match ty {
                        Type::Quad => out.push(IrInstr::QNot { dst, src }),
                        Type::Bool => out.push(IrInstr::BoolNot { dst, src }),
                        _ => {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!("operator ! unsupported for {:?}", ty),
                            })
                        }
                    }
                    Ok((dst, ty))
                }
                UnaryOp::Pos => {
                    if ty == Type::F64 {
                        Ok((src, Type::F64))
                    } else if ty == Type::I32 {
                        Ok((src, Type::I32))
                    } else if ty == Type::Fx {
                        Ok((src, Type::Fx))
                    } else if matches!(ty.measured_parts(), Some((base, _)) if *base == Type::F64) {
                        Ok((src, ty))
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!("operator + unsupported for {:?}", ty),
                        })
                    }
                }
                UnaryOp::Neg => {
                    let result_ty = if ty == Type::I32 {
                        Type::I32
                    } else if ty == Type::Fx {
                        Type::Fx
                    } else if ty == Type::F64 {
                        Type::F64
                    } else if matches!(ty.measured_parts(), Some((base, _)) if *base == Type::F64) {
                        ty.clone()
                    } else {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator - unsupported for {:?}", ty),
                        });
                    };
                    let zero = alloc(next);
                    if ty == Type::I32 {
                        out.push(IrInstr::LoadI32 { dst: zero, val: 0 });
                    } else if ty == Type::Fx {
                        out.push(IrInstr::LoadFx { dst: zero, val: 0 });
                    } else {
                        out.push(IrInstr::LoadF64 {
                            dst: zero,
                            val: 0.0,
                        });
                    }
                    let dst = alloc(next);
                    if ty == Type::I32 {
                        out.push(IrInstr::SubI32 {
                            dst,
                            lhs: zero,
                            rhs: src,
                        });
                    } else if ty == Type::Fx {
                        out.push(IrInstr::SubFx {
                            dst,
                            lhs: zero,
                            rhs: src,
                        });
                    } else {
                        out.push(IrInstr::SubF64 {
                            dst,
                            lhs: zero,
                            rhs: src,
                        });
                    }
                    Ok((dst, result_ty))
                }
            }
        }
        Expr::Binary(left, op, right) => {
            let (lr, lt) = lower_expr_with_expected(
                *left,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                expected.clone(),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let (rr, rt) = lower_expr_with_expected(
                *right,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                expected,
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if lt != rt {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("operator type mismatch: {:?} vs {:?}", lt, rt),
                });
            }
            let dst = alloc(next);
            let erased_lt = lt.erase_units();
            match op {
                BinaryOp::AndAnd => match lt {
                    Type::Quad => out.push(IrInstr::QAnd {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    }),
                    Type::Bool => out.push(IrInstr::BoolAnd {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    }),
                    _ => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator && unsupported for {:?}", lt),
                        })
                    }
                },
                BinaryOp::OrOr => match lt {
                    Type::Quad => out.push(IrInstr::QOr {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    }),
                    Type::Bool => out.push(IrInstr::BoolOr {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    }),
                    _ => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator || unsupported for {:?}", lt),
                        })
                    }
                },
                BinaryOp::Implies => {
                    if lt != Type::Quad {
                        return Err(FrontendError {
                            pos: 0,
                            message: "operator '->' is allowed only for quad".to_string(),
                        });
                    }
                    out.push(IrInstr::QImpl {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::Quad));
                }
                BinaryOp::Eq => {
                    out.push(IrInstr::CmpEq {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::Bool));
                }
                BinaryOp::Ne => {
                    out.push(IrInstr::CmpNe {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::Bool));
                }
                BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                    if lt != Type::I32 || rt != Type::I32 {
                        return Err(FrontendError {
                            pos: 0,
                            message:
                                "relational lowering currently requires same-family i32 operands"
                                    .to_string(),
                        });
                    }
                    match op {
                        BinaryOp::Lt => out.push(IrInstr::CmpI32Lt {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        }),
                        BinaryOp::Le => out.push(IrInstr::CmpI32Le {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        }),
                        BinaryOp::Gt => out.push(IrInstr::CmpI32Lt {
                            dst,
                            lhs: rr,
                            rhs: lr,
                        }),
                        BinaryOp::Ge => out.push(IrInstr::CmpI32Le {
                            dst,
                            lhs: rr,
                            rhs: lr,
                        }),
                        _ => unreachable!("covered relational operator arms"),
                    }
                    return Ok((dst, Type::Bool));
                }
                BinaryOp::Add => {
                    if lt == Type::Text && rt == Type::Text {
                        out.push(IrInstr::ConcatText {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        });
                        return Ok((dst, Type::Text));
                    }
                    if lt == Type::I32 {
                        out.push(IrInstr::AddI32 {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        });
                        return Ok((dst, Type::I32));
                    }
                    if lt == Type::Fx {
                        out.push(IrInstr::AddFx {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        });
                        return Ok((dst, Type::Fx));
                    }
                    if matches!(lt.measured_parts(), Some((_, _))) && erased_lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator + unsupported for {:?}", lt),
                        });
                    }
                    if erased_lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator + unsupported for {:?}", lt),
                        });
                    }
                    out.push(IrInstr::AddF64 {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, lt));
                }
                BinaryOp::Sub => {
                    if lt == Type::I32 {
                        out.push(IrInstr::SubI32 {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        });
                        return Ok((dst, Type::I32));
                    }
                    if lt == Type::Fx {
                        out.push(IrInstr::SubFx {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        });
                        return Ok((dst, Type::Fx));
                    }
                    if matches!(lt.measured_parts(), Some((_, _))) && erased_lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator - unsupported for {:?}", lt),
                        });
                    }
                    if erased_lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator - unsupported for {:?}", lt),
                        });
                    }
                    out.push(IrInstr::SubF64 {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, lt));
                }
                BinaryOp::Mul => {
                    if lt == Type::I32 {
                        out.push(IrInstr::MulI32 {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        });
                        return Ok((dst, Type::I32));
                    }
                    if lt == Type::Fx {
                        out.push(IrInstr::MulFx {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        });
                        return Ok((dst, Type::Fx));
                    }
                    if lt.measured_parts().is_some() {
                        return Err(FrontendError {
                            pos: 0,
                            message:
                                "*, /, % on unit-carrying values are rejected in the first-wave units surface"
                                    .to_string(),
                        });
                    }
                    if lt != Type::F64 {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator * unsupported for {:?}", lt),
                        });
                    }
                    out.push(IrInstr::MulF64 {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::F64));
                }
                BinaryOp::Div | BinaryOp::Mod => {
                    if lt == Type::I32 {
                        if *op == BinaryOp::Div {
                            out.push(IrInstr::DivI32 {
                                dst,
                                lhs: lr,
                                rhs: rr,
                            });
                        } else {
                            out.push(IrInstr::ModI32 {
                                dst,
                                lhs: lr,
                                rhs: rr,
                            });
                        }
                        return Ok((dst, Type::I32));
                    }
                    if lt == Type::Fx && *op == BinaryOp::Div {
                        out.push(IrInstr::DivFx {
                            dst,
                            lhs: lr,
                            rhs: rr,
                        });
                        return Ok((dst, Type::Fx));
                    }
                    if lt.measured_parts().is_some() {
                        return Err(FrontendError {
                            pos: 0,
                            message:
                                "*, /, % on unit-carrying values are rejected in the first-wave units surface"
                                    .to_string(),
                        });
                    }
                    if lt != Type::F64 || *op == BinaryOp::Mod {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "operator {} unsupported for {:?}",
                                if *op == BinaryOp::Div { "/" } else { "%" },
                                lt
                            ),
                        });
                    }
                    out.push(IrInstr::DivF64 {
                        dst,
                        lhs: lr,
                        rhs: rr,
                    });
                    return Ok((dst, Type::F64));
                }
            }
            Ok((dst, lt))
        }
        // M9.4 Wave 1: IfLet lowering is deferred (typecheck-only in M9.4).
        Expr::IfLet(_) => Err(FrontendError {
            pos: 0,
            message: "if-let lowering is not yet implemented in the IR backend".to_string(),
        }),
    }
}

fn bind_tuple_items(
    items: &[TuplePatternItem],
    tuple_reg: u16,
    tuple_ty: &Type,
    tuple_path: Option<&SequenceOwnershipPath>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
    env: &mut ScopeEnv,
) -> Result<(), FrontendError> {
    let Type::Tuple(item_tys) = tuple_ty else {
        return Err(FrontendError {
            pos: 0,
            message: "tuple destructuring bind requires tuple value".to_string(),
        });
    };
    if item_tys.len() != items.len() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "tuple destructuring bind arity mismatch: expected {}, got {}",
                items.len(),
                item_tys.len()
            ),
        });
    }
    let is_dynamic_fallback = tuple_path.is_some_and(SequenceOwnershipPath::is_dynamic_fallback);
    let mut emitted_dynamic_root = false;
    for (index, (item, item_ty)) in items.iter().zip(item_tys.iter()).enumerate() {
        let (name, capture) = match item {
            TuplePatternItem::Bind { name, capture } => (*name, *capture),
            TuplePatternItem::Discard => continue,
            TuplePatternItem::QuadLiteral(_) => {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "quad literal tuple patterns currently require let-else; plain tuple destructuring bind supports only name/_/ref items"
                            .to_string(),
                })
            }
            TuplePatternItem::Nested(_) => {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "nested tuple patterns are not yet supported in plain let bindings; use let-else form"
                            .to_string(),
                })
            }
        };
        let reg = alloc(next);
        let index = u16::try_from(index).map_err(|_| FrontendError {
            pos: 0,
            message: "tuple destructuring bind index exceeds v0 limit".to_string(),
        })?;
        out.push(IrInstr::TupleGet {
            dst: reg,
            src: tuple_reg,
            index,
        });
        env.insert(name, item_ty.clone());
        out.push(IrInstr::StoreVar {
            name: lowered_locals.bind(arena, name)?,
            src: reg,
        });
        if capture == sm_front::types::CaptureMode::Borrow {
            if let Some(tuple_path) = tuple_path {
                if is_dynamic_fallback {
                    if !emitted_dynamic_root {
                        ownership_events.push(OwnershipPathEvent {
                            kind: OwnershipPathEventKind::Borrow,
                            path: tuple_path.as_path().clone(),
                        });
                        emitted_dynamic_root = true;
                    }
                } else {
                    ownership_events.push(OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Borrow,
                        path: tuple_path.as_path().tuple_index(index),
                    });
                }
            }
        }
    }
    Ok(())
}

fn bind_record_items(
    record_name: SymbolId,
    items: &[RecordPatternItem],
    record_reg: u16,
    record_ty: &Type,
    record_path: Option<&AccessPath>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
    env: &mut ScopeEnv,
    record_table: &RecordTable,
    _adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    if *record_ty != Type::Record(record_name) {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "record destructuring bind requires value of type '{}', got {:?}",
                resolve_symbol_name(arena, record_name)?,
                record_ty
            ),
        });
    }
    let record = record_table.get(&record_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown record type '{}' in record destructuring bind",
            resolve_symbol_name(arena, record_name)?
        ),
    })?;
    for item in items {
        let (index, field) = record
            .fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == item.field)
            .ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "record type '{}' has no field named '{}' in destructuring bind",
                    resolve_symbol_name(arena, record_name)?,
                    resolve_symbol_name(arena, item.field)?
                ),
            })?;
        let reg = alloc(next);
        let index = u16::try_from(index).map_err(|_| FrontendError {
            pos: 0,
            message: "record destructuring bind index exceeds v0 limit".to_string(),
        })?;
        out.push(IrInstr::RecordGet {
            dst: reg,
            src: record_reg,
            record_name: resolve_symbol_name(arena, record_name)?.to_string(),
            index,
        });
        match item.target {
            RecordPatternTarget::Bind {
                name: target,
                capture,
            } => {
                env.insert(target, field.ty.clone());
                out.push(IrInstr::StoreVar {
                    name: lowered_locals.bind(arena, target)?,
                    src: reg,
                });
                if capture == sm_front::types::CaptureMode::Borrow {
                    if let Some(record_path) = record_path {
                        ownership_events.push(OwnershipPathEvent {
                            kind: OwnershipPathEventKind::Borrow,
                            path: record_path.field(item.field),
                        });
                    }
                }
            }
            RecordPatternTarget::Discard => {}
            RecordPatternTarget::QuadLiteral(_) => {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "quad literal record field patterns currently require let-else; plain record destructuring bind supports only name/_ items"
                            .to_string(),
                });
            }
        }
    }
    Ok(())
}

fn bind_let_else_record_items(
    record_name: SymbolId,
    items: &[RecordPatternItem],
    record_reg: u16,
    record_ty: &Type,
    record_path: Option<&AccessPath>,
    else_return: Option<ExprId>,
    contract_ensures: &[ExprId],
    contract_result_symbol: Option<SymbolId>,
    contract_invariants: &[ExprId],
    contract_invariant_result_symbol: Option<SymbolId>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
    env: &mut ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
) -> Result<(), FrontendError> {
    if *record_ty != Type::Record(record_name) {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "record let-else requires value of type '{}', got {:?}",
                resolve_symbol_name(arena, record_name)?,
                record_ty
            ),
        });
    }
    let record = record_table.get(&record_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown record type '{}' in record let-else",
            resolve_symbol_name(arena, record_name)?
        ),
    })?;
    let pattern_id = alloc_loop_expr_id(next);
    let mut deferred_binds = Vec::new();
    let mut saw_refutable_item = false;
    for item in items {
        let (index, field) = record
            .fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == item.field)
            .ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "record type '{}' has no field named '{}' in let-else",
                    resolve_symbol_name(arena, record_name)?,
                    resolve_symbol_name(arena, item.field)?
                ),
            })?;
        let reg = alloc(next);
        let index = u16::try_from(index).map_err(|_| FrontendError {
            pos: 0,
            message: "record let-else index exceeds v0 limit".to_string(),
        })?;
        out.push(IrInstr::RecordGet {
            dst: reg,
            src: record_reg,
            record_name: resolve_symbol_name(arena, record_name)?.to_string(),
            index,
        });
        match item.target {
            RecordPatternTarget::Bind {
                name: target,
                capture,
            } => {
                deferred_binds.push((target, reg, field.ty.clone()));
                if capture == sm_front::types::CaptureMode::Borrow {
                    if let Some(record_path) = record_path {
                        ownership_events.push(OwnershipPathEvent {
                            kind: OwnershipPathEventKind::Borrow,
                            path: record_path.field(item.field),
                        });
                    }
                }
            }
            RecordPatternTarget::Discard => {}
            RecordPatternTarget::QuadLiteral(pat) => {
                saw_refutable_item = true;
                if field.ty != Type::Quad {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "record let-else literal pattern requires quad field, got {:?}",
                            field.ty
                        ),
                    });
                }
                let lit_reg = alloc(next);
                out.push(IrInstr::LoadQ {
                    dst: lit_reg,
                    val: pat,
                });
                let cmp_reg = alloc(next);
                out.push(IrInstr::CmpEq {
                    dst: cmp_reg,
                    lhs: reg,
                    rhs: lit_reg,
                });
                let continue_label = format!("let_else_record_{}_field_{}_ok", pattern_id, index);
                out.push(IrInstr::JmpIf {
                    cond: cmp_reg,
                    label: continue_label.clone(),
                });
                lower_return_payload(
                    else_return,
                    contract_ensures,
                    contract_result_symbol,
                    contract_invariants,
                    contract_invariant_result_symbol,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                out.push(IrInstr::Label {
                    name: continue_label,
                });
            }
        }
    }
    if !saw_refutable_item {
        return Err(FrontendError {
            pos: 0,
            message: "record let-else requires at least one refutable quad literal field pattern"
                .to_string(),
        });
    }
    for (name, reg, item_ty) in deferred_binds {
        env.insert(name, item_ty);
        out.push(IrInstr::StoreVar {
            name: lowered_locals.bind(arena, name)?,
            src: reg,
        });
    }
    Ok(())
}

fn assign_tuple_items(
    items: &[Option<SymbolId>],
    tuple_reg: u16,
    tuple_ty: &Type,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
    env: &ScopeEnv,
) -> Result<(), FrontendError> {
    let Type::Tuple(item_tys) = tuple_ty else {
        return Err(FrontendError {
            pos: 0,
            message: "tuple destructuring assignment requires tuple value".to_string(),
        });
    };
    if item_tys.len() != items.len() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "tuple destructuring assignment arity mismatch: expected {}, got {}",
                items.len(),
                item_tys.len()
            ),
        });
    }
    for (index, (item, item_ty)) in items.iter().zip(item_tys.iter()).enumerate() {
        let Some(name) = item else {
            continue;
        };
        let target_ty = env.get(*name).ok_or(FrontendError {
            pos: 0,
            message: format!(
                "unknown tuple assignment target '{}'",
                resolve_symbol_name(arena, *name)?
            ),
        })?;
        if env.is_const(*name)? {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "cannot assign to const binding '{}' in tuple destructuring assignment",
                    resolve_symbol_name(arena, *name)?
                ),
            });
        }
        if target_ty != *item_ty {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "type mismatch in tuple assignment to '{}': {:?} vs {:?}",
                    resolve_symbol_name(arena, *name)?,
                    target_ty,
                    item_ty
                ),
            });
        }
        let reg = alloc(next);
        let index = u16::try_from(index).map_err(|_| FrontendError {
            pos: 0,
            message: "tuple destructuring assignment index exceeds v0 limit".to_string(),
        })?;
        out.push(IrInstr::TupleGet {
            dst: reg,
            src: tuple_reg,
            index,
        });
        out.push(IrInstr::StoreVar {
            name: lowered_locals.resolve(arena, *name)?,
            src: reg,
        });
        ownership_events.push(OwnershipPathEvent {
            kind: OwnershipPathEventKind::Write,
            path: AccessPath::new(*name),
        });
    }
    Ok(())
}

fn lower_for_range_stmt_from_reg(
    name: SymbolId,
    range_reg: u16,
    body: &[StmtId],
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let id = ctx.next_if_id();
    let current_name = format!("__for_range_{}_current", id);
    let start_reg = alloc(&mut ctx.next_reg);
    let end_reg = alloc(&mut ctx.next_reg);
    let inclusive_reg = alloc(&mut ctx.next_reg);
    let one_reg = alloc(&mut ctx.next_reg);
    let cmp_reg = alloc(&mut ctx.next_reg);
    let stop_cmp_reg = alloc(&mut ctx.next_reg);
    let stop_reg = alloc(&mut ctx.next_reg);

    ctx.instrs.push(IrInstr::TupleGet {
        dst: start_reg,
        src: range_reg,
        index: 0,
    });
    ctx.instrs.push(IrInstr::TupleGet {
        dst: end_reg,
        src: range_reg,
        index: 1,
    });
    ctx.instrs.push(IrInstr::TupleGet {
        dst: inclusive_reg,
        src: range_reg,
        index: 2,
    });
    ctx.instrs.push(IrInstr::LoadI32 {
        dst: one_reg,
        val: 1,
    });
    ctx.instrs.push(IrInstr::StoreVar {
        name: current_name.clone(),
        src: start_reg,
    });

    let test_label = format!("for_range_{}_test", id);
    let inclusive_label = format!("for_range_{}_inclusive", id);
    let exclusive_label = format!("for_range_{}_exclusive", id);
    let body_label = format!("for_range_{}_body", id);
    let end_label = format!("for_range_{}_end", id);

    ctx.instrs.push(IrInstr::Label {
        name: test_label.clone(),
    });
    let current_reg = alloc(&mut ctx.next_reg);
    ctx.instrs.push(IrInstr::LoadVar {
        dst: current_reg,
        name: current_name.clone(),
    });
    ctx.instrs.push(IrInstr::JmpIf {
        cond: inclusive_reg,
        label: inclusive_label.clone(),
    });
    ctx.instrs.push(IrInstr::Jmp {
        label: exclusive_label.clone(),
    });

    ctx.instrs.push(IrInstr::Label {
        name: inclusive_label,
    });
    ctx.instrs.push(IrInstr::CmpI32Le {
        dst: cmp_reg,
        lhs: current_reg,
        rhs: end_reg,
    });
    ctx.instrs.push(IrInstr::JmpIf {
        cond: cmp_reg,
        label: body_label.clone(),
    });
    ctx.instrs.push(IrInstr::Jmp {
        label: end_label.clone(),
    });

    ctx.instrs.push(IrInstr::Label {
        name: exclusive_label,
    });
    ctx.instrs.push(IrInstr::CmpI32Lt {
        dst: cmp_reg,
        lhs: current_reg,
        rhs: end_reg,
    });
    ctx.instrs.push(IrInstr::JmpIf {
        cond: cmp_reg,
        label: body_label.clone(),
    });
    ctx.instrs.push(IrInstr::Jmp {
        label: end_label.clone(),
    });

    ctx.instrs.push(IrInstr::Label { name: body_label });
    let mut body_env = env.clone();
    body_env.push_scope();
    body_env.insert_const(name, Type::I32);
    ctx.lowered_locals.push_scope();
    let loop_name = ctx.lowered_locals.bind(arena, name)?;
    ctx.instrs.push(IrInstr::StoreVar {
        name: loop_name,
        src: current_reg,
    });
    for stmt in body {
        lower_stmt(
            *stmt,
            arena,
            ctx,
            &mut body_env,
            ret_ty.clone(),
            fn_table,
            record_table,
            adt_table,
        )?;
    }
    body_env.pop_scope();
    ctx.lowered_locals.pop_scope();

    let reload_reg = alloc(&mut ctx.next_reg);
    let next_reg = alloc(&mut ctx.next_reg);
    ctx.instrs.push(IrInstr::LoadVar {
        dst: reload_reg,
        name: current_name.clone(),
    });
    ctx.instrs.push(IrInstr::CmpEq {
        dst: stop_cmp_reg,
        lhs: reload_reg,
        rhs: end_reg,
    });
    ctx.instrs.push(IrInstr::BoolAnd {
        dst: stop_reg,
        lhs: stop_cmp_reg,
        rhs: inclusive_reg,
    });
    ctx.instrs.push(IrInstr::JmpIf {
        cond: stop_reg,
        label: end_label.clone(),
    });
    ctx.instrs.push(IrInstr::AddI32 {
        dst: next_reg,
        lhs: reload_reg,
        rhs: one_reg,
    });
    ctx.instrs.push(IrInstr::StoreVar {
        name: current_name,
        src: next_reg,
    });
    ctx.instrs.push(IrInstr::Jmp { label: test_label });
    ctx.instrs.push(IrInstr::Label { name: end_label });
    Ok(())
}

fn lower_for_range_stmt(
    name: SymbolId,
    range: ExprId,
    body: &[StmtId],
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let (range_reg, range_ty) = lower_expr_with_expected(
        range,
        arena,
        &mut ctx.next_reg,
        &mut ctx.instrs,
        env,
        &mut ctx.loop_stack,
        fn_table,
        record_table,
        adt_table,
        Some(Type::RangeI32),
        ret_ty.clone(),
        &mut ctx.closure_state,
        &mut ctx.ownership_events,
        &mut ctx.lowered_locals,
    )?;
    if range_ty != Type::RangeI32 {
        return Err(FrontendError {
            pos: 0,
            message: "for-range currently requires i32 range expression".to_string(),
        });
    }
    lower_for_range_stmt_from_reg(
        name,
        range_reg,
        body,
        arena,
        ctx,
        env,
        ret_ty,
        fn_table,
        record_table,
        adt_table,
    )
}

fn lower_while_stmt(
    condition: ExprId,
    body: &[StmtId],
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    append_record_update_write_events_from_expr(condition, arena, &mut ctx.ownership_events);
    let id = ctx.next_if_id();
    let test_label = format!("while_{}_test", id);
    let body_label = format!("while_{}_body", id);
    let end_label = format!("while_{}_end", id);

    ctx.loop_stack.push(LoopLoweringFrame {
        kind: LoopLoweringFrameKind::Control,
        end_label: end_label.clone(),
        continue_label: test_label.clone(),
        result_name: String::new(),
        result_ty: None,
        expected_ty: None,
    });

    ctx.instrs.push(IrInstr::Label {
        name: test_label.clone(),
    });
    let (cond_reg, cond_ty) = lower_expr(
        condition,
        arena,
        &mut ctx.next_reg,
        &mut ctx.instrs,
        env,
        &mut ctx.loop_stack,
        fn_table,
        record_table,
        adt_table,
        ret_ty.clone(),
        &mut ctx.closure_state,
        &mut ctx.ownership_events,
        &mut ctx.lowered_locals,
    )?;
    if cond_ty != Type::Bool {
        return Err(FrontendError {
            pos: 0,
            message: "while condition must be bool".to_string(),
        });
    }

    ctx.instrs.push(IrInstr::JmpIf {
        cond: cond_reg,
        label: body_label.clone(),
    });
    ctx.instrs.push(IrInstr::Jmp {
        label: end_label.clone(),
    });

    ctx.instrs.push(IrInstr::Label { name: body_label });
    let mut body_env = env.clone();
    body_env.push_scope();
    ctx.lowered_locals.push_scope();
    for stmt in body {
        lower_stmt(
            *stmt,
            arena,
            ctx,
            &mut body_env,
            ret_ty.clone(),
            fn_table,
            record_table,
            adt_table,
        )?;
    }
    body_env.pop_scope();
    ctx.lowered_locals.pop_scope();
    let _ = ctx.loop_stack.pop().expect("control loop frame must exist");
    ctx.instrs.push(IrInstr::Jmp { label: test_label });
    ctx.instrs.push(IrInstr::Label { name: end_label });
    Ok(())
}

fn lower_statement_loop(
    body: &[StmtId],
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let id = ctx.next_if_id();
    let start_label = format!("loop_stmt_{}_start", id);
    let end_label = format!("loop_stmt_{}_end", id);
    ctx.loop_stack.push(LoopLoweringFrame {
        kind: LoopLoweringFrameKind::Control,
        end_label: end_label.clone(),
        continue_label: start_label.clone(),
        result_name: String::new(),
        result_ty: None,
        expected_ty: None,
    });
    ctx.instrs.push(IrInstr::Label {
        name: start_label.clone(),
    });
    let mut body_env = env.clone();
    body_env.push_scope();
    ctx.lowered_locals.push_scope();
    for stmt in body {
        lower_stmt(
            *stmt,
            arena,
            ctx,
            &mut body_env,
            ret_ty.clone(),
            fn_table,
            record_table,
            adt_table,
        )?;
    }
    body_env.pop_scope();
    ctx.lowered_locals.pop_scope();
    let _ = ctx.loop_stack.pop().expect("control loop frame must exist");
    ctx.instrs.push(IrInstr::Jmp { label: start_label });
    ctx.instrs.push(IrInstr::Label { name: end_label });
    Ok(())
}

fn lower_for_each_stmt(
    name: SymbolId,
    iterable: ExprId,
    trait_name: SymbolId,
    body: &[StmtId],
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let (iterable_reg, iterable_ty) = lower_expr(
        iterable,
        arena,
        &mut ctx.next_reg,
        &mut ctx.instrs,
        env,
        &mut ctx.loop_stack,
        fn_table,
        record_table,
        adt_table,
        ret_ty.clone(),
        &mut ctx.closure_state,
        &mut ctx.ownership_events,
        &mut ctx.lowered_locals,
    )?;
    if iterable_ty == Type::RangeI32 {
        return lower_for_range_stmt_from_reg(
            name,
            iterable_reg,
            body,
            arena,
            ctx,
            env,
            ret_ty,
            fn_table,
            record_table,
            adt_table,
        );
    }
    if let Type::Sequence(sequence_ty) = &iterable_ty {
        return lower_for_sequence_stmt_from_reg(
            name,
            iterable_reg,
            sequence_ty.item.as_ref().clone(),
            body,
            arena,
            ctx,
            env,
            ret_ty,
            fn_table,
            record_table,
            adt_table,
        );
    }
    if let Some((item_ty, next_fn_name)) =
        resolve_explicit_iterable_loop_contract(&iterable_ty, trait_name, arena, &ctx.impls)?
    {
        return lower_for_explicit_iterable_stmt_from_reg(
            name,
            iterable_reg,
            item_ty,
            &next_fn_name,
            body,
            arena,
            ctx,
            env,
            ret_ty,
            fn_table,
            record_table,
            adt_table,
        );
    }
    Err(FrontendError {
        pos: 0,
        message: format!(
            "{} (`{}` contract)",
            iterable_for_gap_message(),
            resolve_symbol_name(arena, trait_name)?
        ),
    })
}

fn lower_for_sequence_stmt_from_reg(
    name: SymbolId,
    sequence_reg: u16,
    item_ty: Type,
    body: &[StmtId],
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let id = ctx.next_if_id();
    let index_name = format!("__for_each_seq_{}_index", id);

    let zero_reg = alloc(&mut ctx.next_reg);
    let one_reg = alloc(&mut ctx.next_reg);
    let len_reg = alloc(&mut ctx.next_reg);
    let index_reg = alloc(&mut ctx.next_reg);
    let cmp_reg = alloc(&mut ctx.next_reg);
    let item_reg = alloc(&mut ctx.next_reg);
    let next_reg = alloc(&mut ctx.next_reg);

    ctx.instrs.push(IrInstr::LoadI32 {
        dst: zero_reg,
        val: 0,
    });
    ctx.instrs.push(IrInstr::LoadI32 {
        dst: one_reg,
        val: 1,
    });
    ctx.instrs.push(IrInstr::StoreVar {
        name: index_name.clone(),
        src: zero_reg,
    });
    ctx.instrs.push(IrInstr::SequenceLen {
        dst: len_reg,
        src: sequence_reg,
    });

    let test_label = format!("for_each_seq_{}_test", id);
    let body_label = format!("for_each_seq_{}_body", id);
    let end_label = format!("for_each_seq_{}_end", id);

    ctx.instrs.push(IrInstr::Label {
        name: test_label.clone(),
    });
    ctx.instrs.push(IrInstr::LoadVar {
        dst: index_reg,
        name: index_name.clone(),
    });
    ctx.instrs.push(IrInstr::CmpI32Lt {
        dst: cmp_reg,
        lhs: index_reg,
        rhs: len_reg,
    });
    ctx.instrs.push(IrInstr::JmpIf {
        cond: cmp_reg,
        label: body_label.clone(),
    });
    ctx.instrs.push(IrInstr::Jmp {
        label: end_label.clone(),
    });

    ctx.instrs.push(IrInstr::Label { name: body_label });
    ctx.instrs.push(IrInstr::SequenceGet {
        dst: item_reg,
        src: sequence_reg,
        index: index_reg,
    });
    let mut body_env = env.clone();
    body_env.push_scope();
    body_env.insert_const(name, item_ty);
    ctx.lowered_locals.push_scope();
    let loop_name = ctx.lowered_locals.bind(arena, name)?;
    ctx.instrs.push(IrInstr::StoreVar {
        name: loop_name,
        src: item_reg,
    });
    for stmt in body {
        lower_stmt(
            *stmt,
            arena,
            ctx,
            &mut body_env,
            ret_ty.clone(),
            fn_table,
            record_table,
            adt_table,
        )?;
    }
    body_env.pop_scope();
    ctx.lowered_locals.pop_scope();

    ctx.instrs.push(IrInstr::LoadVar {
        dst: index_reg,
        name: index_name.clone(),
    });
    ctx.instrs.push(IrInstr::AddI32 {
        dst: next_reg,
        lhs: index_reg,
        rhs: one_reg,
    });
    ctx.instrs.push(IrInstr::StoreVar {
        name: index_name,
        src: next_reg,
    });
    ctx.instrs.push(IrInstr::Jmp { label: test_label });
    ctx.instrs.push(IrInstr::Label { name: end_label });
    Ok(())
}

fn lower_for_explicit_iterable_stmt_from_reg(
    name: SymbolId,
    iterable_reg: u16,
    item_ty: Type,
    next_fn_name: &str,
    body: &[StmtId],
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let id = ctx.next_if_id();
    let index_name = format!("__for_each_iter_{}_index", id);

    let zero_reg = alloc(&mut ctx.next_reg);
    let one_reg = alloc(&mut ctx.next_reg);
    let index_reg = alloc(&mut ctx.next_reg);
    let next_opt_reg = alloc(&mut ctx.next_reg);
    let tag_reg = alloc(&mut ctx.next_reg);
    let has_item_reg = alloc(&mut ctx.next_reg);
    let item_reg = alloc(&mut ctx.next_reg);
    let next_index_reg = alloc(&mut ctx.next_reg);

    ctx.instrs.push(IrInstr::LoadI32 {
        dst: zero_reg,
        val: 0,
    });
    ctx.instrs.push(IrInstr::LoadI32 {
        dst: one_reg,
        val: 1,
    });
    ctx.instrs.push(IrInstr::StoreVar {
        name: index_name.clone(),
        src: zero_reg,
    });

    let test_label = format!("for_each_iter_{}_test", id);
    let body_label = format!("for_each_iter_{}_body", id);
    let end_label = format!("for_each_iter_{}_end", id);

    ctx.instrs.push(IrInstr::Label {
        name: test_label.clone(),
    });
    ctx.instrs.push(IrInstr::LoadVar {
        dst: index_reg,
        name: index_name.clone(),
    });
    ctx.instrs.push(IrInstr::Call {
        dst: Some(next_opt_reg),
        name: next_fn_name.to_string(),
        args: vec![iterable_reg, index_reg],
    });
    ctx.instrs.push(IrInstr::AdtTag {
        dst: tag_reg,
        src: next_opt_reg,
        adt_name: "Option".to_string(),
    });
    ctx.instrs.push(IrInstr::CmpEq {
        dst: has_item_reg,
        lhs: tag_reg,
        rhs: one_reg,
    });
    ctx.instrs.push(IrInstr::JmpIf {
        cond: has_item_reg,
        label: body_label.clone(),
    });
    ctx.instrs.push(IrInstr::Jmp {
        label: end_label.clone(),
    });

    ctx.instrs.push(IrInstr::Label { name: body_label });
    ctx.instrs.push(IrInstr::AdtGet {
        dst: item_reg,
        src: next_opt_reg,
        adt_name: "Option".to_string(),
        index: 0,
    });
    let mut body_env = env.clone();
    body_env.push_scope();
    body_env.insert_const(name, item_ty);
    ctx.lowered_locals.push_scope();
    let loop_name = ctx.lowered_locals.bind(arena, name)?;
    ctx.instrs.push(IrInstr::StoreVar {
        name: loop_name,
        src: item_reg,
    });
    for stmt in body {
        lower_stmt(
            *stmt,
            arena,
            ctx,
            &mut body_env,
            ret_ty.clone(),
            fn_table,
            record_table,
            adt_table,
        )?;
    }
    body_env.pop_scope();
    ctx.lowered_locals.pop_scope();

    ctx.instrs.push(IrInstr::LoadVar {
        dst: index_reg,
        name: index_name.clone(),
    });
    ctx.instrs.push(IrInstr::AddI32 {
        dst: next_index_reg,
        lhs: index_reg,
        rhs: one_reg,
    });
    ctx.instrs.push(IrInstr::StoreVar {
        name: index_name,
        src: next_index_reg,
    });
    ctx.instrs.push(IrInstr::Jmp { label: test_label });
    ctx.instrs.push(IrInstr::Label { name: end_label });
    Ok(())
}

fn bind_let_else_tuple_items(
    items: &[TuplePatternItem],
    tuple_reg: u16,
    tuple_ty: &Type,
    tuple_path: Option<&SequenceOwnershipPath>,
    else_return: Option<ExprId>,
    contract_ensures: &[ExprId],
    contract_result_symbol: Option<SymbolId>,
    contract_invariants: &[ExprId],
    contract_invariant_result_symbol: Option<SymbolId>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
    env: &mut ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
) -> Result<(), FrontendError> {
    let Type::Tuple(item_tys) = tuple_ty else {
        return Err(FrontendError {
            pos: 0,
            message: "let-else tuple destructuring bind requires tuple value".to_string(),
        });
    };
    if item_tys.len() != items.len() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "let-else tuple destructuring bind arity mismatch: expected {}, got {}",
                items.len(),
                item_tys.len()
            ),
        });
    }

    let pattern_id = alloc_loop_expr_id(next);
    let mut deferred_binds = Vec::new();
    let mut emitted_dynamic_root = false;
    for (index, (item, item_ty)) in items.iter().zip(item_tys.iter()).enumerate() {
        let reg = alloc(next);
        let index = u16::try_from(index).map_err(|_| FrontendError {
            pos: 0,
            message: "let-else tuple destructuring bind index exceeds v0 limit".to_string(),
        })?;
        out.push(IrInstr::TupleGet {
            dst: reg,
            src: tuple_reg,
            index,
        });
        match item {
            TuplePatternItem::Bind { name, capture } => {
                deferred_binds.push((*name, *capture, reg, item_ty.clone(), index))
            }
            TuplePatternItem::Discard => {}
            TuplePatternItem::QuadLiteral(pat) => {
                if *item_ty != Type::Quad {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "let-else tuple literal pattern requires quad element, got {:?}",
                            item_ty
                        ),
                    });
                }
                let lit_reg = alloc(next);
                out.push(IrInstr::LoadQ {
                    dst: lit_reg,
                    val: *pat,
                });
                let cmp_reg = alloc(next);
                out.push(IrInstr::CmpEq {
                    dst: cmp_reg,
                    lhs: reg,
                    rhs: lit_reg,
                });
                let continue_label = format!("let_else_tuple_{}_item_{}_ok", pattern_id, index);
                out.push(IrInstr::JmpIf {
                    cond: cmp_reg,
                    label: continue_label.clone(),
                });
                lower_return_payload(
                    else_return,
                    contract_ensures,
                    contract_result_symbol,
                    contract_invariants,
                    contract_invariant_result_symbol,
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                out.push(IrInstr::Label {
                    name: continue_label,
                });
            }
            // M9.4 Wave 1: nested tuple lowering is deferred.
            TuplePatternItem::Nested(_) => {
                return Err(FrontendError {
                    pos: 0,
                    message: "nested tuple lowering is not yet implemented in the IR backend"
                        .to_string(),
                })
            }
        }
    }

    for (name, capture, reg, item_ty, index) in deferred_binds {
        env.insert(name, item_ty);
        out.push(IrInstr::StoreVar {
            name: lowered_locals.bind(arena, name)?,
            src: reg,
        });
        if capture == sm_front::types::CaptureMode::Borrow {
            if let Some(tuple_path) = tuple_path {
                if tuple_path.is_dynamic_fallback() {
                    if !emitted_dynamic_root {
                        ownership_events.push(OwnershipPathEvent {
                            kind: OwnershipPathEventKind::Borrow,
                            path: tuple_path.as_path().clone(),
                        });
                        emitted_dynamic_root = true;
                    }
                } else {
                    ownership_events.push(OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Borrow,
                        path: tuple_path.as_path().tuple_index(index),
                    });
                }
            }
        }
    }
    Ok(())
}

fn lower_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &mut ScopeEnv,
    ret_ty: Type,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let stmt = arena.stmt(stmt_id);
    match stmt {
        Stmt::Const { name, ty, value } => {
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let (reg, vty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ty.clone(),
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            let final_ty = if let Some(ann) = ty {
                canonicalize_declared_type(ann, record_table, adt_table, arena)?
            } else {
                vty
            };
            env.insert_const(*name, final_ty);
            ctx.instrs.push(IrInstr::StoreVar {
                name: ctx.lowered_locals.bind(arena, *name)?,
                src: reg,
            });
            Ok(())
        }
        Stmt::Let {
            name,
            is_mut,
            ty,
            value,
        } => {
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let (reg, vty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ty.clone(),
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            let final_ty = if let Some(ann) = ty {
                canonicalize_declared_type(ann, record_table, adt_table, arena)?
            } else {
                vty
            };
            if *is_mut {
                env.insert_mut(*name, final_ty);
            } else {
                env.insert(*name, final_ty);
            }
            ctx.instrs.push(IrInstr::StoreVar {
                name: ctx.lowered_locals.bind(arena, *name)?,
                src: reg,
            });
            Ok(())
        }
        Stmt::LetTuple { items, ty, value } => {
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let sequence_path = sequence_access_path_from_expr(*value, arena);
            let (tuple_reg, vty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ty.clone(),
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            let final_ty = if let Some(ann) = ty {
                canonicalize_declared_type(ann, record_table, adt_table, arena)?
            } else {
                vty
            };
            bind_tuple_items(
                items,
                tuple_reg,
                &final_ty,
                sequence_path.as_ref(),
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
                env,
            )
        }
        Stmt::LetRecord {
            record_name,
            items,
            value,
        } => {
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let record_path = direct_record_access_path_from_expr(*value, arena);
            let (record_reg, record_ty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::Record(*record_name)),
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            bind_record_items(
                *record_name,
                items,
                record_reg,
                &record_ty,
                record_path.as_ref(),
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
                env,
                record_table,
                adt_table,
            )
        }
        Stmt::LetElseRecord {
            record_name,
            items,
            value,
            else_return,
        } => {
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let record_path = direct_record_access_path_from_expr(*value, arena);
            let (record_reg, record_ty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::Record(*record_name)),
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            bind_let_else_record_items(
                *record_name,
                items,
                record_reg,
                &record_ty,
                record_path.as_ref(),
                *else_return,
                &ctx.ensures,
                ctx.ensures_result_symbol,
                &ctx.invariants,
                ctx.invariants_result_symbol,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty,
                &mut ctx.closure_state,
            )
        }
        Stmt::LetElseTuple {
            items,
            ty,
            value,
            else_return,
        } => {
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let sequence_path = sequence_access_path_from_expr(*value, arena);
            let (tuple_reg, vty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ty.clone(),
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            let final_ty = if let Some(ann) = ty {
                canonicalize_declared_type(ann, record_table, adt_table, arena)?
            } else {
                vty
            };
            bind_let_else_tuple_items(
                items,
                tuple_reg,
                &final_ty,
                sequence_path.as_ref(),
                *else_return,
                &ctx.ensures,
                ctx.ensures_result_symbol,
                &ctx.invariants,
                ctx.invariants_result_symbol,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty,
                &mut ctx.closure_state,
            )
        }
        Stmt::Discard { ty, value } => {
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let _ = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ty.clone(),
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            Ok(())
        }
        Stmt::Assign { name, value } => {
            let target_ty = env.get(*name).ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "unknown assignment target '{}'",
                    resolve_symbol_name(arena, *name)?
                ),
            })?;
            if env.is_const(*name)? {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "cannot assign to const binding '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                });
            }
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let (reg, _) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(target_ty),
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            ctx.instrs.push(IrInstr::StoreVar {
                name: ctx.lowered_locals.resolve(arena, *name)?,
                src: reg,
            });
            ctx.ownership_events.push(OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new(*name),
            });
            Ok(())
        }
        Stmt::AssignTuple { items, value } => {
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let (tuple_reg, tuple_ty) = lower_expr(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty,
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            assign_tuple_items(
                items,
                tuple_reg,
                &tuple_ty,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
                env,
            )
        }
        Stmt::ForRange { name, range, body } => lower_for_range_stmt(
            *name,
            *range,
            body,
            arena,
            ctx,
            env,
            ret_ty,
            fn_table,
            record_table,
            adt_table,
        ),
        Stmt::While { condition, body } => lower_while_stmt(
            *condition,
            body,
            arena,
            ctx,
            env,
            ret_ty,
            fn_table,
            record_table,
            adt_table,
        ),
        Stmt::Loop { body } => lower_statement_loop(
            body,
            arena,
            ctx,
            env,
            ret_ty,
            fn_table,
            record_table,
            adt_table,
        ),
        Stmt::ForEach {
            name,
            iterable,
            body,
            desugaring,
        } => lower_for_each_stmt(
            *name,
            *iterable,
            desugaring.trait_name,
            body,
            arena,
            ctx,
            env,
            ret_ty,
            fn_table,
            record_table,
            adt_table,
        ),
        Stmt::Break(None) => {
            let frame = ctx.loop_stack.last().ok_or(FrontendError {
                pos: 0,
                message: "bare break is allowed only inside while or statement loop".to_string(),
            })?;
            if !matches!(frame.kind, LoopLoweringFrameKind::Control) {
                return Err(FrontendError {
                    pos: 0,
                    message: "bare break is allowed only inside while or statement loop"
                        .to_string(),
                });
            }
            ctx.instrs.push(IrInstr::Jmp {
                label: frame.end_label.clone(),
            });
            Ok(())
        }
        Stmt::Break(Some(value)) => {
            let (expected_break, end_label, result_name, prior_result_ty) = {
                let frame = ctx.loop_stack.last().ok_or(FrontendError {
                    pos: 0,
                    message: "break with value is allowed only inside loop expression".to_string(),
                })?;
                if !matches!(frame.kind, LoopLoweringFrameKind::Expression) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "break with value is allowed only inside loop expression"
                            .to_string(),
                    });
                }
                (
                    frame.result_ty.clone().or(frame.expected_ty.clone()),
                    frame.end_label.clone(),
                    frame.result_name.clone(),
                    frame.result_ty.clone(),
                )
            };
            append_record_update_write_events_from_expr(*value, arena, &mut ctx.ownership_events);
            let (reg, break_ty) = lower_expr_with_expected(
                *value,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                expected_break,
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            if let Some(expected_ty) = &prior_result_ty {
                if *expected_ty != break_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "loop expression break type mismatch in lowering: expected {:?}, got {:?}",
                            expected_ty, break_ty
                        ),
                    });
                }
            } else if let Some(frame) = ctx.loop_stack.last_mut() {
                frame.result_ty = Some(break_ty);
            } else {
                return Err(FrontendError {
                    pos: 0,
                    message: "break with value is allowed only inside loop expression".to_string(),
                });
            }
            ctx.instrs.push(IrInstr::StoreVar {
                name: result_name,
                src: reg,
            });
            ctx.instrs.push(IrInstr::Jmp { label: end_label });
            Ok(())
        }
        Stmt::Continue => {
            let frame = ctx.loop_stack.last().ok_or(FrontendError {
                pos: 0,
                message: "continue is allowed only inside while or statement loop".to_string(),
            })?;
            if !matches!(frame.kind, LoopLoweringFrameKind::Control) {
                return Err(FrontendError {
                    pos: 0,
                    message: "continue is allowed only inside while or statement loop".to_string(),
                });
            }
            ctx.instrs.push(IrInstr::Jmp {
                label: frame.continue_label.clone(),
            });
            Ok(())
        }
        Stmt::Guard {
            condition,
            else_return,
        } => {
            append_record_update_write_events_from_expr(
                *condition,
                arena,
                &mut ctx.ownership_events,
            );
            let (cond_reg, cond_ty) = lower_expr(
                *condition,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "guard clause condition must be bool".to_string(),
                });
            }

            let id = ctx.next_if_id();
            let continue_label = format!("guard_{}_continue", id);
            ctx.instrs.push(IrInstr::JmpIf {
                cond: cond_reg,
                label: continue_label.clone(),
            });
            lower_return_payload(
                *else_return,
                &ctx.ensures,
                ctx.ensures_result_symbol,
                &ctx.invariants,
                ctx.invariants_result_symbol,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            ctx.instrs.push(IrInstr::Label {
                name: continue_label,
            });
            Ok(())
        }
        Stmt::Expr(expr) => {
            append_record_update_write_events_from_expr(*expr, arena, &mut ctx.ownership_events);
            lower_expr_stmt(
                *expr,
                arena,
                ctx,
                env,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
            )?;
            Ok(())
        }
        Stmt::Return(v) => {
            if let Some(value) = *v {
                append_record_update_write_events_from_expr(
                    value,
                    arena,
                    &mut ctx.ownership_events,
                );
            }
            lower_return_payload(
                *v,
                &ctx.ensures,
                ctx.ensures_result_symbol,
                &ctx.invariants,
                ctx.invariants_result_symbol,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            append_record_update_write_events_from_expr(
                *condition,
                arena,
                &mut ctx.ownership_events,
            );
            let (cond_reg, cond_ty) = lower_expr(
                *condition,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if condition must be bool".to_string(),
                });
            }

            let id = ctx.next_if_id();
            let then_label = format!("if_{}_then", id);
            let else_label = format!("if_{}_else", id);
            let end_label = format!("if_{}_end", id);

            ctx.instrs.push(IrInstr::JmpIf {
                cond: cond_reg,
                label: then_label.clone(),
            });
            ctx.instrs.push(IrInstr::Jmp {
                label: else_label.clone(),
            });

            ctx.instrs.push(IrInstr::Label { name: then_label });
            let mut then_env = env.clone();
            then_env.push_scope();
            ctx.lowered_locals.push_scope();
            for s in then_block {
                lower_stmt(
                    *s,
                    arena,
                    ctx,
                    &mut then_env,
                    ret_ty.clone(),
                    fn_table,
                    record_table,
                    adt_table,
                )?;
            }
            then_env.pop_scope();
            ctx.lowered_locals.pop_scope();
            ctx.instrs.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            ctx.instrs.push(IrInstr::Label { name: else_label });
            let mut else_env = env.clone();
            else_env.push_scope();
            ctx.lowered_locals.push_scope();
            for s in else_block {
                lower_stmt(
                    *s,
                    arena,
                    ctx,
                    &mut else_env,
                    ret_ty.clone(),
                    fn_table,
                    record_table,
                    adt_table,
                )?;
            }
            else_env.pop_scope();
            ctx.lowered_locals.pop_scope();
            ctx.instrs.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            ctx.instrs.push(IrInstr::Label { name: end_label });
            Ok(())
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            append_record_update_write_events_from_expr(
                *scrutinee,
                arena,
                &mut ctx.ownership_events,
            );
            let (scr_reg, scr_ty) = lower_expr(
                *scrutinee,
                arena,
                &mut ctx.next_reg,
                &mut ctx.instrs,
                env,
                &mut ctx.loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                &mut ctx.closure_state,
                &mut ctx.ownership_events,
                &mut ctx.lowered_locals,
            )?;
            if !matches!(
                scr_ty,
                Type::Quad
                    | Type::Adt(_)
                    | Type::Option(_)
                    | Type::Result(_, _)
                    | Type::I32
                    | Type::U32
            ) {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "match scrutinee must be quad, enum, Option(T), Result(T, E), i32, or u32"
                            .to_string(),
                });
            }
            let exhaustive_without_default = if default.is_none() {
                match missing_exhaustive_sum_variants(
                    &scr_ty,
                    arms.iter().map(|arm| (&arm.pat, arm.guard)),
                    arena,
                    adt_table,
                )? {
                    Some((family_label, missing)) if !missing.is_empty() => {
                        return Err(non_exhaustive_match_error(&family_label, &missing, false)?)
                    }
                    Some(_) => true,
                    None => {
                        return Err(FrontendError {
                            pos: 0,
                            message: "match requires default arm '_'".to_string(),
                        });
                    }
                }
            } else {
                false
            };

            let mid = ctx.next_if_id();
            let end_label = format!("match_{}_end", mid);
            let default_label = format!("match_{}_default", mid);
            let arm_labels: Vec<String> = (0..arms.len())
                .map(|i| format!("match_{}_arm_{}", mid, i))
                .collect();
            match scr_ty {
                Type::Quad if arms.iter().all(|arm| arm.guard.is_none()) => {
                    for (i, arm) in arms.iter().enumerate() {
                        let lit_reg = alloc(&mut ctx.next_reg);
                        ctx.instrs.push(IrInstr::LoadQ {
                            dst: lit_reg,
                            val: expect_quad_match_pattern(&arm.pat)?,
                        });
                        let cmp_reg = alloc(&mut ctx.next_reg);
                        ctx.instrs.push(IrInstr::CmpEq {
                            dst: cmp_reg,
                            lhs: scr_reg,
                            rhs: lit_reg,
                        });
                        ctx.instrs.push(IrInstr::JmpIf {
                            cond: cmp_reg,
                            label: arm_labels[i].clone(),
                        });
                    }
                    ctx.instrs.push(IrInstr::Jmp {
                        label: default_label.clone(),
                    });

                    for (i, arm) in arms.iter().enumerate() {
                        ctx.instrs.push(IrInstr::Label {
                            name: arm_labels[i].clone(),
                        });
                        let mut arm_env = env.clone();
                        arm_env.push_scope();
                        ctx.lowered_locals.push_scope();
                        for s in &arm.block {
                            lower_stmt(
                                *s,
                                arena,
                                ctx,
                                &mut arm_env,
                                ret_ty.clone(),
                                fn_table,
                                record_table,
                                adt_table,
                            )?;
                        }
                        arm_env.pop_scope();
                        ctx.lowered_locals.pop_scope();
                        ctx.instrs.push(IrInstr::Jmp {
                            label: end_label.clone(),
                        });
                    }
                }
                Type::Quad => {
                    for (i, arm) in arms.iter().enumerate() {
                        if i > 0 {
                            ctx.instrs.push(IrInstr::Label {
                                name: format!("match_{}_check_{}", mid, i),
                            });
                        }
                        let next_label = if i + 1 < arms.len() {
                            format!("match_{}_check_{}", mid, i + 1)
                        } else {
                            default_label.clone()
                        };

                        let lit_reg = alloc(&mut ctx.next_reg);
                        ctx.instrs.push(IrInstr::LoadQ {
                            dst: lit_reg,
                            val: expect_quad_match_pattern(&arm.pat)?,
                        });
                        let cmp_reg = alloc(&mut ctx.next_reg);
                        ctx.instrs.push(IrInstr::CmpEq {
                            dst: cmp_reg,
                            lhs: scr_reg,
                            rhs: lit_reg,
                        });
                        ctx.instrs.push(IrInstr::JmpIf {
                            cond: cmp_reg,
                            label: arm_labels[i].clone(),
                        });
                        ctx.instrs.push(IrInstr::Jmp {
                            label: next_label.clone(),
                        });

                        ctx.instrs.push(IrInstr::Label {
                            name: arm_labels[i].clone(),
                        });
                        let mut arm_env = env.clone();
                        arm_env.push_scope();
                        ctx.lowered_locals.push_scope();
                        if let Some(guard_reg) = lower_match_guard(
                            arm.guard,
                            arena,
                            &mut ctx.next_reg,
                            &mut ctx.instrs,
                            &arm_env,
                            &mut ctx.loop_stack,
                            fn_table,
                            record_table,
                            adt_table,
                            ret_ty.clone(),
                            &mut ctx.closure_state,
                            &mut ctx.ownership_events,
                            &mut ctx.lowered_locals,
                        )? {
                            let guarded_body_label = format!("match_{}_body_{}", mid, i);
                            ctx.instrs.push(IrInstr::JmpIf {
                                cond: guard_reg,
                                label: guarded_body_label.clone(),
                            });
                            ctx.instrs.push(IrInstr::Jmp { label: next_label });
                            ctx.instrs.push(IrInstr::Label {
                                name: guarded_body_label,
                            });
                        }
                        for s in &arm.block {
                            lower_stmt(
                                *s,
                                arena,
                                ctx,
                                &mut arm_env,
                                ret_ty.clone(),
                                fn_table,
                                record_table,
                                adt_table,
                            )?;
                        }
                        arm_env.pop_scope();
                        ctx.lowered_locals.pop_scope();
                        ctx.instrs.push(IrInstr::Jmp {
                            label: end_label.clone(),
                        });
                    }
                }
                Type::I32 | Type::U32 => {
                    for (i, arm) in arms.iter().enumerate() {
                        if i > 0 {
                            ctx.instrs.push(IrInstr::Label {
                                name: format!("match_{}_check_{}", mid, i),
                            });
                        }
                        let next_label = if i + 1 < arms.len() {
                            format!("match_{}_check_{}", mid, i + 1)
                        } else {
                            default_label.clone()
                        };

                        let lit_reg = alloc(&mut ctx.next_reg);
                        match expect_int_match_pattern(&arm.pat, &scr_ty)? {
                            IntMatchLiteral::I32(val) => {
                                ctx.instrs.push(IrInstr::LoadI32 { dst: lit_reg, val })
                            }
                            IntMatchLiteral::U32(val) => {
                                ctx.instrs.push(IrInstr::LoadU32 { dst: lit_reg, val })
                            }
                        }
                        let cmp_reg = alloc(&mut ctx.next_reg);
                        ctx.instrs.push(IrInstr::CmpEq {
                            dst: cmp_reg,
                            lhs: scr_reg,
                            rhs: lit_reg,
                        });
                        ctx.instrs.push(IrInstr::JmpIf {
                            cond: cmp_reg,
                            label: arm_labels[i].clone(),
                        });
                        ctx.instrs.push(IrInstr::Jmp {
                            label: next_label.clone(),
                        });

                        ctx.instrs.push(IrInstr::Label {
                            name: arm_labels[i].clone(),
                        });
                        let mut arm_env = env.clone();
                        arm_env.push_scope();
                        ctx.lowered_locals.push_scope();
                        if let Some(guard_reg) = lower_match_guard(
                            arm.guard,
                            arena,
                            &mut ctx.next_reg,
                            &mut ctx.instrs,
                            &arm_env,
                            &mut ctx.loop_stack,
                            fn_table,
                            record_table,
                            adt_table,
                            ret_ty.clone(),
                            &mut ctx.closure_state,
                            &mut ctx.ownership_events,
                            &mut ctx.lowered_locals,
                        )? {
                            let guarded_body_label = format!("match_{}_body_{}", mid, i);
                            ctx.instrs.push(IrInstr::JmpIf {
                                cond: guard_reg,
                                label: guarded_body_label.clone(),
                            });
                            ctx.instrs.push(IrInstr::Jmp { label: next_label });
                            ctx.instrs.push(IrInstr::Label {
                                name: guarded_body_label,
                            });
                        }
                        for s in &arm.block {
                            lower_stmt(
                                *s,
                                arena,
                                ctx,
                                &mut arm_env,
                                ret_ty.clone(),
                                fn_table,
                                record_table,
                                adt_table,
                            )?;
                        }
                        arm_env.pop_scope();
                        ctx.lowered_locals.pop_scope();
                        ctx.instrs.push(IrInstr::Jmp {
                            label: end_label.clone(),
                        });
                    }
                }
                Type::Adt(_) | Type::Option(_) | Type::Result(_, _) => {
                    let family = resolve_match_family_for_lowering(&scr_ty, arena, adt_table)?
                        .expect("sum scrutinee family should resolve");
                    let scr_tag_reg = alloc(&mut ctx.next_reg);
                    ctx.instrs.push(IrInstr::AdtTag {
                        dst: scr_tag_reg,
                        src: scr_reg,
                        adt_name: family.family_name.clone(),
                    });
                    let resolved_patterns = arms
                        .iter()
                        .map(|arm| {
                            resolve_sum_match_pattern_for_lowering(
                                &arm.pat,
                                &scr_ty,
                                arena,
                                record_table,
                                adt_table,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    for (i, arm) in arms.iter().enumerate() {
                        if i > 0 {
                            ctx.instrs.push(IrInstr::Label {
                                name: format!("match_{}_check_{}", mid, i),
                            });
                        }
                        let next_label = if i + 1 < arms.len() {
                            format!("match_{}_check_{}", mid, i + 1)
                        } else {
                            default_label.clone()
                        };
                        let expected_tag_reg = alloc(&mut ctx.next_reg);
                        ctx.instrs.push(IrInstr::LoadI32 {
                            dst: expected_tag_reg,
                            val: resolved_patterns[i].tag,
                        });
                        let cmp_reg = alloc(&mut ctx.next_reg);
                        ctx.instrs.push(IrInstr::CmpEq {
                            dst: cmp_reg,
                            lhs: scr_tag_reg,
                            rhs: expected_tag_reg,
                        });
                        ctx.instrs.push(IrInstr::JmpIf {
                            cond: cmp_reg,
                            label: arm_labels[i].clone(),
                        });
                        ctx.instrs.push(IrInstr::Jmp {
                            label: next_label.clone(),
                        });

                        ctx.instrs.push(IrInstr::Label {
                            name: arm_labels[i].clone(),
                        });
                        let mut arm_env = env.clone();
                        arm_env.push_scope();
                        ctx.lowered_locals.push_scope();
                        lower_adt_match_bindings(
                            &resolved_patterns[i],
                            scr_reg,
                            &mut ctx.next_reg,
                            &mut ctx.instrs,
                            &mut arm_env,
                            arena,
                            &mut ctx.lowered_locals,
                        )?;
                        if let Some(guard_reg) = lower_match_guard(
                            arm.guard,
                            arena,
                            &mut ctx.next_reg,
                            &mut ctx.instrs,
                            &arm_env,
                            &mut ctx.loop_stack,
                            fn_table,
                            record_table,
                            adt_table,
                            ret_ty.clone(),
                            &mut ctx.closure_state,
                            &mut ctx.ownership_events,
                            &mut ctx.lowered_locals,
                        )? {
                            let guarded_body_label = format!("match_{}_body_{}", mid, i);
                            ctx.instrs.push(IrInstr::JmpIf {
                                cond: guard_reg,
                                label: guarded_body_label.clone(),
                            });
                            ctx.instrs.push(IrInstr::Jmp { label: next_label });
                            ctx.instrs.push(IrInstr::Label {
                                name: guarded_body_label,
                            });
                        }
                        for s in &arm.block {
                            lower_stmt(
                                *s,
                                arena,
                                ctx,
                                &mut arm_env,
                                ret_ty.clone(),
                                fn_table,
                                record_table,
                                adt_table,
                            )?;
                        }
                        arm_env.pop_scope();
                        ctx.lowered_locals.pop_scope();
                        ctx.instrs.push(IrInstr::Jmp {
                            label: end_label.clone(),
                        });
                    }
                }
                _ => unreachable!("non-matchable scrutinee handled above"),
            }

            ctx.instrs.push(IrInstr::Label {
                name: default_label,
            });
            if exhaustive_without_default {
                let cond = alloc(&mut ctx.next_reg);
                ctx.instrs.push(IrInstr::LoadBool {
                    dst: cond,
                    val: false,
                });
                ctx.instrs.push(IrInstr::Assert { cond });
            } else {
                let default = default
                    .as_ref()
                    .expect("non-exhaustive match statement requires explicit default in lowering");
                let mut def_env = env.clone();
                def_env.push_scope();
                ctx.lowered_locals.push_scope();
                for s in default {
                    lower_stmt(
                        *s,
                        arena,
                        ctx,
                        &mut def_env,
                        ret_ty.clone(),
                        fn_table,
                        record_table,
                        adt_table,
                    )?;
                }
                def_env.pop_scope();
                ctx.lowered_locals.pop_scope();
            }
            ctx.instrs.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            ctx.instrs.push(IrInstr::Label { name: end_label });
            Ok(())
        }
    }
}

fn lower_value_block_expr(
    block: &BlockExpr,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<Type>,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(u16, Type), FrontendError> {
    let mut block_env = env.clone();
    block_env.push_scope();
    lowered_locals.push_scope();
    for stmt in &block.statements {
        match arena.stmt(*stmt) {
            Stmt::Const { name, ty, value } => {
                // #1709: mirrors `lower_stmt`'s `Stmt::Const` arm - this
                // statement's own value has not been scanned by any
                // enclosing authority (the pre-scan that reaches nested
                // blocks only follows a block's `tail`, never its
                // `statements`), so this call is required, not a duplicate.
                append_record_update_write_events_from_expr(*value, arena, ownership_events);
                let (reg, vty) = lower_expr_with_expected(
                    *value,
                    arena,
                    next,
                    out,
                    &block_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ty.clone(),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let final_ty = if let Some(ann) = ty {
                    canonicalize_declared_type(ann, record_table, adt_table, arena)?
                } else {
                    vty
                };
                block_env.insert_const(*name, final_ty);
                out.push(IrInstr::StoreVar {
                    name: lowered_locals.bind(arena, *name)?,
                    src: reg,
                });
            }
            Stmt::Let {
                name,
                is_mut,
                ty,
                value,
            } => {
                append_record_update_write_events_from_expr(*value, arena, ownership_events);
                let (reg, vty) = lower_expr_with_expected(
                    *value,
                    arena,
                    next,
                    out,
                    &block_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ty.clone(),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let final_ty = if let Some(ann) = ty {
                    canonicalize_declared_type(ann, record_table, adt_table, arena)?
                } else {
                    vty
                };
                if *is_mut {
                    block_env.insert_mut(*name, final_ty);
                } else {
                    block_env.insert(*name, final_ty);
                }
                out.push(IrInstr::StoreVar {
                    name: lowered_locals.bind(arena, *name)?,
                    src: reg,
                });
            }
            Stmt::LetTuple { items, ty, value } => {
                append_record_update_write_events_from_expr(*value, arena, ownership_events);
                // #1709: derive the same canonical `AccessPath` the
                // top-level `lower_stmt` arm derives - previously hardcoded
                // to `None` here, which meant `bind_tuple_items` could never
                // emit a Borrow event for this call site regardless of the
                // sink, since it only pushes when a path is present.
                let sequence_path = sequence_access_path_from_expr(*value, arena);
                let (tuple_reg, vty) = lower_expr_with_expected(
                    *value,
                    arena,
                    next,
                    out,
                    &block_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ty.clone(),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                let final_ty = if let Some(ann) = ty {
                    canonicalize_declared_type(ann, record_table, adt_table, arena)?
                } else {
                    vty
                };
                bind_tuple_items(
                    items,
                    tuple_reg,
                    &final_ty,
                    sequence_path.as_ref(),
                    arena,
                    next,
                    out,
                    ownership_events,
                    lowered_locals,
                    &mut block_env,
                )?;
            }
            Stmt::LetRecord {
                record_name,
                items,
                value,
            } => {
                append_record_update_write_events_from_expr(*value, arena, ownership_events);
                let record_path = direct_record_access_path_from_expr(*value, arena);
                let (record_reg, record_ty) = lower_expr_with_expected(
                    *value,
                    arena,
                    next,
                    out,
                    &block_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    Some(Type::Record(*record_name)),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                bind_record_items(
                    *record_name,
                    items,
                    record_reg,
                    &record_ty,
                    record_path.as_ref(),
                    arena,
                    next,
                    out,
                    ownership_events,
                    lowered_locals,
                    &mut block_env,
                    record_table,
                    adt_table,
                )?;
            }
            Stmt::LetElseRecord { .. } => {
                return Err(FrontendError {
                    pos: 0,
                    message: "block expression body currently does not allow record let-else"
                        .to_string(),
                });
            }
            Stmt::Discard { ty, value } => {
                // #1709 corrective (exact-head review of 17e89f63): mirrors
                // `lower_stmt`'s `Stmt::Discard` arm, which prescans before
                // lowering. This nested arm had the correct sink but was
                // never wired to the producer authority itself.
                append_record_update_write_events_from_expr(*value, arena, ownership_events);
                let _ = lower_expr_with_expected(
                    *value,
                    arena,
                    next,
                    out,
                    &block_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ty.clone(),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
            }
            Stmt::Expr(expr) => {
                // #1709 corrective: mirrors `lower_stmt`'s `Stmt::Expr` arm.
                append_record_update_write_events_from_expr(*expr, arena, ownership_events);
                lower_expr_stmt_with_parts(
                    *expr,
                    arena,
                    next,
                    out,
                    &block_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
            }
            _ => {
                return Err(FrontendError {
                    pos: 0,
                    message: "value-producing block currently supports only const-bindings, let-bindings, discard binds, and expression statements before the tail value".to_string(),
                });
            }
        }
    }
    let tail = lower_expr_with_expected(
        block.tail,
        arena,
        next,
        out,
        &block_env,
        loop_stack,
        fn_table,
        record_table,
        adt_table,
        expected,
        ret_ty,
        closure_state,
        ownership_events,
        lowered_locals,
    )?;
    block_env.pop_scope();
    lowered_locals.pop_scope();
    Ok(tail)
}

fn lower_adt_ctor_expr(
    ctor_expr: &AdtCtorExpr,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<Type>,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(u16, Type), FrontendError> {
    if let Some(lowered) = lower_std_form_ctor_expr(
        ctor_expr,
        arena,
        next,
        out,
        env,
        loop_stack,
        fn_table,
        record_table,
        adt_table,
        expected.clone(),
        ret_ty.clone(),
        closure_state,
        ownership_events,
        lowered_locals,
    )? {
        return Ok(lowered);
    }
    let adt = adt_table.get(&ctor_expr.adt_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown enum type '{}' in constructor lowering",
            resolve_symbol_name(arena, ctor_expr.adt_name)?
        ),
    })?;
    let (tag, variant) = adt
        .variants
        .iter()
        .enumerate()
        .find(|(_, variant)| variant.name == ctor_expr.variant_name)
        .ok_or(FrontendError {
            pos: 0,
            message: format!(
                "enum '{}' has no variant named '{}' in constructor lowering",
                resolve_symbol_name(arena, ctor_expr.adt_name)?,
                resolve_symbol_name(arena, ctor_expr.variant_name)?
            ),
        })?;
    if variant.payload.len() != ctor_expr.payload.len() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "enum constructor '{}::{}' expects {} payload items in lowering, got {}",
                resolve_symbol_name(arena, ctor_expr.adt_name)?,
                resolve_symbol_name(arena, ctor_expr.variant_name)?,
                variant.payload.len(),
                ctor_expr.payload.len()
            ),
        });
    }

    let mut regs = Vec::with_capacity(ctor_expr.payload.len());
    for (payload_expr, declared_expected) in ctor_expr.payload.iter().zip(variant.payload.iter()) {
        let expected_ty =
            canonicalize_declared_type(declared_expected, record_table, adt_table, arena)?;
        let (reg, actual_ty) = lower_expr_with_expected(
            *payload_expr,
            arena,
            next,
            out,
            env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            Some(expected_ty.clone()),
            ret_ty.clone(),
            closure_state,
            ownership_events,
            lowered_locals,
        )?;
        if actual_ty != expected_ty {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "enum constructor '{}::{}' payload type mismatch in lowering: expected {:?}, got {:?}",
                    resolve_symbol_name(arena, ctor_expr.adt_name)?,
                    resolve_symbol_name(arena, ctor_expr.variant_name)?,
                    expected_ty,
                    actual_ty
                ),
            });
        }
        regs.push(reg);
    }

    let dst = alloc(next);
    out.push(IrInstr::MakeAdt {
        dst,
        adt_name: resolve_symbol_name(arena, ctor_expr.adt_name)?.to_string(),
        variant_name: resolve_symbol_name(arena, ctor_expr.variant_name)?.to_string(),
        tag: u16::try_from(tag).map_err(|_| FrontendError {
            pos: 0,
            message: "enum variant tag exceeds v0 limit".to_string(),
        })?,
        items: regs,
    });
    Ok((dst, Type::Adt(ctor_expr.adt_name)))
}

fn lower_std_form_ctor_expr(
    ctor_expr: &AdtCtorExpr,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<Type>,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<Option<(u16, Type)>, FrontendError> {
    let type_name = resolve_symbol_name(arena, ctor_expr.adt_name)?;
    let variant_name = resolve_symbol_name(arena, ctor_expr.variant_name)?;

    if type_name == "Option" {
        match variant_name {
            "Some" => {
                if ctor_expr.payload.len() != 1 {
                    return Err(FrontendError {
                        pos: 0,
                        message: "Option::Some expects exactly one payload item in lowering"
                            .to_string(),
                    });
                }
                let item_expected = match expected.as_ref() {
                    Some(Type::Option(item_ty)) => Some((**item_ty).clone()),
                    _ => None,
                };
                let (item_reg, item_ty) = lower_expr_with_expected(
                    ctor_expr.payload[0],
                    arena,
                    next,
                    out,
                    env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    item_expected.clone(),
                    ret_ty,
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                if let Some(expected_item) = item_expected {
                    if item_ty != expected_item {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "Option::Some payload type mismatch in lowering: expected {:?}, got {:?}",
                                expected_item, item_ty
                            ),
                        });
                    }
                }
                let dst = alloc(next);
                out.push(IrInstr::MakeAdt {
                    dst,
                    adt_name: "Option".to_string(),
                    variant_name: "Some".to_string(),
                    tag: 1,
                    items: vec![item_reg],
                });
                return Ok(Some((dst, Type::Option(Box::new(item_ty)))));
            }
            "None" => {
                if !ctor_expr.payload.is_empty() {
                    return Err(FrontendError {
                        pos: 0,
                        message: "Option::None does not accept payload items in lowering"
                            .to_string(),
                    });
                }
                let Some(Type::Option(item_ty)) = expected else {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "Option::None currently requires contextual Option(T) type in v0 lowering"
                                .to_string(),
                    });
                };
                let dst = alloc(next);
                out.push(IrInstr::MakeAdt {
                    dst,
                    adt_name: "Option".to_string(),
                    variant_name: "None".to_string(),
                    tag: 0,
                    items: Vec::new(),
                });
                return Ok(Some((dst, Type::Option(item_ty))));
            }
            _ => {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("Option has no variant named '{}' in lowering", variant_name),
                })
            }
        }
    }

    if type_name == "Result" {
        if ctor_expr.payload.len() != 1 {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "Result::{} expects exactly one payload item in lowering",
                    variant_name
                ),
            });
        }
        let Some(Type::Result(ok_ty, err_ty)) = expected else {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "Result::{} currently requires contextual Result(T, E) type in v0 lowering",
                    variant_name
                ),
            });
        };
        let (payload_expected, tag) = match variant_name {
            "Ok" => ((*ok_ty).clone(), 0),
            "Err" => ((*err_ty).clone(), 1),
            _ => {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("Result has no variant named '{}' in lowering", variant_name),
                })
            }
        };
        let (payload_reg, payload_ty) = lower_expr_with_expected(
            ctor_expr.payload[0],
            arena,
            next,
            out,
            env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            Some(payload_expected.clone()),
            ret_ty,
            closure_state,
            ownership_events,
            lowered_locals,
        )?;
        if payload_ty != payload_expected {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "Result::{} payload type mismatch in lowering: expected {:?}, got {:?}",
                    variant_name, payload_expected, payload_ty
                ),
            });
        }
        let dst = alloc(next);
        out.push(IrInstr::MakeAdt {
            dst,
            adt_name: "Result".to_string(),
            variant_name: variant_name.to_string(),
            tag,
            items: vec![payload_reg],
        });
        return Ok(Some((dst, Type::Result(ok_ty, err_ty))));
    }

    Ok(None)
}

#[derive(Debug, Clone)]
struct LoweredAdtMatchBinding {
    name: SymbolId,
    ty: Type,
    index: u16,
}

#[derive(Debug, Clone)]
struct LoweredAdtMatchPattern {
    adt_name: String,
    tag: i32,
    bindings: Vec<LoweredAdtMatchBinding>,
}

#[derive(Debug, Clone)]
struct LoweredMatchFamilyVariant {
    name: String,
    tag: i32,
    payload: Vec<Type>,
}

#[derive(Debug, Clone)]
struct LoweredMatchFamily {
    family_name: String,
    display_label: String,
    variants: Vec<LoweredMatchFamilyVariant>,
}

fn expect_quad_match_pattern(pat: &MatchPattern) -> Result<QuadVal, FrontendError> {
    match pat {
        MatchPattern::Quad(pat) => Ok(*pat),
        MatchPattern::Adt(_) => Err(FrontendError {
            pos: 0,
            message: "enum match pattern requires enum scrutinee in lowering".to_string(),
        }),
        // M9.4 Wave 1: these patterns are typecheck-only in M9.4; lowering is deferred.
        MatchPattern::Wildcard | MatchPattern::Or(_) | MatchPattern::IntRange(_) => {
            Err(FrontendError {
                pos: 0,
                message: "wildcard/or/range match pattern lowering is not yet implemented in the IR backend".to_string(),
            })
        }
    }
}

/// A lowered integer match-pattern literal, carrying the scrutinee's own
/// runtime type so callers emit `LoadI32`/`LoadU32` consistently with the
/// scrutinee register's actual runtime type instead of always assuming i32.
enum IntMatchLiteral {
    I32(i32),
    U32(u32),
}

fn expect_int_match_pattern(
    pat: &MatchPattern,
    scrutinee_ty: &Type,
) -> Result<IntMatchLiteral, FrontendError> {
    match pat {
        MatchPattern::IntRange(range) if range.start == range.end && range.inclusive => {
            if matches!(scrutinee_ty, Type::U32) {
                u32::try_from(range.start)
                    .map(IntMatchLiteral::U32)
                    .map_err(|_| FrontendError {
                        pos: 0,
                        message: "integer match pattern literal is outside u32 range".to_string(),
                    })
            } else {
                i32::try_from(range.start)
                    .map(IntMatchLiteral::I32)
                    .map_err(|_| FrontendError {
                        pos: 0,
                        message: "integer match pattern literal is outside i32 range".to_string(),
                    })
            }
        }
        // Exclusive equal-bound ranges (`5..5`) are semantically empty and must
        // never be treated as the literal `5`; route them to the same
        // deterministic "not yet implemented" rejection as every other range
        // form lowering does not support, rather than silently miscompiling.
        MatchPattern::IntRange(_) => Err(FrontendError {
            pos: 0,
            message: "integer range match pattern lowering is not yet implemented in the IR backend"
                .to_string(),
        }),
        MatchPattern::Adt(_) => Err(FrontendError {
            pos: 0,
            message: "enum match pattern requires enum scrutinee in lowering".to_string(),
        }),
        // M9.4 Wave 1: these patterns are typecheck-only in M9.4; lowering is deferred.
        MatchPattern::Wildcard | MatchPattern::Or(_) | MatchPattern::Quad(_) => Err(FrontendError {
            pos: 0,
            message: "wildcard/or/quad match pattern lowering is not yet implemented in the IR backend".to_string(),
        }),
    }
}

fn resolve_match_family_for_lowering(
    scrutinee_ty: &Type,
    arena: &AstArena,
    adt_table: &AdtTable,
) -> Result<Option<LoweredMatchFamily>, FrontendError> {
    match scrutinee_ty {
        Type::Adt(adt_name) => {
            let adt = adt_table.get(adt_name).ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "unknown enum type '{}' in match lowering",
                    resolve_symbol_name(arena, *adt_name)?,
                ),
            })?;
            let family_name = resolve_symbol_name(arena, *adt_name)?.to_string();
            let mut variants = Vec::new();
            for (tag, variant) in adt.variants.iter().enumerate() {
                variants.push(LoweredMatchFamilyVariant {
                    name: resolve_symbol_name(arena, variant.name)?.to_string(),
                    tag: i32::try_from(tag).map_err(|_| FrontendError {
                        pos: 0,
                        message: "enum variant tag exceeds v0 lowering limit".to_string(),
                    })?,
                    payload: variant.payload.clone(),
                });
            }
            Ok(Some(LoweredMatchFamily {
                display_label: format!("enum '{}'", family_name),
                family_name,
                variants,
            }))
        }
        Type::Option(item_ty) => Ok(Some(LoweredMatchFamily {
            family_name: "Option".to_string(),
            display_label: "Option(T)".to_string(),
            variants: vec![
                LoweredMatchFamilyVariant {
                    name: "None".to_string(),
                    tag: 0,
                    payload: Vec::new(),
                },
                LoweredMatchFamilyVariant {
                    name: "Some".to_string(),
                    tag: 1,
                    payload: vec![(**item_ty).clone()],
                },
            ],
        })),
        Type::Result(ok_ty, err_ty) => Ok(Some(LoweredMatchFamily {
            family_name: "Result".to_string(),
            display_label: "Result(T, E)".to_string(),
            variants: vec![
                LoweredMatchFamilyVariant {
                    name: "Ok".to_string(),
                    tag: 0,
                    payload: vec![(**ok_ty).clone()],
                },
                LoweredMatchFamilyVariant {
                    name: "Err".to_string(),
                    tag: 1,
                    payload: vec![(**err_ty).clone()],
                },
            ],
        })),
        _ => Ok(None),
    }
}

fn resolve_sum_match_pattern_for_lowering(
    pat: &MatchPattern,
    scrutinee_ty: &Type,
    arena: &AstArena,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<LoweredAdtMatchPattern, FrontendError> {
    let MatchPattern::Adt(adt_pat) = pat else {
        let family = resolve_match_family_for_lowering(scrutinee_ty, arena, adt_table)?
            .expect("non-quad match family should resolve");
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "quad match pattern requires quad scrutinee; {} needs explicit variant patterns in lowering",
                family.display_label,
            ),
        });
    };
    let Some(family) = resolve_match_family_for_lowering(scrutinee_ty, arena, adt_table)? else {
        return Err(FrontendError {
            pos: 0,
            message: "match scrutinee must be quad, enum, Option(T), or Result(T, E)".to_string(),
        });
    };
    let pattern_family = resolve_symbol_name(arena, adt_pat.adt_name)?.to_string();
    if pattern_family != family.family_name {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "match arm pattern type '{}' does not match scrutinee {} in lowering",
                pattern_family, family.display_label,
            ),
        });
    }
    let pattern_variant = resolve_symbol_name(arena, adt_pat.variant_name)?.to_string();
    let variant = family
        .variants
        .iter()
        .find(|variant| variant.name == pattern_variant)
        .ok_or(FrontendError {
            pos: 0,
            message: format!(
                "{} has no variant named '{}' in match lowering",
                family.display_label, pattern_variant,
            ),
        })?;
    if variant.payload.len() != adt_pat.items.len() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "match pattern '{}::{}' expects {} payload items in lowering, got {}",
                family.family_name,
                pattern_variant,
                variant.payload.len(),
                adt_pat.items.len(),
            ),
        });
    }

    let mut bindings = Vec::new();
    for (index, (item, declared_ty)) in adt_pat.items.iter().zip(variant.payload.iter()).enumerate()
    {
        let payload_ty = canonicalize_declared_type(declared_ty, record_table, adt_table, arena)?;
        if let sm_front::types::AdtPatternItem::Bind { name, .. } = item {
            bindings.push(LoweredAdtMatchBinding {
                name: *name,
                ty: payload_ty,
                index: u16::try_from(index).map_err(|_| FrontendError {
                    pos: 0,
                    message: "enum match payload index exceeds v0 limit".to_string(),
                })?,
            });
        }
    }

    Ok(LoweredAdtMatchPattern {
        adt_name: family.family_name,
        tag: variant.tag,
        bindings,
    })
}

fn lower_adt_match_bindings(
    pattern: &LoweredAdtMatchPattern,
    scr_reg: u16,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &mut ScopeEnv,
    arena: &AstArena,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(), FrontendError> {
    for binding in &pattern.bindings {
        let reg = alloc(next);
        out.push(IrInstr::AdtGet {
            dst: reg,
            src: scr_reg,
            adt_name: pattern.adt_name.clone(),
            index: binding.index,
        });
        out.push(IrInstr::StoreVar {
            name: lowered_locals.bind(arena, binding.name)?,
            src: reg,
        });
        env.insert(binding.name, binding.ty.clone());
    }
    Ok(())
}

fn missing_exhaustive_sum_variants<'a>(
    scrutinee_ty: &Type,
    patterns: impl IntoIterator<Item = (&'a MatchPattern, Option<ExprId>)>,
    arena: &AstArena,
    adt_table: &AdtTable,
) -> Result<Option<(String, Vec<String>)>, FrontendError> {
    let Some(family) = resolve_match_family_for_lowering(scrutinee_ty, arena, adt_table)? else {
        return Ok(None);
    };

    let mut covered = BTreeSet::new();
    for (pat, guard) in patterns {
        if guard.is_some() {
            continue;
        }
        if let MatchPattern::Adt(adt_pat) = pat {
            if resolve_symbol_name(arena, adt_pat.adt_name)? == family.family_name {
                covered.insert(resolve_symbol_name(arena, adt_pat.variant_name)?.to_string());
            }
        }
    }

    Ok(Some((
        family.display_label,
        family
            .variants
            .iter()
            .filter(|variant| !covered.contains(&variant.name))
            .map(|variant| variant.name.clone())
            .collect(),
    )))
}

fn non_exhaustive_match_error(
    family_label: &str,
    missing: &[String],
    expression: bool,
) -> Result<FrontendError, FrontendError> {
    Ok(FrontendError {
        pos: 0,
        message: format!(
            "non-exhaustive match{} for {}; missing variants: {}",
            if expression { " expression" } else { "" },
            family_label,
            missing.join(", "),
        ),
    })
}

fn lower_impossible_match_trap(label: String, next: &mut u16, out: &mut Vec<IrInstr>) {
    out.push(IrInstr::Label { name: label });
    let cond = alloc(next);
    out.push(IrInstr::LoadBool {
        dst: cond,
        val: false,
    });
    out.push(IrInstr::Assert { cond });
}

fn lower_match_guard(
    guard: Option<ExprId>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<Option<u16>, FrontendError> {
    let Some(guard_expr) = guard else {
        return Ok(None);
    };
    let (guard_reg, guard_ty) = lower_expr(
        guard_expr,
        arena,
        next,
        out,
        env,
        loop_stack,
        fn_table,
        record_table,
        adt_table,
        ret_ty,
        closure_state,
        ownership_events,
        lowered_locals,
    )?;
    if guard_ty != Type::Bool {
        return Err(FrontendError {
            pos: 0,
            message: "match guard condition must be bool".to_string(),
        });
    }
    Ok(Some(guard_reg))
}

fn lower_ensures_clauses(
    contract_ensures: &[ExprId],
    contract_result_symbol: Option<SymbolId>,
    result_value: Option<(u16, Type)>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(), FrontendError> {
    if contract_ensures.is_empty() {
        return Ok(());
    }

    let mut contract_env = env.clone();
    if let Some(result_symbol) = contract_result_symbol {
        let (result_reg, result_ty) = result_value.ok_or(FrontendError {
            pos: 0,
            message: "ensures clause referencing result requires explicit return value".to_string(),
        })?;
        contract_env.insert_const(result_symbol, result_ty);
        // #1724 (FA-04-018): `result` is a synthetic single-use contract
        // slot, not a genuine source declaration - it can never be
        // shadowed (fresh `contract_env` per clause activation, never
        // nested). `bind_raw` keeps this binding's key identical to the
        // literal "result" text the StoreVar below already uses, so any
        // `Expr::Var` reference to it inside the clause resolves correctly.
        lowered_locals.bind_raw(arena, result_symbol)?;
        out.push(IrInstr::StoreVar {
            name: "result".to_string(),
            src: result_reg,
        });
    }

    for condition in contract_ensures {
        let (cond_reg, cond_ty) = lower_expr(
            *condition,
            arena,
            next,
            out,
            &contract_env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            ret_ty.clone(),
            closure_state,
            ownership_events,
            lowered_locals,
        )?;
        if cond_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "ensures clause condition must be bool in lowering, got {:?}",
                    cond_ty
                ),
            });
        }
        out.push(IrInstr::Assert { cond: cond_reg });
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractInvariantPhase {
    Entry,
    Exit,
}

fn lower_invariant_clauses(
    contract_invariants: &[ExprId],
    contract_result_symbol: Option<SymbolId>,
    result_value: Option<(u16, Type)>,
    phase: ContractInvariantPhase,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(), FrontendError> {
    if contract_invariants.is_empty() {
        return Ok(());
    }

    let mut contract_env = env.clone();
    if let Some(result_symbol) = contract_result_symbol {
        if let Some((result_reg, result_ty)) = result_value.clone() {
            contract_env.insert_const(result_symbol, result_ty);
            // #1724 (FA-04-018): see the matching comment in
            // `lower_ensures_clauses`.
            lowered_locals.bind_raw(arena, result_symbol)?;
            out.push(IrInstr::StoreVar {
                name: "result".to_string(),
                src: result_reg,
            });
        }
    }

    for condition in contract_invariants {
        let references_result = contract_clause_references_result(*condition, arena)?;
        if references_result && phase == ContractInvariantPhase::Entry {
            continue;
        }
        if references_result && result_value.is_none() {
            return Err(FrontendError {
                pos: 0,
                message: "invariant clause referencing result requires explicit return value"
                    .to_string(),
            });
        }
        let (cond_reg, cond_ty) = lower_expr(
            *condition,
            arena,
            next,
            out,
            &contract_env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            ret_ty.clone(),
            closure_state,
            ownership_events,
            lowered_locals,
        )?;
        if cond_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "invariant clause condition must be bool in lowering, got {:?}",
                    cond_ty
                ),
            });
        }
        out.push(IrInstr::Assert { cond: cond_reg });
    }

    Ok(())
}

fn lower_return_payload(
    value: Option<ExprId>,
    contract_ensures: &[ExprId],
    contract_result_symbol: Option<SymbolId>,
    contract_invariants: &[ExprId],
    contract_invariant_result_symbol: Option<SymbolId>,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(), FrontendError> {
    match value {
        Some(expr_id) => {
            let (reg, ty) = lower_expr_with_expected(
                expr_id,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(ret_ty.clone()),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if ty != ret_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "return type mismatch in lowering: expected {:?}, got {:?}",
                        ret_ty, ty
                    ),
                });
            }
            lower_ensures_clauses(
                contract_ensures,
                contract_result_symbol,
                Some((reg, ty.clone())),
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            lower_invariant_clauses(
                contract_invariants,
                contract_invariant_result_symbol,
                Some((reg, ty.clone())),
                ContractInvariantPhase::Exit,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            out.push(IrInstr::Ret { src: Some(reg) });
            Ok(())
        }
        None => {
            if ret_ty != Type::Unit {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("return without value in non-unit function ({:?})", ret_ty),
                });
            }
            lower_ensures_clauses(
                contract_ensures,
                contract_result_symbol,
                None,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            lower_invariant_clauses(
                contract_invariants,
                contract_invariant_result_symbol,
                None,
                ContractInvariantPhase::Exit,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            out.push(IrInstr::Ret { src: None });
            Ok(())
        }
    }
}

fn lower_loop_expr(
    loop_expr: &LoopExpr,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<Type>,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(u16, Type), FrontendError> {
    let id = alloc_loop_expr_id(next);
    let start_label = format!("loop_expr_{}_start", id);
    let end_label = format!("loop_expr_{}_end", id);
    let result_name = format!("__loop_expr_{}_result", id);

    loop_stack.push(LoopLoweringFrame {
        kind: LoopLoweringFrameKind::Expression,
        end_label: end_label.clone(),
        continue_label: start_label.clone(),
        result_name: result_name.clone(),
        result_ty: None,
        expected_ty: expected.clone(),
    });

    out.push(IrInstr::Label {
        name: start_label.clone(),
    });

    let mut body_env = env.clone();
    body_env.push_scope();
    for stmt in &loop_expr.body {
        lower_loop_expr_stmt(
            *stmt,
            arena,
            next,
            out,
            &mut body_env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            ret_ty.clone(),
            closure_state,
            ownership_events,
            lowered_locals,
        )?;
    }
    body_env.pop_scope();
    out.push(IrInstr::Jmp { label: start_label });
    out.push(IrInstr::Label { name: end_label });

    let frame = loop_stack.pop().expect("loop frame must exist");
    let result_ty = frame.result_ty.ok_or(FrontendError {
        pos: 0,
        message: "loop expression requires at least one break value".to_string(),
    })?;
    if let Some(expected_ty) = expected {
        if expected_ty != result_ty {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "loop expression result type mismatch in lowering: expected {:?}, got {:?}",
                    expected_ty, result_ty
                ),
            });
        }
    }
    let dst = alloc(next);
    out.push(IrInstr::LoadVar {
        dst,
        name: result_name,
    });
    Ok((dst, result_ty))
}

fn lower_loop_expr_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &mut ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(), FrontendError> {
    match arena.stmt(stmt_id) {
        Stmt::LetElseTuple { .. } | Stmt::LetElseRecord { .. } => Err(FrontendError {
            pos: 0,
            message: "loop expression body currently does not allow let-else".to_string(),
        }),
        Stmt::ForRange { .. } => Err(FrontendError {
            pos: 0,
            message: "loop expression body currently does not allow for-range".to_string(),
        }),
        Stmt::While { .. } => Err(FrontendError {
            pos: 0,
            message: "loop expression body currently does not allow while statement".to_string(),
        }),
        Stmt::Loop { .. } => Err(FrontendError {
            pos: 0,
            message: "loop expression body currently does not allow statement loop".to_string(),
        }),
        Stmt::ForEach { .. } => Err(FrontendError {
            pos: 0,
            message: "loop expression body currently does not allow iterable for-each".to_string(),
        }),
        Stmt::Guard { .. } | Stmt::Return(..) | Stmt::Continue => Err(FrontendError {
            pos: 0,
            message: "loop expression body currently does not allow guard clause or return"
                .to_string(),
        }),
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            // #1709 corrective: mirrors `lower_stmt`'s `Stmt::If` arm, which
            // prescans `condition` before lowering it. This dedicated
            // loop-expression `If` implementation does not delegate to
            // `lower_stmt` (unlike everything reaching the `_` fallback
            // below), so it needs the same call independently.
            append_record_update_write_events_from_expr(*condition, arena, ownership_events);
            let (cond_reg, cond_ty) = lower_expr(
                *condition,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if condition must be bool".to_string(),
                });
            }

            let id = alloc_loop_expr_id(next);
            let then_label = format!("loop_if_{}_then", id);
            let else_label = format!("loop_if_{}_else", id);
            let end_label = format!("loop_if_{}_end", id);

            out.push(IrInstr::JmpIf {
                cond: cond_reg,
                label: then_label.clone(),
            });
            out.push(IrInstr::Jmp {
                label: else_label.clone(),
            });

            out.push(IrInstr::Label { name: then_label });
            let mut then_env = env.clone();
            then_env.push_scope();
            lowered_locals.push_scope();
            for stmt in then_block {
                lower_loop_expr_stmt(
                    *stmt,
                    arena,
                    next,
                    out,
                    &mut then_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
            }
            then_env.pop_scope();
            lowered_locals.pop_scope();
            out.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            out.push(IrInstr::Label { name: else_label });
            let mut else_env = env.clone();
            else_env.push_scope();
            lowered_locals.push_scope();
            for stmt in else_block {
                lower_loop_expr_stmt(
                    *stmt,
                    arena,
                    next,
                    out,
                    &mut else_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
            }
            else_env.pop_scope();
            lowered_locals.pop_scope();
            out.push(IrInstr::Jmp {
                label: end_label.clone(),
            });

            out.push(IrInstr::Label { name: end_label });
            Ok(())
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            // #1709 corrective: mirrors `lower_stmt`'s `Stmt::Match` arm,
            // which prescans `scrutinee` before lowering it. This dedicated
            // loop-expression `Match` implementation does not delegate to
            // `lower_stmt`, so it needs the same call independently.
            append_record_update_write_events_from_expr(*scrutinee, arena, ownership_events);
            let (scr_reg, scr_ty) = lower_expr(
                *scrutinee,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if !matches!(
                scr_ty,
                Type::Quad
                    | Type::Adt(_)
                    | Type::Option(_)
                    | Type::Result(_, _)
                    | Type::I32
                    | Type::U32
            ) {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "match scrutinee must be quad, enum, Option(T), Result(T, E), i32, or u32"
                            .to_string(),
                });
            }
            let exhaustive_without_default = if default.is_none() {
                match missing_exhaustive_sum_variants(
                    &scr_ty,
                    arms.iter().map(|arm| (&arm.pat, arm.guard)),
                    arena,
                    adt_table,
                )? {
                    Some((family_label, missing)) if !missing.is_empty() => {
                        return Err(non_exhaustive_match_error(&family_label, &missing, false)?)
                    }
                    Some(_) => true,
                    None => {
                        return Err(FrontendError {
                            pos: 0,
                            message: "match requires default arm '_'".to_string(),
                        });
                    }
                }
            } else {
                false
            };

            let id = alloc_loop_expr_id(next);
            let end_label = format!("loop_match_{}_end", id);
            let default_label = format!("loop_match_{}_default", id);
            let arm_labels: Vec<String> = (0..arms.len())
                .map(|i| format!("loop_match_{}_arm_{}", id, i))
                .collect();

            match scr_ty {
                Type::Quad => {
                    for (i, arm) in arms.iter().enumerate() {
                        if i > 0 {
                            out.push(IrInstr::Label {
                                name: format!("loop_match_{}_check_{}", id, i),
                            });
                        }
                        let next_label = if i + 1 < arms.len() {
                            format!("loop_match_{}_check_{}", id, i + 1)
                        } else {
                            default_label.clone()
                        };

                        let lit_reg = alloc(next);
                        out.push(IrInstr::LoadQ {
                            dst: lit_reg,
                            val: expect_quad_match_pattern(&arm.pat)?,
                        });
                        let cmp_reg = alloc(next);
                        out.push(IrInstr::CmpEq {
                            dst: cmp_reg,
                            lhs: scr_reg,
                            rhs: lit_reg,
                        });
                        out.push(IrInstr::JmpIf {
                            cond: cmp_reg,
                            label: arm_labels[i].clone(),
                        });
                        out.push(IrInstr::Jmp {
                            label: next_label.clone(),
                        });

                        out.push(IrInstr::Label {
                            name: arm_labels[i].clone(),
                        });
                        let mut arm_env = env.clone();
                        arm_env.push_scope();
                        lowered_locals.push_scope();
                        if let Some(guard_reg) = lower_match_guard(
                            arm.guard,
                            arena,
                            next,
                            out,
                            &arm_env,
                            loop_stack,
                            fn_table,
                            record_table,
                            adt_table,
                            ret_ty.clone(),
                            closure_state,
                            ownership_events,
                            lowered_locals,
                        )? {
                            let guarded_body_label = format!("loop_match_{}_body_{}", id, i);
                            out.push(IrInstr::JmpIf {
                                cond: guard_reg,
                                label: guarded_body_label.clone(),
                            });
                            out.push(IrInstr::Jmp { label: next_label });
                            out.push(IrInstr::Label {
                                name: guarded_body_label,
                            });
                        }
                        for stmt in &arm.block {
                            lower_loop_expr_stmt(
                                *stmt,
                                arena,
                                next,
                                out,
                                &mut arm_env,
                                loop_stack,
                                fn_table,
                                record_table,
                                adt_table,
                                ret_ty.clone(),
                                closure_state,
                                ownership_events,
                                lowered_locals,
                            )?;
                        }
                        arm_env.pop_scope();
                        lowered_locals.pop_scope();
                        out.push(IrInstr::Jmp {
                            label: end_label.clone(),
                        });
                    }
                }
                Type::I32 | Type::U32 => {
                    for (i, arm) in arms.iter().enumerate() {
                        if i > 0 {
                            out.push(IrInstr::Label {
                                name: format!("loop_match_{}_check_{}", id, i),
                            });
                        }
                        let next_label = if i + 1 < arms.len() {
                            format!("loop_match_{}_check_{}", id, i + 1)
                        } else {
                            default_label.clone()
                        };

                        let lit_reg = alloc(next);
                        match expect_int_match_pattern(&arm.pat, &scr_ty)? {
                            IntMatchLiteral::I32(val) => {
                                out.push(IrInstr::LoadI32 { dst: lit_reg, val })
                            }
                            IntMatchLiteral::U32(val) => {
                                out.push(IrInstr::LoadU32 { dst: lit_reg, val })
                            }
                        }
                        let cmp_reg = alloc(next);
                        out.push(IrInstr::CmpEq {
                            dst: cmp_reg,
                            lhs: scr_reg,
                            rhs: lit_reg,
                        });
                        out.push(IrInstr::JmpIf {
                            cond: cmp_reg,
                            label: arm_labels[i].clone(),
                        });
                        out.push(IrInstr::Jmp {
                            label: next_label.clone(),
                        });

                        out.push(IrInstr::Label {
                            name: arm_labels[i].clone(),
                        });
                        let mut arm_env = env.clone();
                        arm_env.push_scope();
                        lowered_locals.push_scope();
                        if let Some(guard_reg) = lower_match_guard(
                            arm.guard,
                            arena,
                            next,
                            out,
                            &arm_env,
                            loop_stack,
                            fn_table,
                            record_table,
                            adt_table,
                            ret_ty.clone(),
                            closure_state,
                            ownership_events,
                            lowered_locals,
                        )? {
                            let guarded_body_label = format!("loop_match_{}_body_{}", id, i);
                            out.push(IrInstr::JmpIf {
                                cond: guard_reg,
                                label: guarded_body_label.clone(),
                            });
                            out.push(IrInstr::Jmp { label: next_label });
                            out.push(IrInstr::Label {
                                name: guarded_body_label,
                            });
                        }
                        for stmt in &arm.block {
                            lower_loop_expr_stmt(
                                *stmt,
                                arena,
                                next,
                                out,
                                &mut arm_env,
                                loop_stack,
                                fn_table,
                                record_table,
                                adt_table,
                                ret_ty.clone(),
                                closure_state,
                                ownership_events,
                                lowered_locals,
                            )?;
                        }
                        arm_env.pop_scope();
                        lowered_locals.pop_scope();
                        out.push(IrInstr::Jmp {
                            label: end_label.clone(),
                        });
                    }
                }
                Type::Adt(_) | Type::Option(_) | Type::Result(_, _) => {
                    let family = resolve_match_family_for_lowering(&scr_ty, arena, adt_table)?
                        .expect("sum scrutinee family should resolve");
                    let scr_tag_reg = alloc(next);
                    out.push(IrInstr::AdtTag {
                        dst: scr_tag_reg,
                        src: scr_reg,
                        adt_name: family.family_name.clone(),
                    });
                    let resolved_patterns = arms
                        .iter()
                        .map(|arm| {
                            resolve_sum_match_pattern_for_lowering(
                                &arm.pat,
                                &scr_ty,
                                arena,
                                record_table,
                                adt_table,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    for (i, arm) in arms.iter().enumerate() {
                        if i > 0 {
                            out.push(IrInstr::Label {
                                name: format!("loop_match_{}_check_{}", id, i),
                            });
                        }
                        let next_label = if i + 1 < arms.len() {
                            format!("loop_match_{}_check_{}", id, i + 1)
                        } else {
                            default_label.clone()
                        };

                        let expected_tag_reg = alloc(next);
                        out.push(IrInstr::LoadI32 {
                            dst: expected_tag_reg,
                            val: resolved_patterns[i].tag,
                        });
                        let cmp_reg = alloc(next);
                        out.push(IrInstr::CmpEq {
                            dst: cmp_reg,
                            lhs: scr_tag_reg,
                            rhs: expected_tag_reg,
                        });
                        out.push(IrInstr::JmpIf {
                            cond: cmp_reg,
                            label: arm_labels[i].clone(),
                        });
                        out.push(IrInstr::Jmp {
                            label: next_label.clone(),
                        });

                        out.push(IrInstr::Label {
                            name: arm_labels[i].clone(),
                        });
                        let mut arm_env = env.clone();
                        arm_env.push_scope();
                        lowered_locals.push_scope();
                        lower_adt_match_bindings(
                            &resolved_patterns[i],
                            scr_reg,
                            next,
                            out,
                            &mut arm_env,
                            arena,
                            lowered_locals,
                        )?;
                        if let Some(guard_reg) = lower_match_guard(
                            arm.guard,
                            arena,
                            next,
                            out,
                            &arm_env,
                            loop_stack,
                            fn_table,
                            record_table,
                            adt_table,
                            ret_ty.clone(),
                            closure_state,
                            ownership_events,
                            lowered_locals,
                        )? {
                            let guarded_body_label = format!("loop_match_{}_body_{}", id, i);
                            out.push(IrInstr::JmpIf {
                                cond: guard_reg,
                                label: guarded_body_label.clone(),
                            });
                            out.push(IrInstr::Jmp { label: next_label });
                            out.push(IrInstr::Label {
                                name: guarded_body_label,
                            });
                        }
                        for stmt in &arm.block {
                            lower_loop_expr_stmt(
                                *stmt,
                                arena,
                                next,
                                out,
                                &mut arm_env,
                                loop_stack,
                                fn_table,
                                record_table,
                                adt_table,
                                ret_ty.clone(),
                                closure_state,
                                ownership_events,
                                lowered_locals,
                            )?;
                        }
                        arm_env.pop_scope();
                        lowered_locals.pop_scope();
                        out.push(IrInstr::Jmp {
                            label: end_label.clone(),
                        });
                    }
                }
                _ => unreachable!("non-matchable scrutinee handled above"),
            }

            out.push(IrInstr::Label {
                name: default_label,
            });
            if exhaustive_without_default {
                let cond = alloc(next);
                out.push(IrInstr::LoadBool {
                    dst: cond,
                    val: false,
                });
                out.push(IrInstr::Assert { cond });
            } else {
                let default = default
                    .as_ref()
                    .expect("non-exhaustive match statement requires explicit default in lowering");
                let mut def_env = env.clone();
                def_env.push_scope();
                lowered_locals.push_scope();
                for stmt in default {
                    lower_loop_expr_stmt(
                        *stmt,
                        arena,
                        next,
                        out,
                        &mut def_env,
                        loop_stack,
                        fn_table,
                        record_table,
                        adt_table,
                        ret_ty.clone(),
                        closure_state,
                        ownership_events,
                        lowered_locals,
                    )?;
                }
                def_env.pop_scope();
                lowered_locals.pop_scope();
                out.push(IrInstr::Jmp {
                    label: end_label.clone(),
                });
            }
            out.push(IrInstr::Label { name: end_label });
            Ok(())
        }
        _ => {
            // #1709 (FA-04-003): `ownership_events` is moved into `ctx` (not
            // reset to a fresh `Vec::new()`) and moved back out below, on the
            // same `core::mem::take`/restore pattern already used here for
            // `closure_state` - a statement lowered through this fallback can
            // itself generate real ownership events (e.g. a tuple/record
            // destructuring bind), and this is the one function-owned sink
            // for the enclosing loop expression; a fresh temporary here would
            // silently discard them.
            let mut ctx = LoweringCtx {
                next_reg: *next,
                next_label_id: out.len() as u32,
                loop_stack: loop_stack.clone(),
                closure_state: core::mem::take(closure_state),
                ensures: Vec::new(),
                ensures_result_symbol: None,
                invariants: Vec::new(),
                invariants_result_symbol: None,
                instrs: core::mem::take(out),
                ownership_events: core::mem::take(ownership_events),
                impls: Vec::new(),
                // #1724 (FA-04-018): same reasoning and pattern as
                // `ownership_events` immediately above - a statement
                // lowered through this fallback can introduce or resolve
                // real lexical bindings (e.g. `let x = ...;`), and this is
                // the one function-owned scope stack for the enclosing
                // loop expression; a fresh temporary here would silently
                // lose every binding introduced inside the fallback,
                // making later sibling statements in the same loop body
                // unable to resolve them.
                lowered_locals: core::mem::take(lowered_locals),
            };
            let result = lower_stmt(
                stmt_id,
                arena,
                &mut ctx,
                env,
                ret_ty,
                fn_table,
                record_table,
                adt_table,
            );
            *next = ctx.next_reg;
            *out = ctx.instrs;
            *loop_stack = ctx.loop_stack;
            *closure_state = ctx.closure_state;
            *ownership_events = ctx.ownership_events;
            *lowered_locals = ctx.lowered_locals;
            result
        }
    }
}

fn lower_match_expr(
    match_expr: &MatchExpr,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<Type>,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(u16, Type), FrontendError> {
    let (scr_reg, scr_ty) = lower_expr(
        match_expr.scrutinee,
        arena,
        next,
        out,
        env,
        loop_stack,
        fn_table,
        record_table,
        adt_table,
        ret_ty.clone(),
        closure_state,
        ownership_events,
        lowered_locals,
    )?;
    if !matches!(
        scr_ty,
        Type::Quad | Type::Adt(_) | Type::Option(_) | Type::Result(_, _) | Type::I32 | Type::U32
    ) {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "match expression scrutinee must be quad, enum, Option(T), Result(T, E), i32, or u32. Got: {:?}",
                scr_ty
            ),
        });
    }
    let exhaustive_without_default = if match_expr.default.is_none() {
        match missing_exhaustive_sum_variants(
            &scr_ty,
            match_expr.arms.iter().map(|arm| (&arm.pat, arm.guard)),
            arena,
            adt_table,
        )? {
            Some((family_label, missing)) if !missing.is_empty() => {
                return Err(non_exhaustive_match_error(&family_label, &missing, true)?)
            }
            Some(_) => true,
            None => {
                return Err(FrontendError {
                    pos: 0,
                    message: "match expression requires default arm '_'".to_string(),
                });
            }
        }
    } else {
        false
    };

    let id = alloc_match_expr_id(next);
    let end_label = format!("match_expr_{}_end", id);
    let default_label = format!("match_expr_{}_default", id);
    let arm_labels: Vec<String> = (0..match_expr.arms.len())
        .map(|i| format!("match_expr_{}_arm_{}", id, i))
        .collect();
    let result_name = format!("__match_expr_{}_result", id);

    let mut result_ty = None;
    match scr_ty {
        Type::Quad => {
            for (i, arm) in match_expr.arms.iter().enumerate() {
                if i > 0 {
                    out.push(IrInstr::Label {
                        name: format!("match_expr_{}_check_{}", id, i),
                    });
                }
                let next_label = if i + 1 < match_expr.arms.len() {
                    format!("match_expr_{}_check_{}", id, i + 1)
                } else {
                    default_label.clone()
                };

                let lit_reg = alloc(next);
                out.push(IrInstr::LoadQ {
                    dst: lit_reg,
                    val: expect_quad_match_pattern(&arm.pat)?,
                });
                let cmp_reg = alloc(next);
                out.push(IrInstr::CmpEq {
                    dst: cmp_reg,
                    lhs: scr_reg,
                    rhs: lit_reg,
                });
                out.push(IrInstr::JmpIf {
                    cond: cmp_reg,
                    label: arm_labels[i].clone(),
                });
                out.push(IrInstr::Jmp {
                    label: next_label.clone(),
                });

                out.push(IrInstr::Label {
                    name: arm_labels[i].clone(),
                });
                let mut arm_env = env.clone();
                arm_env.push_scope();
                lowered_locals.push_scope();
                if let Some(guard_reg) = lower_match_guard(
                    arm.guard,
                    arena,
                    next,
                    out,
                    &arm_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )? {
                    let guarded_body_label = format!("match_expr_{}_body_{}", id, i);
                    out.push(IrInstr::JmpIf {
                        cond: guard_reg,
                        label: guarded_body_label.clone(),
                    });
                    out.push(IrInstr::Jmp { label: next_label });
                    out.push(IrInstr::Label {
                        name: guarded_body_label,
                    });
                }
                let (arm_reg, arm_ty) = lower_value_block_expr(
                    &arm.block,
                    arena,
                    next,
                    out,
                    &arm_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    expected.clone(),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                arm_env.pop_scope();
                lowered_locals.pop_scope();
                if let Some(ref expected_ty) = result_ty {
                    if *expected_ty != arm_ty {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "match expression branch type mismatch in lowering: expected {:?}, got {:?}",
                                expected_ty, arm_ty
                            ),
                        });
                    }
                } else {
                    result_ty = Some(arm_ty);
                }
                out.push(IrInstr::StoreVar {
                    name: result_name.clone(),
                    src: arm_reg,
                });
                out.push(IrInstr::Jmp {
                    label: end_label.clone(),
                });
            }
        }
        Type::I32 | Type::U32 => {
            for (i, arm) in match_expr.arms.iter().enumerate() {
                if i > 0 {
                    out.push(IrInstr::Label {
                        name: format!("match_expr_{}_check_{}", id, i),
                    });
                }
                let next_label = if i + 1 < match_expr.arms.len() {
                    format!("match_expr_{}_check_{}", id, i + 1)
                } else {
                    default_label.clone()
                };

                let lit_reg = alloc(next);
                match expect_int_match_pattern(&arm.pat, &scr_ty)? {
                    IntMatchLiteral::I32(val) => out.push(IrInstr::LoadI32 { dst: lit_reg, val }),
                    IntMatchLiteral::U32(val) => out.push(IrInstr::LoadU32 { dst: lit_reg, val }),
                }
                let cmp_reg = alloc(next);
                out.push(IrInstr::CmpEq {
                    dst: cmp_reg,
                    lhs: scr_reg,
                    rhs: lit_reg,
                });
                out.push(IrInstr::JmpIf {
                    cond: cmp_reg,
                    label: arm_labels[i].clone(),
                });
                out.push(IrInstr::Jmp {
                    label: next_label.clone(),
                });

                out.push(IrInstr::Label {
                    name: arm_labels[i].clone(),
                });
                let mut arm_env = env.clone();
                arm_env.push_scope();
                lowered_locals.push_scope();
                if let Some(guard_reg) = lower_match_guard(
                    arm.guard,
                    arena,
                    next,
                    out,
                    &arm_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )? {
                    let guarded_body_label = format!("match_expr_{}_body_{}", id, i);
                    out.push(IrInstr::JmpIf {
                        cond: guard_reg,
                        label: guarded_body_label.clone(),
                    });
                    out.push(IrInstr::Jmp { label: next_label });
                    out.push(IrInstr::Label {
                        name: guarded_body_label,
                    });
                }
                let (arm_reg, arm_ty) = lower_value_block_expr(
                    &arm.block,
                    arena,
                    next,
                    out,
                    &arm_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    expected.clone(),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                arm_env.pop_scope();
                lowered_locals.pop_scope();
                if let Some(ref expected_ty) = result_ty {
                    if *expected_ty != arm_ty {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "match expression branch type mismatch in lowering: expected {:?}, got {:?}",
                                expected_ty, arm_ty
                            ),
                        });
                    }
                } else {
                    result_ty = Some(arm_ty);
                }
                out.push(IrInstr::StoreVar {
                    name: result_name.clone(),
                    src: arm_reg,
                });
                out.push(IrInstr::Jmp {
                    label: end_label.clone(),
                });
            }
        }
        Type::Adt(_) | Type::Option(_) | Type::Result(_, _) => {
            let family = resolve_match_family_for_lowering(&scr_ty, arena, adt_table)?
                .expect("sum scrutinee family should resolve");
            let scr_tag_reg = alloc(next);
            out.push(IrInstr::AdtTag {
                dst: scr_tag_reg,
                src: scr_reg,
                adt_name: family.family_name.clone(),
            });
            let resolved_patterns = match_expr
                .arms
                .iter()
                .map(|arm| {
                    resolve_sum_match_pattern_for_lowering(
                        &arm.pat,
                        &scr_ty,
                        arena,
                        record_table,
                        adt_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            for (i, arm) in match_expr.arms.iter().enumerate() {
                if i > 0 {
                    out.push(IrInstr::Label {
                        name: format!("match_expr_{}_check_{}", id, i),
                    });
                }
                let next_label = if i + 1 < match_expr.arms.len() {
                    format!("match_expr_{}_check_{}", id, i + 1)
                } else {
                    default_label.clone()
                };

                let expected_tag_reg = alloc(next);
                out.push(IrInstr::LoadI32 {
                    dst: expected_tag_reg,
                    val: resolved_patterns[i].tag,
                });
                let cmp_reg = alloc(next);
                out.push(IrInstr::CmpEq {
                    dst: cmp_reg,
                    lhs: scr_tag_reg,
                    rhs: expected_tag_reg,
                });
                out.push(IrInstr::JmpIf {
                    cond: cmp_reg,
                    label: arm_labels[i].clone(),
                });
                out.push(IrInstr::Jmp {
                    label: next_label.clone(),
                });

                out.push(IrInstr::Label {
                    name: arm_labels[i].clone(),
                });
                let mut arm_env = env.clone();
                arm_env.push_scope();
                lowered_locals.push_scope();
                lower_adt_match_bindings(
                    &resolved_patterns[i],
                    scr_reg,
                    next,
                    out,
                    &mut arm_env,
                    arena,
                    lowered_locals,
                )?;
                if let Some(guard_reg) = lower_match_guard(
                    arm.guard,
                    arena,
                    next,
                    out,
                    &arm_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )? {
                    let guarded_body_label = format!("match_expr_{}_body_{}", id, i);
                    out.push(IrInstr::JmpIf {
                        cond: guard_reg,
                        label: guarded_body_label.clone(),
                    });
                    out.push(IrInstr::Jmp { label: next_label });
                    out.push(IrInstr::Label {
                        name: guarded_body_label,
                    });
                }
                let (arm_reg, arm_ty) = lower_value_block_expr(
                    &arm.block,
                    arena,
                    next,
                    out,
                    &arm_env,
                    loop_stack,
                    fn_table,
                    record_table,
                    adt_table,
                    expected.clone(),
                    ret_ty.clone(),
                    closure_state,
                    ownership_events,
                    lowered_locals,
                )?;
                arm_env.pop_scope();
                lowered_locals.pop_scope();
                if let Some(ref expected_ty) = result_ty {
                    if *expected_ty != arm_ty {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "match expression branch type mismatch in lowering: expected {:?}, got {:?}",
                                expected_ty, arm_ty
                            ),
                        });
                    }
                } else {
                    result_ty = Some(arm_ty);
                }
                out.push(IrInstr::StoreVar {
                    name: result_name.clone(),
                    src: arm_reg,
                });
                out.push(IrInstr::Jmp {
                    label: end_label.clone(),
                });
            }
        }
        _ => unreachable!("non-matchable scrutinee handled above"),
    }

    if exhaustive_without_default {
        lower_impossible_match_trap(default_label, next, out);
    } else {
        let default = match_expr
            .default
            .as_ref()
            .expect("non-exhaustive match expression requires explicit default in lowering");
        out.push(IrInstr::Label {
            name: default_label,
        });
        let (default_reg, default_ty) = lower_value_block_expr(
            default,
            arena,
            next,
            out,
            env,
            loop_stack,
            fn_table,
            record_table,
            adt_table,
            expected,
            ret_ty,
            closure_state,
            ownership_events,
            lowered_locals,
        )?;
        if let Some(ref expected_ty) = result_ty {
            if *expected_ty != default_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "match expression branch type mismatch in lowering: expected {:?}, got {:?}",
                        expected_ty, default_ty
                    ),
                });
            }
        } else {
            result_ty = Some(default_ty);
        }
        out.push(IrInstr::StoreVar {
            name: result_name.clone(),
            src: default_reg,
        });
        out.push(IrInstr::Jmp {
            label: end_label.clone(),
        });
    }

    out.push(IrInstr::Label { name: end_label });
    let dst = alloc(next);
    out.push(IrInstr::LoadVar {
        dst,
        name: result_name,
    });
    Ok((
        dst,
        result_ty.expect("match expression lowering must establish a result type"),
    ))
}

fn lower_expr_stmt(
    expr_id: ExprId,
    arena: &AstArena,
    ctx: &mut LoweringCtx,
    env: &ScopeEnv,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
) -> Result<(), FrontendError> {
    lower_expr_stmt_with_parts(
        expr_id,
        arena,
        &mut ctx.next_reg,
        &mut ctx.instrs,
        env,
        &mut ctx.loop_stack,
        fn_table,
        record_table,
        adt_table,
        ret_ty,
        &mut ctx.closure_state,
        &mut ctx.ownership_events,
        &mut ctx.lowered_locals,
    )
}

fn alloc_if_expr_id(next: &mut u16) -> u16 {
    let id = *next;
    *next += 1;
    id
}

fn alloc_match_expr_id(next: &mut u16) -> u16 {
    let id = *next;
    *next += 1;
    id
}

fn alloc_loop_expr_id(next: &mut u16) -> u16 {
    let id = *next;
    *next += 1;
    id
}

fn lower_expr_stmt_with_parts(
    expr_id: ExprId,
    arena: &AstArena,
    next: &mut u16,
    out: &mut Vec<IrInstr>,
    env: &ScopeEnv,
    loop_stack: &mut Vec<LoopLoweringFrame>,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    closure_state: &mut ClosureLoweringState,
    ownership_events: &mut Vec<OwnershipPathEvent>,
    lowered_locals: &mut LoweredLocalEnv,
) -> Result<(), FrontendError> {
    let expr = arena.expr(expr_id);
    if let Expr::Call(name, args) = expr {
        if is_builtin_assert_name(*name, arena, fn_table)? {
            if args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "assert builtin takes exactly one positional argument".to_string(),
                });
            }
            if args.len() != 1 {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("assert builtin expects 1 arg, got {}", args.len()),
                });
            }
            let (cond, cond_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::Bool),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("assert builtin requires bool condition, got {:?}", cond_ty),
                });
            }
            out.push(IrInstr::Assert { cond });
            return Ok(());
        }
        // builtin len(sequence) — allowed as statement (result discarded)
        if resolve_symbol_name(arena, *name)? == "len" {
            if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "builtin 'len' takes exactly one positional argument".to_string(),
                });
            }
            let (src, arg_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                None,
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            return match &arg_ty {
                Type::Sequence(_) => {
                    let dst = alloc(next);
                    out.push(IrInstr::SequenceLen { dst, src });
                    Ok(())
                }
                _ => Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin 'len' expects a Sequence argument, got {:?}",
                        arg_ty
                    ),
                }),
            };
        }
        // builtin is_empty(sequence) — allowed as statement (result discarded)
        if resolve_symbol_name(arena, *name)? == "is_empty" {
            if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "builtin 'is_empty' takes exactly one positional argument".to_string(),
                });
            }
            let (src, arg_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                None,
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            return match &arg_ty {
                Type::Sequence(_) => {
                    let dst = alloc(next);
                    out.push(IrInstr::SequenceIsEmpty { dst, src });
                    Ok(())
                }
                _ => Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin 'is_empty' expects a Sequence argument, got {:?}",
                        arg_ty
                    ),
                }),
            };
        }
        // builtin push / prepend — allowed as statement (result discarded, but should be assigned)
        let name_str_stmt = resolve_symbol_name(arena, *name)?;
        if name_str_stmt == "push" || name_str_stmt == "prepend" {
            if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin '{name_str_stmt}' takes exactly two positional arguments"
                    ),
                });
            }
            let (seq, seq_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                None,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let Type::Sequence(seq_type) = &seq_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin '{name_str_stmt}' first argument must be a Sequence, got {:?}",
                        seq_ty
                    ),
                });
            };
            let elem_ty = seq_type.item.as_ref().clone();
            let (val, val_ty) = lower_expr_with_expected(
                args[1].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(elem_ty.clone()),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if val_ty != elem_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin '{name_str_stmt}' second argument type {:?} does not match \
                         sequence element type {:?}",
                        val_ty, elem_ty
                    ),
                });
            }
            let dst = alloc(next);
            if name_str_stmt == "push" {
                out.push(IrInstr::SequencePush { dst, seq, val });
            } else {
                out.push(IrInstr::SequencePrepend { dst, seq, val });
            }
            return Ok(());
        }
        // builtin contains(sequence, value) — allowed as statement (result discarded)
        if resolve_symbol_name(arena, *name)? == "contains" {
            if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "builtin 'contains' takes exactly two positional arguments"
                        .to_string(),
                });
            }
            let (seq, seq_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                None,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let Type::Sequence(seq_type) = &seq_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin 'contains' first argument must be a Sequence, got {:?}",
                        seq_ty
                    ),
                });
            };
            let elem_ty = seq_type.item.as_ref().clone();
            let (val, val_ty) = lower_expr_with_expected(
                args[1].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(elem_ty.clone()),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if val_ty != elem_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin 'contains' value type {:?} does not match element type {:?}",
                        val_ty, elem_ty
                    ),
                });
            }
            let dst = alloc(next);
            out.push(IrInstr::SequenceContains { dst, seq, val });
            return Ok(());
        }
        // builtin pop(sequence) — allowed as statement (result discarded)
        if resolve_symbol_name(arena, *name)? == "pop" {
            if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "builtin 'pop' takes exactly one positional argument".to_string(),
                });
            }
            let (src, arg_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                None,
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            return match &arg_ty {
                Type::Sequence(_) => {
                    let dst = alloc(next);
                    out.push(IrInstr::SequencePop { dst, src });
                    Ok(())
                }
                _ => Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin 'pop' expects a Sequence argument, got {:?}",
                        arg_ty
                    ),
                }),
            };
        }
        // builtin map_empty() as statement — rejected; result must be bound to a Map variable
        if resolve_symbol_name(arena, *name)? == "map_empty" {
            return Err(FrontendError {
                pos: 0,
                message: "map_empty() requires a contextual Map(K, V) type and cannot be \
                          used as a statement; use 'let q: Map(K, V) = map_empty()'"
                    .to_string(),
            });
        }
        // builtin map_contains(Map(K,V), K) as statement
        if resolve_symbol_name(arena, *name)? == "map_contains" {
            if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "builtin 'map_contains' takes exactly two positional arguments"
                        .to_string(),
                });
            }
            let (map_reg, map_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                None,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let Type::Map(ref map_type) = map_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin 'map_contains' first argument must be Map, got {:?}",
                        map_ty
                    ),
                });
            };
            let key_ty = map_type.key.as_ref().clone();
            let (key_reg, _) = lower_expr_with_expected(
                args[1].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(key_ty),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let dst = alloc(next);
            out.push(IrInstr::MapContains {
                dst,
                map: map_reg,
                key: key_reg,
            });
            return Ok(());
        }
        // builtin map_get(Map(K,V), K, V) as statement
        if resolve_symbol_name(arena, *name)? == "map_get" {
            if args.len() != 3 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "builtin 'map_get' takes exactly three positional arguments"
                        .to_string(),
                });
            }
            let (map_reg, map_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                None,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let Type::Map(ref map_type) = map_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin 'map_get' first argument must be Map, got {:?}",
                        map_ty
                    ),
                });
            };
            let key_ty = map_type.key.as_ref().clone();
            let val_ty = map_type.val.as_ref().clone();
            let (key_reg, _) = lower_expr_with_expected(
                args[1].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(key_ty),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let (default_reg, _) = lower_expr_with_expected(
                args[2].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(val_ty),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let dst = alloc(next);
            out.push(IrInstr::MapGet {
                dst,
                map: map_reg,
                key: key_reg,
                default_val: default_reg,
            });
            return Ok(());
        }
        // builtin map_set(Map(K,V), K, V) as statement
        if resolve_symbol_name(arena, *name)? == "map_set" {
            if args.len() != 3 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "builtin 'map_set' takes exactly three positional arguments"
                        .to_string(),
                });
            }
            let (map_reg, map_ty) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                None,
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let Type::Map(ref map_type) = map_ty else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "builtin 'map_set' first argument must be Map, got {:?}",
                        map_ty
                    ),
                });
            };
            let key_ty = map_type.key.as_ref().clone();
            let val_ty = map_type.val.as_ref().clone();
            let (key_reg, _) = lower_expr_with_expected(
                args[1].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(key_ty),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let (val_reg, _) = lower_expr_with_expected(
                args[2].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(val_ty),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let dst = alloc(next);
            out.push(IrInstr::MapSet {
                dst,
                map: map_reg,
                key: key_reg,
                val: val_reg,
            });
            return Ok(());
        }
        // builtin print(msg: text) as statement
        if resolve_symbol_name(arena, *name)? == "print" {
            if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message: "builtin 'print' takes exactly one positional argument (msg: text)"
                        .to_string(),
                });
            }
            let (arg_reg, _) = lower_expr(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            out.push(IrInstr::Call {
                dst: None,
                name: "print".to_string(),
                args: vec![arg_reg],
            });
            return Ok(());
        }
        // builtin random_seed(seed: i32) as statement — valid (discards Unit result)
        if resolve_symbol_name(arena, *name)? == "random_seed" {
            if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "builtin 'random_seed' takes exactly one positional argument (seed: i32)"
                            .to_string(),
                });
            }
            let (seed_reg, _) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::I32),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let dst = alloc(next);
            out.push(IrInstr::RngSeed {
                dst,
                seed: seed_reg,
            });
            return Ok(());
        }
        // builtin random_next_i32(lo, hi) as statement — valid (discards i32 result)
        if resolve_symbol_name(arena, *name)? == "random_next_i32" {
            if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "builtin 'random_next_i32' takes exactly two positional arguments (lo: i32, hi: i32)"
                            .to_string(),
                });
            }
            let (lo_reg, _) = lower_expr_with_expected(
                args[0].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::I32),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let (hi_reg, _) = lower_expr_with_expected(
                args[1].value,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(Type::I32),
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            let dst = alloc(next);
            out.push(IrInstr::RngNextI32 {
                dst,
                lo: lo_reg,
                hi: hi_reg,
            });
            return Ok(());
        }
        let sig = if let Some(s) = fn_table.get(name) {
            s.clone()
        } else if let Some(s) = builtin_sig(resolve_symbol_name(arena, *name)?) {
            s
        } else if let Some(Type::Closure(closure_ty)) = env.get(*name) {
            return lower_direct_closure_call_stmt(
                *name,
                args,
                &closure_ty,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                ret_ty,
                closure_state,
                ownership_events,
                lowered_locals,
            );
        } else {
            return Err(FrontendError {
                pos: 0,
                message: format!("unknown function '{}'", resolve_symbol_name(arena, *name)?),
            });
        };
        let ordered_args = reorder_call_args(*name, args, &sig, arena)?;
        // See the call-expression path above (FA-04-016 / #1722): evaluate in
        // source order, assign into declared parameter slots.
        let mut regs: Vec<Option<u16>> = vec![None; ordered_args.slots.len()];
        for &slot in &ordered_args.eval_order {
            let arg = ordered_args.slots[slot];
            let (r, t) = lower_expr_with_expected(
                arg,
                arena,
                next,
                out,
                env,
                loop_stack,
                fn_table,
                record_table,
                adt_table,
                Some(sig.params[slot].clone()),
                ret_ty.clone(),
                closure_state,
                ownership_events,
                lowered_locals,
            )?;
            if t != sig.params[slot] {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "arg {} for '{}' type mismatch",
                        slot,
                        resolve_symbol_name(arena, *name)?
                    ),
                });
            }
            regs[slot] = Some(r);
        }
        let regs: Vec<u16> = regs.into_iter().flatten().collect();
        let dst = if sig.ret == Type::Unit {
            None
        } else {
            Some(alloc(next))
        };
        out.push(IrInstr::Call {
            dst,
            name: resolve_symbol_name(arena, *name)?.to_string(),
            args: regs,
        });
        return Ok(());
    }

    let _ = lower_expr(
        expr_id,
        arena,
        next,
        out,
        env,
        loop_stack,
        fn_table,
        record_table,
        adt_table,
        ret_ty,
        closure_state,
        ownership_events,
        lowered_locals,
    )?;
    Ok(())
}

/// #1724 (FA-04-018): the canonical authority mapping a lexical source
/// binding to its lowered runtime-local key. `SymbolId` is a pure
/// spelling-interned identity (`AstArena::intern_symbol` is a
/// `BTreeMap<String, SymbolId>` keyed only by text - confirmed by direct
/// inspection and empirically: `let x = 1; if true { let x = 2; ... }`
/// gives the outer declaration, the inner declaration, and every use the
/// exact same `SymbolId`) - it is not declaration-unique, so it cannot be
/// used directly as a runtime-local key, and neither can
/// `format!("__local_{}", symbol_id.0)`, since two shadowed declarations
/// can share it. Only this scope stack (deliberately mirroring
/// `ScopeEnv`'s own scope push/pop shape) distinguishes them. Knows as
/// little as possible: scope, source symbol, lowered local key - it
/// duplicates none of `ScopeEnv`'s type/mutability/constness/ownership
/// state. One `LoweredLocalEnv` per `IrFunction`: same-function nested
/// lowering shares it; a lifted closure child gets its own fresh instance,
/// the same as `ownership_events` (#1709) and `local_next`/
/// `local_loop_stack` already do at that exact boundary.
#[derive(Debug, Default)]
struct LoweredLocalEnv {
    scopes: Vec<BTreeMap<SymbolId, String>>,
    next_id: u32,
}

impl LoweredLocalEnv {
    fn new() -> Self {
        Self {
            scopes: vec![BTreeMap::new()],
            next_id: 0,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Introduces a fresh lexical binding for `symbol` in the current
    /// (innermost) scope, allocating a new, deterministic,
    /// source-unreachable runtime-local key (`is_ascii_alphabetic()` gates
    /// every admitted identifier's first character - confirmed by direct
    /// lexer inspection - so no admitted source identifier can ever begin
    /// with `_`, making the `__sm_local_` prefix collision-safe). Must be
    /// called exactly once per lexical declaration (`let`/`const`/
    /// parameter/pattern binding/loop variable) - never for a use or an
    /// assignment to an already-existing binding.
    fn bind(&mut self, arena: &AstArena, symbol: SymbolId) -> Result<String, FrontendError> {
        let spelling = resolve_symbol_name(arena, symbol)?;
        let id = self.next_id;
        self.next_id += 1;
        let key = format!("__sm_local_{}_{}", id, spelling);
        self.scopes
            .last_mut()
            .expect("LoweredLocalEnv always has at least one scope")
            .insert(symbol, key.clone());
        Ok(key)
    }

    /// Resolves a use (read) or an assignment target (an existing binding,
    /// not a new declaration) to the runtime-local key of the exact
    /// binding currently selected by lexical scope - the innermost scope
    /// that has one. Fails closed: no fallback to the raw source spelling
    /// on a missing mapping (#1724 §16) - that would reintroduce exactly
    /// the identity collapse this authority exists to repair.
    fn resolve(&self, arena: &AstArena, symbol: SymbolId) -> Result<String, FrontendError> {
        for scope in self.scopes.iter().rev() {
            if let Some(key) = scope.get(&symbol) {
                return Ok(key.clone());
            }
        }
        Err(FrontendError {
            pos: 0,
            message: format!(
                "cannot resolve lexical binding '{}' during lowering",
                resolve_symbol_name(arena, symbol)?
            ),
        })
    }

    /// Seeds a binding using the *exact* source spelling as its own
    /// lowered key, bypassing the mangled/counter-based scheme `bind`
    /// uses. Reserved for `lower_expr_to_ir`'s pre-populated `var_types`
    /// map: those symbols name pre-existing VM frame locals supplied by
    /// the caller under their raw source spelling, not a fresh lexical
    /// declaration this function itself lowers - there is no
    /// corresponding `StoreVar` to mangle, and no possibility of
    /// shadowing in a flat, statement-free `HashMap<SymbolId, Type>`.
    /// Must not be used anywhere lexical scoping/shadowing applies.
    fn bind_raw(&mut self, arena: &AstArena, symbol: SymbolId) -> Result<(), FrontendError> {
        let spelling = resolve_symbol_name(arena, symbol)?.to_string();
        self.scopes
            .last_mut()
            .expect("LoweredLocalEnv always has at least one scope")
            .insert(symbol, spelling);
        Ok(())
    }
}

#[derive(Debug, Default)]
struct ClosureLoweringState {
    parent_fn_name: String,
    next_closure_id: u32,
    lifted_funcs: Vec<IrFunction>,
}

#[derive(Debug, Default)]
struct LoweringCtx {
    next_reg: u16,
    next_label_id: u32,
    loop_stack: Vec<LoopLoweringFrame>,
    closure_state: ClosureLoweringState,
    ensures: Vec<ExprId>,
    ensures_result_symbol: Option<SymbolId>,
    invariants: Vec<ExprId>,
    invariants_result_symbol: Option<SymbolId>,
    instrs: Vec<IrInstr>,
    ownership_events: Vec<OwnershipPathEvent>,
    impls: Vec<sm_front::ImplDecl>,
    lowered_locals: LoweredLocalEnv,
}

#[derive(Debug, Clone)]
struct LoopLoweringFrame {
    kind: LoopLoweringFrameKind,
    end_label: String,
    continue_label: String,
    result_name: String,
    result_ty: Option<Type>,
    expected_ty: Option<Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopLoweringFrameKind {
    Expression,
    Control,
}

impl LoweringCtx {
    fn new(
        parent_fn_name: String,
        ensures: Vec<ExprId>,
        ensures_result_symbol: Option<SymbolId>,
        invariants: Vec<ExprId>,
        invariants_result_symbol: Option<SymbolId>,
        impl_list: &[sm_front::ImplDecl],
    ) -> Self {
        Self {
            next_reg: 0,
            next_label_id: 0,
            loop_stack: Vec::new(),
            closure_state: ClosureLoweringState {
                parent_fn_name,
                next_closure_id: 0,
                lifted_funcs: Vec::new(),
            },
            ensures,
            ensures_result_symbol,
            invariants,
            invariants_result_symbol,
            instrs: Vec::new(),
            ownership_events: Vec::new(),
            impls: impl_list.to_vec(),
            lowered_locals: LoweredLocalEnv::new(),
        }
    }

    fn next_if_id(&mut self) -> u32 {
        let id = self.next_label_id;
        self.next_label_id += 1;
        id
    }

    fn ends_with_ret(&self) -> bool {
        matches!(self.instrs.last(), Some(IrInstr::Ret { .. }))
    }
}

fn find_contract_result_symbol(
    contract_ensures: &[ExprId],
    arena: &AstArena,
) -> Result<Option<SymbolId>, FrontendError> {
    for condition in contract_ensures {
        if let Some(symbol) = find_named_var_symbol(*condition, arena, "result")? {
            return Ok(Some(symbol));
        }
    }
    Ok(None)
}

fn contract_clause_references_result(
    expr_id: ExprId,
    arena: &AstArena,
) -> Result<bool, FrontendError> {
    Ok(find_named_var_symbol(expr_id, arena, "result")?.is_some())
}

fn find_named_var_symbol(
    expr_id: ExprId,
    arena: &AstArena,
    name: &str,
) -> Result<Option<SymbolId>, FrontendError> {
    match arena.expr(expr_id) {
        Expr::Var(symbol_id) => {
            if resolve_symbol_name(arena, *symbol_id)? == name {
                Ok(Some(*symbol_id))
            } else {
                Ok(None)
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                if let Some(symbol) = find_named_var_symbol(*item, arena, name)? {
                    return Ok(Some(symbol));
                }
            }
            Ok(None)
        }
        Expr::RecordField(field_expr) => find_named_var_symbol(field_expr.base, arena, name),
        Expr::Unary(_, inner) => find_named_var_symbol(*inner, arena, name),
        Expr::Binary(lhs, _, rhs) => {
            if let Some(symbol) = find_named_var_symbol(*lhs, arena, name)? {
                return Ok(Some(symbol));
            }
            find_named_var_symbol(*rhs, arena, name)
        }
        _ => Ok(None),
    }
}

fn sequence_access_path_from_expr(
    expr_id: ExprId,
    arena: &AstArena,
) -> Option<SequenceOwnershipPath> {
    match arena.expr(expr_id) {
        Expr::Var(name) => Some(SequenceOwnershipPath::Exact(AccessPath::new(*name))),
        Expr::SequenceIndex(index_expr) => {
            let base = sequence_access_path_from_expr(index_expr.base, arena)?;
            let base_path = base.as_path().clone();
            if base.is_dynamic_fallback() {
                return Some(SequenceOwnershipPath::DynamicFallback(base_path));
            }
            let Expr::NumericLiteral(NumericLiteral::I32(index)) = arena.expr(index_expr.index)
            else {
                return Some(SequenceOwnershipPath::DynamicFallback(base_path));
            };
            if *index < 0 {
                return Some(SequenceOwnershipPath::DynamicFallback(base_path));
            }
            let index = u32::try_from(*index).ok()?;
            Some(SequenceOwnershipPath::Exact(
                base_path.sequence_index_static(index),
            ))
        }
        _ => None,
    }
}

fn direct_record_access_path_from_expr(expr_id: ExprId, arena: &AstArena) -> Option<AccessPath> {
    match arena.expr(expr_id) {
        Expr::Var(name) => Some(AccessPath::new(*name)),
        _ => None,
    }
}

fn append_record_update_write_events_from_expr(
    expr_id: ExprId,
    arena: &AstArena,
    ownership_events: &mut Vec<OwnershipPathEvent>,
) {
    match arena.expr(expr_id) {
        Expr::Tuple(items) => {
            for item in items {
                append_record_update_write_events_from_expr(*item, arena, ownership_events);
            }
        }
        Expr::RecordLiteral(record_literal) => {
            for field in &record_literal.fields {
                append_record_update_write_events_from_expr(field.value, arena, ownership_events);
            }
        }
        Expr::RecordField(field_expr) => {
            append_record_update_write_events_from_expr(field_expr.base, arena, ownership_events);
        }
        Expr::SequenceLiteral(sequence) => {
            for item in &sequence.items {
                append_record_update_write_events_from_expr(*item, arena, ownership_events);
            }
        }
        Expr::SequenceIndex(index_expr) => {
            append_record_update_write_events_from_expr(index_expr.base, arena, ownership_events);
            append_record_update_write_events_from_expr(index_expr.index, arena, ownership_events);
        }
        Expr::RecordUpdate(update_expr) => {
            append_record_update_write_events_from_expr(update_expr.base, arena, ownership_events);
            for field in &update_expr.fields {
                append_record_update_write_events_from_expr(field.value, arena, ownership_events);
            }
            if let Some(record_path) = direct_record_access_path_from_expr(update_expr.base, arena)
            {
                for field in &update_expr.fields {
                    ownership_events.push(OwnershipPathEvent {
                        kind: OwnershipPathEventKind::Write,
                        path: record_path.field(field.name),
                    });
                }
            }
        }
        Expr::Call(_, args) => {
            for arg in args {
                append_record_update_write_events_from_expr(arg.value, arena, ownership_events);
            }
        }
        Expr::Unary(_, inner) => {
            append_record_update_write_events_from_expr(*inner, arena, ownership_events);
        }
        Expr::Binary(lhs, _, rhs) => {
            append_record_update_write_events_from_expr(*lhs, arena, ownership_events);
            append_record_update_write_events_from_expr(*rhs, arena, ownership_events);
        }
        Expr::Range(range) => {
            append_record_update_write_events_from_expr(range.start, arena, ownership_events);
            append_record_update_write_events_from_expr(range.end, arena, ownership_events);
        }
        Expr::If(if_expr) => {
            append_record_update_write_events_from_expr(if_expr.condition, arena, ownership_events);
            append_record_update_write_events_from_expr(
                if_expr.then_block.tail,
                arena,
                ownership_events,
            );
            append_record_update_write_events_from_expr(
                if_expr.else_block.tail,
                arena,
                ownership_events,
            );
        }
        Expr::IfLet(if_let_expr) => {
            append_record_update_write_events_from_expr(if_let_expr.value, arena, ownership_events);
            append_record_update_write_events_from_expr(
                if_let_expr.then_block.tail,
                arena,
                ownership_events,
            );
            append_record_update_write_events_from_expr(
                if_let_expr.else_block.tail,
                arena,
                ownership_events,
            );
        }
        Expr::Block(block) => {
            append_record_update_write_events_from_expr(block.tail, arena, ownership_events);
        }
        Expr::Match(match_expr) => {
            append_record_update_write_events_from_expr(
                match_expr.scrutinee,
                arena,
                ownership_events,
            );

            let scrutinee_path = sequence_access_path_from_expr(match_expr.scrutinee, arena);
            let mut borrowed_dynamic_scrutinee_root = false;
            for arm in &match_expr.arms {
                if let sm_front::types::MatchPattern::Adt(adt_pat) = &arm.pat {
                    if let Some(path) = &scrutinee_path {
                        if path.is_dynamic_fallback() {
                            let should_borrow = adt_pat.items.iter().any(|item| {
                                matches!(
                                    item,
                                    sm_front::types::AdtPatternItem::Bind {
                                        capture: sm_front::types::CaptureMode::Borrow,
                                        ..
                                    }
                                )
                            });
                            if should_borrow && !borrowed_dynamic_scrutinee_root {
                                ownership_events.push(OwnershipPathEvent {
                                    kind: OwnershipPathEventKind::Borrow,
                                    path: path.as_path().clone(),
                                });
                                borrowed_dynamic_scrutinee_root = true;
                            }
                        } else {
                            for (idx, item) in adt_pat.items.iter().enumerate() {
                                if let sm_front::types::AdtPatternItem::Bind {
                                    capture: sm_front::types::CaptureMode::Borrow,
                                    ..
                                } = item
                                {
                                    ownership_events.push(OwnershipPathEvent {
                                        kind: OwnershipPathEventKind::Borrow,
                                        path: path
                                            .as_path()
                                            .adt_payload(adt_pat.variant_name, idx as u16),
                                    });
                                }
                            }
                        }
                    }
                }

                if let Some(guard) = arm.guard {
                    append_record_update_write_events_from_expr(guard, arena, ownership_events);
                }
                append_record_update_write_events_from_expr(
                    arm.block.tail,
                    arena,
                    ownership_events,
                );
            }
            if let Some(default) = &match_expr.default {
                append_record_update_write_events_from_expr(default.tail, arena, ownership_events);
            }
        }
        Expr::QuadLiteral(_)
        | Expr::BoolLiteral(_)
        | Expr::TextLiteral(_)
        | Expr::NumericLiteral(_)
        | Expr::Closure(_)
        | Expr::AdtCtor(_)
        | Expr::Var(_)
        | Expr::Loop(_) => {}
    }
}

#[inline]
fn alloc(next: &mut u16) -> u16 {
    let out = *next;
    *next += 1;
    out
}

#[cfg(test)]
mod opt_tests {
    use super::*;
    use crate::passes::run_default_opt_passes;
    use sm_format::semcode_decode::{decode_semcode_envelope, DecodedAccessPathComponent};
    use sm_front::parse_program;

    #[test]
    fn storage_admission_sequence_and_map_aggregate_storage_lowers() {
        // FA-02-038 / #1861 corrective round: Sequence/Map have no prior
        // normative record-field statement (unlike Tuple/Measured/Option/
        // Result, documented in docs/spec/types.md); this is their sole
        // aggregate-storage lowering evidence.
        let cases = [
            (
                "record Sequence",
                "record R { x: Sequence(i32) } fn main() { let r: R = R { x: [1, 2] }; let s: Sequence(i32) = r.x; let _ = s; return; }",
            ),
            (
                "adt Sequence",
                "enum E { V(Sequence(i32)) } fn main() { let e: E = E::V([1, 2]); match e { E::V(s) => { let _ = s; } } return; }",
            ),
            (
                "record Map",
                "record R { x: Map(i32, i32) } fn main() { let r: R = R { x: map_empty() }; let m: Map(i32, i32) = r.x; let _ = m; return; }",
            ),
            (
                "adt Map",
                "enum E { V(Map(i32, i32)) } fn main() { let e: E = E::V(map_empty()); match e { E::V(m) => { let _ = m; } } return; }",
            ),
        ];
        for (label, src) in cases {
            compile_program_to_ir(src)
                .unwrap_or_else(|e| panic!("{label}: aggregate storage must lower: {e:?}"));
        }
    }

    #[test]
    fn storage_admission_record_closure_field_lowers_to_working_ir() {
        // FA-02-038 / #1861 corrective round: proves the record-closure
        // path lowers to real, composing opcodes (MakeClosure -> MakeRecord
        // -> RecordGet -> ClosureCall), not merely that the frontend admits
        // the declaration. Full VM execution is proven in sm-vm's test
        // suite (crates/sm-vm/src/lib.rs).
        let src = r#"
            record Holder {
                f: Closure(f64 -> f64),
            }

            fn main() {
                let h: Holder = Holder { f: (x => x + 1.0) };
                let g: Closure(f64 -> f64) = h.f;
                let total: f64 = g(2.0);
                return;
            }
        "#;
        let ir = compile_program_to_ir(src).expect("record closure field storage must lower");
        let main = &ir[0];
        assert!(
            main.instrs
                .iter()
                .any(|instr| matches!(instr, IrInstr::MakeClosure { .. })),
            "expected a MakeClosure instruction"
        );
        assert!(
            main.instrs
                .iter()
                .any(|instr| matches!(instr, IrInstr::MakeRecord { .. })),
            "expected the closure to be stored via MakeRecord"
        );
        assert!(
            main.instrs
                .iter()
                .any(|instr| matches!(instr, IrInstr::RecordGet { .. })),
            "expected the closure to be read back out via RecordGet"
        );
        assert!(
            main.instrs
                .iter()
                .any(|instr| matches!(instr, IrInstr::ClosureCall { .. })),
            "expected the extracted closure to be invoked via ClosureCall"
        );
    }

    #[test]
    fn storage_admission_adt_closure_payload_lowers_to_working_ir() {
        let src = r#"
            enum Holder {
                Wrap(Closure(f64 -> f64)),
            }

            fn main() {
                let h: Holder = Holder::Wrap((x => x + 1.0));
                let total: f64 = match h {
                    Holder::Wrap(g) => { g(2.0) }
                };
                return;
            }
        "#;
        let ir = compile_program_to_ir(src).expect("ADT closure payload storage must lower");
        let main = &ir[0];
        assert!(
            main.instrs
                .iter()
                .any(|instr| matches!(instr, IrInstr::MakeClosure { .. })),
            "expected a MakeClosure instruction"
        );
        assert!(
            main.instrs
                .iter()
                .any(|instr| matches!(instr, IrInstr::MakeAdt { .. })),
            "expected the closure to be stored via MakeAdt"
        );
        assert!(
            main.instrs
                .iter()
                .any(|instr| matches!(instr, IrInstr::ClosureCall { .. })),
            "expected the extracted closure to be invoked via ClosureCall"
        );
    }

    // FA-04-011 / #1717: sm-ir's executable boundary has no monomorphisation
    // pass, so a generic function's declaration (Function.type_params
    // non-empty) is rejected deterministically at IR lowering -- used or
    // unused, called or uncalled, direct or nested, single or multiply
    // instantiated at call sites. Frontend generic-function type semantics
    // (#1634/#1648/#1649) remain fully admitted; only IR/SemCode execution
    // is gated. See ensure_function_is_ir_concrete's doc comment for the
    // full architecture rationale.

    fn expect_ir_generic_rejection(result: Result<Vec<IrFunction>, FrontendError>, fn_name: &str) {
        let err = result.expect_err(&format!(
            "a generic function ('{fn_name}') must be rejected at IR compilation"
        ));
        assert!(
            err.message.contains(fn_name)
                && err
                    .message
                    .contains("not executable in the current IR contract")
                && err
                    .message
                    .contains("concrete IR monomorphisation is not implemented"),
            "unexpected error: {}",
            err.message
        );
        assert!(
            !err.message.contains("deferred to M9.1 Wave 2"),
            "rejection must come from the deliberate IR generic-execution boundary, not the \
             old accidental, misleading construction-time TypeVar-canonicalization failure: {}",
            err.message
        );
    }

    #[test]
    fn compile_program_to_ir_rejects_unused_generic_type_param() {
        // Central regression: T never appears in params/return/body types,
        // so the pre-#1717 accidental TypeVar-canonicalization rejection
        // never fired and this function silently lowered as an ordinary
        // IrFunction -- pure type erasure, not partial monomorphisation
        // support.
        let src = r#"
            fn marker<T>(x: i32) -> i32 {
                return x;
            }
            fn main() {
                return;
            }
        "#;
        expect_ir_generic_rejection(compile_program_to_ir(src), "marker");
    }

    #[test]
    fn compile_program_to_ir_rejects_direct_generic_type_param() {
        let src = r#"
            fn id<T>(x: T) -> T {
                return x;
            }
            fn main() {
                let y: i32 = id(1);
                let _ = y;
                return;
            }
        "#;
        expect_ir_generic_rejection(compile_program_to_ir(src), "id");
    }

    #[test]
    fn compile_program_to_ir_rejects_nested_generic_type_param() {
        let src = r#"
            fn keep<T>(x: Option(T)) -> Option(T) {
                return x;
            }
            fn main() {
                return;
            }
        "#;
        expect_ir_generic_rejection(compile_program_to_ir(src), "keep");
    }

    #[test]
    fn compile_program_to_ir_rejects_generic_regardless_of_instantiation_count() {
        // No per-call-site specialization identity exists: the same
        // function-level rejection fires whether id() is called zero, one,
        // or many times with different concrete types -- proving there is
        // no monomorphisation model that could ever distinguish id<i32>
        // from id<text>.
        let called_multiple = r#"
            fn id<T>(x: T) -> T {
                return x;
            }
            fn main() {
                let a: i32 = id(1);
                let b: text = id("x");
                let _ = a;
                let _ = b;
                return;
            }
        "#;
        expect_ir_generic_rejection(compile_program_to_ir(called_multiple), "id");

        let never_called = r#"
            fn id<T>(x: T) -> T {
                return x;
            }
            fn main() {
                return;
            }
        "#;
        expect_ir_generic_rejection(compile_program_to_ir(never_called), "id");
    }

    #[test]
    fn compile_program_to_ir_rejects_generic_to_generic_delegation() {
        // Preserves #1648's legitimate frontend behavior (asserted below)
        // while proving IR compilation rejects the delegating wrapper
        // itself, not only the innermost primitive it calls. `outer` is
        // declared first so its own rejection is what surfaces (the
        // compile loop fails on the first generic declaration it
        // encounters; `id` would reject independently too, but ordering
        // it second here specifically proves `outer`'s own type_params
        // triggers admission failure rather than something specific to
        // calling `id`).
        let src = r#"
            fn outer<T>(x: T) -> T {
                return id(x);
            }
            fn id<T>(x: T) -> T {
                return x;
            }
            fn main() {
                let y: i32 = outer(1);
                let _ = y;
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        sm_front::type_check_program(&program)
            .expect("frontend generic-to-generic delegation must remain admitted (#1648)");
        expect_ir_generic_rejection(compile_program_to_ir(src), "outer");
    }

    #[test]
    fn compile_program_to_ir_rejects_generic_with_satisfied_trait_bound() {
        let src = r#"
            trait Zeroable {
                fn zero(v: ZeroInt) -> i32;
            }
            record ZeroInt { n: i32 }
            impl Zeroable for ZeroInt {
                fn zero(v: ZeroInt) -> i32 { return 0; }
            }
            fn wrap<T: Zeroable>(x: Option(T)) -> i32 {
                return 0;
            }
            fn main() {
                let v: Option(ZeroInt) = Option::Some(ZeroInt { n: 0 });
                let y: i32 = wrap(v);
                let _ = y;
                return;
            }
        "#;
        let program = parse_program(src).expect("parse");
        sm_front::type_check_program(&program)
            .expect("frontend must admit a bound-satisfied generic call (#1649)");
        expect_ir_generic_rejection(compile_program_to_ir(src), "wrap");
    }

    #[test]
    fn lower_function_to_ir_directly_rejects_generic_function() {
        // Proves the shared owner boundary cannot be bypassed by calling
        // the other public single-function lowering entrypoint directly.
        let src = r#"
            fn id<T>(x: T) -> T {
                return x;
            }
        "#;
        let program = parse_program(src).expect("parse");
        let fn_table = sm_front::build_fn_table(&program).expect("fn table");
        let func = &program.functions[0];
        let err = lower_function_to_ir(func, &program.arena, &fn_table)
            .expect_err("direct lower_function_to_ir must also reject a generic function");
        assert!(
            err.message.contains("id")
                && err
                    .message
                    .contains("not executable in the current IR contract"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn compile_program_to_semcode_rejects_generic_function_before_any_emission() {
        // compile_program_to_semcode_with_options_debug calls
        // compile_program_to_immutable_ir, which calls the same central
        // compile_program_to_ir_with_options_and_profile this PR gates --
        // proving no partial/malformed SemCode artifact can ever be
        // produced for a generic function; the rejection propagates before
        // emit_semcode is ever reached.
        let src = r#"
            fn id<T>(x: T) -> T {
                return x;
            }
            fn main() {
                let y: i32 = id(1);
                let _ = y;
                return;
            }
        "#;
        let err = compile_program_to_semcode(src)
            .expect_err("SemCode compilation of a generic function must reject deterministically");
        assert!(
            err.message.contains("id")
                && err
                    .message
                    .contains("not executable in the current IR contract"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn compile_program_to_ir_admits_ordinary_non_generic_functions_with_structural_params() {
        // Positive control: non-generic functions using the same
        // structural families (Option, Sequence, Tuple, Record, ADT) that
        // #1648 qualified for generic inference must remain fully
        // executable -- only functions with a non-empty type_params are
        // affected by this PR.
        let src = r#"
            record Point { x: i32, y: i32 }
            enum Color { Red, Green, Blue }
            fn first(xs: Sequence(i32)) -> i32 {
                return xs[0];
            }
            fn unwrap_opt(x: Option(i32)) -> i32 {
                return 0;
            }
            fn make_pair(a: i32, b: text) -> (i32, text) {
                return (a, b);
            }
            fn make_point(x: i32, y: i32) -> Point {
                return Point { x: x, y: y };
            }
            fn pick_color() -> Color {
                return Color::Red;
            }
            fn main() {
                return;
            }
        "#;
        compile_program_to_ir(src)
            .expect("ordinary non-generic functions over structural families must lower");
    }

    #[test]
    fn compile_program_to_ir_admits_ordinary_non_generic_identity_function() {
        let src = r#"
            fn inc(x: i32) -> i32 {
                return x + 1;
            }
            fn main() {
                let y: i32 = inc(1);
                let _ = y;
                return;
            }
        "#;
        compile_program_to_ir(src).expect("ordinary non-generic function must lower");
    }

    // #1732 (FA-05-002) review follow-up: proves emit_semcode_function's
    // opcode-revision tracking is genuinely mechanical (reads the real byte
    // emit_instr wrote), not a second hand-maintained table - this is what
    // lets emit_semcode's post-loop check catch a future opcode whose
    // Opcode::minimum_semcode_revision() is elevated without a matching
    // header-selection promotion, instead of silently emitting bytes its
    // own verifier would reject.
    #[test]
    fn emit_semcode_function_tracks_max_opcode_revision_for_qtruth() {
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadI32 { dst: 0, val: 1 },
                IrInstr::QTruthAnd {
                    dst: 1,
                    lhs: 0,
                    rhs: 0,
                },
                IrInstr::Ret { src: None },
            ],
            ownership_events: Vec::new(),
            params: Vec::new(),
        };
        let (_, max_rev) = emit_semcode_function(&func, false, false, 19).expect("emit");
        assert_eq!(max_rev, 19);
    }

    #[test]
    fn emit_semcode_function_tracks_baseline_revision_without_qtruth() {
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadI32 { dst: 0, val: 1 },
                IrInstr::Ret { src: None },
            ],
            ownership_events: Vec::new(),
            params: Vec::new(),
        };
        let (_, max_rev) = emit_semcode_function(&func, false, false, 1).expect("emit");
        assert_eq!(max_rev, 1);
    }

    // #1732 (FA-05-002) review follow-up: opcode_minimum_revision_at must
    // fail closed - never silently skip - if the opcode byte it's asked to
    // read is missing or unrecognized. Both failure modes are unreachable
    // through the public compile/emit API today (emit_instr always writes a
    // real Opcode byte for every non-Label instruction), so these test the
    // pure helper directly with hand-built byte slices rather than trying
    // to force emit_instr to misbehave.
    #[test]
    fn opcode_minimum_revision_at_fails_closed_on_missing_byte() {
        let code: &[u8] = &[];
        let err = opcode_minimum_revision_at(code, 0).expect_err("must fail closed");
        assert!(err.message.contains("no opcode byte"));
    }

    #[test]
    fn opcode_minimum_revision_at_fails_closed_on_unrecognized_byte() {
        // 0x00 is not assigned to any Opcode variant.
        let code: &[u8] = &[0x00];
        let err = opcode_minimum_revision_at(code, 0).expect_err("must fail closed");
        assert!(err.message.contains("unrecognized opcode byte"));
    }

    #[test]
    fn opcode_minimum_revision_at_returns_correct_revision_for_known_opcode() {
        let code: &[u8] = &[Opcode::QTruthAnd.byte(), 0, 0, 0, 0, 0, 0];
        assert_eq!(opcode_minimum_revision_at(code, 0), Ok(19));

        let code: &[u8] = &[Opcode::LoadI32.byte(), 0, 0, 0, 0, 0, 0];
        assert_eq!(opcode_minimum_revision_at(code, 0), Ok(1));
    }

    #[test]
    fn access_path_root_starts_with_empty_component_list() {
        let path = AccessPath::new(SymbolId(7));
        assert_eq!(path.root, SymbolId(7));
        assert!(path.components.is_empty());
    }

    #[test]
    fn access_path_tuple_indices_preserve_append_order() {
        let path = AccessPath::new(SymbolId(3)).tuple_index(1).tuple_index(4);
        assert_eq!(path.root, SymbolId(3));
        assert_eq!(
            path.components,
            vec![PathComponent::TupleIndex(1), PathComponent::TupleIndex(4)]
        );
    }

    #[test]
    fn access_path_record_field_can_be_represented() {
        let camera = SymbolId(11);
        let path = AccessPath::new(SymbolId(3)).field(camera);
        assert_eq!(path.root, SymbolId(3));
        assert_eq!(path.components, vec![PathComponent::Field(camera)]);
    }

    #[test]
    fn access_path_sequence_index_static_can_be_represented() {
        let path = AccessPath::new(SymbolId(3))
            .sequence_index_static(2)
            .sequence_index_static(4);
        assert_eq!(path.root, SymbolId(3));
        assert_eq!(
            path.components,
            vec![
                PathComponent::SequenceIndexStatic(2),
                PathComponent::SequenceIndexStatic(4),
            ]
        );
    }

    #[test]
    fn access_path_component_order_is_deterministic() {
        let field = SymbolId(12);
        let left = AccessPath::new(SymbolId(9)).tuple_index(0).field(field);
        let right = AccessPath::new(SymbolId(9)).tuple_index(0).field(field);
        let different = AccessPath::new(SymbolId(9)).field(field).tuple_index(0);
        assert_eq!(left, right);
        assert_ne!(left, different);
    }

    fn lower_single_function_with_program(
        src: &str,
        fn_name: &str,
    ) -> (sm_front::Program, IrFunction) {
        let program = parse_program(src).expect("program should parse");
        let fn_table = build_fn_table(&program).expect("function table should build");
        let record_table = build_record_table(&program).expect("record table should build");
        let adt_table = build_adt_table(&program).expect("adt table should build");
        type_check_program(&program).expect("program should type-check");
        let func = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == fn_name)
            .expect("function should exist");
        let lowered = lower_function_to_ir_with_tables(
            func,
            &program.arena,
            &fn_table,
            &record_table,
            &adt_table,
            &program.impls,
        )
        .expect("function should lower");
        (program, lowered.primary)
    }

    #[test]
    fn lower_block_expression_tail_to_ir() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    let base: f64 = 1.0;
                    base + 2.0
                };
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("block expression should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_base")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_total")
        )));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
    }

    #[test]
    fn block_expression_rejects_control_statements_in_body() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    if true { return; } else { return; }
                    1.0
                };
                return;
            }
        "#;

        let err = compile_program_to_ir(src).expect_err("control statements must reject");
        assert!(err.message.contains(
            "value-producing block currently supports only const-bindings, let-bindings, discard binds, and expression statements before the tail value"
        ));
    }

    #[test]
    fn lower_if_expression_to_ir() {
        let src = r#"
            fn main() {
                let total: f64 = if true { 1.0 } else { 2.0 };
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("if expression should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("if_expr_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.starts_with("__if_expr_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::LoadVar { name, .. } if name.starts_with("__if_expr_")
        )));
    }

    #[test]
    fn lowering_if_expression_rejects_branch_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = if true { 1.0 } else { true };
                return;
            }
        "#;

        let err = compile_program_to_ir(src).expect_err("mismatched branch types must reject");
        assert!(err.message.contains("if expression branch type mismatch"));
    }

    #[test]
    fn lower_match_expression_to_ir() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T if true => { 1.0 }
                    _ => { 2.0 }
                };
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("match expression should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("match_expr_")
        )));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadBool { .. })));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.starts_with("__match_expr_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::LoadVar { name, .. } if name.starts_with("__match_expr_")
        )));
    }

    #[test]
    fn lower_adt_match_expression_to_tag_and_payload_ir() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn main() {
                let total: f64 = match Maybe::Some(1.0) {
                    Maybe::Some(inner) => { inner }
                    _ => { 0.0 }
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        let ir = compile_program_to_ir(src).expect("ADT match expression should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AdtTag { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AdtGet { index: 0, .. })));
    }

    #[test]
    fn lower_exhaustive_adt_match_expression_without_default_to_trap_backstop() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn main() {
                let total: f64 = match Maybe::Some(1.0) {
                    Maybe::None => { 0.0 }
                    Maybe::Some(inner) => { inner }
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        let ir = compile_program_to_ir(src)
            .expect("exhaustive ADT match expression without default should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AdtTag { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Assert { .. })));
    }

    // FA-02-007 / #1639, items 8-9: statement-form match lowering must
    // preserve the same presence-vs-absence distinction as the AST -- an
    // explicitly present but empty default (`Some(vec![])`) lowers as a real,
    // reachable (if empty) branch, never collapsed into the "provably
    // unreachable" trap backstop that a genuinely absent default (`None`)
    // over exhaustive arms gets.

    #[test]
    fn lower_statement_match_exhaustive_without_default_to_trap_backstop() {
        // Item 9: a legally exhaustive statement-form match with `None`
        // (no `_` arm at all) must still lower correctly, with the
        // impossible-to-reach default branch backed by an `Assert` trap.
        let src = r#"
            enum Flag { A, B }

            fn main() {
                let f: Flag = Flag::A;
                match f {
                    Flag::A => { }
                    Flag::B => { }
                }
                return;
            }
        "#;

        let ir = compile_program_to_ir(src)
            .expect("exhaustive statement match without default should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Assert { .. })));
    }

    #[test]
    fn lower_statement_match_present_empty_default_does_not_emit_trap() {
        // Item 8: an explicitly present but empty default (`_ => {}`, i.e.
        // `Some(vec![])`) over non-exhaustive arms must lower successfully
        // as a real branch. It must not be treated as absent and must not
        // be routed through the impossible-match `Assert` trap -- if
        // lowering ever regressed to inferring presence from body
        // emptiness, this case (non-exhaustive arms, so the trap path would
        // otherwise be unreachable) would either wrongly reject with
        // "match requires default arm '_'" or wrongly emit an assert(false).
        let src = r#"
            enum Flag { A, B, C }

            fn main() {
                let f: Flag = Flag::A;
                match f {
                    Flag::A => { }
                    _ => { }
                }
                return;
            }
        "#;

        let ir = compile_program_to_ir(src)
            .expect("non-exhaustive statement match with a present empty default should lower");
        let main = &ir[0];
        assert!(
            !main
                .instrs
                .iter()
                .any(|instr| matches!(instr, IrInstr::Assert { .. })),
            "a present, empty default must not be lowered as the impossible-match trap"
        );
    }

    #[test]
    fn lower_guard_clause_to_ir() {
        let src = r#"
            fn main() {
                guard true else return;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("guard clause should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::JmpIf { label, .. } if label.starts_with("guard_")
        )));
        assert!(
            main.instrs
                .iter()
                .filter(|instr| matches!(instr, IrInstr::Ret { .. }))
                .count()
                >= 2
        );
    }

    #[test]
    fn lower_pipeline_expression_to_ordinary_calls() {
        let src = r#"
            fn inc(x: f64) -> f64 = x + 1.0;
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = 1.0 |> inc() |> scale(3.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        let ir = compile_program_to_ir(src).expect("pipeline should lower through ordinary calls");
        let main = &ir[2];
        let call_names: Vec<_> = main
            .instrs
            .iter()
            .filter_map(|instr| match instr {
                IrInstr::Call { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert!(call_names.contains(&"inc"));
        assert!(call_names.contains(&"scale"));
    }

    #[test]
    fn lower_named_arguments_to_ordinary_call_order() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;
            fn main() {
                let total: f64 = scale(factor = 3.0, x = 2.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("named arguments should lower");
        let main = &ir[1];
        let call = main
            .instrs
            .iter()
            .find(|instr| matches!(instr, IrInstr::Call { name, .. } if name == "scale"));
        assert!(call.is_some());
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadF64 { val, .. } if (*val - 2.0).abs() < f64::EPSILON)));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadF64 { val, .. } if (*val - 3.0).abs() < f64::EPSILON)));
    }

    #[test]
    fn lower_named_arguments_evaluates_in_source_order_not_parameter_order() {
        // FA-04-016 / #1722: `scale(factor = 3.0, x = 2.0)` must evaluate
        // `factor`'s expression (3.0) before `x`'s expression (2.0) because
        // it is written first in source, even though `x` is the declared
        // first parameter. Named-argument slot assignment (x <- 2.0,
        // factor <- 3.0) must not change evaluation order.
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;
            fn main() {
                let total: f64 = scale(factor = 3.0, x = 2.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("named arguments should lower");
        let main = &ir[1];
        let load_order: Vec<f64> = main
            .instrs
            .iter()
            .filter_map(|instr| match instr {
                IrInstr::LoadF64 { val, .. }
                    if (*val - 2.0).abs() < f64::EPSILON || (*val - 3.0).abs() < f64::EPSILON =>
                {
                    Some(*val)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            load_order,
            vec![3.0, 2.0],
            "expected source-order evaluation (factor=3.0 loaded before x=2.0), got {:?}",
            load_order
        );
    }

    #[test]
    fn lower_named_arguments_evaluates_in_source_order_for_call_statement() {
        // FA-04-016 / #1722: the call-statement lowering path (a bare,
        // Unit-returning call used as a statement) owns independent
        // lowering logic from the call-expression path and must preserve
        // the same source-order evaluation guarantee.
        let src = r#"
            fn take(x: f64, factor: f64) {
                return;
            }
            fn main() {
                take(factor = 3.0, x = 2.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("named arguments should lower");
        let main = &ir[1];
        let load_order: Vec<f64> = main
            .instrs
            .iter()
            .filter_map(|instr| match instr {
                IrInstr::LoadF64 { val, .. }
                    if (*val - 2.0).abs() < f64::EPSILON || (*val - 3.0).abs() < f64::EPSILON =>
                {
                    Some(*val)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            load_order,
            vec![3.0, 2.0],
            "expected source-order evaluation (factor=3.0 loaded before x=2.0), got {:?}",
            load_order
        );
    }

    #[test]
    fn lowering_rejects_builtin_named_arguments() {
        let src = r#"
            fn main() {
                let total: f64 = sqrt(x = 4.0);
                return;
            }
        "#;

        let err = compile_program_to_ir(src).expect_err("builtin named arguments must reject");
        assert!(err
            .message
            .contains("named arguments are not supported for builtin 'sqrt'"));
    }

    #[test]
    fn lower_mixed_named_arguments_and_trailing_default_evaluates_in_source_order() {
        // FA-04-016 / #1722: named-argument reordering must still preserve
        // source-order evaluation when combined with a trailing default
        // parameter. `combo(b = 3.0, a = 1.0)` writes `b` before `a`, and
        // `c`'s default (a compiler-synthesized, non-source expression) is
        // evaluated last, after every explicitly written argument.
        let src = r#"
            fn combo(a: f64, b: f64, c: f64 = 9.0) -> f64 = a + b + c;
            fn main() {
                let total: f64 = combo(b = 3.0, a = 1.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("mixed named/default call should lower");
        let main = &ir[1];
        let load_order: Vec<f64> = main
            .instrs
            .iter()
            .filter_map(|instr| match instr {
                IrInstr::LoadF64 { val, .. }
                    if [1.0, 3.0, 9.0]
                        .iter()
                        .any(|expected| (*val - expected).abs() < f64::EPSILON) =>
                {
                    Some(*val)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            load_order,
            vec![3.0, 1.0, 9.0],
            "expected source-order evaluation (b=3.0, then a=1.0, then c's default=9.0 last), got {:?}",
            load_order
        );
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Call { name, args, .. } if name == "combo" && args.len() == 3)));
    }

    #[test]
    fn lower_default_parameters_to_ordinary_call_order() {
        let src = r#"
            fn scale(x: f64, factor: f64 = 2.0) -> f64 = x * factor;
            fn main() {
                let total: f64 = scale(3.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("default parameters should lower");
        let main = &ir[1];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Call { name, args, .. } if name == "scale" && args.len() == 2)));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadF64 { val, .. } if (*val - 2.0).abs() < f64::EPSILON)));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadF64 { val, .. } if (*val - 3.0).abs() < f64::EPSILON)));
    }

    #[test]
    fn lowering_rejects_non_const_safe_default_parameter_initializer() {
        let src = r#"
            fn scale(x: f64, factor: f64 = sqrt(4.0)) -> f64 = x * factor;
            fn main() {
                return;
            }
        "#;

        let err =
            compile_program_to_ir(src).expect_err("non-const-safe default parameter must reject");
        assert!(err.message.contains("default parameter 'factor'"));
    }

    #[test]
    fn lower_immediate_short_lambda_without_indirect_call_path() {
        let src = r#"
            fn main() {
                let total: f64 = (x => x + 1.0)(2.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("short lambda should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_x"))));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(!main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Call { .. })));
    }

    #[test]
    fn lower_pipeline_short_lambda_without_indirect_call_path() {
        let src = r#"
            fn main() {
                let total: f64 = 2.0 |> (x => x + 1.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("pipeline short lambda should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_x"))));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(!main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Call { .. })));
    }

    #[test]
    fn lower_const_declaration_to_existing_store_path() {
        let src = r#"
            fn main() {
                const total: f64 = 1.0 + 2.0;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("const declaration should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(main.instrs.iter().any(
            |instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_total"))
        ));
    }

    #[test]
    fn lowering_rejects_assignment_to_const_binding() {
        let src = r#"
            fn main() {
                const total: f64 = 1.0;
                total += 2.0;
                return;
            }
        "#;

        let err = compile_program_to_ir(src).expect_err("assignment to const must reject");
        assert!(err
            .message
            .contains("cannot assign to const binding 'total'"));
    }

    #[test]
    fn lowering_rejects_tuple_assignment_to_const_binding() {
        // #1664 completion: is_const's fail-closed migration (bool -> ?)
        // must not disturb this rejection for the tuple-destructuring
        // assignment call site.
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                const count: i32 = 0;
                let ready: bool = false;
                (count, ready) = pair(true);
                return;
            }
        "#;

        let err =
            compile_program_to_ir(src).expect_err("tuple assignment to const target must reject");
        assert!(err
            .message
            .contains("cannot assign to const binding 'count'"));
    }

    #[test]
    fn lowering_succeeds_for_closure_capturing_const_binding() {
        // #1664 completion. What this proves: the closure-capture call
        // site's is_const(*capture)? migration (bool -> Result) does not
        // regress the ordinary case -- a closure capturing a const
        // binding still lowers successfully and emits its lifted helper.
        //
        // What this does NOT prove: that the captured binding was
        // actually inserted into the lifted helper's own environment via
        // lifted_env.insert_const(...) rather than lifted_env.insert(...)
        // -- i.e. this does not independently verify constness
        // preservation *within* the lifted environment.
        //
        // That could not be observed directly without architectural
        // distortion. The only operation whose outcome differs between
        // insert_const and insert is an assignment to the captured name
        // rejecting as const -- and a closure body is a value-producing
        // block (`infer_value_block_type` in
        // crates/sm-front/src/typecheck.rs), which admits only
        // `Stmt::Const | Let | LetTuple | Discard | Expr(_)`; `Stmt::Assign`
        // is categorically not admitted inside any value-producing block,
        // closure bodies included (confirmed by direct inspection, and
        // empirically: `(x => { offset = 2.0; x })` fails to parse with
        // "expected '}' after value-producing block", independent of
        // constness). No admitted source can place a reassignment inside
        // a closure body at all, so no admitted source can make lowering
        // outcome differ based on which of the two `lifted_env` calls
        // fired. Exposing `lifted_env` itself from
        // `lower_closure_literal_expr` for direct inspection would be a
        // production hook added solely for this test, which is exactly
        // what this migration must not do.
        let src = r#"
            fn main() {
                const offset: f64 = 1.0;
                let add: Closure(f64 -> f64) = (x => x + offset);
                let total: f64 = add(2.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src)
            .expect("closure capturing a const binding should lower successfully");
        assert!(ir
            .iter()
            .any(|func| func.name.starts_with("__closure_main_")));
    }

    #[test]
    fn lower_extended_numeric_literals_to_typed_loads() {
        let src = r#"
            fn main() {
                let hex: i32 = 0xff;
                let unsigned: u32 = 1_000u32;
                let fixed: fx = 1.25fx;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("extended numeric literals should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadI32 { val, .. } if *val == 255)));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadU32 { val, .. } if *val == 1000)));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadFx { val, .. } if *val == 1250)));
    }

    #[test]
    fn plain_fx_arithmetic_lowers_to_fx_ops() {
        let src = r#"
            fn main() {
                let a: fx = 2.0;
                let b: fx = 3.0;
                let sum: fx = a + b;
                let diff: fx = a - b;
                let prod: fx = a * b;
                let quo: fx = a / b;
                let neg: fx = -a;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("plain fx arithmetic should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddFx { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::SubFx { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::MulFx { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::DivFx { .. })));
    }

    #[test]
    fn text_literals_lower_to_load_text_and_semcod19() {
        let src = r#"
            fn main() {
                let left: text = "alpha";
                let right: text = "alpha";
                assert(left == right);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("text literals should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadText { .. })));

        // #1773 (FA-09-005): every compiled artifact now carries a canonical
        // callable-signature record per function, which only a header at or
        // above SEMCODE_SIGNATURE_MIN_REVISION can structurally carry - so
        // SEMCOD19 is now the floor regardless of which lesser opcodes this
        // program happens to use (was SEMCODE8, text's own promotion floor).
        let bytes = compile_program_to_semcode(src).expect("text semcode should emit");
        assert_eq!(&bytes[0..8], b"SEMCOD19");
    }

    #[test]
    fn sequence_literals_indexing_and_equality_lower_to_semcod19() {
        let src = r#"
            fn head(values: Sequence(i32), index: i32) -> i32 {
                return values[index];
            }

            fn main() {
                let left: Sequence(i32) = [1, 2, 3];
                let right: Sequence(i32) = [1, 2, 3];
                let first: i32 = head(left, 0);
                assert(first == 1);
                assert(left == right);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("ordered sequence runtime surface should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::MakeSequence { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpEq { .. })));
        let head = ir.iter().find(|func| func.name == "head").expect("head fn");
        assert!(head
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::SequenceGet { .. })));

        // #1773 (FA-09-005): SEMCOD19 is now the floor for every compiled
        // artifact (was SEMCODE9, sequences' own promotion floor) - see the
        // comment in `text_literals_lower_to_load_text_and_semcod19` above.
        let bytes = compile_program_to_semcode(src).expect("ordered sequence semcode should emit");
        assert_eq!(&bytes[0..8], b"SEMCOD19");
    }

    #[test]
    fn first_class_closures_lower_to_runtime_carrier_and_semcod19() {
        let src = r#"
            fn main() {
                let offset: f64 = 1.0;
                let add: Closure(f64 -> f64) = (x => x + offset);
                let total: f64 = add(2.0);
                assert(total == 3.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("first-class closures should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::MakeClosure { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::ClosureCall { .. })));
        let helper = ir
            .iter()
            .find(|func| func.name.starts_with("__closure_main_"))
            .expect("lifted closure helper");
        assert!(helper
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));

        // #1773 (FA-09-005): SEMCOD19 is now the floor for every compiled
        // artifact (was SEMCOD10, closures' own promotion floor) - see the
        // comment in `text_literals_lower_to_load_text_and_semcod19` above.
        let bytes = compile_program_to_semcode(src).expect("closure semcode should emit");
        assert_eq!(&bytes[0..8], b"SEMCOD19");
    }

    // #1773 (FA-09-005) permanent regressions: callable-signature contract
    // preservation and the QVec architecture-decision boundary. These
    // promote the original architecture-checkpoint RED reproductions (Cases
    // A-E, posted to issue #1773) to permanent coverage.

    #[test]
    fn callable_signature_preserves_multi_family_parameters_in_declared_order() {
        let src = r#"
            fn describe(count: i32, active: bool, label: text) -> i32 {
                return count;
            }

            fn main() {
                return;
            }
        "#;
        let ir = compile_program_to_ir(src).expect("multi-param function should lower");
        let describe = ir
            .iter()
            .find(|func| func.name == "describe")
            .expect("describe fn");
        assert_eq!(
            describe.params,
            vec![
                CallableValueFamily::I32,
                CallableValueFamily::Bool,
                CallableValueFamily::Text,
            ]
        );
    }

    #[test]
    fn callable_signature_is_empty_for_zero_parameter_function() {
        let src = "fn main() { return; }";
        let ir = compile_program_to_ir(src).expect("zero-param function should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.params.is_empty());
    }

    #[test]
    fn callable_signature_measured_parameter_erases_to_base_family() {
        let src = r#"
            fn scale(amount: f64[meters]) -> f64[meters] {
                return amount;
            }

            fn main() {
                return;
            }
        "#;
        let ir = compile_program_to_ir(src).expect("measured param should lower");
        let scale = ir
            .iter()
            .find(|func| func.name == "scale")
            .expect("scale fn");
        assert_eq!(scale.params, vec![CallableValueFamily::F64]);
    }

    #[test]
    fn callable_signature_option_and_result_parameters_map_to_adt_family() {
        let src = r#"
            fn first(items: Option(i32)) -> i32 {
                return 0;
            }

            fn main() {
                return;
            }
        "#;
        let ir = compile_program_to_ir(src).expect("option param should lower");
        let first = ir
            .iter()
            .find(|func| func.name == "first")
            .expect("first fn");
        assert_eq!(first.params, vec![CallableValueFamily::Adt]);
    }

    #[test]
    fn qvec_callable_parameter_is_a_deterministic_compile_time_rejection_no_semcode_emitted() {
        // #1773 owner decision: qvec is real, parser-writable, typechecking
        // syntax with no corresponding sm-vm::Value variant and no lowering
        // path that ever constructs one. It typechecks fine today (this is
        // Case-equivalent to the original architecture-checkpoint evidence),
        // so this must fail specifically at the new callable-signature
        // boundary inside lowering - not at parse/typecheck - and must never
        // reach SemCode emission.
        let src = "fn f(x: qvec) -> i32 { return 0; } fn main() { return; }";

        let ir_err = compile_program_to_ir(src)
            .expect_err("qvec callable parameter must be rejected, not silently lowered");
        assert!(
            ir_err.message.contains("qvec"),
            "rejection must name the offending type, got: {}",
            ir_err.message
        );
        assert!(
            ir_err.message.contains("executable"),
            "rejection must explain the executable-family gap, got: {}",
            ir_err.message
        );

        let semcode_err = compile_program_to_semcode(src)
            .expect_err("no SemCode bytes may be emitted for a qvec callable parameter");
        assert!(semcode_err.message.contains("qvec"));
    }

    /// Codex review follow-up on #1773: the SIG0 wire field's `u16` count
    /// alone permits up to 65,535 parameters, but the decoder bounds
    /// acceptance at `MAX_SIGNATURE_PARAMETERS_PER_FUNCTION` (4,096) -
    /// without an equal check on the emit side, a function with
    /// 4,097..=65,535 parameters would emit successfully and then be
    /// unconditionally rejected by every verified execution route
    /// downstream. Built via `emit_ir_to_semcode` directly since no source
    /// program plausibly declares this many parameters.
    #[test]
    fn callable_signature_rejects_parameter_count_over_decoder_limit() {
        let too_many = vec![CallableValueFamily::I32; MAX_SIGNATURE_PARAMETERS_PER_FUNCTION + 1];
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![IrInstr::Ret { src: None }],
            ownership_events: Vec::new(),
            params: too_many,
        };
        let err = emit_ir_to_semcode(&[func], false)
            .expect_err("over-limit parameter count must be rejected before emission");
        assert!(
            err.message
                .contains("too many callable-signature parameters"),
            "got: {}",
            err.message
        );
    }

    /// The boundary immediately below the same limit must still emit
    /// successfully - this is a decoder capacity bound, not an
    /// artificially-lowered one.
    #[test]
    fn callable_signature_accepts_parameter_count_at_decoder_limit() {
        let at_limit = vec![CallableValueFamily::I32; MAX_SIGNATURE_PARAMETERS_PER_FUNCTION];
        let func = IrFunction {
            name: "main".to_string(),
            instrs: vec![IrInstr::Ret { src: None }],
            ownership_events: Vec::new(),
            params: at_limit,
        };
        emit_ir_to_semcode(&[func], false).expect("at-limit parameter count must emit");
    }

    #[test]
    fn lower_compound_assignment_to_read_modify_write() {
        let src = r#"
            fn main() {
                let mut total: f64 = 1.0;
                total += 2.0;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("compound assignment should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(
            |instr| matches!(instr, IrInstr::LoadVar { name, .. } if name.ends_with("_total"))
        ));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(
            main.instrs
                .iter()
                .filter(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_total")))
                .count()
                >= 2
        );
    }

    #[test]
    fn lower_mutable_local_reassignment_to_store_path() {
        let src = r#"
            fn main() {
                let mut score: i32 = 0;
                score = 1;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("mutable local reassignment should lower");
        let main = &ir[0];
        assert!(
            main.instrs
                .iter()
                .filter(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_score")))
                .count()
                >= 2
        );
    }

    #[test]
    fn lower_plain_local_reassignment_to_store_path() {
        let src = r#"
            fn main() {
                let score: i32 = 0;
                score = 1;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("plain local reassignment should lower");
        let main = &ir[0];
        assert!(
            main.instrs
                .iter()
                .filter(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_score")))
                .count()
                >= 2
        );
    }

    #[test]
    fn lower_discard_bind_evaluates_rhs_without_store() {
        let src = r#"
            fn main() {
                let _ = 1.0 + 2.0;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("discard bind should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(!main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name == "_"
        )));
    }

    #[test]
    fn lower_assert_statement_to_assert_ir() {
        let src = r#"
            fn main() {
                assert(true);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("assert statement should lower");
        let main = &ir[0];
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::Assert { .. })));
    }

    #[test]
    fn lower_function_requires_clause_to_entry_asserts() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn decide(ctx: DecisionContext, expected: quad) -> quad
                requires(ctx.camera == expected)
                requires(ctx.quality == 0.75) {
                return ctx.camera;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let seen: quad = decide(ctx, T);
                assert(seen == T);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("requires clause should lower");
        let decide = ir
            .iter()
            .find(|func| func.name == "decide")
            .expect("decide fn");
        let first_assert = decide
            .instrs
            .iter()
            .position(|instr| matches!(instr, IrInstr::Assert { .. }))
            .expect("requires clause should lower to assert");
        let param_store = decide
            .instrs
            .iter()
            .position(
                |instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_ctx")),
            )
            .expect("parameter store should exist");
        assert!(param_store < first_assert);
        let assert_count = decide
            .instrs
            .iter()
            .filter(|instr| matches!(instr, IrInstr::Assert { .. }))
            .count();
        assert_eq!(assert_count, 2);
    }

    #[test]
    fn lower_function_ensures_clause_to_exit_asserts() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn decide(ctx: DecisionContext) -> quad
                ensures(result == ctx.camera)
                ensures(ctx.quality == 0.75) {
                return ctx.camera;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let seen: quad = decide(ctx);
                assert(seen == T);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("ensures clause should lower");
        let decide = ir
            .iter()
            .find(|func| func.name == "decide")
            .expect("decide fn");
        let ret_index = decide
            .instrs
            .iter()
            .position(|instr| matches!(instr, IrInstr::Ret { src: Some(_) }))
            .expect("return should exist");
        let result_store = decide
            .instrs
            .iter()
            .position(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name == "result"))
            .expect("ensures should store return value into synthetic result binding");
        let assert_positions: Vec<_> = decide
            .instrs
            .iter()
            .enumerate()
            .filter_map(|(idx, instr)| matches!(instr, IrInstr::Assert { .. }).then_some(idx))
            .collect();
        assert_eq!(assert_positions.len(), 2);
        assert!(result_store < assert_positions[0]);
        assert!(assert_positions[0] < ret_index);
        assert!(assert_positions[1] < ret_index);
    }

    #[test]
    fn lower_function_invariant_clauses_to_entry_and_exit_asserts() {
        let src = r#"
            fn keep(flag: bool) -> bool
                invariant(flag == true)
                invariant(result == flag) {
                return flag;
            }

            fn main() {
                let seen: bool = keep(true);
                assert(seen == true);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("invariant clauses should lower");
        let keep = ir.iter().find(|func| func.name == "keep").expect("keep fn");
        let ret_index = keep
            .instrs
            .iter()
            .position(|instr| matches!(instr, IrInstr::Ret { src: Some(_) }))
            .expect("return should exist");
        let result_store = keep
            .instrs
            .iter()
            .position(|instr| matches!(instr, IrInstr::StoreVar { name, .. } if name == "result"))
            .expect("exit invariant path should store synthetic result binding");
        let assert_positions: Vec<_> = keep
            .instrs
            .iter()
            .enumerate()
            .filter_map(|(idx, instr)| matches!(instr, IrInstr::Assert { .. }).then_some(idx))
            .collect();
        assert_eq!(assert_positions.len(), 3);
        assert!(assert_positions[0] < result_store);
        assert!(result_store < assert_positions[1]);
        assert!(result_store < assert_positions[2]);
        assert!(assert_positions[2] < ret_index);
    }

    #[test]
    fn lower_tuple_literal_to_make_tuple_ir() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) {
                return (1, flag);
            }

            fn main() {
                let pair: (i32, bool) = pair(true);
                assert(pair == (1, true));
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("tuple literal should lower");
        let pair_fn = ir.iter().find(|func| func.name == "pair").expect("pair fn");
        assert!(pair_fn
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::MakeTuple { items, .. } if items.len() == 2)));
    }

    #[test]
    fn lower_tuple_destructuring_bind_to_tuple_get_ir() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let (count, ready): (i32, bool) = pair(true);
                assert(ready == true);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("tuple destructuring bind should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::TupleGet { index: 0, .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::TupleGet { index: 1, .. })));
    }

    #[test]
    fn lower_adt_match_borrow_capture_records_borrow_path_event() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }
            fn use_e(e: Maybe) -> f64 {
                let total: f64 = match e {
                    Maybe::Some(ref v) => { 1.0 }
                    _ => { 0.0 }
                };
                return total;
            }
            fn main() { return; }
        "#;

        let (program, func) = lower_single_function_with_program(src, "use_e");
        let use_e_func = program
            .functions
            .iter()
            .find(|f| program.arena.symbol_name(f.name) == "use_e")
            .unwrap();
        let e_name = use_e_func.params[0].0;

        let adt = program
            .adts
            .iter()
            .find(|adt| program.arena.symbol_name(adt.name) == "Maybe")
            .unwrap();
        let variant = adt
            .variants
            .iter()
            .find(|v| program.arena.symbol_name(v.name) == "Some")
            .unwrap();

        assert_eq!(
            func.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Borrow,
                path: AccessPath::new(e_name).adt_payload(variant.name, 0),
            }]
        );
    }

    #[test]
    fn lower_adt_match_move_capture_does_not_record_borrow_path_event() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }
            fn use_e(e: Maybe) -> f64 {
                let total: f64 = match e {
                    Maybe::Some(v) => { 1.0 }
                    _ => { 0.0 }
                };
                return total;
            }
            fn main() { return; }
        "#;

        let (_, func) = lower_single_function_with_program(src, "use_e");

        assert_eq!(func.ownership_events, vec![]);
    }

    #[test]
    fn lower_option_ref_emits_borrow_path() {
        let src = r#"
            fn use_opt(opt: Option(f64)) -> f64 {
                let total: f64 = match opt {
                    Option::Some(ref value) => { 1.0 }
                    Option::None => { 0.0 }
                };
                return total;
            }
            fn main() { return; }
        "#;

        let (mut program, func) = lower_single_function_with_program(src, "use_opt");

        let opt_name = program.arena.intern_symbol("opt");
        let some_name = program.arena.intern_symbol("Some");

        assert_eq!(
            func.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Borrow,
                path: AccessPath::new(opt_name).adt_payload(some_name, 0),
            }]
        );
    }

    #[test]
    fn lower_option_move_does_not_emit_borrow() {
        let src = r#"
            fn use_opt(opt: Option(f64)) -> f64 {
                let total: f64 = match opt {
                    Option::Some(value) => { 1.0 }
                    Option::None => { 0.0 }
                };
                return total;
            }
            fn main() { return; }
        "#;

        let (_, func) = lower_single_function_with_program(src, "use_opt");

        assert_eq!(func.ownership_events, vec![]);
    }

    #[test]
    fn lower_result_ref_emits_borrow_path() {
        let src = r#"
            fn use_result(res: Result(f64, i32)) -> f64 {
                let total: f64 = match res {
                    Result::Ok(ref value) => { 1.0 }
                    Result::Err(err) => { 0.0 }
                };
                return total;
            }
            fn main() { return; }
        "#;

        let (mut program, func) = lower_single_function_with_program(src, "use_result");

        let res_name = program.arena.intern_symbol("res");
        let ok_name = program.arena.intern_symbol("Ok");

        assert_eq!(
            func.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Borrow,
                path: AccessPath::new(res_name).adt_payload(ok_name, 0),
            }]
        );
    }

    #[test]
    fn lower_result_err_ref_emits_borrow_path() {
        let src = r#"
            fn use_result(res: Result(f64, i32)) -> f64 {
                let total: f64 = match res {
                    Result::Ok(value) => { 1.0 }
                    Result::Err(ref err) => { 0.0 }
                };
                return total;
            }
            fn main() { return; }
        "#;

        let (mut program, func) = lower_single_function_with_program(src, "use_result");

        let res_name = program.arena.intern_symbol("res");
        let err_name = program.arena.intern_symbol("Err");

        assert_eq!(
            func.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Borrow,
                path: AccessPath::new(res_name).adt_payload(err_name, 0),
            }]
        );
    }

    #[test]
    fn lower_tuple_borrow_capture_records_borrow_path_event() {
        let src = r#"
            fn pair() -> (i32, i32) = (1, 2);

            fn main() {
                let pair: (i32, i32) = pair();
                let (ref left, _): (i32, i32) = pair;
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let {
            name: pair_name, ..
        } = program.arena.stmt(main_fn.body[0])
        else {
            panic!("expected pair binding");
        };
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Borrow,
                path: AccessPath::new(*pair_name).tuple_index(0),
            }]
        );
    }

    #[test]
    fn lower_tuple_elements_emit_distinct_ownership_paths() {
        let src = r#"
            fn pair() -> (i32, i32) = (1, 2);

            fn main() {
                let pair: (i32, i32) = pair();
                let (ref left, ref right): (i32, i32) = pair;
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let {
            name: pair_name, ..
        } = program.arena.stmt(main_fn.body[0])
        else {
            panic!("expected pair binding");
        };
        assert_eq!(
            main.ownership_events,
            vec![
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*pair_name).tuple_index(0),
                },
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*pair_name).tuple_index(1),
                },
            ]
        );
    }

    #[test]
    fn lower_tuple_destructuring_assignment_to_tuple_get_ir() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let count: i32 = 0;
                let ready: bool = false;
                (count, ready) = pair(true);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("tuple destructuring assignment should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::TupleGet { index: 0, .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::TupleGet { index: 1, .. })));
    }

    #[test]
    fn lower_tuple_assignment_records_write_path_events() {
        let src = r#"
            fn pair() -> (i32, bool) = (1, true);

            fn main() {
                let count: i32 = 0;
                let ready: bool = false;
                (count, ready) = pair();
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let {
            name: count_name, ..
        } = program.arena.stmt(main_fn.body[0])
        else {
            panic!("expected count binding");
        };
        let Stmt::Let {
            name: ready_name, ..
        } = program.arena.stmt(main_fn.body[1])
        else {
            panic!("expected ready binding");
        };
        assert_eq!(
            main.ownership_events,
            vec![
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Write,
                    path: AccessPath::new(*count_name),
                },
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Write,
                    path: AccessPath::new(*ready_name),
                },
            ]
        );
    }

    #[test]
    fn lower_tuple_let_else_to_tuple_get_and_early_return_ir() {
        let src = r#"
            fn pair() -> (i32, quad) = (1, T);

            fn main() {
                let (count, T): (i32, quad) = pair() else return;
                assert(count == 1);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("tuple let-else should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::TupleGet { index: 0, .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::TupleGet { index: 1, .. })));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::LoadQ {
                val: QuadVal::T,
                ..
            }
        )));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpEq { .. })));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::JmpIf { label, .. } if label.starts_with("let_else_tuple_")
        )));
        assert!(
            main.instrs
                .iter()
                .filter(|instr| matches!(instr, IrInstr::Ret { .. }))
                .count()
                >= 2
        );
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_count")
        )));
    }

    #[test]
    fn lower_where_clause_via_existing_block_path() {
        let src = r#"
            fn magnitude_sq(x: f64, y: f64) -> f64 =
                total where
                    xx = x * x,
                    yy = y * y,
                    total = xx + yy;

            fn main() {
                let value: f64 = magnitude_sq(3.0, 4.0);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("where-clause should lower");
        let func = ir
            .iter()
            .find(|func| func.name == "magnitude_sq")
            .expect("magnitude_sq fn");
        assert!(func.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_xx")
        )));
        assert!(func.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_yy")
        )));
        assert!(func.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_total")
        )));
    }

    #[test]
    fn lower_range_literal_to_hidden_tuple_carrier() {
        let src = r#"
            fn main() {
                let interval = 0..=10;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("range literal should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadI32 { val: 0, .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadI32 { val: 10, .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadBool { val: true, .. })));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeTuple { items, .. } if items.len() == 3
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_interval")
        )));
    }

    #[test]
    fn lower_for_range_to_i32_compare_and_increment_path() {
        let src = r#"
            fn main() {
                for i in 0..=2 {
                    assert(i == i);
                }
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("for-range should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpI32Le { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpI32Lt { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddI32 { .. })));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_i")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.starts_with("__for_range_")
        )));
    }

    #[test]
    fn lower_for_range_through_variable_keeps_existing_execution_path() {
        let src = r#"
            fn main() {
                let window = 0..=2;
                for i in window {
                    assert(i == i);
                }
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("range-valued variable loop should still lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(
            |instr| matches!(instr, IrInstr::LoadVar { name, .. } if name.ends_with("_window"))
        ));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpI32Le { .. })));
    }

    #[test]
    fn lower_i32_relational_surface_through_existing_cmp_path() {
        let src = r#"
            fn main() {
                let lt: bool = 1 < 2;
                let ge: bool = 3 >= 3;
                assert(lt == true);
                assert(ge == true);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("i32 relational surface should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpI32Lt { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpI32Le { .. })));
    }

    #[test]
    fn lower_sequence_iterable_loop_to_length_and_index_path() {
        let src = r#"
            fn main() {
                let items: Sequence(i32) = [1, 2, 3];
                for item in items {
                    assert(item == item);
                }
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("Sequence(T) iterable loop should now lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::SequenceLen { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::SequenceGet { .. })));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_item")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.starts_with("__for_each_seq_")
        )));
    }

    #[test]
    fn compile_program_lowers_explicit_iterable_impl_dispatch() {
        let src = r#"
            trait Iterable {
                fn next(self: Self, index: i32) -> Option(i32);
            }

            record Numbers {
                current: i32,
            }

            impl Iterable for Numbers {
                fn next(self: Self, index: i32) -> Option(i32) {
                    if index == 0 {
                        return Option::Some(0);
                    }
                    if index == 1 {
                        return Option::Some(1);
                    }
                    if index == 2 {
                        return Option::Some(index);
                    }
                    return Option::None;
                }
            }

            fn main() {
                let numbers: Numbers = Numbers { current: 0 };
                for value in numbers {
                    let _ = value;
                }
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("explicit Iterable impl loop should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Call { name, args, .. } if name == "__impl::Iterable::Numbers::next" && args.len() == 2
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::AdtTag { adt_name, .. } if adt_name == "Option"
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::AdtGet { adt_name, index, .. } if adt_name == "Option" && *index == 0
        )));
    }

    #[test]
    fn compile_program_rejects_explicit_iterable_impl_with_wrong_contract() {
        let src = r#"
            trait Iterable {
                fn next(self: Self) -> Option(i32);
            }

            record Numbers {
                current: i32,
            }

            impl Iterable for Numbers {
                fn next(self: Self) -> Option(i32) {
                    return Option::None;
                }
            }

            fn main() {
                let numbers: Numbers = Numbers { current: 0 };
                for value in numbers {
                    let _ = value;
                }
                return;
            }
        "#;

        let err =
            compile_program_to_ir(src).expect_err("wrong executable Iterable contract must reject");
        assert!(err
            .message
            .contains("fn next(self: Self, index: i32) -> Option(Item)"));
    }

    #[test]
    fn compile_program_lowers_impl_methods_to_internal_functions() {
        let src = r#"
            trait Iterable {
                fn next(self: Self, index: i32) -> Option(i32);
            }

            record Numbers {
                current: i32,
            }

            impl Iterable for Numbers {
                fn next(self: Self, index: i32) -> Option(i32) {
                    let _ = index;
                    return Option::None;
                }
            }

            fn main() {
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("impl methods should now lower to IR");
        assert!(ir
            .iter()
            .any(|func| func.name == "__impl::Iterable::Numbers::next"));
    }

    #[test]
    fn compile_program_with_top_level_record_declaration_and_ordinary_main() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                return;
            }
        "#;

        let ir = compile_program_to_ir(src)
            .expect("record declaration should not break ordinary lowering");
        assert_eq!(ir.len(), 1);
        assert_eq!(ir[0].name, "main");
    }

    #[test]
    fn lower_record_param_return_and_safe_equality_path() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            fn echo(ctx: DecisionContext) -> DecisionContext {
                return ctx;
            }

            fn main() {
                let left: DecisionContext = DecisionContext { camera: T };
                let right: DecisionContext = echo(left);
                assert(right == right);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("record params/returns should lower");
        assert!(ir.iter().any(|func| func.name == "echo"));
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Call { name, .. } if name == "echo"
        )));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpEq { .. })));
    }

    #[test]
    fn lower_record_literal_to_make_record_in_declaration_slot_order() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let mirror: DecisionContext = ctx;
                let _ = mirror;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("record literal should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeRecord { name, items, .. } if name == "DecisionContext" && items.len() == 2
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_ctx")
        )));
    }

    #[test]
    fn lower_enum_constructor_to_make_adt_ir() {
        let src = r#"
            enum Maybe {
                None,
                Some(bool),
            }

            fn choose(flag: bool) -> Maybe {
                return Maybe::Some(flag);
            }

            fn main() {
                let value: Maybe = choose(true);
                let fallback: Maybe = Maybe::None;
                let _ = value;
                let _ = fallback;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("enum constructor should lower");
        let choose = ir
            .iter()
            .find(|func| func.name == "choose")
            .expect("choose fn");
        assert!(choose.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeAdt { adt_name, variant_name, tag, items, .. }
                if adt_name == "Maybe" && variant_name == "Some" && *tag == 1 && items.len() == 1
        )));
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeAdt { adt_name, variant_name, tag, items, .. }
                if adt_name == "Maybe" && variant_name == "None" && *tag == 0 && items.is_empty()
        )));
    }

    #[test]
    fn lower_option_and_result_standard_forms_to_canonical_make_adt_ir() {
        let src = r#"
            fn keep(flag: bool) -> Option(bool) {
                let fallback: Option(bool) = Option::None;
                let _ = fallback;
                return Option::Some(flag);
            }

            fn settle(flag: bool) -> Result(bool, quad) {
                if flag {
                    let value: Result(bool, quad) = Result::Ok(true);
                    return value;
                }
                let value: Result(bool, quad) = Result::Err(N);
                return value;
            }

            fn main() {
                let left: Option(bool) = keep(true);
                let right: Result(bool, quad) = settle(false);
                let _ = left;
                let _ = right;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("Option/Result standard forms should lower");
        let keep = ir.iter().find(|func| func.name == "keep").expect("keep fn");
        assert!(keep.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeAdt { adt_name, variant_name, tag, items, .. }
                if adt_name == "Option" && variant_name == "None" && *tag == 0 && items.is_empty()
        )));
        assert!(keep.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeAdt { adt_name, variant_name, tag, items, .. }
                if adt_name == "Option" && variant_name == "Some" && *tag == 1 && items.len() == 1
        )));
        let settle = ir
            .iter()
            .find(|func| func.name == "settle")
            .expect("settle fn");
        assert!(settle.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeAdt { adt_name, variant_name, tag, items, .. }
                if adt_name == "Result" && variant_name == "Ok" && *tag == 0 && items.len() == 1
        )));
        assert!(settle.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeAdt { adt_name, variant_name, tag, items, .. }
                if adt_name == "Result" && variant_name == "Err" && *tag == 1 && items.len() == 1
        )));
    }

    #[test]
    fn lower_option_and_result_match_patterns_to_existing_adt_tag_path() {
        let src = r#"
            fn unwrap(opt: Option(bool)) -> bool {
                let out: bool = match opt {
                    Option::Some(value) => { value }
                    Option::None => { false }
                };
                return out;
            }

            fn settle(res: Result(quad, quad)) -> quad {
                let out: quad = match res {
                    Result::Ok(value) => { value }
                    Result::Err(code) => { code }
                };
                return out;
            }

            fn main() {
                let left: bool = unwrap(Option::Some(true));
                let right: quad = settle(Result::Err(S));
                assert(left == true);
                assert(right == S);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("Option/Result match ergonomics should lower");
        let unwrap = ir
            .iter()
            .find(|func| func.name == "unwrap")
            .expect("unwrap fn");
        assert!(unwrap.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::AdtTag { adt_name, .. } if adt_name == "Option"
        )));
        assert!(unwrap.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::AdtGet { adt_name, index, .. } if adt_name == "Option" && *index == 0
        )));
        let settle = ir
            .iter()
            .find(|func| func.name == "settle")
            .expect("settle fn");
        assert!(settle.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::AdtTag { adt_name, .. } if adt_name == "Result"
        )));
        assert!(settle.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::AdtGet { adt_name, index, .. } if adt_name == "Result" && *index == 0
        )));
    }

    #[test]
    fn lower_units_of_measure_through_existing_numeric_ir_path() {
        let src = r#"
            record Measurement {
                distance: f64[m],
            }

            fn echo(distance: f64[m], sample: Measurement) -> f64[m] {
                let total: f64[m] = distance + sample.distance;
                let same: bool = total == distance;
                assert(same == false || same == true);
                return total;
            }

            fn main() {
                let sample: Measurement = Measurement { distance: 2.0 };
                let total: f64[m] = echo(3.0, sample);
                let expected: f64[m] = 5.0;
                assert(total == expected);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("units-of-measure values should lower");
        let echo = ir.iter().find(|func| func.name == "echo").expect("echo fn");
        assert!(echo
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::RecordGet { record_name, index, .. } if record_name == "Measurement" && *index == 0)));
        assert!(echo
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::AddF64 { .. })));
        assert!(echo
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::CmpEq { .. })));

        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadF64 { val, .. } if (*val - 2.0).abs() < f64::EPSILON)));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadF64 { val, .. } if (*val - 3.0).abs() < f64::EPSILON)));
        assert!(!main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::MakeTuple { .. })));
    }

    #[test]
    fn lower_measured_u32_literal_through_existing_integer_carrier() {
        let src = r#"
            fn main() {
                let ticks: u32[ms] = 1_000u32;
                let _ = ticks;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("measured u32 literal should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::LoadU32 { val, .. } if *val == 1000)));
    }

    #[test]
    fn lower_record_field_access_to_record_get_slot() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let seen: quad = ctx.camera;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("record field access should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::RecordGet { record_name, index, .. }
                if record_name == "DecisionContext" && *index == 0
        )));
    }

    #[test]
    fn lower_record_copy_with_to_record_get_and_make_record_ir() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let patched: DecisionContext = ctx with { quality: 1.0 };
                assert(patched.camera == T);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("record copy-with should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::RecordGet { record_name, index, .. }
                if record_name == "DecisionContext" && *index == 0
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeRecord { name, items, .. }
                if name == "DecisionContext" && items.len() == 2
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_patched")
        )));
    }

    #[test]
    fn lower_record_punning_shorthand_via_existing_record_paths() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let camera: quad = T;
                let quality: f64 = 0.75;
                let ctx: DecisionContext = DecisionContext { camera, quality };
                let patched: DecisionContext = ctx with { quality };
                assert(patched.camera == T);
                assert(patched.quality == 0.75);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("record punning shorthand should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::MakeRecord { name, items, .. }
                if name == "DecisionContext" && items.len() == 2
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::RecordGet { record_name, .. }
                if record_name == "DecisionContext"
        )));
    }

    #[test]
    fn lower_record_destructuring_bind_to_record_get_ir() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: seen_camera, quality: _ } =
                    DecisionContext { quality: 0.75, camera: T };
                assert(seen_camera == T);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("record destructuring bind should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::RecordGet { record_name, index, .. }
                if record_name == "DecisionContext" && *index == 0
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_seen_camera")
        )));
    }

    #[test]
    fn lower_record_borrow_capture_records_borrow_path_event() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let DecisionContext { camera: ref seen_camera, quality: _ } = ctx;
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let camera_field = program.records[0].fields[0].name;
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Borrow,
                path: AccessPath::new(*ctx_name).field(camera_field),
            }]
        );
    }

    #[test]
    fn lower_record_copy_with_emits_field_write_events() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { quality: 0.75, camera: T };
                let patched: DecisionContext = ctx with { quality: 1.0 };
                assert(patched.camera == T);
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new(*ctx_name).field(quality_field),
            }]
        );
    }

    #[test]
    fn lower_record_borrow_capture_records_distinct_field_paths() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let DecisionContext { camera: ref seen_camera, quality: ref seen_quality } = ctx;
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let camera_field = program.records[0].fields[0].name;
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*ctx_name).field(camera_field),
                },
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*ctx_name).field(quality_field),
                },
            ]
        );
    }

    #[test]
    fn lower_record_copy_with_emits_distinct_field_write_events() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let patched: DecisionContext = ctx with { quality: 1.0, camera: T };
                assert(patched.camera == T);
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let camera_field = program.records[0].fields[0].name;
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Write,
                    path: AccessPath::new(*ctx_name).field(quality_field),
                },
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Write,
                    path: AccessPath::new(*ctx_name).field(camera_field),
                },
            ]
        );
    }

    #[test]
    fn lower_sequence_index_borrow_records_sequence_index_static_path() {
        let src = r#"
            fn main() {
                let seq: Sequence((i32, i32)) = [(1, 2), (3, 4)];
                let (ref left, ref right): (i32, i32) = seq[0];
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: seq_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected sequence binding");
        };
        assert_eq!(
            main.ownership_events,
            vec![
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*seq_name)
                        .sequence_index_static(0)
                        .tuple_index(0),
                },
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*seq_name)
                        .sequence_index_static(0)
                        .tuple_index(1),
                },
            ]
        );
    }

    #[test]
    fn lower_sequence_index_borrow_encodes_sequence_index_static_component() {
        let src = r#"
            fn main() {
                let seq: Sequence((i32, i32)) = [(1, 2), (3, 4)];
                let (ref left, ref right): (i32, i32) = seq[0];
                return;
            }
        "#;

        let bytes = compile_program_to_semcode(src).expect("sequence semcode should emit");
        let (_, functions) = decode_semcode_envelope(&bytes).expect("decode sequence semcode");
        let main = functions
            .iter()
            .find(|func| func.name == "main")
            .expect("main fn");
        assert_eq!(main.borrowed_paths.len(), 2);
        let root_symbol_id = main.borrowed_paths[0].root_symbol_id;
        assert_eq!(main.borrowed_paths[1].root_symbol_id, root_symbol_id);
        assert_eq!(
            main.borrowed_paths[0].components,
            vec![
                DecodedAccessPathComponent::SequenceIndexStatic(0),
                DecodedAccessPathComponent::TupleIndex(0),
            ]
        );
        assert_eq!(
            main.borrowed_paths[1].components,
            vec![
                DecodedAccessPathComponent::SequenceIndexStatic(0),
                DecodedAccessPathComponent::TupleIndex(1),
            ]
        );
    }

    #[test]
    fn lower_sequence_indexes_emit_distinct_static_paths() {
        let src = r#"
            fn main() {
                let seq: Sequence((i32, i32)) = [(1, 2), (3, 4)];
                let (ref left0, ref right0): (i32, i32) = seq[0];
                let (ref left1, ref right1): (i32, i32) = seq[1];
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: seq_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected sequence binding");
        };
        assert_eq!(
            main.ownership_events,
            vec![
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*seq_name)
                        .sequence_index_static(0)
                        .tuple_index(0),
                },
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*seq_name)
                        .sequence_index_static(0)
                        .tuple_index(1),
                },
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*seq_name)
                        .sequence_index_static(1)
                        .tuple_index(0),
                },
                OwnershipPathEvent {
                    kind: OwnershipPathEventKind::Borrow,
                    path: AccessPath::new(*seq_name)
                        .sequence_index_static(1)
                        .tuple_index(1),
                },
            ]
        );
    }

    #[test]
    fn lower_record_let_else_to_record_get_and_early_return_ir() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: T, quality: score } =
                    DecisionContext { quality: 0.75, camera: T } else return;
                assert(score == 0.75);
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("record let-else should lower");
        let main = ir.iter().find(|func| func.name == "main").expect("main fn");
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::RecordGet { record_name, index, .. }
                if record_name == "DecisionContext" && (*index == 0 || *index == 1)
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::LoadQ {
                val: QuadVal::T,
                ..
            }
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::JmpIf { label, .. } if label.starts_with("let_else_record_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.ends_with("_score")
        )));
    }

    #[test]
    fn lower_loop_expression_with_break_value_to_labels_and_result_slot() {
        let src = r#"
            fn main() {
                let total: f64 = loop {
                    if true {
                        break 1.0;
                    } else {
                        break 2.0;
                    }
                };
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("loop expression should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("loop_expr_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::StoreVar { name, .. } if name.starts_with("__loop_expr_")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::LoadVar { name, .. } if name.starts_with("__loop_expr_")
        )));
    }

    #[test]
    fn lower_while_statement_to_test_body_and_end_labels() {
        let src = r#"
            fn main() {
                let mut i: i32 = 0;
                while i < 3 {
                    i = i + 1;
                }
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("while statement should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("while_") && name.ends_with("_test")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("while_") && name.ends_with("_body")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("while_") && name.ends_with("_end")
        )));
    }

    #[test]
    fn lower_statement_loop_with_continue_and_bare_break() {
        let src = r#"
            fn main() {
                let mut i: i32 = 0;
                loop {
                    i = i + 1;
                    if i < 3 {
                        continue;
                    }
                    break;
                }
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("statement loop should lower");
        let main = &ir[0];
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("loop_stmt_") && name.ends_with("_start")
        )));
        assert!(main.instrs.iter().any(|instr| matches!(
            instr,
            IrInstr::Label { name } if name.starts_with("loop_stmt_") && name.ends_with("_end")
        )));
    }

    #[test]
    fn lower_ufcs_method_call_to_ordinary_call_order() {
        let method_src = r#"
            fn scale(value: f64, factor: f64) -> f64 = value * factor;

            fn main() {
                let total: f64 = 2.0.scale(3.0);
                return;
            }
        "#;

        let plain_src = r#"
            fn scale(value: f64, factor: f64) -> f64 = value * factor;

            fn main() {
                let total: f64 = scale(2.0, 3.0);
                return;
            }
        "#;

        let method_ir = compile_program_to_ir(method_src).expect("UFCS method call should lower");
        let plain_ir = compile_program_to_ir(plain_src).expect("plain call should lower");
        let method_main = method_ir
            .iter()
            .find(|func| func.name == "main")
            .expect("main fn");
        let plain_main = plain_ir
            .iter()
            .find(|func| func.name == "main")
            .expect("main fn");
        assert_eq!(method_main.instrs, plain_main.instrs);
    }

    #[test]
    fn lowering_match_expression_rejects_branch_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T => { 1.0 }
                    _ => { true }
                };
                return;
            }
        "#;

        let err = compile_program_to_ir(src)
            .expect_err("mismatched match expression branches must reject");
        assert!(err
            .message
            .contains("match expression branch type mismatch"));
    }

    #[test]
    fn opt_removes_unreachable_and_noop_jmp() {
        let mut ir = vec![IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::Label {
                    name: "entry".to_string(),
                },
                IrInstr::Jmp {
                    label: "l1".to_string(),
                },
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::Label {
                    name: "l1".to_string(),
                },
                IrInstr::Ret { src: None },
            ],
            ownership_events: Vec::new(),
            params: Vec::new(),
        }];
        let report = run_default_opt_passes(&mut ir);
        assert!(report.changed);
        assert!(matches!(ir[0].instrs[0], IrInstr::Label { .. }));
        assert!(ir[0]
            .instrs
            .iter()
            .all(|i| !matches!(i, IrInstr::LoadBool { dst: 0, val: true })));
    }

    #[test]
    fn opt_removes_redundant_consecutive_loads() {
        let mut ir = vec![IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadI32 { dst: 1, val: 10 },
                IrInstr::LoadI32 { dst: 1, val: 11 },
                IrInstr::Ret { src: Some(1) },
            ],
            ownership_events: Vec::new(),
            params: Vec::new(),
        }];
        let report = run_default_opt_passes(&mut ir);
        assert!(report.changed);
        let loads = ir[0]
            .instrs
            .iter()
            .filter(|i| matches!(i, IrInstr::LoadI32 { dst: 1, .. }))
            .count();
        assert_eq!(loads, 1);
        assert!(matches!(
            ir[0].instrs[0],
            IrInstr::LoadI32 { dst: 1, val: 11 }
        ));
    }

    #[test]
    fn opt_folds_bool_and_f64_constants() {
        let f = IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::LoadBool { dst: 1, val: false },
                IrInstr::BoolAnd {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::LoadF64 { dst: 3, val: 2.0 },
                IrInstr::LoadF64 { dst: 4, val: 3.0 },
                IrInstr::AddF64 {
                    dst: 5,
                    lhs: 3,
                    rhs: 4,
                },
                IrInstr::Ret { src: Some(5) },
            ],
            ownership_events: Vec::new(),
            params: Vec::new(),
        };
        let mut ir = vec![f];
        let report = crate::passes::run_default_opt_passes(&mut ir);
        assert!(report.changed);
        let f = &ir[0];
        assert!(f
            .instrs
            .iter()
            .any(|i| matches!(i, IrInstr::LoadBool { dst: 2, val: false })));
        assert!(f.instrs.iter().any(|i| matches!(
            i,
            IrInstr::LoadF64 { dst: 5, val } if (*val - 5.0).abs() < f64::EPSILON
        )));
    }

    #[test]
    fn qtruth_instructions_are_distinct_from_legacy_lattice_instructions() {
        assert_ne!(
            IrInstr::QTruthAnd {
                dst: 0,
                lhs: 1,
                rhs: 2,
            },
            IrInstr::QAnd {
                dst: 0,
                lhs: 1,
                rhs: 2,
            }
        );
        assert_ne!(
            IrInstr::QTruthOr {
                dst: 0,
                lhs: 1,
                rhs: 2,
            },
            IrInstr::QOr {
                dst: 0,
                lhs: 1,
                rhs: 2,
            }
        );
        assert_ne!(
            IrInstr::QTruthNot { dst: 0, src: 1 },
            IrInstr::QNot { dst: 0, src: 1 }
        );
        assert_ne!(
            IrInstr::QTruthImpl {
                dst: 0,
                lhs: 1,
                rhs: 2,
            },
            IrInstr::QImpl {
                dst: 0,
                lhs: 1,
                rhs: 2,
            }
        );
    }

    #[test]
    fn qtruth_and_legacy_instructions_encode_to_distinct_opcode_bytes() {
        fn encode(instr: IrInstr) -> Vec<u8> {
            let mut out = Vec::new();
            emit_instr(
                &instr,
                &HashMap::new(),
                &StringInterner::default(),
                &mut out,
            )
            .expect("instruction should encode");
            out
        }

        assert_eq!(
            encode(IrInstr::QTruthAnd {
                dst: 1,
                lhs: 2,
                rhs: 3,
            }),
            vec![Opcode::QTruthAnd.byte(), 1, 0, 2, 0, 3, 0]
        );
        assert_eq!(
            encode(IrInstr::QTruthOr {
                dst: 1,
                lhs: 2,
                rhs: 3,
            }),
            vec![Opcode::QTruthOr.byte(), 1, 0, 2, 0, 3, 0]
        );
        assert_eq!(
            encode(IrInstr::QTruthNot { dst: 1, src: 2 }),
            vec![Opcode::QTruthNot.byte(), 1, 0, 2, 0]
        );
        assert_eq!(
            encode(IrInstr::QTruthImpl {
                dst: 1,
                lhs: 2,
                rhs: 3,
            }),
            vec![Opcode::QTruthImpl.byte(), 1, 0, 2, 0, 3, 0]
        );

        assert_eq!(
            encode(IrInstr::QAnd {
                dst: 1,
                lhs: 2,
                rhs: 3,
            }),
            vec![Opcode::QAnd.byte(), 1, 0, 2, 0, 3, 0]
        );
        assert_eq!(
            encode(IrInstr::QOr {
                dst: 1,
                lhs: 2,
                rhs: 3,
            }),
            vec![Opcode::QOr.byte(), 1, 0, 2, 0, 3, 0]
        );
        assert_eq!(
            encode(IrInstr::QNot { dst: 1, src: 2 }),
            vec![Opcode::QNot.byte(), 1, 0, 2, 0]
        );
        assert_eq!(
            encode(IrInstr::QImpl {
                dst: 1,
                lhs: 2,
                rhs: 3,
            }),
            vec![Opcode::QImpl.byte(), 1, 0, 2, 0, 3, 0]
        );
    }

    #[test]
    fn qtruth_instructions_survive_semcode_envelope_emission() {
        fn assert_opcode_in_envelope(instr: IrInstr, expected_opcode: u8) {
            let semcode = emit_ir_to_semcode(
                &[IrFunction {
                    name: "main".to_string(),
                    instrs: vec![
                        IrInstr::LoadQ {
                            dst: 0,
                            val: QuadVal::N,
                        },
                        IrInstr::LoadQ {
                            dst: 1,
                            val: QuadVal::T,
                        },
                        instr,
                        IrInstr::Ret { src: Some(2) },
                    ],
                    ownership_events: Vec::new(),
                    params: Vec::new(),
                }],
                false,
            )
            .expect("QTruth IR should emit through the SemCode envelope");

            let (_, functions) =
                decode_semcode_envelope(&semcode).expect("SemCode envelope should decode");
            let main = functions
                .iter()
                .find(|function| function.name == "main")
                .expect("main function should be present");
            let instruction_stream = &main.code_slice[main.instr_start_offset..];
            assert_eq!(instruction_stream[8], expected_opcode);
        }

        assert_opcode_in_envelope(
            IrInstr::QTruthAnd {
                dst: 2,
                lhs: 0,
                rhs: 1,
            },
            Opcode::QTruthAnd.byte(),
        );
        assert_opcode_in_envelope(
            IrInstr::QTruthOr {
                dst: 2,
                lhs: 0,
                rhs: 1,
            },
            Opcode::QTruthOr.byte(),
        );
        assert_opcode_in_envelope(
            IrInstr::QTruthNot { dst: 2, src: 0 },
            Opcode::QTruthNot.byte(),
        );
        assert_opcode_in_envelope(
            IrInstr::QTruthImpl {
                dst: 2,
                lhs: 0,
                rhs: 1,
            },
            Opcode::QTruthImpl.byte(),
        );
    }

    #[test]
    fn qtruth_intrinsics_lower_to_explicit_ir_variants() {
        let src = r#"
            fn main() {
                let a: quad = qtruth_and(T, F);
                let b: quad = qtruth_or(T, F);
                let c: quad = qtruth_not(T);
                let d: quad = qtruth_impl(T, F);
                return;
            }
        "#;
        let ir = compile_program_to_ir(src).expect("QTruth intrinsics should lower");
        let main = ir
            .iter()
            .find(|function| function.name == "main")
            .expect("main function should be present");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::QTruthAnd { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::QTruthOr { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::QTruthNot { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::QTruthImpl { .. })));
    }

    #[test]
    fn legacy_quad_operators_keep_legacy_ir_variants() {
        let src = r#"
            fn main() {
                let a: quad = T && F;
                let b: quad = T || F;
                let c: quad = !T;
                let d: quad = T -> F;
                return;
            }
        "#;
        let ir = compile_program_to_ir(src).expect("legacy quad operators should lower");
        let main = ir
            .iter()
            .find(|function| function.name == "main")
            .expect("main function should be present");
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::QAnd { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::QOr { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::QNot { .. })));
        assert!(main
            .instrs
            .iter()
            .any(|instr| matches!(instr, IrInstr::QImpl { .. })));
        assert!(main.instrs.iter().all(|instr| {
            !matches!(
                instr,
                IrInstr::QTruthAnd { .. }
                    | IrInstr::QTruthOr { .. }
                    | IrInstr::QTruthNot { .. }
                    | IrInstr::QTruthImpl { .. }
            )
        }));
    }

    #[test]
    fn qtruth_intrinsics_reject_invalid_arguments_arity_and_named_args() {
        let non_quad = r#"
            fn main() {
                let x: quad = qtruth_and(true, T);
                return;
            }
        "#;
        assert!(compile_program_to_ir(non_quad).is_err());

        let wrong_arity = r#"
            fn main() {
                let x: quad = qtruth_not(T, F);
                return;
            }
        "#;
        assert!(compile_program_to_ir(wrong_arity).is_err());

        let named_args = r#"
            fn main() {
                let x: quad = qtruth_and(a: T, b: F);
                return;
            }
        "#;
        assert!(compile_program_to_ir(named_args).is_err());
    }

    // #1709 (FA-04-003): nested value/loop-expression lowering can erase
    // frozen ownership events. Each test below pins one previously-lost
    // event through its specific loss path (§10-13 of the governing spec),
    // asserting exact event *count* (not mere containment) to catch both
    // event loss and event duplication in the same assertion.

    #[test]
    fn ssf08_1709_nested_value_block_tuple_borrow_preserves_ownership_event() {
        // Baseline defect: `lower_value_block_expr`'s `LetTuple` arm passed
        // `&mut Vec::new()` to `bind_tuple_items` *and* hardcoded the
        // source `AccessPath` to `None` - so the Borrow event was both
        // unretained and never generated at this call site. Fixed: this
        // nested `let (ref .., ..)` inside a value-producing block derives
        // the same canonical path `lower_stmt`'s top-level twin derives,
        // and writes into the enclosing function's one retained sink.
        let src = r#"
            fn pair() -> (i32, i32) = (1, 2);

            fn main() {
                let source: (i32, i32) = pair();
                let total: i32 = {
                    let (ref a, b): (i32, i32) = source;
                    a
                };
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let {
            name: source_name, ..
        } = program.arena.stmt(main_fn.body[0])
        else {
            panic!("expected source binding");
        };
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Borrow,
                path: AccessPath::new(*source_name).tuple_index(0),
            }]
        );
    }

    // No `ssf08_1709_nested_value_block_record_borrow_preserves_ownership_event`
    // test exists here. `sm-front::typecheck::infer_value_block_type`'s
    // statement whitelist for any value-producing block (closure bodies,
    // if/match arm bodies) is exactly `Const | Let | LetTuple | Discard |
    // Expr(_)` (confirmed by direct inspection) - `LetRecord` is not
    // admitted there at all, so `lower_value_block_expr`'s own `LetRecord`
    // arm (fixed alongside `LetTuple`'s for the same #1709 defect: real
    // `AccessPath` derivation instead of `None`, the real retained sink
    // instead of `&mut Vec::new()`) is currently unreachable through the
    // standard `parse -> typecheck -> lower` pipeline. This mirrors the
    // #1664 closure-capture-constness precedent: the repair is real and
    // architecturally required (the same producer/consumer pair the
    // admitted `LetTuple` case already proves correct), but no admitted
    // source can independently exercise it, and adding one would require
    // widening frontend syntax, which is explicitly out of #1709's scope.
    // The `LetTuple` regression above proves the shared mechanism.

    #[test]
    fn ssf08_1709_nested_value_block_record_update_write_event_preserved() {
        // Baseline defect: `lower_value_block_expr`'s `Stmt::Let` arm never
        // called `append_record_update_write_events_from_expr` at all (its
        // top-level `lower_stmt` twin calls it as the first line of every
        // relevant arm) - so a record-update write nested inside a value
        // block produced zero events regardless of the sink.
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let total: f64 = {
                    let patched: DecisionContext = ctx with { quality: 1.0 };
                    patched.quality
                };
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new(*ctx_name).field(quality_field),
            }]
        );
    }

    #[test]
    fn ssf08_1709_loop_expr_fallback_preserves_ownership_event() {
        // Baseline defect: `lower_loop_expr_stmt`'s `_` fallback arm built a
        // temporary `LoweringCtx` with `ownership_events: Vec::new()`,
        // delegated to `lower_stmt`, and copied back every piece of state
        // except `ownership_events` - so a `LetTuple` reaching this
        // fallback (it is not one of the `If`/`Match` special-cased forms)
        // emitted a real Borrow event into a channel that was then
        // silently dropped when the temporary `ctx` went out of scope.
        let src = r#"
            fn pair() -> (i32, i32) = (1, 2);

            fn main() {
                let source: (i32, i32) = pair();
                let total: i32 = loop {
                    let (ref a, _): (i32, i32) = source;
                    break a;
                };
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let {
            name: source_name, ..
        } = program.arena.stmt(main_fn.body[0])
        else {
            panic!("expected source binding");
        };
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Borrow,
                path: AccessPath::new(*source_name).tuple_index(0),
            }]
        );
    }

    #[test]
    fn ssf08_1709_lifted_closure_owns_body_ownership_event_not_parent() {
        // #1709 is not solved if the event is merely moved into the parent
        // function (§13/§26): a lifted closure is a new `IrFunction`, hence
        // a new function-owned event sink. The closure body here contains
        // the same ref-capture tuple destructuring as the other #1709
        // regressions; the resulting Borrow event must land on the lifted
        // helper's own `ownership_events`, and must be absent from the
        // parent (`main`) function's event stream.
        let src = r#"
            fn pair() -> (i32, i32) = (1, 2);

            fn main() {
                let source: (i32, i32) = pair();
                let f: Closure(i32 -> i32) = (x => {
                    let (ref a, b): (i32, i32) = source;
                    a + x
                });
                let total: i32 = f(10);
                return;
            }
        "#;

        let program = parse_program(src).expect("program should parse");
        let fn_table = build_fn_table(&program).expect("function table should build");
        let record_table = build_record_table(&program).expect("record table should build");
        let adt_table = build_adt_table(&program).expect("adt table should build");
        type_check_program(&program).expect("program should type-check");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn should exist");
        let lowered = lower_function_to_ir_with_tables(
            main_fn,
            &program.arena,
            &fn_table,
            &record_table,
            &adt_table,
            &program.impls,
        )
        .expect("main should lower");

        let Stmt::Let {
            name: source_name, ..
        } = program.arena.stmt(main_fn.body[0])
        else {
            panic!("expected source binding");
        };
        let expected_event = OwnershipPathEvent {
            kind: OwnershipPathEventKind::Borrow,
            path: AccessPath::new(*source_name).tuple_index(0),
        };

        let lifted = lowered
            .lifted
            .iter()
            .find(|func| func.name.starts_with("__closure_main_"))
            .expect("closure lowering should produce a lifted helper");
        assert_eq!(lifted.ownership_events, vec![expected_event.clone()]);
        assert!(
            !lowered.primary.ownership_events.contains(&expected_event),
            "closure-body-local ownership event must not leak into the parent function"
        );
    }

    // #1709 corrective round (exact-head review of 17e89f63): the sink was
    // correctly threaded everywhere, but several admitted nested roots
    // still never *called* `append_record_update_write_events_from_expr` -
    // the producer authority - even though they had the right place to put
    // its output. These regressions pin each of those roots independently.

    #[test]
    fn ssf08_1709_value_block_discard_write_event_preserved() {
        // `lower_value_block_expr`'s `Stmt::Discard` arm had the sink but
        // never called the producer authority, unlike `lower_stmt`'s twin.
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let total: f64 = {
                    let _ = ctx with { quality: 1.0 };
                    ctx.quality
                };
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new(*ctx_name).field(quality_field),
            }]
        );
    }

    #[test]
    fn ssf08_1709_value_block_expr_statement_write_event_preserved() {
        // `lower_value_block_expr`'s `Stmt::Expr` arm had the sink but
        // never called the producer authority, unlike `lower_stmt`'s twin.
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn sink(x: DecisionContext) {
                return;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let total: f64 = {
                    sink(ctx with { quality: 1.0 });
                    ctx.quality
                };
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new(*ctx_name).field(quality_field),
            }]
        );
    }

    #[test]
    fn ssf08_1709_loop_expr_special_if_condition_write_event_preserved() {
        // `lower_loop_expr_stmt`'s dedicated `Stmt::If` arm does not
        // delegate to `lower_stmt` (unlike everything reaching its `_`
        // fallback), so it needs its own call to the producer authority on
        // `condition`, mirroring `lower_stmt`'s `Stmt::If` arm exactly.
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn ready(ctx: DecisionContext) -> bool = ctx.camera == T;

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let total: i32 = loop {
                    if ready(ctx with { quality: 1.0 }) {
                        break 1;
                    } else {
                        break 2;
                    }
                };
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new(*ctx_name).field(quality_field),
            }]
        );
    }

    #[test]
    fn ssf08_1709_loop_expr_special_match_scrutinee_write_event_preserved() {
        // `lower_loop_expr_stmt`'s dedicated `Stmt::Match` arm does not
        // delegate to `lower_stmt`, so it needs its own call to the
        // producer authority on `scrutinee`, mirroring `lower_stmt`'s
        // `Stmt::Match` arm exactly.
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn tag_of(ctx: DecisionContext) -> i32 = 1;

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let total: i32 = loop {
                    match tag_of(ctx with { quality: 1.0 }) {
                        1 => { break 10; }
                        _ => { break 20; }
                    }
                };
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new(*ctx_name).field(quality_field),
            }]
        );
    }

    #[test]
    fn ssf08_1709_loop_expr_break_value_write_event_preserved() {
        // Canonical `lower_stmt`'s `Stmt::Break(Some(value))` arm itself
        // never called the producer authority on the break payload, unlike
        // its `Stmt::Return(Some(value))` sibling. `break value;` is only
        // ever reachable nested inside a loop expression, so this is
        // squarely within #1709's contour, not a separate top-level defect.
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let total: DecisionContext = loop {
                    break ctx with { quality: 1.0 };
                };
                return;
            }
        "#;

        let (program, main) = lower_single_function_with_program(src, "main");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn");
        let Stmt::Let { name: ctx_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected ctx binding");
        };
        let quality_field = program.records[0].fields[1].name;
        assert_eq!(
            main.ownership_events,
            vec![OwnershipPathEvent {
                kind: OwnershipPathEventKind::Write,
                path: AccessPath::new(*ctx_name).field(quality_field),
            }]
        );
    }

    #[test]
    fn ssf08_1709_closure_body_write_event_owned_by_lifted_helper() {
        // `append_record_update_write_events_from_expr` treats
        // `Expr::Closure` as a deliberate leaf, so no enclosing scan ever
        // reaches `closure.body`. `lower_closure_literal_expr` must
        // therefore prescan the body itself, into the child's own sink.
        let src = r#"
            record R {
                x: i32,
            }

            fn main() {
                let r: R = R { x: 1 };
                let f: Closure(i32 -> R) = (n => r with { x: n });
                let total: R = f(5);
                return;
            }
        "#;

        let program = parse_program(src).expect("program should parse");
        let fn_table = build_fn_table(&program).expect("function table should build");
        let record_table = build_record_table(&program).expect("record table should build");
        let adt_table = build_adt_table(&program).expect("adt table should build");
        type_check_program(&program).expect("program should type-check");
        let main_fn = program
            .functions
            .iter()
            .find(|func| program.arena.symbol_name(func.name) == "main")
            .expect("main fn should exist");
        let lowered = lower_function_to_ir_with_tables(
            main_fn,
            &program.arena,
            &fn_table,
            &record_table,
            &adt_table,
            &program.impls,
        )
        .expect("main should lower");

        let Stmt::Let { name: r_name, .. } = program.arena.stmt(main_fn.body[0]) else {
            panic!("expected r binding");
        };
        let x_field = program.records[0].fields[0].name;
        let expected_event = OwnershipPathEvent {
            kind: OwnershipPathEventKind::Write,
            path: AccessPath::new(*r_name).field(x_field),
        };

        let lifted = lowered
            .lifted
            .iter()
            .find(|func| func.name.starts_with("__closure_main_"))
            .expect("closure lowering should produce a lifted helper");
        assert_eq!(lifted.ownership_events, vec![expected_event.clone()]);
        assert!(
            !lowered.primary.ownership_events.contains(&expected_event),
            "closure-body-local write event must not leak into the parent function"
        );
    }

    // SSF-08 Lane 2b (#1724 / FA-04-018): IR-structural companion to the
    // VM-behavioral proofs in `tests/lexical_binding_identity_e2e.rs`.
    // Deliberately uses relational assertions (distinctness, adjacency),
    // not the exact generated key text - the mangled format is an
    // implementation detail of `LoweredLocalEnv`, not part of #1724's
    // contract.

    #[test]
    fn ssf08_1724_if_branch_shadow_uses_distinct_lowered_local_keys() {
        let src = r#"
            fn main() {
                let x: i32 = 1;
                if true {
                    let x: i32 = 2;
                    let y: i32 = x;
                    let _ = y;
                }
                let z: i32 = x;
                let _ = z;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("if-branch shadow should lower");
        let main = &ir[0];

        let outer_x_key = main
            .instrs
            .iter()
            .find_map(|instr| match instr {
                IrInstr::StoreVar { name, .. } => Some(name.clone()),
                _ => None,
            })
            .expect("outer x StoreVar must exist as the function's first StoreVar");

        // The LoadVar immediately preceding the `y` StoreVar must resolve
        // to a *different* key than the outer binding's - the inner `x`.
        let y_store_pos = main
            .instrs
            .iter()
            .position(
                |instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_y")),
            )
            .expect("y StoreVar must exist");
        let inner_x_key = match &main.instrs[y_store_pos - 1] {
            IrInstr::LoadVar { name, .. } => name.clone(),
            other => panic!("expected LoadVar feeding y, got {other:?}"),
        };
        assert_ne!(
            outer_x_key, inner_x_key,
            "outer and inner 'x' must lower to distinct runtime-local keys"
        );

        // The LoadVar immediately preceding the post-if `z` StoreVar must
        // resolve back to the *outer* binding's exact key.
        let z_store_pos = main
            .instrs
            .iter()
            .position(
                |instr| matches!(instr, IrInstr::StoreVar { name, .. } if name.ends_with("_z")),
            )
            .expect("z StoreVar must exist");
        let post_scope_x_key = match &main.instrs[z_store_pos - 1] {
            IrInstr::LoadVar { name, .. } => name.clone(),
            other => panic!("expected LoadVar feeding z, got {other:?}"),
        };
        assert_eq!(
            post_scope_x_key, outer_x_key,
            "use of 'x' after the if block must resolve to the outer binding's exact key"
        );
    }

    #[test]
    fn ssf08_1724_reassignment_reuses_existing_lowered_local_key() {
        let src = r#"
            fn main() {
                let mut x: i32 = 1;
                x = 2;
                return;
            }
        "#;

        let ir = compile_program_to_ir(src).expect("reassignment should lower");
        let main = &ir[0];
        let store_keys: Vec<&str> = main
            .instrs
            .iter()
            .filter_map(|instr| match instr {
                IrInstr::StoreVar { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            store_keys.len(),
            2,
            "expected exactly the declaration StoreVar and the reassignment StoreVar, got {store_keys:?}"
        );
        assert_eq!(
            store_keys[0], store_keys[1],
            "reassignment to an existing binding must reuse its exact lowered key, not mint a new one"
        );
    }
}
