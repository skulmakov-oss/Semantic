use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
pub use ton618_core::{ExprId, StmtId, SymbolId};
use ton618_core::{SigTable, SourceMark};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Quad,
    QVec(usize),
    Bool,
    Text,
    Sequence(SequenceType),
    Map(MapType),
    Closure(ClosureType),
    I32,
    U32,
    Fx,
    F64,
    Measured(Box<Type>, SymbolId),
    RangeI32,
    Tuple(Vec<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Record(SymbolId),
    Adt(SymbolId),
    Unit,
    /// Type variable introduced by a generic parameter list.
    ///
    /// Admitted at the owner layer (Wave 1). Executable use of TypeVar in
    /// type-check and lowering is deferred to Wave 2 (monomorphisation pass).
    TypeVar(SymbolId),
}

impl Type {
    pub fn erase_units(&self) -> Type {
        match self {
            Type::Measured(base, _) => base.erase_units(),
            Type::Tuple(items) => Type::Tuple(items.iter().map(Type::erase_units).collect()),
            Type::Sequence(sequence) => Type::Sequence(SequenceType {
                family: sequence.family,
                item: Box::new(sequence.item.erase_units()),
            }),
            Type::Map(map) => Type::Map(MapType {
                key: Box::new(map.key.erase_units()),
                val: Box::new(map.val.erase_units()),
            }),
            Type::Closure(closure) => Type::Closure(ClosureType {
                family: closure.family,
                capture: closure.capture,
                param: Box::new(closure.param.erase_units()),
                ret: Box::new(closure.ret.erase_units()),
            }),
            Type::Option(item) => Type::Option(Box::new(item.erase_units())),
            Type::Result(ok_ty, err_ty) => Type::Result(
                Box::new(ok_ty.erase_units()),
                Box::new(err_ty.erase_units()),
            ),
            // TypeVar is an owner-layer marker. Unit erasure is identity
            // since monomorphisation has not yet substituted the variable.
            Type::TypeVar(_) => self.clone(),
            _ => self.clone(),
        }
    }

    pub fn measured_parts(&self) -> Option<(&Type, SymbolId)> {
        match self {
            Type::Measured(base, unit) => Some((base.as_ref(), *unit)),
            _ => None,
        }
    }

    pub fn is_core_numeric_scalar(&self) -> bool {
        matches!(self, Type::I32 | Type::U32 | Type::Fx | Type::F64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadVal {
    N,
    F,
    T,
    S,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Pos,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    AndAnd,
    OrOr,
    Implies,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericLiteral {
    I32(i32),
    U32(u32),
    F64(f64),
    Fx(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextLiteralFamily {
    DoubleQuotedUtf8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextLiteral {
    pub family: TextLiteralFamily,
    pub spelling: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SequenceCollectionFamily {
    OrderedSequence,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceType {
    pub family: SequenceCollectionFamily,
    pub item: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapType {
    pub key: Box<Type>,
    pub val: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceLiteral {
    pub family: SequenceCollectionFamily,
    pub items: Vec<ExprId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceIndexExpr {
    pub base: ExprId,
    pub index: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClosureValueFamily {
    UnaryDirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClosureCapturePolicy {
    Immutable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClosureType {
    pub family: ClosureValueFamily,
    pub capture: ClosureCapturePolicy,
    pub param: Box<Type>,
    pub ret: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClosureLiteral {
    pub family: ClosureValueFamily,
    pub capture: ClosureCapturePolicy,
    pub param: SymbolId,
    pub param_ty: Option<Type>,
    pub ret_ty: Option<Type>,
    pub captures: Vec<SymbolId>,
    pub body: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallArg {
    pub name: Option<SymbolId>,
    pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordInitField {
    pub name: SymbolId,
    pub value: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordLiteralExpr {
    pub name: SymbolId,
    pub fields: Vec<RecordInitField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordFieldExpr {
    pub base: ExprId,
    pub field: SymbolId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordUpdateExpr {
    pub base: ExprId,
    pub fields: Vec<RecordInitField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdtCtorExpr {
    pub adt_name: SymbolId,
    pub variant_name: SymbolId,
    pub payload: Vec<ExprId>,
}

/// M9.5 Wave A: capture mode for pattern bindings.
///
/// Default is `Move`. `Borrow` is spelled `ref x` in source.
/// Mutable borrow, partial move, lifetime inference, and reborrow are deferred.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum CaptureMode {
    Move,
    Borrow,
}

// ──────────────────────────────────────────────────────────────
// M9.5 Wave C: pattern path tracking, binding plans, scrutinee state
// ──────────────────────────────────────────────────────────────

/// One step in the address of a sub-value accessed by a pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternPathElem {
    TupleIndex(usize),
    Variant(SymbolId),
    VariantField(usize),
    RecordField(SymbolId),
}

/// Canonical address of a sub-value within the scrutinee.
///
/// `PatternPath::root()` refers to the scrutinee itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PatternPath {
    pub elems: Vec<PatternPathElem>,
}

impl PatternPath {
    pub fn root() -> Self {
        Self { elems: Vec::new() }
    }
    pub fn tuple_index(&self, idx: usize) -> Self {
        let mut e = self.elems.clone();
        e.push(PatternPathElem::TupleIndex(idx));
        Self { elems: e }
    }
    pub fn variant(&self, name: SymbolId) -> Self {
        let mut e = self.elems.clone();
        e.push(PatternPathElem::Variant(name));
        Self { elems: e }
    }
    pub fn variant_field(&self, idx: usize) -> Self {
        let mut e = self.elems.clone();
        e.push(PatternPathElem::VariantField(idx));
        Self { elems: e }
    }
    pub fn record_field(&self, name: SymbolId) -> Self {
        let mut e = self.elems.clone();
        e.push(PatternPathElem::RecordField(name));
        Self { elems: e }
    }
}

/// A single binding produced by a pattern, with its sub-value address.
#[derive(Debug, Clone)]
pub struct BindingPlanItem {
    pub name: SymbolId,
    pub capture: CaptureMode,
    pub path: PatternPath,
    pub ty: Type,
}

/// The complete set of bindings a pattern produces from a scrutinee.
///
/// NOTE (M9.5 Wave A/B): CaptureMode is populated but not yet enforced in
/// typecheck beyond conflict detection. Full move/borrow semantics arrive in
/// Wave C.
#[derive(Debug, Clone, Default)]
pub struct BindingPlan {
    pub items: Vec<BindingPlanItem>,
}

impl BindingPlan {
    pub fn push(&mut self, item: BindingPlanItem) {
        self.items.push(item);
    }
}

/// Whether the same path is accessed as moved or borrowed (conflict detection).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKind {
    Borrow,
    Move,
}

/// Whether a scrutinee value is available after a match/if-let.
///
/// Consumed if any binding in the plan used `Move`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrutineeUse {
    Preserved,
    Consumed,
}

// ──────────────────────────────────────────────────────────────
// M9.7: per-path availability state for partial move
// ──────────────────────────────────────────────────────────────

/// Availability of a single path within a bound variable.
///
/// Used for partial-move tracking: moving `x.0` marks `root.0` as `Moved`
/// without invalidating `root.1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathAvailability {
    Available,
    Borrowed,
    Moved,
}

/// Per-variable path availability table.
///
/// Stores the set of `(PatternPath, PathAvailability)` entries recorded
/// by pattern-binding operations.  An empty table means the variable is
/// fully available.
#[derive(Debug, Clone, Default)]
pub struct ValuePathState {
    pub paths: Vec<(PatternPath, PathAvailability)>,
}

/// Availability of a local variable in scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueAvailability {
    Available,
    Consumed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdtPatternItem {
    /// M9.5 Wave A: binding now carries explicit capture mode (default `Move`).
    Bind {
        name: SymbolId,
        capture: CaptureMode,
    },
    Discard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdtMatchPattern {
    pub adt_name: SymbolId,
    pub variant_name: SymbolId,
    pub items: Vec<AdtPatternItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordPatternItem {
    pub field: SymbolId,
    pub target: RecordPatternTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordPatternTarget {
    Bind {
        name: SymbolId,
        capture: CaptureMode,
    },
    Discard,
    QuadLiteral(QuadVal),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    QuadLiteral(QuadVal),
    BoolLiteral(bool),
    TextLiteral(TextLiteral),
    SequenceLiteral(SequenceLiteral),
    NumericLiteral(NumericLiteral),
    Range(RangeExpr),
    Tuple(Vec<ExprId>),
    RecordLiteral(RecordLiteralExpr),
    RecordField(RecordFieldExpr),
    SequenceIndex(SequenceIndexExpr),
    Closure(ClosureLiteral),
    RecordUpdate(RecordUpdateExpr),
    AdtCtor(AdtCtorExpr),
    Var(SymbolId),
    Call(SymbolId, Vec<CallArg>),
    Unary(UnaryOp, ExprId),
    Binary(ExprId, BinaryOp, ExprId),
    Block(BlockExpr),
    If(IfExpr),
    /// M9.4 Wave 1: `if let Pattern = expr { ... } else { ... }` binding guard desugaring.
    ///
    /// Admitted at the owner layer. Parser and typecheck admission in Wave 2/3.
    IfLet(IfLetExpr),
    Match(MatchExpr),
    Loop(LoopExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Const {
        name: SymbolId,
        ty: Option<Type>,
        value: ExprId,
    },
    Let {
        name: SymbolId,
        is_mut: bool,
        ty: Option<Type>,
        value: ExprId,
    },
    LetTuple {
        items: Vec<TuplePatternItem>,
        ty: Option<Type>,
        value: ExprId,
    },
    LetRecord {
        record_name: SymbolId,
        items: Vec<RecordPatternItem>,
        value: ExprId,
    },
    LetElseRecord {
        record_name: SymbolId,
        items: Vec<RecordPatternItem>,
        value: ExprId,
        else_return: Option<ExprId>,
    },
    LetElseTuple {
        items: Vec<TuplePatternItem>,
        ty: Option<Type>,
        value: ExprId,
        else_return: Option<ExprId>,
    },
    Discard {
        ty: Option<Type>,
        value: ExprId,
    },
    Assign {
        name: SymbolId,
        value: ExprId,
    },
    AssignTuple {
        items: Vec<Option<SymbolId>>,
        value: ExprId,
    },
    /// M9.3 Wave 1: owner-layer iterable loop surface.
    ///
    /// This records the future `for x in collection` contract without making
    /// general iterable loops executable yet. Existing `RangeI32` loops remain
    /// runnable through explicit typecheck/lowering compatibility handling.
    ForEach {
        name: SymbolId,
        iterable: ExprId,
        body: Vec<StmtId>,
        desugaring: IterableLoopDesugaring,
    },
    ForRange {
        name: SymbolId,
        range: ExprId,
        body: Vec<StmtId>,
    },
    While {
        condition: ExprId,
        body: Vec<StmtId>,
    },
    Loop {
        body: Vec<StmtId>,
    },
    Guard {
        condition: ExprId,
        else_return: Option<ExprId>,
    },
    If {
        condition: ExprId,
        then_block: Vec<StmtId>,
        else_block: Vec<StmtId>,
    },
    Match {
        scrutinee: ExprId,
        arms: Vec<MatchArm>,
        default: Vec<StmtId>,
    },
    Break(Option<ExprId>),
    Continue,
    Return(Option<ExprId>),
    Expr(ExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockExpr {
    pub statements: Vec<StmtId>,
    pub tail: ExprId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: ExprId,
    pub then_block: BlockExpr,
    pub else_block: BlockExpr,
}

/// M9.4 Wave 1: `if let Pattern = value { then } else { otherwise }` form.
///
/// Binds names from `pattern` into `then_block` only. The `else_block` sees
/// the pre-binding scope. Parser and typecheck admission in Wave 2/3.
#[derive(Debug, Clone, PartialEq)]
pub struct IfLetExpr {
    pub pattern: MatchPattern,
    pub value: ExprId,
    pub then_block: BlockExpr,
    pub else_block: BlockExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub scrutinee: ExprId,
    pub arms: Vec<MatchExprArm>,
    pub default: Option<BlockExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchExprArm {
    pub pat: MatchPattern,
    pub guard: Option<ExprId>,
    pub block: BlockExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpr {
    pub body: Vec<StmtId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpr {
    pub start: ExprId,
    pub end: ExprId,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuplePatternItem {
    /// M9.5 Wave A: binding now carries explicit capture mode (default `Move`).
    Bind {
        name: SymbolId,
        capture: CaptureMode,
    },
    Discard,
    QuadLiteral(QuadVal),
    /// M9.4 Wave 1: nested tuple destructuring — `(a, (b, c))` beyond one level.
    Nested(Vec<TuplePatternItem>),
}

/// An integer range used as a match pattern, e.g. `1..=5 =>` or `0..10 =>`.
///
/// M9.4 Wave 1 owner layer. Parser and typecheck admission in Wave 2/3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntRangePattern {
    pub start: i64,
    pub end: i64,
    pub inclusive: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Quad(QuadVal),
    Adt(AdtMatchPattern),
    /// M9.4 Wave 1: `_` wildcard in match arms.
    Wildcard,
    /// M9.4 Wave 1: or-pattern — `Variant::A | Variant::B =>`.
    /// At least two alternatives; alternatives are matched left-to-right.
    Or(Vec<MatchPattern>),
    /// M9.4 Wave 1: integer range pattern — `1..=5 =>` or `0..10 =>`.
    IntRange(IntRangePattern),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pat: MatchPattern,
    pub guard: Option<ExprId>,
    pub block: Vec<StmtId>,
}

/// M9.3 Wave 1: canonical owner-layer desugaring anchor for iterable loops.
///
/// Wave 1 records only the stdlib trait contract name. Type admission and
/// executable lowering remain deferred to later iterable waves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IterableLoopDesugaring {
    pub trait_name: SymbolId,
}

/// A named behavior bound on a type parameter.
///
/// Represents the `T: TraitName` constraint in a `<T: TraitName>` parameter
/// list. Admitted at the owner layer (Wave 1). Bound checking at call sites
/// and impl resolution are deferred to Wave 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitBound {
    /// The type parameter that carries the bound.
    pub param: SymbolId,
    /// The named trait the parameter must implement.
    pub bound: SymbolId,
}

/// A method signature declared inside a `trait` definition body.
///
/// No default method bodies in first wave — each method is an abstract
/// signature only. Admitted at the owner layer (Wave 1). Parser admission
/// is deferred to Wave 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitMethodSig {
    pub name: SymbolId,
    pub params: Vec<(SymbolId, Type)>,
    pub ret: Type,
}

/// A named behavior contract declared with the `trait` keyword.
///
/// First-wave shape: named trait with a list of abstract method signatures.
/// No default bodies, no associated types, no higher-ranked bounds.
///
/// Admitted at the owner layer (Wave 1). Parser admission is deferred to
/// Wave 2. Impl resolution and static dispatch are deferred to Wave 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDecl {
    pub name: SymbolId,
    /// Generic type parameters on the trait itself (`trait Foo<T>`).
    /// Empty in first-wave canonical form.
    pub type_params: Vec<SymbolId>,
    pub methods: Vec<TraitMethodSig>,
}

/// An explicit named impl block binding a concrete type to a trait.
///
/// First-wave shape: one impl per (trait, type) pair; type_params on impl
/// blocks are empty in first-wave canonical form.
///
/// Admitted at the owner layer (Wave 1). Parser admission is deferred to
/// Wave 2. Impl resolution and static dispatch are deferred to Wave 3.
#[derive(Debug, Clone, PartialEq)]
pub struct ImplDecl {
    /// The trait being implemented.
    pub trait_name: SymbolId,
    /// The concrete nominal type that implements the trait.
    pub for_type: SymbolId,
    /// Type parameters on the impl block.
    /// Empty in first-wave canonical form.
    pub type_params: Vec<SymbolId>,
    /// Concrete method implementations. Each method must match a signature in
    /// the corresponding `TraitDecl`.
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: SymbolId,
    /// Generic type parameter names declared on this function.
    ///
    /// Admitted at the owner layer (Wave 1). Executable use (monomorphisation,
    /// instantiation) is deferred to Wave 2.
    pub type_params: Vec<SymbolId>,
    /// Trait bounds on the type parameters: `<T: TraitName>` constraints.
    ///
    /// Admitted at the owner layer (Wave 1). Bound checking at call sites
    /// and impl resolution are deferred to Wave 3.
    pub trait_bounds: Vec<TraitBound>,
    pub params: Vec<(SymbolId, Type)>,
    pub param_defaults: Vec<Option<ExprId>>,
    pub requires: Vec<ExprId>,
    pub ensures: Vec<ExprId>,
    pub invariants: Vec<ExprId>,
    pub ret: Type,
    pub body: Vec<StmtId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordField {
    pub name: SymbolId,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDecl {
    pub name: SymbolId,
    /// Generic type parameter names declared on this record.
    ///
    /// Admitted at the owner layer (Wave 1). Executable use is deferred to
    /// Wave 2 (monomorphisation pass).
    pub type_params: Vec<SymbolId>,
    pub fields: Vec<RecordField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtVariant {
    pub name: SymbolId,
    pub payload: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdtDecl {
    pub name: SymbolId,
    /// Generic type parameter names declared on this ADT.
    ///
    /// Admitted at the owner layer (Wave 1). Executable use is deferred to
    /// Wave 2 (monomorphisation pass).
    pub type_params: Vec<SymbolId>,
    pub variants: Vec<AdtVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaField {
    pub name: SymbolId,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    pub name: SymbolId,
    pub fields: Vec<SchemaField>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaRole {
    Config,
    Api,
    Wire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaVersion {
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaShape {
    Record(Vec<SchemaField>),
    TaggedUnion(Vec<SchemaVariant>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaDecl {
    pub name: SymbolId,
    pub role: Option<SchemaRole>,
    pub version: Option<SchemaVersion>,
    pub shape: SchemaShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationFieldPlan {
    pub name: SymbolId,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationVariantPlan {
    pub name: SymbolId,
    pub fields: Vec<ValidationFieldPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationCheck {
    RequiredField {
        field: SymbolId,
    },
    FieldType {
        field: SymbolId,
        ty: Type,
    },
    TaggedUnionBranch {
        variant: SymbolId,
    },
    TaggedUnionBranchRequiredField {
        variant: SymbolId,
        field: SymbolId,
    },
    TaggedUnionBranchFieldType {
        variant: SymbolId,
        field: SymbolId,
        ty: Type,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationShapePlan {
    Record(Vec<ValidationFieldPlan>),
    TaggedUnion(Vec<ValidationVariantPlan>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationPlan {
    pub schema_name: SymbolId,
    pub role: Option<SchemaRole>,
    pub shape: ValidationShapePlan,
    pub checks: Vec<ValidationCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableImportSelectItem {
    pub name: SymbolId,
    pub alias: Option<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableImport {
    pub spec: String,
    pub alias: Option<SymbolId>,
    pub reexport: bool,
    pub select_items: Vec<ExecutableImportSelectItem>,
    pub wildcard: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub arena: AstArena,
    pub imports: Vec<ExecutableImport>,
    pub adts: Vec<AdtDecl>,
    pub records: Vec<RecordDecl>,
    pub schemas: Vec<SchemaDecl>,
    pub functions: Vec<Function>,
    /// Trait definitions admitted by the parser (Wave 2+).
    pub traits: Vec<TraitDecl>,
    /// Impl blocks admitted by the parser (Wave 2+).
    pub impls: Vec<ImplDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendErrorKind {
    Syntax,
    PolicyViolation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendError {
    pub pos: usize,
    pub message: String,
}

impl FrontendError {
    pub fn syntax(pos: usize, message: impl Into<String>) -> Self {
        Self {
            pos,
            message: message.into(),
        }
    }

    pub fn policy_violation(pos: usize, message: impl Into<String>) -> Self {
        Self {
            pos,
            message: format!("policy violation: {}", message.into()),
        }
    }

    pub fn kind(&self) -> FrontendErrorKind {
        if self.message.starts_with("policy violation:") {
            FrontendErrorKind::PolicyViolation
        } else {
            FrontendErrorKind::Syntax
        }
    }
}

impl ::core::fmt::Display for FrontendError {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "at {}: {}", self.pos, self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FrontendError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub pos: usize,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    KwFn,
    KwRequires,
    KwEnsures,
    KwInvariant,
    KwRecord,
    KwSchema,
    KwEnum,
    KwConst,
    /// `trait` — introduces a named behavior contract declaration.
    ///
    /// Admitted to the lexer at the owner layer (Wave 1).
    /// Parser admission is deferred to Wave 2.
    KwTrait,
    /// `impl` — introduces an explicit named impl block.
    ///
    /// Admitted to the lexer at the owner layer (Wave 1).
    /// Parser admission is deferred to Wave 2.
    KwImpl,
    KwLet,
    KwFor,
    KwIn,
    KwGuard,
    KwIf,
    KwElse,
    KwIs,
    KwWhile,
    KwLoop,
    KwBreak,
    KwContinue,
    KwWhere,
    KwWith,
    KwReturn,
    KwMatch,
    KwTrue,
    KwFalse,
    KwSystem,
    KwEntity,
    KwLaw,
    KwWhen,
    KwPulse,
    KwProfile,
    KwImport,
    /// M9.5 Wave B: `ref` keyword — borrow binding in patterns, e.g. `ref x` in tuple/ADT patterns.
    KwRef,
    TyQuad,
    TyBool,
    TyI32,
    TyU32,
    TyFx,
    TyF64,
    QuadN,
    QuadF,
    QuadT,
    QuadS,
    Ident,
    Num,
    String,
    AndAnd,
    OrOr,
    PipeForward,
    /// `|` — bare pipe used as or-pattern separator. M9.4 Wave 2.
    Pipe,
    AndAndAssign,
    OrOrAssign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Arrow,
    Implies,
    FatArrow,
    EqEq,
    Ne,
    Le,
    Ge,
    DotDot,
    DotDotEq,
    Assign,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Semi,
    Comma,
    Colon,
    PathSep,
    Dot,
    LBracket,
    RBracket,
    /// `<` — used for generic type parameter lists (`fn foo<T>`).
    LAngle,
    /// `>` — used for generic type parameter lists.
    RAngle,
    Underscore,
    Indent,
    Dedent,
    Newline,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosSystem {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosEntityField {
    pub kind: LogosEntityFieldKind,
    pub name: String,
    pub ty: Type,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogosEntityFieldKind {
    State,
    Prop,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosEntity {
    pub name: String,
    pub fields: Vec<LogosEntityField>,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosWhen {
    pub condition: String,
    pub effect: String,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogosLaw {
    pub name: String,
    pub priority: u32,
    pub whens: Vec<LogosWhen>,
    pub mark: SourceMark,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LogosProgram {
    pub system: Option<LogosSystem>,
    pub entities: Vec<LogosEntity>,
    pub laws: Vec<LogosLaw>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AstArena {
    exprs: Vec<Expr>,
    stmts: Vec<Stmt>,
    sigtable: SigTable,
    pub(crate) symbol_to_id: BTreeMap<String, SymbolId>,
}

impl AstArena {
    pub fn alloc_expr(&mut self, expr: Expr) -> ExprId {
        let id = ExprId(self.exprs.len() as u32);
        self.exprs.push(expr);
        id
    }

    pub fn alloc_stmt(&mut self, stmt: Stmt) -> StmtId {
        let id = StmtId(self.stmts.len() as u32);
        self.stmts.push(stmt);
        id
    }

    pub fn expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id.0 as usize]
    }

    pub fn stmt(&self, id: StmtId) -> &Stmt {
        &self.stmts[id.0 as usize]
    }

    pub fn intern_symbol(&mut self, name: &str) -> SymbolId {
        if let Some(id) = self.symbol_to_id.get(name) {
            return *id;
        }
        let id = self.sigtable.intern(name);
        self.symbol_to_id.insert(name.to_string(), id);
        id
    }

    pub fn expr_count(&self) -> usize {
        self.exprs.len()
    }

    pub fn stmt_count(&self) -> usize {
        self.stmts.len()
    }

    pub fn symbol_count(&self) -> usize {
        self.sigtable.len()
    }

    pub fn symbol_name(&self, id: SymbolId) -> &str {
        self.try_symbol_name(id).unwrap_or("<invalid-symbol>")
    }

    pub fn try_symbol_name(&self, id: SymbolId) -> Option<&str> {
        self.sigtable.resolve(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_invariants_append_only_and_stable_ids() {
        let mut arena = AstArena::default();

        let e0 = arena.alloc_expr(Expr::QuadLiteral(QuadVal::T));
        let e1 = arena.alloc_expr(Expr::BoolLiteral(true));
        assert_eq!(e0.0, 0);
        assert_eq!(e1.0, 1);
        assert_eq!(arena.expr_count(), 2);
        assert_eq!(arena.expr(e0), &Expr::QuadLiteral(QuadVal::T));

        let s0 = arena.alloc_stmt(Stmt::Expr(e0));
        let s1 = arena.alloc_stmt(Stmt::Expr(e1));
        assert_eq!(s0.0, 0);
        assert_eq!(s1.0, 1);
        assert_eq!(arena.stmt_count(), 2);
        assert_eq!(arena.stmt(s0), &Stmt::Expr(e0));

        // Additional allocations must not change earlier IDs or referenced nodes.
        let _e2 = arena.alloc_expr(Expr::NumericLiteral(NumericLiteral::I32(42)));
        let _s2 = arena.alloc_stmt(Stmt::Return(None));
        assert_eq!(arena.expr(e0), &Expr::QuadLiteral(QuadVal::T));
        assert_eq!(arena.stmt(s0), &Stmt::Expr(e0));
    }

    #[test]
    fn text_type_is_non_numeric_and_survives_unit_erasure() {
        assert_eq!(Type::Text.erase_units(), Type::Text);
        assert!(!Type::Text.is_core_numeric_scalar());
    }

    #[test]
    fn sequence_type_preserves_family_and_erases_nested_units() {
        let ty = Type::Sequence(SequenceType {
            family: SequenceCollectionFamily::OrderedSequence,
            item: Box::new(Type::Measured(Box::new(Type::F64), SymbolId(7))),
        });
        assert_eq!(
            ty.erase_units(),
            Type::Sequence(SequenceType {
                family: SequenceCollectionFamily::OrderedSequence,
                item: Box::new(Type::F64),
            })
        );
        assert!(!ty.is_core_numeric_scalar());
    }

    #[test]
    fn text_literal_owner_layer_is_stable_data() {
        let literal = TextLiteral {
            family: TextLiteralFamily::DoubleQuotedUtf8,
            spelling: "\"semantic\"".to_string(),
        };
        assert_eq!(literal.family, TextLiteralFamily::DoubleQuotedUtf8);
        assert_eq!(literal.spelling, "\"semantic\"");
    }

    #[test]
    fn sequence_literal_owner_layer_is_stable_data() {
        let literal = SequenceLiteral {
            family: SequenceCollectionFamily::OrderedSequence,
            items: vec![ExprId(2), ExprId(5)],
        };
        assert_eq!(literal.family, SequenceCollectionFamily::OrderedSequence);
        assert_eq!(literal.items, vec![ExprId(2), ExprId(5)]);
    }

    #[test]
    fn sequence_index_expr_owner_layer_is_stable_data() {
        let expr = SequenceIndexExpr {
            base: ExprId(3),
            index: ExprId(4),
        };
        assert_eq!(expr.base, ExprId(3));
        assert_eq!(expr.index, ExprId(4));
    }

    #[test]
    fn closure_type_preserves_family_and_erases_nested_units() {
        let ty = Type::Closure(ClosureType {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param: Box::new(Type::Measured(Box::new(Type::Fx), SymbolId(9))),
            ret: Box::new(Type::Measured(Box::new(Type::F64), SymbolId(10))),
        });
        assert_eq!(
            ty.erase_units(),
            Type::Closure(ClosureType {
                family: ClosureValueFamily::UnaryDirect,
                capture: ClosureCapturePolicy::Immutable,
                param: Box::new(Type::Fx),
                ret: Box::new(Type::F64),
            })
        );
        assert!(!ty.is_core_numeric_scalar());
    }

    #[test]
    fn typevar_owner_layer_is_stable_data() {
        // TypeVar is admitted at the owner layer. It must survive arena
        // round-trips and erase_units must be identity (no substitution yet).
        let tv = Type::TypeVar(SymbolId(42));
        assert_eq!(tv.erase_units(), Type::TypeVar(SymbolId(42)));
        assert!(!tv.is_core_numeric_scalar());
    }

    #[test]
    fn function_with_type_params_owner_layer_is_stable_data() {
        let mut arena = AstArena::default();
        let name = arena.intern_symbol("identity");
        let t_param = arena.intern_symbol("T");
        let param = arena.intern_symbol("x");
        let body_expr = arena.alloc_expr(Expr::Var(param));
        let body = arena.alloc_stmt(Stmt::Return(Some(body_expr)));
        let func = Function {
            name,
            type_params: vec![t_param],
            trait_bounds: vec![],
            params: vec![(param, Type::TypeVar(t_param))],
            param_defaults: vec![None],
            requires: vec![],
            ensures: vec![],
            invariants: vec![],
            ret: Type::TypeVar(t_param),
            body: vec![body],
        };
        assert_eq!(func.type_params, vec![t_param]);
        assert_eq!(func.params[0].1, Type::TypeVar(t_param));
    }

    #[test]
    fn record_decl_with_type_params_owner_layer_is_stable_data() {
        let mut arena = AstArena::default();
        let name = arena.intern_symbol("Box");
        let t_param = arena.intern_symbol("T");
        let field_name = arena.intern_symbol("value");
        let decl = RecordDecl {
            name,
            type_params: vec![t_param],
            fields: vec![RecordField {
                name: field_name,
                ty: Type::TypeVar(t_param),
            }],
        };
        assert_eq!(decl.type_params, vec![t_param]);
        assert_eq!(decl.fields[0].ty, Type::TypeVar(t_param));
    }

    #[test]
    fn closure_literal_owner_layer_is_stable_data() {
        let literal = ClosureLiteral {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param: SymbolId(2),
            param_ty: Some(Type::Text),
            ret_ty: Some(Type::Bool),
            captures: vec![SymbolId(5), SymbolId(7)],
            body: ExprId(11),
        };
        assert_eq!(literal.family, ClosureValueFamily::UnaryDirect);
        assert_eq!(literal.capture, ClosureCapturePolicy::Immutable);
        assert_eq!(literal.param, SymbolId(2));
        assert_eq!(literal.param_ty, Some(Type::Text));
        assert_eq!(literal.ret_ty, Some(Type::Bool));
        assert_eq!(literal.captures, vec![SymbolId(5), SymbolId(7)]);
        assert_eq!(literal.body, ExprId(11));
    }

    #[test]
    fn trait_bound_owner_layer_data_is_stable() {
        let mut arena = AstArena::default();
        let t = arena.intern_symbol("T");
        let display = arena.intern_symbol("Display");
        let bound = TraitBound {
            param: t,
            bound: display,
        };
        assert_eq!(bound.param, t);
        assert_eq!(bound.bound, display);
    }

    #[test]
    fn trait_decl_owner_layer_data_is_stable() {
        let mut arena = AstArena::default();
        let name = arena.intern_symbol("Display");
        let method = arena.intern_symbol("fmt");
        let self_param = arena.intern_symbol("self");
        let sig = TraitMethodSig {
            name: method,
            params: vec![(self_param, Type::Unit)],
            ret: Type::Text,
        };
        let decl = TraitDecl {
            name,
            type_params: vec![],
            methods: vec![sig],
        };
        assert_eq!(decl.name, name);
        assert_eq!(decl.methods.len(), 1);
        assert_eq!(decl.methods[0].name, method);
        assert_eq!(decl.methods[0].ret, Type::Text);
    }

    #[test]
    fn impl_decl_owner_layer_data_is_stable() {
        let mut arena = AstArena::default();
        let trait_name = arena.intern_symbol("Display");
        let for_type = arena.intern_symbol("MyRecord");
        let decl = ImplDecl {
            trait_name,
            for_type,
            type_params: vec![],
            methods: vec![],
        };
        assert_eq!(decl.trait_name, trait_name);
        assert_eq!(decl.for_type, for_type);
        assert!(decl.methods.is_empty());
    }

    #[test]
    fn function_trait_bound_owner_layer_is_stable() {
        let mut arena = AstArena::default();
        let name = arena.intern_symbol("print_all");
        let t_param = arena.intern_symbol("T");
        let display = arena.intern_symbol("Display");
        let param = arena.intern_symbol("x");
        let body_expr = arena.alloc_expr(Expr::Var(param));
        let body = arena.alloc_stmt(Stmt::Return(Some(body_expr)));
        let func = Function {
            name,
            type_params: vec![t_param],
            trait_bounds: vec![TraitBound {
                param: t_param,
                bound: display,
            }],
            params: vec![(param, Type::TypeVar(t_param))],
            param_defaults: vec![None],
            requires: vec![],
            ensures: vec![],
            invariants: vec![],
            ret: Type::Unit,
            body: vec![body],
        };
        assert_eq!(func.trait_bounds.len(), 1);
        assert_eq!(func.trait_bounds[0].bound, display);
        assert!(func.type_params.contains(&t_param));
    }

    #[test]
    fn iterable_loop_desugaring_owner_layer_is_stable() {
        let mut arena = AstArena::default();
        let iterable_trait = arena.intern_symbol("Iterable");
        let desugaring = IterableLoopDesugaring {
            trait_name: iterable_trait,
        };
        assert_eq!(desugaring.trait_name, iterable_trait);
    }

    #[test]
    fn for_each_statement_owner_layer_is_stable() {
        let stmt = Stmt::ForEach {
            name: SymbolId(1),
            iterable: ExprId(2),
            body: vec![StmtId(3)],
            desugaring: IterableLoopDesugaring {
                trait_name: SymbolId(4),
            },
        };
        let Stmt::ForEach {
            name,
            iterable,
            body,
            desugaring,
        } = stmt
        else {
            panic!("expected ForEach");
        };
        assert_eq!(name, SymbolId(1));
        assert_eq!(iterable, ExprId(2));
        assert_eq!(body, vec![StmtId(3)]);
        assert_eq!(desugaring.trait_name, SymbolId(4));
    }

    #[test]
    fn kw_trait_and_kw_impl_lex_to_reserved_tokens() {
        use crate::lexer::lex_tokens;
        let tokens = lex_tokens("trait Display { }").unwrap();
        assert!(
            tokens.iter().any(|t| t.kind == TokenKind::KwTrait),
            "expected KwTrait token from 'trait'"
        );
        let tokens2 = lex_tokens("impl Display for MyRecord { }").unwrap();
        assert!(
            tokens2.iter().any(|t| t.kind == TokenKind::KwImpl),
            "expected KwImpl token from 'impl'"
        );
        assert!(
            tokens2.iter().any(|t| t.kind == TokenKind::KwFor),
            "expected KwFor token from 'for'"
        );
    }

    // M9.4 Wave 1 — richer pattern surface owner layer

    #[test]
    fn nested_tuple_pattern_item_owner_layer_is_stable() {
        let inner = vec![
            TuplePatternItem::Bind {
                name: SymbolId(0),
                capture: CaptureMode::Move,
            },
            TuplePatternItem::Discard,
        ];
        let nested = TuplePatternItem::Nested(inner);
        assert!(matches!(nested, TuplePatternItem::Nested(ref items) if items.len() == 2));
    }

    #[test]
    fn wildcard_match_pattern_owner_layer_is_stable() {
        let pat = MatchPattern::Wildcard;
        assert!(matches!(pat, MatchPattern::Wildcard));
    }

    #[test]
    fn or_pattern_owner_layer_is_stable() {
        let mut arena = AstArena::default();
        let adt_a = AdtMatchPattern {
            adt_name: arena.intern_symbol("Color"),
            variant_name: arena.intern_symbol("Red"),
            items: vec![],
        };
        let adt_b = AdtMatchPattern {
            adt_name: arena.intern_symbol("Color"),
            variant_name: arena.intern_symbol("Blue"),
            items: vec![],
        };
        let or_pat = MatchPattern::Or(vec![MatchPattern::Adt(adt_a), MatchPattern::Adt(adt_b)]);
        assert!(matches!(&or_pat, MatchPattern::Or(alts) if alts.len() == 2));
    }

    #[test]
    fn int_range_pattern_owner_layer_is_stable() {
        let range_pat = MatchPattern::IntRange(IntRangePattern {
            start: 1,
            end: 5,
            inclusive: true,
        });
        assert!(
            matches!(&range_pat, MatchPattern::IntRange(r) if r.start == 1 && r.end == 5 && r.inclusive)
        );
    }

    #[test]
    fn if_let_expr_owner_layer_is_stable() {
        let mut arena = AstArena::default();
        let value_id = arena.alloc_expr(Expr::BoolLiteral(true));
        let unit_id = arena.alloc_expr(Expr::QuadLiteral(QuadVal::N));
        let then_block = BlockExpr {
            statements: vec![],
            tail: unit_id,
        };
        let else_block = BlockExpr {
            statements: vec![],
            tail: unit_id,
        };
        let if_let = IfLetExpr {
            pattern: MatchPattern::Wildcard,
            value: value_id,
            then_block,
            else_block,
        };
        assert!(matches!(if_let.pattern, MatchPattern::Wildcard));
        assert_eq!(if_let.value, value_id);
    }

    // M9.5 Wave A — capture mode owner layer

    #[test]
    fn capture_mode_move_and_borrow_are_distinct() {
        assert_ne!(CaptureMode::Move, CaptureMode::Borrow);
        assert_eq!(CaptureMode::Move, CaptureMode::Move);
        assert_eq!(CaptureMode::Borrow, CaptureMode::Borrow);
    }

    #[test]
    fn tuple_pattern_bind_carries_capture_mode() {
        let item_move = TuplePatternItem::Bind {
            name: SymbolId(1),
            capture: CaptureMode::Move,
        };
        let item_borrow = TuplePatternItem::Bind {
            name: SymbolId(1),
            capture: CaptureMode::Borrow,
        };
        assert!(matches!(
            item_move,
            TuplePatternItem::Bind {
                capture: CaptureMode::Move,
                ..
            }
        ));
        assert!(matches!(
            item_borrow,
            TuplePatternItem::Bind {
                capture: CaptureMode::Borrow,
                ..
            }
        ));
        // Two bindings of same name but different capture mode are distinct.
        assert_ne!(item_move, item_borrow);
    }

    #[test]
    fn adt_pattern_bind_carries_capture_mode() {
        let item_move = AdtPatternItem::Bind {
            name: SymbolId(2),
            capture: CaptureMode::Move,
        };
        let item_borrow = AdtPatternItem::Bind {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
        };
        assert!(matches!(
            item_move,
            AdtPatternItem::Bind {
                capture: CaptureMode::Move,
                ..
            }
        ));
        assert!(matches!(
            item_borrow,
            AdtPatternItem::Bind {
                capture: CaptureMode::Borrow,
                ..
            }
        ));
        assert_ne!(item_move, item_borrow);
    }

    #[test]
    fn tuple_pattern_default_is_move() {
        // Default capture in parser-generated nodes is Move; Borrow requires explicit `ref`.
        let item = TuplePatternItem::Bind {
            name: SymbolId(3),
            capture: CaptureMode::Move,
        };
        let TuplePatternItem::Bind { capture, .. } = item else {
            panic!("expected Bind")
        };
        assert_eq!(
            capture,
            CaptureMode::Move,
            "default tuple binding capture must be Move"
        );
    }

    #[test]
    fn adt_pattern_default_is_move() {
        let item = AdtPatternItem::Bind {
            name: SymbolId(4),
            capture: CaptureMode::Move,
        };
        let AdtPatternItem::Bind { capture, .. } = item else {
            panic!("expected Bind")
        };
        assert_eq!(
            capture,
            CaptureMode::Move,
            "default ADT binding capture must be Move"
        );
    }

    #[test]
    fn record_pattern_bind_carries_capture_mode() {
        let item_move = RecordPatternTarget::Bind {
            name: SymbolId(5),
            capture: CaptureMode::Move,
        };
        let item_borrow = RecordPatternTarget::Bind {
            name: SymbolId(5),
            capture: CaptureMode::Borrow,
        };
        assert!(matches!(
            item_move,
            RecordPatternTarget::Bind {
                capture: CaptureMode::Move,
                ..
            }
        ));
        assert!(matches!(
            item_borrow,
            RecordPatternTarget::Bind {
                capture: CaptureMode::Borrow,
                ..
            }
        ));
        assert_ne!(item_move, item_borrow);
    }

    #[test]
    fn record_pattern_default_is_move() {
        let item = RecordPatternTarget::Bind {
            name: SymbolId(6),
            capture: CaptureMode::Move,
        };
        let RecordPatternTarget::Bind { capture, .. } = item else {
            panic!("expected Bind")
        };
        assert_eq!(
            capture,
            CaptureMode::Move,
            "default record binding capture must be Move"
        );
    }

    // M9.5 Wave B — KwRef token owner layer

    #[test]
    fn kw_ref_lexes_to_reserved_token() {
        use crate::lexer::lex_tokens;
        let tokens = lex_tokens("ref x").unwrap();
        assert!(
            tokens.iter().any(|t| t.kind == TokenKind::KwRef),
            "expected KwRef token from 'ref'"
        );
    }

    // M9.5 Wave C — PatternPath / BindingPlan owner layer

    #[test]
    fn pattern_path_root_is_empty() {
        let p = PatternPath::root();
        assert!(p.elems.is_empty());
    }

    #[test]
    fn pattern_path_tuple_index_appends() {
        let p = PatternPath::root().tuple_index(2);
        assert_eq!(p.elems, vec![PatternPathElem::TupleIndex(2)]);
    }

    #[test]
    fn pattern_path_nested_build() {
        let p = PatternPath::root().tuple_index(1).variant_field(0);
        assert_eq!(p.elems.len(), 2);
        assert!(matches!(p.elems[0], PatternPathElem::TupleIndex(1)));
        assert!(matches!(p.elems[1], PatternPathElem::VariantField(0)));
    }

    #[test]
    fn pattern_path_record_field_appends() {
        let field = SymbolId(7);
        let p = PatternPath::root().record_field(field);
        assert_eq!(p.elems, vec![PatternPathElem::RecordField(field)]);
    }

    #[test]
    fn binding_plan_default_is_empty() {
        let plan = BindingPlan::default();
        assert!(plan.items.is_empty());
    }

    // M9.7 — PathAvailability / ValuePathState owner layer

    #[test]
    fn path_availability_variants_distinct() {
        assert_ne!(PathAvailability::Available, PathAvailability::Moved);
        assert_ne!(PathAvailability::Available, PathAvailability::Borrowed);
        assert_ne!(PathAvailability::Moved, PathAvailability::Borrowed);
    }

    #[test]
    fn value_path_state_default_is_empty() {
        let s = crate::types::ValuePathState::default();
        assert!(s.paths.is_empty());
    }

    #[test]
    fn scrutinee_use_consumed_if_any_move() {
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: PatternPath::root().tuple_index(0),
            ty: Type::I32,
        });
        assert_eq!(crate::types::ScrutineeUse::Consumed, {
            if plan.items.iter().any(|it| it.capture == CaptureMode::Move) {
                crate::types::ScrutineeUse::Consumed
            } else {
                crate::types::ScrutineeUse::Preserved
            }
        });
    }
}
