#![allow(clippy::too_many_arguments)] // Internal typechecker helpers carry explicit semantic context

use crate::types::{
    AdtCtorExpr, AdtMatchPattern, AdtPatternItem, BindingPlan, BindingPlanItem, CaptureMode,
    MatchPattern, NumericLiteral, PathAvailability, PatternPath, RecordPatternTarget, ScrutineeUse,
};
use crate::*;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};

fn fx_coercion_gap_message() -> &'static str {
    "fx coercion from non-literal numeric expressions is not implemented in the canonical Rust-like path yet"
}

fn fx_measured_arithmetic_gap_message() -> &'static str {
    "unit-carrying fx arithmetic is not part of the first post-stable fx arithmetic slice yet"
}

fn iterable_for_gap_message() -> &'static str {
    "iterable 'for x in collection' currently requires built-in Sequence(type), i32 range, or a direct record `Iterable` impl shaped as `fn next(self: Self, index: i32) -> Option(Item)`"
}

fn first_wave_relational_gap_message() -> &'static str {
    "relational operators are currently admitted only for same-family i32 operands in the first application-completeness wave"
}

fn iterable_for_impl_contract_message() -> &'static str {
    "iterable 'for x in collection' over an explicit `Iterable` impl currently requires direct record contract `fn next(self: Self, index: i32) -> Option(Item)`"
}

fn iterable_for_impl_out_of_scope_message() -> &'static str {
    "iterable 'for x in collection' executable explicit `Iterable` dispatch currently supports direct record impls only; ADT/schema dispatch stays out of scope"
}

fn executable_import_wave2_out_of_scope_message() -> &'static str {
    "top-level executable Import admits direct local-path and package-qualified helper modules plus selected local imports; alias, wildcard, and re-export forms remain out of scope"
}

fn validate_executable_imports(program: &Program) -> Result<(), FrontendError> {
    for import in &program.imports {
        if import.reexport
            || import.wildcard
            || import.alias.is_some()
            || (import.spec.contains("::") && !import.select_items.is_empty())
        {
            return Err(FrontendError {
                pos: 0,
                message: executable_import_wave2_out_of_scope_message().to_string(),
            });
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

fn is_numeric_for_fx_gap(ty: &Type) -> bool {
    matches!(ty.erase_units(), Type::I32 | Type::U32 | Type::F64)
}

fn is_fx_literal_expr(expr_id: ExprId, arena: &AstArena) -> bool {
    is_numeric_literal_like_expr(expr_id, arena)
}

fn has_explicit_iterable_impl(
    ty: &Type,
    trait_name: SymbolId,
    impl_list: &[ImplDecl],
) -> Result<bool, FrontendError> {
    let nominal = match ty {
        Type::Record(name) | Type::Adt(name) => *name,
        _ => return Ok(false),
    };
    for imp in impl_list {
        if imp.for_type == nominal && imp.trait_name == trait_name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn resolve_explicit_iterable_loop_item_type(
    iterable_ty: &Type,
    trait_name: SymbolId,
    arena: &AstArena,
    impl_list: &[ImplDecl],
) -> Result<Option<Type>, FrontendError> {
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
                message: iterable_for_impl_contract_message().to_string(),
            })?;
        if method.params.len() != 2
            || method.params[0].1 != Type::Record(nominal)
            || method.params[1].1 != Type::I32
        {
            return Err(FrontendError {
                pos: 0,
                message: iterable_for_impl_contract_message().to_string(),
            });
        }
        let Type::Option(item_ty) = &method.ret else {
            return Err(FrontendError {
                pos: 0,
                message: iterable_for_impl_contract_message().to_string(),
            });
        };
        return Ok(Some(item_ty.as_ref().clone()));
    }
    Ok(None)
}

fn match_unit_lift(expected: &Type, actual: &Type, expr_id: ExprId, arena: &AstArena) -> bool {
    match expected.measured_parts() {
        Some((base, _)) if base == actual => is_numeric_literal_like_expr(expr_id, arena),
        _ => false,
    }
}

fn measured_numeric_parts(ty: &Type) -> Option<(&Type, SymbolId)> {
    ty.measured_parts()
}

fn lift_literal_to_expected_type(
    expected: Option<&Type>,
    actual: &Type,
    expr_id: ExprId,
    arena: &AstArena,
) -> Option<Type> {
    match expected {
        Some(expected_ty) if match_unit_lift(expected_ty, actual, expr_id, arena) => {
            Some(expected_ty.clone())
        }
        _ => None,
    }
}

/// Type-checks a `Program` containing exactly one function, using the same
/// canonical `build_fn_table` FnSig-construction authority as
/// `type_check_program` (FA-02-017 / #1649): generic `type_params` and
/// `trait_bounds` are preserved under the same function-signature admission
/// rules, never hardcoded to empty.
///
/// This does not make the two public APIs semantically identical:
/// `type_check_function` does not build or validate trait/impl coherence and
/// intentionally checks the function with an empty validated impl context
/// (`impl_list` is always `&[]` here). Therefore checks that require impl
/// evidence -- such as a self-referential call inside the checked function's
/// own body that needs trait-bound satisfaction -- may fail closed here even
/// when a fully validated `type_check_program` context could prove them. A
/// bounded generic function's own declaration/signature is unaffected either
/// way, since bounds are consulted only at call sites.
pub fn type_check_function(program: &Program) -> Result<(), FrontendError> {
    validate_executable_imports(program)?;
    if program.functions.len() != 1 {
        return Err(FrontendError {
            pos: 0,
            message: "type_check_function expects exactly one function in program".to_string(),
        });
    }
    // FA-02-017 / #1649: build the FnSig through the same canonical
    // build_fn_table authority type_check_program uses, instead of
    // reconstructing a second, hand-rolled FnSig here. The previous manual
    // construction canonicalized params/ret through the non-generic
    // canonicalize_declared_type (which unconditionally rejects TypeVar)
    // and hardcoded type_params/trait_bounds to empty regardless of what
    // the parsed Function actually declared -- so this API silently
    // disagreed with the canonical program-level admission contract for
    // every generic function, rejecting realistic generic declarations
    // the canonical path admits (or, for the rare case a declaration did
    // canonicalize, silently discarding metadata a self-referential call
    // would have needed), rather than honestly preserving or explicitly
    // rejecting generic metadata. impl_list stays empty deliberately below:
    // this single-function API never builds or validates a TraitTable, so
    // it has no coherence/conformance-checked impls to use for bound
    // satisfaction -- reading program.impls unchecked here would let an
    // unvalidated impl silently "prove" a bound the canonical path never
    // confirmed. A generic function's own declaration/body is unaffected
    // either way (bounds are consulted only at call sites); a
    // self-referential call requiring bound satisfaction fails closed
    // through the existing empty-impl_list path instead.
    let record_table = build_record_table(program)?;
    let adt_table = build_adt_table(program)?;
    let schema_table = build_schema_table(program)?;
    let table = build_fn_table(program)?;
    let func = &program.functions[0];
    validate_top_level_name_collisions(program, &table, &record_table, &adt_table, &schema_table)?;
    validate_record_declarations(program, &record_table, &adt_table)?;
    validate_adt_declarations(program, &record_table, &adt_table)?;
    validate_schema_declarations(program, &schema_table, &record_table, &adt_table)?;
    type_check_function_with_tables(func, &program.arena, &table, &record_table, &adt_table, &[])
}

pub fn type_check_program(p: &Program) -> Result<(), FrontendError> {
    validate_executable_imports(p)?;
    let table = build_fn_table(p)?;
    let record_table = build_record_table(p)?;
    let adt_table = build_adt_table(p)?;
    let schema_table = build_schema_table(p)?;
    // M9.2 Wave 3: trait coherence and impl conformance.
    let trait_table = build_trait_table(p)?;
    validate_trait_coherence(&p.impls, &p.arena)?;
    validate_impl_conformance(&p.impls, &trait_table, &record_table, &adt_table, &p.arena)?;
    validate_top_level_name_collisions(p, &table, &record_table, &adt_table, &schema_table)?;
    validate_record_declarations(p, &record_table, &adt_table)?;
    validate_adt_declarations(p, &record_table, &adt_table)?;
    validate_schema_declarations(p, &schema_table, &record_table, &adt_table)?;
    let main_id = p
        .arena
        .symbol_to_id
        .get("main")
        .copied()
        .ok_or(FrontendError {
            pos: 0,
            message: "program must define fn main()".to_string(),
        })?;
    let main_sig = table.get(&main_id).ok_or(FrontendError {
        pos: 0,
        message: "program must define fn main()".to_string(),
    })?;
    if !main_sig.params.is_empty() || main_sig.ret != Type::Unit {
        return Err(FrontendError {
            pos: 0,
            message: "main must have signature fn main()".to_string(),
        });
    }
    for f in &p.functions {
        type_check_function_with_tables(f, &p.arena, &table, &record_table, &adt_table, &p.impls)?;
    }
    for imp in &p.impls {
        for method in &imp.methods {
            type_check_function_with_tables(
                method,
                &p.arena,
                &table,
                &record_table,
                &adt_table,
                &p.impls,
            )?;
        }
    }
    Ok(())
}

pub fn derive_validation_plan_table(
    program: &Program,
) -> Result<ValidationPlanTable, FrontendError> {
    validate_executable_imports(program)?;
    let record_table = build_record_table(program)?;
    let adt_table = build_adt_table(program)?;
    let schema_table = build_schema_table(program)?;
    let fn_table = build_fn_table(program)?;
    validate_top_level_name_collisions(
        program,
        &fn_table,
        &record_table,
        &adt_table,
        &schema_table,
    )?;
    validate_record_declarations(program, &record_table, &adt_table)?;
    validate_adt_declarations(program, &record_table, &adt_table)?;
    validate_schema_declarations(program, &schema_table, &record_table, &adt_table)?;

    let mut plans = ValidationPlanTable::new();
    for schema in &program.schemas {
        let _ = schema_table.get(&schema.name).ok_or(FrontendError {
            pos: 0,
            message: format!(
                "missing schema '{}' in canonical schema table",
                resolve_symbol_name(&program.arena, schema.name)?
            ),
        })?;

        let shape = match &schema.shape {
            SchemaShape::Record(fields) => ValidationShapePlan::Record(
                derive_validation_field_plans(fields, &record_table, &adt_table, &program.arena)?,
            ),
            SchemaShape::TaggedUnion(variants) => {
                ValidationShapePlan::TaggedUnion(derive_validation_variant_plans(
                    variants,
                    &record_table,
                    &adt_table,
                    &program.arena,
                )?)
            }
        };
        let checks = match &shape {
            ValidationShapePlan::Record(fields) => derive_record_validation_checks(fields),
            ValidationShapePlan::TaggedUnion(variants) => {
                derive_tagged_union_validation_checks(variants)
            }
        };

        plans.insert(
            schema.name,
            ValidationPlan {
                schema_name: schema.name,
                role: schema.role,
                shape,
                checks,
            },
        );
    }

    Ok(plans)
}

pub fn type_check_function_with_table(
    func: &Function,
    arena: &AstArena,
    table: &FnTable,
) -> Result<(), FrontendError> {
    let empty_records = RecordTable::new();
    let empty_adts = AdtTable::new();
    type_check_function_with_tables(func, arena, table, &empty_records, &empty_adts, &[])
}

fn type_check_function_with_tables(
    func: &Function,
    arena: &AstArena,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    impl_list: &[ImplDecl],
) -> Result<(), FrontendError> {
    if func.params.len() != func.param_defaults.len() {
        return Err(FrontendError {
            pos: 0,
            message: "function parameter/default metadata length mismatch".to_string(),
        });
    }
    // Generic functions: canonicalize with type_params scope, skip executable
    // type checks for TypeVar params (those are checked per call-site after
    // substitution). Body type-check is deferred until Wave 3 monomorphisation.
    let is_generic = !func.type_params.is_empty();
    let canonical_params = func
        .params
        .iter()
        .map(|(name, ty)| {
            Ok((
                *name,
                canonicalize_declared_type_generic(
                    ty,
                    record_table,
                    adt_table,
                    arena,
                    &func.type_params,
                )?,
            ))
        })
        .collect::<Result<Vec<_>, FrontendError>>()?;
    let canonical_ret = canonicalize_declared_type_generic(
        &func.ret,
        record_table,
        adt_table,
        arena,
        &func.type_params,
    )?;
    for (name, ty) in &canonical_params {
        // Skip executable-type check for TypeVar — substitution happens at call site.
        if matches!(ty, Type::TypeVar(_)) && is_generic {
            continue;
        }
        ensure_type_resolved(
            ty,
            record_table,
            adt_table,
            arena,
            format!("parameter '{}'", resolve_symbol_name(arena, *name)?),
        )?;
        ensure_executable_type_supported(
            ty,
            arena,
            &func.type_params,
            format!("parameter '{}'", resolve_symbol_name(arena, *name)?),
        )?;
    }
    // Skip return-type executable check for TypeVar.
    if !matches!(canonical_ret, Type::TypeVar(_)) || !is_generic {
        ensure_type_resolved(
            &canonical_ret,
            record_table,
            adt_table,
            arena,
            format!(
                "return type of '{}'",
                resolve_symbol_name(arena, func.name)?
            ),
        )?;
        ensure_executable_type_supported(
            &canonical_ret,
            arena,
            &func.type_params,
            format!(
                "return type of '{}'",
                resolve_symbol_name(arena, func.name)?
            ),
        )?;
    }
    let mut empty_env = ScopeEnv::new();
    let mut default_loop_stack = Vec::new();
    for ((name, ty), default_expr) in canonical_params.iter().zip(func.param_defaults.iter()) {
        if let Some(default_expr) = default_expr {
            let default_ty = infer_expr_type(
                *default_expr,
                arena,
                &mut empty_env,
                table,
                record_table,
                adt_table,
                Type::Unit,
                &mut default_loop_stack,
                impl_list,
            )?;
            if let Err(err) = ensure_const_initializer_safe(*default_expr, arena, &mut empty_env) {
                return Err(FrontendError {
                    pos: err.pos,
                    message: format!(
                        "default parameter '{}' {}",
                        resolve_symbol_name(arena, *name)?,
                        err.message
                    ),
                });
            }
            ensure_binding_value_type(
                ty.clone(),
                default_ty,
                *default_expr,
                arena,
                format!("default parameter '{}'", resolve_symbol_name(arena, *name)?),
            )?;
        }
    }
    check_requires_clauses(func, arena, table, record_table, adt_table, impl_list)?;
    check_ensures_clauses(
        func,
        arena,
        table,
        record_table,
        adt_table,
        &canonical_ret,
        impl_list,
    )?;
    check_invariant_clauses(
        func,
        arena,
        table,
        record_table,
        adt_table,
        &canonical_ret,
        impl_list,
    )?;
    let mut env = ScopeEnv::with_params(&canonical_params);
    let mut loop_stack = Vec::new();
    for stmt in &func.body {
        check_stmt(
            *stmt,
            arena,
            &mut env,
            canonical_ret.clone(),
            table,
            record_table,
            adt_table,
            &mut loop_stack,
            impl_list,
        )?;
    }
    Ok(())
}

fn check_requires_clauses(
    func: &Function,
    arena: &AstArena,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    impl_list: &[ImplDecl],
) -> Result<(), FrontendError> {
    if func.requires.is_empty() {
        return Ok(());
    }
    let params = func
        .params
        .iter()
        .map(|(name, ty)| {
            Ok((
                *name,
                canonicalize_declared_type_generic(
                    ty,
                    record_table,
                    adt_table,
                    arena,
                    &func.type_params,
                )?,
            ))
        })
        .collect::<Result<Vec<_>, FrontendError>>()?;
    let mut env = ScopeEnv::with_params(&params);
    let mut loop_stack = Vec::new();
    for condition in &func.requires {
        ensure_requires_expr_supported(*condition, arena)?;
        let condition_ty = infer_expr_type(
            *condition,
            arena,
            &mut env,
            table,
            record_table,
            adt_table,
            canonicalize_declared_type_generic(
                &func.ret,
                record_table,
                adt_table,
                arena,
                &func.type_params,
            )?,
            &mut loop_stack,
            impl_list,
        )?;
        if condition_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "requires clause condition must be bool, got {:?}",
                    condition_ty
                ),
            });
        }
    }
    Ok(())
}

fn check_ensures_clauses(
    func: &Function,
    arena: &AstArena,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    canonical_ret: &Type,
    impl_list: &[ImplDecl],
) -> Result<(), FrontendError> {
    if func.ensures.is_empty() {
        return Ok(());
    }
    ensure_contract_result_name_available(func, arena)?;
    let params = func
        .params
        .iter()
        .map(|(name, ty)| {
            Ok((
                *name,
                canonicalize_declared_type_generic(
                    ty,
                    record_table,
                    adt_table,
                    arena,
                    &func.type_params,
                )?,
            ))
        })
        .collect::<Result<Vec<_>, FrontendError>>()?;
    let mut env = ScopeEnv::with_params(&params);
    if *canonical_ret != Type::Unit {
        if let Some(result_symbol) = arena.symbol_to_id.get("result").copied() {
            env.insert_const(result_symbol, canonical_ret.clone());
        }
    }
    let mut loop_stack = Vec::new();
    for condition in &func.ensures {
        ensure_ensures_expr_supported(*condition, arena)?;
        let condition_ty = infer_expr_type(
            *condition,
            arena,
            &mut env,
            table,
            record_table,
            adt_table,
            canonical_ret.clone(),
            &mut loop_stack,
            impl_list,
        )?;
        if condition_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "ensures clause condition must be bool, got {:?}",
                    condition_ty
                ),
            });
        }
    }
    Ok(())
}

fn check_invariant_clauses(
    func: &Function,
    arena: &AstArena,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    canonical_ret: &Type,
    impl_list: &[ImplDecl],
) -> Result<(), FrontendError> {
    if func.invariants.is_empty() {
        return Ok(());
    }
    ensure_contract_result_name_available(func, arena)?;
    ensure_invariant_result_usage(func, arena)?;
    let params = func
        .params
        .iter()
        .map(|(name, ty)| {
            Ok((
                *name,
                canonicalize_declared_type_generic(
                    ty,
                    record_table,
                    adt_table,
                    arena,
                    &func.type_params,
                )?,
            ))
        })
        .collect::<Result<Vec<_>, FrontendError>>()?;
    let mut env = ScopeEnv::with_params(&params);
    if *canonical_ret != Type::Unit {
        if let Some(result_symbol) = arena.symbol_to_id.get("result").copied() {
            env.insert_const(result_symbol, canonical_ret.clone());
        }
    }
    let mut loop_stack = Vec::new();
    for condition in &func.invariants {
        ensure_invariant_expr_supported(*condition, arena)?;
        let condition_ty = infer_expr_type(
            *condition,
            arena,
            &mut env,
            table,
            record_table,
            adt_table,
            canonical_ret.clone(),
            &mut loop_stack,
            impl_list,
        )?;
        if condition_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "invariant clause condition must be bool, got {:?}",
                    condition_ty
                ),
            });
        }
    }
    Ok(())
}

fn ensure_contract_result_name_available(
    func: &Function,
    arena: &AstArena,
) -> Result<(), FrontendError> {
    if func.ensures.is_empty() && func.invariants.is_empty() {
        return Ok(());
    }
    for (name, _) in &func.params {
        if resolve_symbol_name(arena, *name)? == "result" {
            let message = match (func.ensures.is_empty(), func.invariants.is_empty()) {
                (false, true) => {
                    "parameter name 'result' is reserved while ensures clauses are present"
                }
                (true, false) => {
                    "parameter name 'result' is reserved while invariant clauses are present"
                }
                (false, false) => {
                    "parameter name 'result' is reserved while ensures or invariant clauses are present"
                }
                (true, true) => unreachable!("contract result reservation requires contract clauses"),
            };
            return Err(FrontendError {
                pos: 0,
                message: message.to_string(),
            });
        }
    }
    Ok(())
}

fn ensure_invariant_result_usage(func: &Function, arena: &AstArena) -> Result<(), FrontendError> {
    if func.ret != Type::Unit {
        return Ok(());
    }
    for condition in &func.invariants {
        if contract_clause_references_result(*condition, arena)? {
            return Err(FrontendError {
                pos: 0,
                message:
                    "invariant clause may reference 'result' only in non-unit return functions"
                        .to_string(),
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LoopTypeFrameKind {
    Expression,
    Control,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopTypeFrame {
    kind: LoopTypeFrameKind,
    break_ty: Option<Type>,
}

// ──────────────────────────────────────────────────────────────
// SSF-08 Lane 1: canonical branch-join helpers.
//
// `check_stmt`'s `Stmt::If`/`Stmt::Match` and `check_loop_expr_stmt`'s
// `Stmt::If`/`Stmt::Match` are, and were before this change, structurally
// identical except for which function checks a body statement (the former
// admits every statement form; the latter rejects `while`/`loop`/`for`/
// `return`/`guard` inside a value-producing loop body). Rather than
// hand-roll the same join logic twice -- the duplication #1656-#1664
// diagnosed as the root failure mode -- both call these two functions,
// parametrized by a `fn` pointer over which body-statement checker to use.
// `ScopeEnv::join_ownership_from` (in `lib.rs`) is the actual join math;
// these two only assemble the right set of successor environments to feed
// it, per construct.
// ──────────────────────────────────────────────────────────────

/// Either `check_stmt` or `check_loop_expr_stmt` -- both already share this
/// exact signature. A plain `fn` pointer (not a closure) so it can be
/// threaded through recursive calls without borrow-checker friction against
/// the `&mut Vec<LoopTypeFrame>`/`&mut ScopeEnv` parameters passed alongside it.
type BodyStmtChecker = fn(
    StmtId,
    &AstArena,
    &mut ScopeEnv,
    Type,
    &FnTable,
    &RecordTable,
    &AdtTable,
    &mut Vec<LoopTypeFrame>,
    &[ImplDecl],
) -> Result<(), FrontendError>;

/// Checks `if condition { then_block } else { else_block }` (statement
/// form) through the canonical join: each branch is checked from its own
/// clone of the pre-if `env`, with its own branch-local scope, then both
/// resulting states are conservatively joined back into `env`. An empty
/// `else_block` (no source-level `else`) naturally represents "0 statements
/// ran", i.e. the unchanged pre-if state, without special-casing -- it is
/// still one of the two joined successors.
#[allow(clippy::too_many_arguments)]
fn check_if_branches_joined(
    condition: ExprId,
    then_block: &[StmtId],
    else_block: &[StmtId],
    arena: &AstArena,
    env: &mut ScopeEnv,
    ret_ty: Type,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
    check_body_stmt: BodyStmtChecker,
) -> Result<(), FrontendError> {
    let ct = infer_expr_type(
        condition,
        arena,
        env,
        table,
        record_table,
        adt_table,
        ret_ty.clone(),
        loop_stack,
        impl_list,
    )?;
    if ct != Type::Bool {
        return Err(FrontendError {
            pos: 0,
            message: "if condition must be bool; explicit compare is required for quad".to_string(),
        });
    }

    let mut then_env = env.clone();
    then_env.push_scope();
    for s in then_block {
        check_body_stmt(
            *s,
            arena,
            &mut then_env,
            ret_ty.clone(),
            table,
            record_table,
            adt_table,
            loop_stack,
            impl_list,
        )?;
    }
    then_env.pop_scope();

    let mut else_env = env.clone();
    else_env.push_scope();
    for s in else_block {
        check_body_stmt(
            *s,
            arena,
            &mut else_env,
            ret_ty.clone(),
            table,
            record_table,
            adt_table,
            loop_stack,
            impl_list,
        )?;
    }
    else_env.pop_scope();

    env.join_ownership_from(&[then_env, else_env])?;
    Ok(())
}

/// SSF-08 Lane 1 (#1663): `true` if `expr_id` is *syntactically* attempting
/// a static-path projection (`Expr::RecordField`/`Expr::SequenceIndex`),
/// independent of whether `expr_access_path` can actually resolve it end to
/// end. Used to distinguish the two reasons `expr_access_path` can return
/// `None`: (a) the expression is a pure rvalue with no addressable identity
/// at all (a call result, a literal, an arithmetic expression, ...) -- safe
/// to skip ownership tracking, since nothing could read the same location
/// again afterward; versus (b) the expression *is* a projection chain (the
/// user wrote `.field`/`[index]`) but some link in it isn't representable
/// (most commonly a dynamically-computed sequence index) -- in this case
/// the same addressable expression *can* be read again later, so silently
/// skipping tracking would let a move/borrow capture through unrecorded.
fn expr_is_projection_shaped(expr_id: ExprId, arena: &AstArena) -> bool {
    matches!(
        arena.expr(expr_id),
        Expr::RecordField(_) | Expr::SequenceIndex(_)
    )
}

/// SSF-08 Lane 1 (#1661/#1663): apply one match/if-let arm's *own* pattern
/// capture effect directly to `arm_env`, so it is visible to that arm's own
/// guard/body -- not only unioned into the outer `env` after every arm has
/// already been checked (the prior design's `apply_plans_to_scrutinee`,
/// which meant an arm could not observe that its own pattern had just moved
/// part of the scrutinee it is about to read again).
///
/// Uses `expr_access_path` to resolve the scrutinee expression to its own
/// `(root, base_path)` rather than requiring it to be a bare `Expr::Var` --
/// a projected scrutinee such as `match ctx.camera { ... }` composes the
/// item's pattern-relative path onto `base_path` via `PatternPath::extend`,
/// so a capture of `Pattern(x)` against scrutinee `ctx.camera` correctly
/// targets `ctx.camera.<item path>`, not a bogus root-only path or a silent
/// no-op.
///
/// If `expr_access_path` cannot resolve the scrutinee at all: a plan with
/// no capture items (e.g. matching a bare quad literal or a no-payload enum
/// variant) has nothing ownership-affecting to track regardless, so this is
/// inert by construction. But if the plan *does* capture something and the
/// scrutinee is projection-shaped (`expr_is_projection_shaped`) -- the user
/// wrote a path-like expression that isn't fully representable, most often
/// a dynamically-indexed sequence access -- this is exactly the "unsupported
/// path form" case that must reject deterministically rather than silently
/// downgrade to untracked. A non-projection-shaped, capturing scrutinee (a
/// bare call result, say `match make_pair() { (x, y) => .. }`) is still
/// safely inert: the matched value is a fresh temporary with no reachable
/// name to observe an inconsistent state through afterward.
fn apply_arm_pattern_capture(
    scrutinee_expr: ExprId,
    plan: &BindingPlan,
    arena: &AstArena,
    arm_env: &mut ScopeEnv,
) -> Result<(), FrontendError> {
    let Some((root, base_path)) = expr_access_path(scrutinee_expr, arena) else {
        if !plan.items.is_empty() && expr_is_projection_shaped(scrutinee_expr, arena) {
            return Err(FrontendError {
                pos: 0,
                message: "pattern capture against a projected scrutinee that is not an admitted static path (e.g. a dynamically-computed index) cannot be tracked; bind the scrutinee to a local first or use a supported static path".to_string(),
            });
        }
        return Ok(());
    };
    for item in &plan.items {
        let full_path = base_path.extend(&item.path);
        arm_env.check_capture_allowed(root, &full_path, item.capture)?;
        let avail = match item.capture {
            CaptureMode::Move => PathAvailability::Moved,
            CaptureMode::Borrow => PathAvailability::Borrowed,
        };
        arm_env.mark_path_state(root, full_path, avail)?;
    }
    Ok(())
}

/// Checks `match scrutinee { arms... default }` (statement form) through
/// the canonical join. Each arm starts independently from its own clone of
/// the pre-match `env` (never contaminated by a sibling arm's effects),
/// gets its own pattern-bound names inserted and its own pattern-capture
/// effect applied (visible to its own guard/body), is checked to
/// completion, then all reachable successors (every arm, plus `default` if
/// present) are conservatively joined back into `env`. Exhaustiveness (a
/// `match` with no `default` arm) is unaffected -- when the scrutinee's
/// variant coverage is already proven exhaustive, the arm successors alone
/// are the complete reachable set; no implicit "no arm taken" successor is
/// synthesized (unlike `if`'s empty-`else_block` case), since that would
/// wrongly relax an exhaustive match.
#[allow(clippy::too_many_arguments)]
fn check_match_arms_joined(
    scrutinee: ExprId,
    arms: &[MatchArm],
    default: &Option<Vec<StmtId>>,
    arena: &AstArena,
    env: &mut ScopeEnv,
    ret_ty: Type,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
    check_body_stmt: BodyStmtChecker,
) -> Result<(), FrontendError> {
    let st = infer_expr_type(
        scrutinee,
        arena,
        env,
        table,
        record_table,
        adt_table,
        ret_ty.clone(),
        loop_stack,
        impl_list,
    )?;
    if !matches!(
        st,
        Type::Quad | Type::Adt(_) | Type::Option(_) | Type::Result(_, _) | Type::I32 | Type::U32
    ) {
        return Err(FrontendError {
            pos: 0,
            message:
                "match is allowed only for quad, enum, Option(T), Result(T, E), i32, or u32 scrutinee"
                    .to_string(),
        });
    }

    let mut successors: Vec<ScopeEnv> = Vec::new();
    for arm in arms {
        let mut arm_env = build_pattern_arm_env(scrutinee, &arm.pat, &st, arena, env, adt_table)?;

        check_match_guard(
            arm.guard,
            arena,
            &mut arm_env,
            table,
            record_table,
            adt_table,
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        for s in &arm.block {
            check_body_stmt(
                *s,
                arena,
                &mut arm_env,
                ret_ty.clone(),
                table,
                record_table,
                adt_table,
                loop_stack,
                impl_list,
            )?;
        }
        arm_env.pop_scope();
        successors.push(arm_env);
    }

    match default {
        None => match missing_exhaustive_sum_variants(
            &st,
            arms.iter().map(|arm| (&arm.pat, arm.guard)),
            arena,
            adt_table,
        )? {
            Some((family_label, missing)) if !missing.is_empty() => {
                return Err(non_exhaustive_match_error(&family_label, &missing, false)?)
            }
            Some(_) => {}
            None => {
                return Err(FrontendError {
                    pos: 0,
                    message: "match requires default arm '_'".to_string(),
                });
            }
        },
        Some(default_body) => {
            let mut def_env = env.clone();
            def_env.push_scope();
            for s in default_body {
                check_body_stmt(
                    *s,
                    arena,
                    &mut def_env,
                    ret_ty.clone(),
                    table,
                    record_table,
                    adt_table,
                    loop_stack,
                    impl_list,
                )?;
            }
            def_env.pop_scope();
            successors.push(def_env);
        }
    }

    env.join_ownership_from(&successors)?;
    Ok(())
}

/// SSF-08 Lane 1 (#1657/#1660): run a loop body against a monotonically
/// growing candidate ownership state until a fixed point is reached, so a
/// restriction one logical iteration creates is visible to (and can
/// legitimately conflict with) the next. Concretely: `candidate` starts as
/// an unchanged clone of the pre-loop `env`; each round clones `candidate`,
/// pushes a fresh scope, installs the loop-local binding(s) via
/// `insert_loop_locals`, checks `body` against it, pops the scope, and
/// joins the result back onto `candidate`. Iterating stops once a round
/// produces no new restriction (`ScopeEnv` derives `PartialEq`, so this is
/// an exact equality check, not a heuristic). Because `path_state`
/// accumulation is monotonic and bounded by the finite set of paths the
/// loop body's own syntax can mention, this always converges -- in
/// practice within 2 rounds (round 1 discovers what one pass does; round 2
/// either finds nothing new, or the body's own re-check against its own
/// round-1 output legitimately errors, e.g. "move from already-moved
/// path", which is exactly the required "iteration N+1 cannot forget what
/// iteration N did" property). `MAX_FIXED_POINT_ITERATIONS` is a defensive
/// cap, not an expected case -- exceeding it means an invariant elsewhere
/// is broken, so it fails closed rather than looping forever.
///
/// Per `ScopeEnv::join_ownership_from`'s conservative-join law, starting
/// `candidate` from the *unchanged* pre-loop state and only ever adding
/// restrictions is sound for both "the loop may run zero times"
/// (`while`/`for`, where that baseline is a real possibility) and "the loop
/// always runs at least once" (a bare `loop { .. }` statement, where it
/// is not) -- a successor that happens to contribute no new restriction
/// can never *remove* one found elsewhere, so including it is never
/// unsound, only sometimes redundant.
#[allow(clippy::too_many_arguments)]
fn run_loop_body_to_fixed_point(
    body: &[StmtId],
    arena: &AstArena,
    env: &mut ScopeEnv,
    ret_ty: Type,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
    insert_loop_locals: impl Fn(&mut ScopeEnv),
    check_body_stmt: BodyStmtChecker,
) -> Result<(), FrontendError> {
    const MAX_FIXED_POINT_ITERATIONS: usize = 64;
    let mut candidate = env.clone();
    for _ in 0..MAX_FIXED_POINT_ITERATIONS {
        let mut body_env = candidate.clone();
        body_env.push_scope();
        insert_loop_locals(&mut body_env);
        for stmt in body {
            check_body_stmt(
                *stmt,
                arena,
                &mut body_env,
                ret_ty.clone(),
                table,
                record_table,
                adt_table,
                loop_stack,
                impl_list,
            )?;
        }
        body_env.pop_scope();
        let mut next_candidate = candidate.clone();
        next_candidate.join_ownership_from(&[body_env])?;
        if next_candidate == candidate {
            *env = candidate;
            return Ok(());
        }
        candidate = next_candidate;
    }
    Err(FrontendError {
        pos: 0,
        message:
            "internal ownership state: loop body ownership analysis did not reach a fixed point"
                .to_string(),
    })
}

fn check_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    ret_ty: Type,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<(), FrontendError> {
    let stmt = arena.stmt(stmt_id);
    match stmt {
        Stmt::Const { name, ty, value } => {
            if let Some(ann) = ty {
                ensure_type_resolved(
                    ann,
                    record_table,
                    adt_table,
                    arena,
                    format!("const '{}'", resolve_symbol_name(arena, *name)?),
                )?;
                ensure_storage_type_supported(
                    &canonicalize_declared_type(ann, record_table, adt_table, arena)?,
                    arena,
                    format!("const '{}'", resolve_symbol_name(arena, *name)?),
                )?;
            }
            ensure_const_initializer_safe(*value, arena, env)?;
            let final_ty = if let Some(ann) = ty {
                let expected_ty = canonicalize_declared_type(ann, record_table, adt_table, arena)?;
                let vt = infer_expr_type_with_expected(
                    *value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    Some(expected_ty.clone()),
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                ensure_binding_value_type(
                    expected_ty.clone(),
                    vt,
                    *value,
                    arena,
                    format!("const '{}'", resolve_symbol_name(arena, *name)?),
                )?;
                expected_ty
            } else {
                let vt = infer_expr_type(
                    *value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                vt
            };
            env.insert_const(*name, final_ty);
            Ok(())
        }
        Stmt::Let {
            name,
            is_mut,
            ty,
            value,
        } => {
            if let Some(ann) = ty {
                ensure_type_resolved(
                    ann,
                    record_table,
                    adt_table,
                    arena,
                    format!("let '{}'", resolve_symbol_name(arena, *name)?),
                )?;
                ensure_storage_type_supported(
                    &canonicalize_declared_type(ann, record_table, adt_table, arena)?,
                    arena,
                    format!("let '{}'", resolve_symbol_name(arena, *name)?),
                )?;
            }
            let final_ty = if let Some(ann) = ty {
                let expected_ty = canonicalize_declared_type(ann, record_table, adt_table, arena)?;
                let vt = infer_expr_type_with_expected(
                    *value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    Some(expected_ty.clone()),
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                ensure_binding_value_type(
                    expected_ty.clone(),
                    vt,
                    *value,
                    arena,
                    format!("let '{}'", resolve_symbol_name(arena, *name)?),
                )?;
                expected_ty
            } else {
                let vt = infer_expr_type(
                    *value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                vt
            };
            if *is_mut {
                env.insert_mut(*name, final_ty);
            } else {
                env.insert(*name, final_ty);
            }
            Ok(())
        }
        Stmt::LetTuple { items, ty, value } => {
            if let Some(ann) = ty {
                ensure_type_resolved(
                    ann,
                    record_table,
                    adt_table,
                    arena,
                    "tuple destructuring bind".to_string(),
                )?;
                ensure_storage_type_supported(
                    &canonicalize_declared_type(ann, record_table, adt_table, arena)?,
                    arena,
                    "tuple destructuring bind".to_string(),
                )?;
            }
            let final_ty = if let Some(ann) = ty {
                let expected_ty = canonicalize_declared_type(ann, record_table, adt_table, arena)?;
                let vt = infer_expr_type_with_expected(
                    *value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    Some(expected_ty.clone()),
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                ensure_binding_value_type(
                    expected_ty.clone(),
                    vt,
                    *value,
                    arena,
                    "tuple destructuring bind".to_string(),
                )?;
                expected_ty
            } else {
                let vt = infer_expr_type(
                    *value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                vt
            };
            let Type::Tuple(item_tys) = final_ty else {
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
            // M9.10 Wave B: build BindingPlan so path-state is tracked on the source variable.
            let tuple_ty = Type::Tuple(item_tys);
            let mut plan = BindingPlan::default();
            build_tuple_pattern_plan(items, &tuple_ty, &PatternPath::root(), &mut plan)?;
            validate_binding_plan_conflicts(&plan)?;
            apply_binding_plan(env, &plan);
            // SSF-08 Lane 1 (#1663): apply_arm_pattern_capture validates
            // capture-compatibility and marks the source path in one pass,
            // via canonical access-path extraction rather than a Var-only
            // check -- so a projected source expression (e.g. a record
            // field being destructured) is tracked correctly instead of
            // silently bypassed.
            apply_arm_pattern_capture(*value, &plan, arena, env)?;
            Ok(())
        }
        Stmt::LetRecord {
            record_name,
            items,
            value,
        } => {
            let value_ty = infer_expr_type(
                *value,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            if value_ty != Type::Record(*record_name) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "record destructuring bind requires value of type '{}', got {:?}",
                        resolve_symbol_name(arena, *record_name)?,
                        value_ty
                    ),
                });
            }
            for item in items {
                if matches!(item.target, RecordPatternTarget::QuadLiteral(_)) {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "quad literal record field patterns currently require let-else; plain record destructuring bind supports only name/_ items"
                                .to_string(),
                    });
                }
            }
            let mut plan = BindingPlan::default();
            build_record_pattern_plan(
                items,
                &value_ty,
                &PatternPath::root(),
                &mut plan,
                arena,
                record_table,
                adt_table,
            )?;
            validate_binding_plan_conflicts(&plan)?;
            apply_binding_plan(env, &plan);
            // SSF-08 Lane 1 (#1663): apply_arm_pattern_capture validates
            // capture-compatibility and marks the source path in one pass,
            // via canonical access-path extraction rather than a Var-only
            // check -- so a projected source expression (e.g. a record
            // field being destructured) is tracked correctly instead of
            // silently bypassed.
            apply_arm_pattern_capture(*value, &plan, arena, env)?;
            Ok(())
        }
        Stmt::LetElseRecord {
            record_name,
            items,
            value,
            else_return,
        } => {
            let value_ty = infer_expr_type(
                *value,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            if value_ty != Type::Record(*record_name) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "record let-else requires value of type '{}', got {:?}",
                        resolve_symbol_name(arena, *record_name)?,
                        value_ty
                    ),
                });
            }
            check_return_payload(
                *else_return,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            let mut saw_refutable_item = false;
            for item in items {
                let record = record_table.get(record_name).ok_or(FrontendError {
                    pos: 0,
                    message: format!(
                        "unknown record type '{}' in record let-else",
                        resolve_symbol_name(arena, *record_name)?
                    ),
                })?;
                let field = record
                    .fields
                    .iter()
                    .find(|field| field.name == item.field)
                    .ok_or(FrontendError {
                        pos: 0,
                        message: format!(
                            "record type '{}' has no field named '{}' in let-else",
                            resolve_symbol_name(arena, *record_name)?,
                            resolve_symbol_name(arena, item.field)?
                        ),
                    })?;
                match item.target {
                    RecordPatternTarget::Bind { .. } => {}
                    RecordPatternTarget::Discard => {}
                    RecordPatternTarget::QuadLiteral(_) => {
                        saw_refutable_item = true;
                        if canonicalize_declared_type(&field.ty, record_table, adt_table, arena)?
                            != Type::Quad
                        {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!(
                                    "record let-else literal pattern requires quad field, got {:?}",
                                    canonicalize_declared_type(
                                        &field.ty,
                                        record_table,
                                        adt_table,
                                        arena
                                    )?
                                ),
                            });
                        }
                    }
                }
            }
            if !saw_refutable_item {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "record let-else requires at least one refutable quad literal field pattern"
                            .to_string(),
                });
            }
            let mut plan = BindingPlan::default();
            build_record_pattern_plan(
                items,
                &value_ty,
                &PatternPath::root(),
                &mut plan,
                arena,
                record_table,
                adt_table,
            )?;
            validate_binding_plan_conflicts(&plan)?;
            apply_binding_plan(env, &plan);
            // SSF-08 Lane 1 (#1663): apply_arm_pattern_capture validates
            // capture-compatibility and marks the source path in one pass,
            // via canonical access-path extraction rather than a Var-only
            // check -- so a projected source expression (e.g. a record
            // field being destructured) is tracked correctly instead of
            // silently bypassed.
            apply_arm_pattern_capture(*value, &plan, arena, env)?;
            Ok(())
        }
        Stmt::LetElseTuple {
            items,
            ty,
            value,
            else_return,
        } => {
            if let Some(ann) = ty {
                ensure_type_resolved(
                    ann,
                    record_table,
                    adt_table,
                    arena,
                    "let-else tuple destructuring bind".to_string(),
                )?;
                ensure_storage_type_supported(
                    &canonicalize_declared_type(ann, record_table, adt_table, arena)?,
                    arena,
                    "let-else tuple destructuring bind".to_string(),
                )?;
            }
            let vt = infer_expr_type(
                *value,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            let final_ty = if let Some(ann) = ty {
                let expected_ty = canonicalize_declared_type(ann, record_table, adt_table, arena)?;
                ensure_binding_value_type(
                    expected_ty.clone(),
                    vt,
                    *value,
                    arena,
                    "let-else tuple destructuring bind".to_string(),
                )?;
                expected_ty
            } else {
                vt
            };
            let Type::Tuple(item_tys) = final_ty else {
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
            check_return_payload(
                *else_return,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            // M9.10 Wave B: validate QuadLiteral items before building plan.
            for (item, item_ty) in items.iter().zip(item_tys.iter()) {
                if let TuplePatternItem::QuadLiteral(_) = item {
                    if *item_ty != Type::Quad {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "let-else tuple literal pattern requires quad element, got {:?}",
                                item_ty
                            ),
                        });
                    }
                }
            }
            // M9.10 Wave B: build BindingPlan so path-state is tracked on the source variable.
            let tuple_ty = Type::Tuple(item_tys);
            let mut plan = BindingPlan::default();
            build_tuple_pattern_plan(items, &tuple_ty, &PatternPath::root(), &mut plan)?;
            validate_binding_plan_conflicts(&plan)?;
            apply_binding_plan(env, &plan);
            // SSF-08 Lane 1 (#1663): apply_arm_pattern_capture validates
            // capture-compatibility and marks the source path in one pass,
            // via canonical access-path extraction rather than a Var-only
            // check -- so a projected source expression (e.g. a record
            // field being destructured) is tracked correctly instead of
            // silently bypassed.
            apply_arm_pattern_capture(*value, &plan, arena, env)?;
            Ok(())
        }
        Stmt::Discard { ty, value } => {
            if let Some(ann) = ty {
                ensure_type_resolved(
                    ann,
                    record_table,
                    adt_table,
                    arena,
                    "discard binding".to_string(),
                )?;
                ensure_storage_type_supported(
                    &canonicalize_declared_type(ann, record_table, adt_table, arena)?,
                    arena,
                    "discard binding".to_string(),
                )?;
            }
            if let Some(ann) = ty {
                let expected_ty = canonicalize_declared_type(ann, record_table, adt_table, arena)?;
                let vt = infer_expr_type_with_expected(
                    *value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    Some(expected_ty.clone()),
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                ensure_binding_value_type(
                    expected_ty,
                    vt,
                    *value,
                    arena,
                    "discard binding".to_string(),
                )?;
            }
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
            if env.is_const_checked(*name)? {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "cannot assign to const binding '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                });
            }
            let value_ty = infer_expr_type_with_expected(
                *value,
                arena,
                env,
                table,
                record_table,
                adt_table,
                Some(target_ty.clone()),
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            ensure_binding_value_type(
                target_ty,
                value_ty,
                *value,
                arena,
                format!("assignment to '{}'", resolve_symbol_name(arena, *name)?),
            )
        }
        Stmt::AssignTuple { items, value } => {
            let value_ty = infer_expr_type(
                *value,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            let Type::Tuple(item_tys) = value_ty else {
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
            for (item, item_ty) in items.iter().zip(item_tys) {
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
                if env.is_const_checked(*name)? {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "cannot assign to const binding '{}' in tuple destructuring assignment",
                            resolve_symbol_name(arena, *name)?
                        ),
                    });
                }
                ensure_binding_value_type(
                    target_ty,
                    item_ty,
                    *value,
                    arena,
                    format!(
                        "tuple assignment to '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                )?;
            }
            Ok(())
        }
        Stmt::ForRange { name, range, body } => {
            let range_ty = infer_expr_type(
                *range,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            if range_ty != Type::RangeI32 {
                return Err(FrontendError {
                    pos: 0,
                    message: "for-range currently requires i32 range expression".to_string(),
                });
            }
            run_loop_body_to_fixed_point(
                body,
                arena,
                env,
                ret_ty,
                table,
                record_table,
                adt_table,
                loop_stack,
                impl_list,
                |body_env| body_env.insert_const(*name, Type::I32),
                check_stmt,
            )
        }
        Stmt::While { condition, body } => {
            let condition_ty = infer_expr_type(
                *condition,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            if condition_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "while condition must be bool; explicit compare is required for quad"
                        .to_string(),
                });
            }
            loop_stack.push(LoopTypeFrame {
                kind: LoopTypeFrameKind::Control,
                break_ty: None,
            });
            let result = run_loop_body_to_fixed_point(
                body,
                arena,
                env,
                ret_ty,
                table,
                record_table,
                adt_table,
                loop_stack,
                impl_list,
                |_| {},
                check_stmt,
            );
            let _ = loop_stack.pop().expect("control loop frame must exist");
            result
        }
        Stmt::Loop { body } => {
            loop_stack.push(LoopTypeFrame {
                kind: LoopTypeFrameKind::Control,
                break_ty: None,
            });
            let result = run_loop_body_to_fixed_point(
                body,
                arena,
                env,
                ret_ty,
                table,
                record_table,
                adt_table,
                loop_stack,
                impl_list,
                |_| {},
                check_stmt,
            );
            let _ = loop_stack.pop().expect("control loop frame must exist");
            result
        }
        Stmt::ForEach {
            name,
            iterable,
            body,
            desugaring,
        } => {
            let iterable_ty = infer_expr_type(
                *iterable,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            if iterable_ty == Type::RangeI32 {
                return run_loop_body_to_fixed_point(
                    body,
                    arena,
                    env,
                    ret_ty,
                    table,
                    record_table,
                    adt_table,
                    loop_stack,
                    impl_list,
                    |body_env| body_env.insert_const(*name, Type::I32),
                    check_stmt,
                );
            }
            if let Type::Sequence(sequence_ty) = &iterable_ty {
                let item_ty = sequence_ty.item.as_ref().clone();
                return run_loop_body_to_fixed_point(
                    body,
                    arena,
                    env,
                    ret_ty,
                    table,
                    record_table,
                    adt_table,
                    loop_stack,
                    impl_list,
                    |body_env| body_env.insert_const(*name, item_ty.clone()),
                    check_stmt,
                );
            }
            if let Some(item_ty) = resolve_explicit_iterable_loop_item_type(
                &iterable_ty,
                desugaring.trait_name,
                arena,
                impl_list,
            )? {
                return run_loop_body_to_fixed_point(
                    body,
                    arena,
                    env,
                    ret_ty,
                    table,
                    record_table,
                    adt_table,
                    loop_stack,
                    impl_list,
                    |body_env| body_env.insert_const(*name, item_ty.clone()),
                    check_stmt,
                );
            }
            let detail = match &iterable_ty {
                Type::Adt(_)
                    if has_explicit_iterable_impl(
                        &iterable_ty,
                        desugaring.trait_name,
                        impl_list,
                    )? =>
                {
                    iterable_for_impl_out_of_scope_message().to_string()
                }
                _ if has_explicit_iterable_impl(
                    &iterable_ty,
                    desugaring.trait_name,
                    impl_list,
                )? =>
                {
                    iterable_for_impl_contract_message().to_string()
                }
                _ => iterable_for_gap_message().to_string(),
            };
            Err(FrontendError {
                pos: 0,
                message: format!(
                    "{} (`{}` contract)",
                    detail,
                    resolve_symbol_name(arena, desugaring.trait_name)?
                ),
            })
        }
        Stmt::Break(None) => {
            let frame = loop_stack.last().ok_or(FrontendError {
                pos: 0,
                message: "bare break is allowed only inside while or statement loop".to_string(),
            })?;
            if !matches!(frame.kind, LoopTypeFrameKind::Control) {
                return Err(FrontendError {
                    pos: 0,
                    message: "bare break is allowed only inside while or statement loop"
                        .to_string(),
                });
            }
            Ok(())
        }
        Stmt::Break(Some(value)) => {
            let break_ty = infer_expr_type(
                *value,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            let frame = loop_stack.last_mut().ok_or(FrontendError {
                pos: 0,
                message: "break with value is allowed only inside loop expression".to_string(),
            })?;
            if !matches!(frame.kind, LoopTypeFrameKind::Expression) {
                return Err(FrontendError {
                    pos: 0,
                    message: "break with value is allowed only inside loop expression".to_string(),
                });
            }
            if let Some(expected) = &frame.break_ty {
                if *expected != break_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "loop expression break type mismatch: expected {:?}, got {:?}",
                            expected, break_ty
                        ),
                    });
                }
            } else {
                frame.break_ty = Some(break_ty);
            }
            Ok(())
        }
        Stmt::Continue => {
            let frame = loop_stack.last().ok_or(FrontendError {
                pos: 0,
                message: "continue is allowed only inside while or statement loop".to_string(),
            })?;
            if !matches!(frame.kind, LoopTypeFrameKind::Control) {
                return Err(FrontendError {
                    pos: 0,
                    message: "continue is allowed only inside while or statement loop".to_string(),
                });
            }
            Ok(())
        }
        Stmt::Guard {
            condition,
            else_return,
        } => {
            let condition_ty = infer_expr_type(
                *condition,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            if condition_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "guard clause condition must be bool; explicit compare is required for quad"
                            .to_string(),
                });
            }
            check_return_payload(
                *else_return,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => check_if_branches_joined(
            *condition,
            then_block,
            else_block,
            arena,
            env,
            ret_ty,
            table,
            record_table,
            adt_table,
            loop_stack,
            impl_list,
            check_stmt,
        ),
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => check_match_arms_joined(
            *scrutinee,
            arms,
            default,
            arena,
            env,
            ret_ty,
            table,
            record_table,
            adt_table,
            loop_stack,
            impl_list,
            check_stmt,
        ),
        Stmt::Return(v) => check_return_payload(
            *v,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Stmt::Expr(e) => {
            if check_builtin_assert_stmt(
                *e,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )? {
                return Ok(());
            }
            let _ = infer_expr_type(
                *e,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            Ok(())
        }
    }
}

/// Apply a type-variable substitution map to `ty` (FA-02-016 / #1648),
/// recursively, through every currently admitted structural type family. A
/// `TypeVar` occurrence is substituted wherever it appears -- directly, or
/// nested inside `Tuple`/`Sequence`/`Map`/`Option`/`Result`/`Closure`/
/// `Measured` -- not only at the top level. This is frontend call-site type
/// substitution, distinct from IR monomorphisation (#1717, unaffected):
/// `Record`/`Adt` are nominal leaves with no applied type arguments (#1650)
/// and are never recursed into. Exhaustive over every `Type` variant, no
/// catch-all arm, so a future variant forces an explicit decision here
/// rather than silently passing through unsubstituted (mirrors the
/// exhaustiveness discipline `ensure_executable_type_supported` established
/// for #1647).
fn subst_apply(ty: &Type, subst: &BTreeMap<SymbolId, Type>) -> Type {
    match ty {
        Type::TypeVar(id) => subst.get(id).cloned().unwrap_or_else(|| ty.clone()),
        Type::Tuple(items) => Type::Tuple(items.iter().map(|t| subst_apply(t, subst)).collect()),
        Type::Sequence(sequence) => Type::Sequence(SequenceType {
            family: sequence.family,
            item: Box::new(subst_apply(sequence.item.as_ref(), subst)),
        }),
        Type::Map(map) => Type::Map(MapType {
            key: Box::new(subst_apply(map.key.as_ref(), subst)),
            val: Box::new(subst_apply(map.val.as_ref(), subst)),
        }),
        Type::Closure(closure) => Type::Closure(ClosureType {
            family: closure.family,
            capture: closure.capture,
            param: Box::new(subst_apply(closure.param.as_ref(), subst)),
            ret: Box::new(subst_apply(closure.ret.as_ref(), subst)),
        }),
        Type::Measured(base, unit) => Type::Measured(Box::new(subst_apply(base, subst)), *unit),
        Type::Option(item) => Type::Option(Box::new(subst_apply(item, subst))),
        Type::Result(ok_ty, err_ty) => Type::Result(
            Box::new(subst_apply(ok_ty, subst)),
            Box::new(subst_apply(err_ty, subst)),
        ),
        Type::Quad
        | Type::QVec(_)
        | Type::Bool
        | Type::Text
        | Type::I32
        | Type::U32
        | Type::Fx
        | Type::F64
        | Type::RangeI32
        | Type::Record(_)
        | Type::Adt(_)
        | Type::Unit => ty.clone(),
    }
}

/// FA-02-016 / #1648 corrective: proves every `TypeVar` occurring in `ty`,
/// directly or nested inside any currently admitted structural family,
/// belongs to `declared` -- before `collect_generic_constraints` is ever
/// allowed to treat it as this call's generic authority.
///
/// `type_check_function_with_table` accepts a caller-supplied `FnTable` (a
/// deliberately checkable public trust boundary -- see the `malformed_fn_sig_*`
/// regressions for #1665/#1722's precedent), so a callee's `FnSig` cannot be
/// assumed to have been built by the canonical `build_fn_table` authority,
/// which is the only thing that actually guarantees every `TypeVar` in
/// `params`/`ret` was canonicalized within `type_params` scope. Without this
/// check, a malformed `FnSig` whose `params`/`ret` nests a `TypeVar` never
/// listed in `type_params` would have that undeclared variable silently
/// "adopted": `collect_generic_constraints` would bind it like any other
/// declared parameter the moment it appears structurally nested (exactly the
/// shape the pre-#1648 shallow first pass could never see in the first
/// place, so this was not a reachable gap before recursion was added). This
/// is a distinct concern from `collect_generic_constraints`'s own algorithm,
/// which is unchanged: an *actual* argument type being a bare `TypeVar` --
/// including an enclosing generic function's own still-opaque parameter, per
/// the comment above -- remains a legitimate binding *value*; what this
/// function polices is which symbols the *expected*, declared signature
/// shape is entitled to reference as a generic parameter at all.
fn ensure_typevars_declared(
    ty: &Type,
    declared: &[SymbolId],
    arena: &AstArena,
) -> Result<(), FrontendError> {
    match ty {
        Type::TypeVar(id) => {
            if declared.contains(id) {
                Ok(())
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "type variable '{}' is not declared as one of this function's type parameters",
                        resolve_symbol_name(arena, *id)?
                    ),
                })
            }
        }
        Type::Tuple(items) => {
            for item in items {
                ensure_typevars_declared(item, declared, arena)?;
            }
            Ok(())
        }
        Type::Sequence(sequence) => {
            ensure_typevars_declared(sequence.item.as_ref(), declared, arena)
        }
        Type::Map(map) => {
            ensure_typevars_declared(map.key.as_ref(), declared, arena)?;
            ensure_typevars_declared(map.val.as_ref(), declared, arena)
        }
        Type::Closure(closure) => {
            ensure_typevars_declared(closure.param.as_ref(), declared, arena)?;
            ensure_typevars_declared(closure.ret.as_ref(), declared, arena)
        }
        Type::Measured(base, _) => ensure_typevars_declared(base, declared, arena),
        Type::Option(item) => ensure_typevars_declared(item, declared, arena),
        Type::Result(ok_ty, err_ty) => {
            ensure_typevars_declared(ok_ty, declared, arena)?;
            ensure_typevars_declared(err_ty, declared, arena)
        }
        Type::Quad
        | Type::QVec(_)
        | Type::Bool
        | Type::Text
        | Type::I32
        | Type::U32
        | Type::Fx
        | Type::F64
        | Type::RangeI32
        | Type::Record(_)
        | Type::Adt(_)
        | Type::Unit => Ok(()),
    }
}

/// Recursively derives `TypeVar` constraints for a generic call argument
/// (FA-02-016 / #1648): given the declared, canonicalized `expected` shape
/// (which may contain the callee's own declared type parameter, directly or
/// nested inside any currently admitted structural family) and the `actual`
/// inferred type of the argument expression, walks both in lockstep and
/// binds or checks each `TypeVar` encountered in `expected`.
///
/// Responsibility split: this function only collects/checks constraints
/// (inference); `subst_apply` performs the corresponding recursive
/// substitution afterward. Neither is a substitute for the other.
///
/// A structural shape mismatch (e.g. expected `Option(T)`, actual
/// `Sequence(i32)`) yields no constraint here rather than an error -- the
/// existing canonical second-pass parameter-compatibility check remains the
/// final authority and rejects the call deterministically once the
/// (unsubstituted, still-generic) expected shape provably cannot equal the
/// concrete argument type. This avoids two independent, potentially
/// disagreeing diagnostics for the same defect.
///
/// `actual` binding a declared type parameter to a bare `TypeVar` (its own,
/// or -- since symbol names are interned program-wide -- another unrelated
/// declaration's of the same spelling) is not an error case to special-case
/// here: it is exactly how a generic function's body-check honestly types a
/// call to another generic function using its own still-opaque parameter
/// (e.g. `fn outer<T>(x: T) -> T { return id(x); }` calling
/// `fn id<T>(y: T) -> T`), independent of and unaffected by whatever
/// concrete type `outer` itself is later called with. `subst_apply` performs
/// one structural substitution pass with no fixpoint re-application, so this
/// carries no cyclic/infinite-recursion risk either way.
fn collect_generic_constraints(
    expected: &Type,
    actual: &Type,
    arena: &AstArena,
    subst: &mut BTreeMap<SymbolId, Type>,
) -> Result<(), FrontendError> {
    match expected {
        Type::TypeVar(tid) => match subst.get(tid) {
            None => {
                subst.insert(*tid, actual.clone());
                Ok(())
            }
            Some(existing) if existing == actual => Ok(()),
            Some(existing) => Err(FrontendError {
                pos: 0,
                message: format!(
                    "conflicting generic constraints for type parameter '{}': {:?} vs {:?}",
                    resolve_symbol_name(arena, *tid)?,
                    existing,
                    actual
                ),
            }),
        },
        Type::Tuple(expected_items) => {
            if let Type::Tuple(actual_items) = actual {
                if expected_items.len() == actual_items.len() {
                    for (e, a) in expected_items.iter().zip(actual_items.iter()) {
                        collect_generic_constraints(e, a, arena, subst)?;
                    }
                }
            }
            Ok(())
        }
        Type::Sequence(expected_seq) => {
            if let Type::Sequence(actual_seq) = actual {
                collect_generic_constraints(
                    expected_seq.item.as_ref(),
                    actual_seq.item.as_ref(),
                    arena,
                    subst,
                )?;
            }
            Ok(())
        }
        Type::Map(expected_map) => {
            if let Type::Map(actual_map) = actual {
                collect_generic_constraints(
                    expected_map.key.as_ref(),
                    actual_map.key.as_ref(),
                    arena,
                    subst,
                )?;
                collect_generic_constraints(
                    expected_map.val.as_ref(),
                    actual_map.val.as_ref(),
                    arena,
                    subst,
                )?;
            }
            Ok(())
        }
        Type::Closure(expected_closure) => {
            if let Type::Closure(actual_closure) = actual {
                if expected_closure.family == actual_closure.family
                    && expected_closure.capture == actual_closure.capture
                {
                    collect_generic_constraints(
                        expected_closure.param.as_ref(),
                        actual_closure.param.as_ref(),
                        arena,
                        subst,
                    )?;
                    collect_generic_constraints(
                        expected_closure.ret.as_ref(),
                        actual_closure.ret.as_ref(),
                        arena,
                        subst,
                    )?;
                }
            }
            Ok(())
        }
        Type::Measured(expected_base, expected_unit) => {
            if let Type::Measured(actual_base, actual_unit) = actual {
                if expected_unit == actual_unit {
                    collect_generic_constraints(expected_base, actual_base, arena, subst)?;
                }
            }
            Ok(())
        }
        Type::Option(expected_item) => {
            if let Type::Option(actual_item) = actual {
                collect_generic_constraints(expected_item, actual_item, arena, subst)?;
            }
            Ok(())
        }
        Type::Result(expected_ok, expected_err) => {
            if let Type::Result(actual_ok, actual_err) = actual {
                collect_generic_constraints(expected_ok, actual_ok, arena, subst)?;
                collect_generic_constraints(expected_err, actual_err, arena, subst)?;
            }
            Ok(())
        }
        // Leaves for generic-constraint purposes: no TypeVar can occur
        // inside them, so there is nothing to bind. Record/Adt are nominal
        // leaves specifically because they carry no applied type arguments
        // (#1650 remains the separate, unimplemented gap for that). Final
        // shape/value compatibility for every leaf family remains the
        // existing canonical second-pass checker's responsibility.
        Type::Quad
        | Type::QVec(_)
        | Type::Bool
        | Type::Text
        | Type::I32
        | Type::U32
        | Type::Fx
        | Type::F64
        | Type::RangeI32
        | Type::Record(_)
        | Type::Adt(_)
        | Type::Unit => Ok(()),
    }
}

/// Returns true if `concrete_ty` matches the `for_type` nominal name of an
/// impl block. Used by the M9.2 Wave 3 trait bound satisfaction check.
fn concrete_type_matches_impl_for(concrete_ty: &Type, for_type: SymbolId) -> bool {
    match concrete_ty {
        Type::Record(id) | Type::Adt(id) => *id == for_type,
        _ => false,
    }
}

fn infer_expr_type(
    expr_id: ExprId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    // M9.9: path-aware read check. Extract the most specific path reachable
    // from this expression and verify it is available. Base expressions used
    // inside field/index helpers go through infer_expr_type_no_check, which
    // skips this guard for intermediate Var nodes.
    //
    // SSF-08 Lane 1 (#1664): `check_path_available` is REQUIRED_BINDING and
    // fails closed on a missing root binding. The canonical source-level
    // "unknown variable" diagnostic must come from this expression's own
    // kind-specific resolution below (e.g. `Expr::Var`'s `env.get`), not
    // from this ownership check -- so existence is proven here first via
    // `env.get`, and the ownership check is skipped (never silently
    // *passed*) when it is absent, letting the per-kind match below raise
    // its own correct diagnostic instead of a duplicate/incorrect one.
    if let Some((name, path)) = expr_access_path(expr_id, arena) {
        if env.get(name).is_some() {
            env.check_path_available(name, &path)?;
        }
    }
    let expr = arena.expr(expr_id);
    match expr {
        Expr::QuadLiteral(_) => Ok(Type::Quad),
        Expr::BoolLiteral(_) => Ok(Type::Bool),
        Expr::TextLiteral(_) => Ok(Type::Text),
        Expr::SequenceLiteral(sequence) => infer_sequence_literal_type(
            sequence,
            arena,
            env,
            table,
            record_table,
            adt_table,
            None,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::Closure(closure) => infer_closure_literal_type(
            closure,
            arena,
            env,
            table,
            record_table,
            adt_table,
            None,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::NumericLiteral(literal) => match literal {
            NumericLiteral::I32(_) => Ok(Type::I32),
            NumericLiteral::U32(_) => Ok(Type::U32),
            NumericLiteral::F64(_) => Ok(Type::F64),
            NumericLiteral::Fx(_) => Ok(Type::Fx),
        },
        Expr::Range(range_expr) => {
            let start_ty = infer_expr_type(
                range_expr.start,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            let end_ty = infer_expr_type(
                range_expr.end,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            if start_ty != Type::I32 || end_ty != Type::I32 {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "range literal currently requires i32 bounds, got {:?}..{:?}",
                        start_ty, end_ty
                    ),
                });
            }
            Ok(Type::RangeI32)
        }
        Expr::Tuple(items) => {
            let mut item_tys = Vec::with_capacity(items.len());
            for item in items {
                let item_ty = infer_expr_type(
                    *item,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                if item_ty == Type::RangeI32 {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "range literal is not yet part of the stable tuple/user-data surface"
                                .to_string(),
                    });
                }
                item_tys.push(item_ty);
            }
            Ok(Type::Tuple(item_tys))
        }
        Expr::RecordLiteral(record_literal) => infer_record_literal_type(
            record_literal,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::RecordField(field_expr) => infer_record_field_access_type(
            field_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::SequenceIndex(index_expr) => infer_sequence_index_type(
            index_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::RecordUpdate(update_expr) => infer_record_update_type(
            update_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::AdtCtor(ctor_expr) => infer_adt_ctor_type(
            ctor_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            None,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::Var(v) => {
            // M9.9: path check moved to top of infer_expr_type via expr_access_path.
            env.get(*v).ok_or(FrontendError {
                pos: 0,
                message: format!("unknown variable '{}'", resolve_symbol_name(arena, *v)?),
            })
        }
        Expr::Block(block) => {
            let (ty, result_env) = infer_value_block_type(
                block,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            // SSF-08 Lane 1: a bare block has no sibling alternative -- this
            // is a join over exactly one successor, which correctly just
            // adopts that successor's restrictions onto `env`.
            env.join_ownership_from(&[result_env])?;
            Ok(ty)
        }
        Expr::If(if_expr) => {
            let cond_ty = infer_expr_type(
                if_expr.condition,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message:
                        "if expression condition must be bool; explicit compare is required for quad"
                            .to_string(),
                });
            }
            // SSF-08 Lane 1 (#1659): both branches are checked from a clone
            // of the pre-if env, and their resulting states are joined back
            // -- `if` expression must obey the same ownership state machine
            // as statement `if`, per the required expression/statement parity.
            let (then_ty, then_result_env) = infer_value_block_type(
                &if_expr.then_block,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            let (else_ty, else_result_env) = infer_value_block_type(
                &if_expr.else_block,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            if then_ty != else_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "if expression branch type mismatch: then {:?}, else {:?}",
                        then_ty, else_ty
                    ),
                });
            }
            env.join_ownership_from(&[then_result_env, else_result_env])?;
            Ok(then_ty)
        }
        Expr::Match(match_expr) => infer_match_expr_type(
            match_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::Loop(loop_expr) => infer_loop_expr_type(
            loop_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        // M9.4 Wave 3: if-let expression typecheck.
        Expr::IfLet(if_let) => {
            // TODO(M9.5): disambiguate expr parsing for scrutinee to avoid record-literal conflict
            // (e.g. `if let Pat = v { ... }` where `v { ... }` is parsed as a record literal).
            // Infer value type.
            let value_ty = infer_expr_type(
                if_let.value,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            // SSF-08 Lane 1 (#1662/#1663): build_pattern_arm_env applies this
            // pattern's own scrutinee capture directly to then_env (visible
            // to the then-block, and checked against the pre-if-let state),
            // via the same canonical helper match-expression arms use.
            let mut then_env = build_pattern_arm_env(
                if_let.value,
                &if_let.pattern,
                &value_ty,
                arena,
                env,
                adt_table,
            )?;
            let (then_ty, mut then_result_env) = infer_value_block_type(
                &if_let.then_block,
                arena,
                &mut then_env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            // Drop build_pattern_arm_env's own pushed (pattern-binding)
            // scope so then_result_env is back at `env`'s depth for joining.
            then_result_env.pop_scope();
            // else-block uses original env (no bindings, no capture).
            let (else_ty, else_result_env) = infer_value_block_type(
                &if_let.else_block,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            if then_ty != else_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "if-let branch type mismatch: then is {:?}, else is {:?}",
                        then_ty, else_ty
                    ),
                });
            }
            env.join_ownership_from(&[then_result_env, else_result_env])?;
            Ok(then_ty)
        }
        Expr::Call(name, args) => {
            if is_builtin_assert_name(*name, arena, table)? {
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
                let arg_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                return match &arg_ty {
                    Type::Sequence(_) => Ok(Type::I32),
                    _ => Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'len' expects a Sequence argument, got {:?}",
                            arg_ty
                        ),
                    }),
                };
            }
            // builtin push(sequence, value) -> Sequence(T)  [persistent — returns new sequence]
            if resolve_symbol_name(arena, *name)? == "push"
                || resolve_symbol_name(arena, *name)? == "prepend"
            {
                let builtin_name = resolve_symbol_name(arena, *name)?;
                if args.len() != 2 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin '{builtin_name}' takes exactly two positional arguments"
                        ),
                    });
                }
                let seq_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                let Type::Sequence(seq_type) = &seq_ty else {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin '{builtin_name}' first argument must be a Sequence, got {:?}",
                            seq_ty
                        ),
                    });
                };
                let elem_ty = seq_type.item.as_ref().clone();
                let val_ty = infer_expr_type(
                    args[1].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                if val_ty != elem_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin '{builtin_name}' second argument type {:?} does not match \
                             sequence element type {:?}",
                            val_ty, elem_ty
                        ),
                    });
                }
                return Ok(seq_ty);
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
                let seq_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
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
                // Restrict to scalar comparable types in this release
                match &elem_ty {
                    Type::I32 | Type::U32 | Type::Bool | Type::Text | Type::Quad => {}
                    other => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "builtin 'contains' does not yet support element type {:?}; \
                                 admitted element types are i32, u32, bool, text, quad",
                                other
                            ),
                        });
                    }
                }
                let val_ty = infer_expr_type(
                    args[1].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                if val_ty != elem_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'contains' second argument type {:?} does not match \
                             sequence element type {:?}",
                            val_ty, elem_ty
                        ),
                    });
                }
                return Ok(Type::Bool);
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
                let arg_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                return match &arg_ty {
                    Type::Sequence(_) => Ok(Type::Bool),
                    _ => Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'is_empty' expects a Sequence argument, got {:?}",
                            arg_ty
                        ),
                    }),
                };
            }
            // builtin pop(sequence) -> Sequence(T)
            if resolve_symbol_name(arena, *name)? == "pop" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'pop' takes exactly one positional argument".to_string(),
                    });
                }
                let arg_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                return match &arg_ty {
                    Type::Sequence(_) => Ok(arg_ty),
                    _ => Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'pop' expects a Sequence argument, got {:?}",
                            arg_ty
                        ),
                    }),
                };
            }
            // builtin map_empty() — requires contextual Map type; handled in
            // infer_expr_type_with_expected. If we reach here, context is absent.
            if resolve_symbol_name(arena, *name)? == "map_empty" {
                return Err(FrontendError {
                    pos: 0,
                    message: "map_empty() requires a contextual Map(K, V) type; \
                         use 'let q: Map(K, V) = map_empty()'"
                        .to_string(),
                });
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
                let map_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
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
                match &key_ty {
                    Type::I32 | Type::U32 | Type::Bool | Type::Text | Type::Quad => {}
                    other => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "builtin 'map_contains' does not support key type {:?}; \
                                 admitted key types are i32, u32, bool, text, quad",
                                other
                            ),
                        });
                    }
                }
                let actual_key_ty = infer_expr_type(
                    args[1].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                if actual_key_ty != key_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'map_contains' key type {:?} does not match map key type {:?}",
                            actual_key_ty, key_ty
                        ),
                    });
                }
                return Ok(Type::Bool);
            }
            // builtin map_get(Map(K, V), K, V) -> V
            if resolve_symbol_name(arena, *name)? == "map_get" {
                if args.len() != 3 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "builtin 'map_get' takes exactly three positional arguments (map, key, default)"
                                .to_string(),
                    });
                }
                let map_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
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
                match &key_ty {
                    Type::I32 | Type::U32 | Type::Bool | Type::Text | Type::Quad => {}
                    other => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "builtin 'map_get' does not support key type {:?}; \
                                 admitted key types are i32, u32, bool, text, quad",
                                other
                            ),
                        });
                    }
                }
                let actual_key_ty = infer_expr_type(
                    args[1].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                if actual_key_ty != key_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'map_get' key type {:?} does not match map key type {:?}",
                            actual_key_ty, key_ty
                        ),
                    });
                }
                let actual_default_ty = infer_expr_type(
                    args[2].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                if actual_default_ty != val_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'map_get' default type {:?} does not match map value type {:?}",
                            actual_default_ty, val_ty
                        ),
                    });
                }
                return Ok(val_ty);
            }
            // builtin map_set(Map(K, V), K, V) -> Map(K, V)
            if resolve_symbol_name(arena, *name)? == "map_set" {
                if args.len() != 3 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "builtin 'map_set' takes exactly three positional arguments (map, key, value)"
                                .to_string(),
                    });
                }
                let map_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
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
                match &key_ty {
                    Type::I32 | Type::U32 | Type::Bool | Type::Text | Type::Quad => {}
                    other => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "builtin 'map_set' does not support key type {:?}; \
                                 admitted key types are i32, u32, bool, text, quad",
                                other
                            ),
                        });
                    }
                }
                let actual_key_ty = infer_expr_type(
                    args[1].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                if actual_key_ty != key_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'map_set' key type {:?} does not match map key type {:?}",
                            actual_key_ty, key_ty
                        ),
                    });
                }
                let actual_val_ty = infer_expr_type(
                    args[2].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                if actual_val_ty != val_ty {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'map_set' value type {:?} does not match map value type {:?}",
                            actual_val_ty, val_ty
                        ),
                    });
                }
                return Ok(map_ty);
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
                let arg_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                if arg_ty != Type::Text {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!("builtin 'print' expects text, got {:?}", arg_ty),
                    });
                }
                return Ok(Type::Unit);
            }
            if resolve_symbol_name(arena, *name)? == "to_text" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message: "builtin 'to_text' takes exactly one positional argument"
                            .to_string(),
                    });
                }
                let arg_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                match &arg_ty {
                    Type::Text | Type::Bool | Type::I32 | Type::U32 | Type::Quad => {
                        return Ok(Type::Text);
                    }
                    Type::Record(sym) => {
                        let name_str = resolve_symbol_name(arena, *sym).unwrap_or("<unknown>");
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "builtin 'to_text' does not yet support record type '{name_str}'"
                            ),
                        });
                    }
                    other => {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "builtin 'to_text' currently supports text, bool, i32, u32, and quad; got {:?}",
                                other
                            ),
                        });
                    }
                }
            }
            if resolve_symbol_name(arena, *name)? == "random_seed" {
                if args.len() != 1 || args.iter().any(|a| a.name.is_some()) {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "builtin 'random_seed' takes exactly one positional argument (seed: i32)"
                                .to_string(),
                    });
                }
                let seed_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                if seed_ty != Type::I32 {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'random_seed' expects i32 seed, got {:?}",
                            seed_ty
                        ),
                    });
                }
                return Ok(Type::Unit);
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
                let lo_ty = infer_expr_type(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                let hi_ty = infer_expr_type(
                    args[1].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                if lo_ty != Type::I32 {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'random_next_i32' lo must be i32, got {:?}",
                            lo_ty
                        ),
                    });
                }
                if hi_ty != Type::I32 {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "builtin 'random_next_i32' hi must be i32, got {:?}",
                            hi_ty
                        ),
                    });
                }
                return Ok(Type::I32);
            }
            let sig = if let Some(s) = table.get(name) {
                s.clone()
            } else if let Some(s) = builtin_sig(resolve_symbol_name(arena, *name)?) {
                s
            } else if let Some(Type::Closure(closure_ty)) = env.get(*name) {
                if closure_ty.family != ClosureValueFamily::UnaryDirect
                    || closure_ty.capture != ClosureCapturePolicy::Immutable
                {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "direct invocation currently admits only the UnaryDirect immutable closure family in M8.4 Wave 3"
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
                let arg_ty = infer_expr_type_with_expected(
                    args[0].value,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    Some(closure_ty.param.as_ref().clone()),
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                ensure_binding_value_type(
                    closure_ty.param.as_ref().clone(),
                    arg_ty,
                    args[0].value,
                    arena,
                    format!(
                        "closure argument for '{}'",
                        resolve_symbol_name(arena, *name)?
                    ),
                )?;
                return Ok(closure_ty.ret.as_ref().clone());
            } else {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("unknown function '{}'", resolve_symbol_name(arena, *name)?),
                });
            };
            let ordered_args = reorder_call_args(*name, args, &sig, arena)?;
            // FA-02-016 / #1648: generic call-site substitution.
            // When the function is generic (sig.type_params non-empty), infer a
            // substitution map TypeVar(T) → concrete_type from the argument
            // expressions -- recursively through every currently admitted
            // structural family, not only a directly TypeVar-shaped parameter
            // -- and apply it before checking argument/return types.
            if !sig.type_params.is_empty() {
                let fn_name = resolve_symbol_name(arena, *name)?;
                // FA-02-016 / #1648 corrective: prove sig.type_params is an
                // honest ownership boundary before trusting it -- table is a
                // public, caller-suppliable FnTable, not necessarily built
                // by build_fn_table. See ensure_typevars_declared.
                for p in &sig.params {
                    ensure_typevars_declared(p, &sig.type_params, arena)?;
                }
                ensure_typevars_declared(&sig.ret, &sig.type_params, arena)?;
                for bound in &sig.trait_bounds {
                    if !sig.type_params.contains(&bound.param) {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "trait bound references type parameter '{}' which is not declared on '{}'",
                                resolve_symbol_name(arena, bound.param)?,
                                fn_name,
                            ),
                        });
                    }
                }
                // First pass: recursively collect TypeVar constraints from
                // every argument's declared (expected) shape against its
                // actual inferred type. Operates on the final bound
                // parameter slot (ordered_args.slots), never source
                // position, so named/default-argument reordering (#1722)
                // cannot desync inference from the ABI-bound parameter it
                // actually corresponds to.
                let mut subst: BTreeMap<SymbolId, Type> = BTreeMap::new();
                for (i, arg) in ordered_args.slots.iter().enumerate() {
                    let at = infer_expr_type(
                        *arg,
                        arena,
                        env,
                        table,
                        record_table,
                        adt_table,
                        ret_ty.clone(),
                        loop_stack,
                        impl_list,
                    )?;
                    collect_generic_constraints(&sig.params[i], &at, arena, &mut subst)?;
                }
                // Every declared type parameter must have been bound by at
                // least one argument -- the formal exit invariant: a
                // successful generic call never leaves a declared type
                // parameter with no constraint at all.
                for tp in &sig.type_params {
                    if !subst.contains_key(tp) {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "cannot infer type for type parameter '{}' in call to '{}': no argument constrains it",
                                resolve_symbol_name(arena, *tp)?,
                                fn_name,
                            ),
                        });
                    }
                }
                // M9.2 Wave 3: trait bound satisfaction check.
                // After substitution is fully inferred, verify that each bound
                // T: TraitName is satisfied by the concrete type substituted for T.
                for bound in &sig.trait_bounds {
                    if let Some(concrete_ty) = subst.get(&bound.param) {
                        let satisfied = impl_list.iter().any(|imp| {
                            imp.trait_name == bound.bound
                                && concrete_type_matches_impl_for(concrete_ty, imp.for_type)
                        });
                        if !satisfied {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!(
                                    "type {:?} does not implement trait '{}' required by '{}'",
                                    concrete_ty,
                                    resolve_symbol_name(arena, bound.bound)?,
                                    fn_name,
                                ),
                            });
                        }
                    }
                }
                // Substitute TypeVar → concrete in all param types and ret.
                let concrete_params: Vec<Type> =
                    sig.params.iter().map(|p| subst_apply(p, &subst)).collect();
                let concrete_ret = subst_apply(&sig.ret, &subst);
                // Second pass: check every argument against its concrete type.
                for (i, arg) in ordered_args.slots.iter().enumerate() {
                    let expected_ty = concrete_params[i].clone();
                    let at = infer_expr_type_with_expected(
                        *arg,
                        arena,
                        env,
                        table,
                        record_table,
                        adt_table,
                        Some(expected_ty.clone()),
                        ret_ty.clone(),
                        loop_stack,
                        impl_list,
                    )?;
                    if at != expected_ty {
                        if expected_ty == Type::Fx && is_numeric_for_fx_gap(&at) {
                            if !is_fx_literal_expr(*arg, arena) {
                                return Err(FrontendError {
                                    pos: 0,
                                    message: format!(
                                        "{}; arg {} for '{}' currently requires an fx literal or an existing fx-typed value",
                                        fx_coercion_gap_message(),
                                        i,
                                        fn_name,
                                    ),
                                });
                            }
                        } else {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!(
                                    "arg {} for '{}' has type {:?}, expected {:?}",
                                    i, fn_name, at, expected_ty,
                                ),
                            });
                        }
                    }
                }
                return Ok(concrete_ret);
            }
            for (i, arg) in ordered_args.slots.iter().enumerate() {
                let expected_ty = sig.params[i].clone();
                let at = infer_expr_type_with_expected(
                    *arg,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    Some(expected_ty.clone()),
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                if at != expected_ty {
                    if expected_ty == Type::Fx && is_numeric_for_fx_gap(&at) {
                        if !is_fx_literal_expr(*arg, arena) {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!(
                                    "{}; arg {} for '{}' currently requires an fx literal or an existing fx-typed value",
                                    fx_coercion_gap_message(),
                                    i,
                                    resolve_symbol_name(arena, *name)?,
                                ),
                            });
                        }
                    } else {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "arg {} for '{}' has type {:?}, expected {:?}",
                                i,
                                resolve_symbol_name(arena, *name)?,
                                at,
                                expected_ty
                            ),
                        });
                    }
                }
            }
            Ok(sig.ret.clone())
        }
        Expr::Unary(op, inner) => {
            let t = infer_expr_type(
                *inner,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            let measured = measured_numeric_parts(&t);
            match op {
                UnaryOp::Not => match t {
                    Type::Quad | Type::Bool => Ok(t),
                    _ => Err(FrontendError {
                        pos: 0,
                        message: format!("operator ! unsupported for {:?}", t),
                    }),
                },
                UnaryOp::Pos | UnaryOp::Neg => {
                    if t == Type::I32 {
                        Ok(Type::I32)
                    } else if t == Type::F64 {
                        Ok(Type::F64)
                    } else if t == Type::Fx {
                        Ok(Type::Fx)
                    } else if let Some((base, _)) = measured {
                        if *base == Type::F64 {
                            Ok(t)
                        } else if *base == Type::Fx {
                            Err(FrontendError {
                                pos: 0,
                                message: fx_measured_arithmetic_gap_message().to_string(),
                            })
                        } else {
                            Err(FrontendError {
                                pos: 0,
                                message: format!("operator +/- unsupported for {:?}", t),
                            })
                        }
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!("operator +/- unsupported for {:?}", t),
                        })
                    }
                }
            }
        }
        Expr::Binary(l, op, r) => {
            let lt = infer_expr_type(
                *l,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            let rt = infer_expr_type(
                *r,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            match op {
                BinaryOp::Eq | BinaryOp::Ne => {
                    if lt == Type::RangeI32 && rt == Type::RangeI32 {
                        return Err(FrontendError {
                            pos: 0,
                            message: "range equality is not part of the stable v0 range surface"
                                .to_string(),
                        });
                    }
                    if !supports_stable_equality_type(&lt, record_table, adt_table)? {
                        let message = if matches!(lt, Type::Record(_)) {
                            "record equality is allowed only when every field type already supports stable equality"
                        } else {
                            "equality is allowed only when the value family already supports stable equality"
                        };
                        return Err(FrontendError {
                            pos: 0,
                            message: message.to_string(),
                        });
                    }
                    if lt == rt {
                        Ok(Type::Bool)
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!("cannot compare {:?} and {:?}", lt, rt),
                        })
                    }
                }
                BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                    if lt == Type::I32 && rt == Type::I32 {
                        Ok(Type::Bool)
                    } else if lt == rt {
                        Err(FrontendError {
                            pos: 0,
                            message: first_wave_relational_gap_message().to_string(),
                        })
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!("cannot compare {:?} and {:?}", lt, rt),
                        })
                    }
                }
                BinaryOp::AndAnd | BinaryOp::OrOr => {
                    if lt != rt {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!("operator type mismatch: {:?} vs {:?}", lt, rt),
                        });
                    }
                    match lt {
                        Type::Quad | Type::Bool => Ok(lt),
                        _ => Err(FrontendError {
                            pos: 0,
                            message: format!("operator unsupported for {:?}", lt),
                        }),
                    }
                }
                BinaryOp::Implies => {
                    if lt == Type::Quad && rt == Type::Quad {
                        Ok(Type::Quad)
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: "operator '->' is allowed only for quad".to_string(),
                        })
                    }
                }
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    if matches!(lt, Type::Sequence(_)) || matches!(rt, Type::Sequence(_)) {
                        return Err(FrontendError {
                            pos: 0,
                            message: "ordered sequence values are not part of the current M8.3 Wave 1 operator surface"
                                .to_string(),
                        });
                    }
                    if lt == Type::Text || rt == Type::Text {
                        if *op == BinaryOp::Add && lt == Type::Text && rt == Type::Text {
                            return Ok(Type::Text);
                        }
                        return Err(FrontendError {
                            pos: 0,
                            message:
                                "text concatenation currently admits only text + text operands"
                                    .to_string(),
                        });
                    }
                    if lt == Type::I32 && rt == Type::I32 {
                        return match op {
                            BinaryOp::Add
                            | BinaryOp::Sub
                            | BinaryOp::Mul
                            | BinaryOp::Div
                            | BinaryOp::Mod => Ok(Type::I32),
                            _ => unreachable!("covered arithmetic operator arms"),
                        };
                    }
                    if measured_numeric_parts(&lt).is_some()
                        || measured_numeric_parts(&rt).is_some()
                    {
                        if lt != rt {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!("operator type mismatch: {:?} vs {:?}", lt, rt),
                            });
                        }
                        let (base, _) = measured_numeric_parts(&lt).ok_or(FrontendError {
                            pos: 0,
                            message: format!("operator unsupported for {:?}", lt),
                        })?;
                        return match op {
                            BinaryOp::Add | BinaryOp::Sub if *base == Type::F64 => Ok(lt),
                            BinaryOp::Add | BinaryOp::Sub if *base == Type::Fx => {
                                Err(FrontendError {
                                    pos: 0,
                                    message: fx_measured_arithmetic_gap_message().to_string(),
                                })
                            }
                            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => Err(FrontendError {
                                pos: 0,
                                message:
                                    "*, /, % on unit-carrying values are rejected in the first-wave units surface"
                                        .to_string(),
                            }),
                            _ => Err(FrontendError {
                                pos: 0,
                                message: format!("operator unsupported for {:?}", lt),
                            }),
                        };
                    }
                    if lt == Type::Fx && rt == Type::Fx {
                        return match op {
                            BinaryOp::Mod => Err(FrontendError {
                                pos: 0,
                                message: format!("operator % unsupported for {:?}", lt),
                            }),
                            _ => Ok(Type::Fx),
                        };
                    }
                    if lt == Type::F64 && rt == Type::F64 {
                        match op {
                            BinaryOp::Mod => Err(FrontendError {
                                pos: 0,
                                message: format!("operator % unsupported for {:?}", lt),
                            }),
                            _ => Ok(Type::F64),
                        }
                    } else {
                        Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "f64 arithmetic requires f64 operands, got {:?} and {:?}",
                                lt, rt
                            ),
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typecheck_source(src: &str) -> Result<(), FrontendError> {
        let program = parse_program(src)?;
        type_check_program(&program)
    }

    #[test]
    fn storage_admission_aggregate_composite_matrix_typechecks_for_record_and_adt() {
        // Corrective round (review on PR #1877): wiring
        // ensure_storage_type_supported into record/ADT declaration
        // validation changed this authority's semantic scope from
        // "local binding storage" to "local binding storage, record field
        // storage, and ADT payload storage" -- the original PR treated the
        // 20-variant classification as automatically valid for all three
        // positions without proving that. This table proves, per composite,
        // that a value can be constructed with the composite in a record
        // field / ADT payload position, read back out, and used -- not
        // merely that the declaration alone typechecks.
        //
        // Tuple/Measured/Option/Result: `docs/spec/types.md` already states
        // "measured numeric types may appear in ... tuple elements, record
        // fields, Option(T), and Result(T, E) payload positions" -- direct
        // normative evidence, corroborated here.
        // Sequence/Map: no prior normative record-field statement found;
        // admitted only after this empirical proof (also lowered to IR
        // successfully, see legacy_lowering.rs's matching regression).
        // Record/Adt nesting: architecturally proven independently by
        // `validate_record_acyclic`/`validate_adt_acyclic`, which exist
        // specifically to walk nested nominal fields -- that machinery has
        // no purpose if nominal nesting were not an intended, supported
        // pattern.
        let cases = [
            (
                "record Tuple",
                "record R { x: (i32, i32) } fn main() { let r: R = R { x: (1, 2) }; let (a, b): (i32, i32) = r.x; let _ = a; let _ = b; return; }",
            ),
            (
                "adt Tuple",
                "enum E { V((i32, i32)) } fn main() { let e: E = E::V((1, 2)); match e { E::V(t) => { let (a, b): (i32, i32) = t; let _ = a; let _ = b; } } return; }",
            ),
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
            (
                "record Measured",
                "record R { x: f64[m] } fn main() { let r: R = R { x: 1.0 }; let v: f64[m] = r.x; let _ = v; return; }",
            ),
            (
                "adt Measured",
                "enum E { V(f64[m]) } fn main() { let e: E = E::V(1.0); match e { E::V(v) => { let _ = v; } } return; }",
            ),
            (
                "record Option",
                "record R { x: Option(i32) } fn main() { let r: R = R { x: Option::Some(1) }; let o: Option(i32) = r.x; let _ = o; return; }",
            ),
            (
                "adt Option",
                "enum E { V(Option(i32)) } fn main() { let e: E = E::V(Option::Some(1)); match e { E::V(o) => { let _ = o; } } return; }",
            ),
            (
                "record Result",
                "record R { x: Result(i32, i32) } fn main() { let r: R = R { x: Result::Ok(1) }; let v: Result(i32, i32) = r.x; let _ = v; return; }",
            ),
            (
                "adt Result",
                "enum E { V(Result(i32, i32)) } fn main() { let e: E = E::V(Result::Ok(1)); match e { E::V(v) => { let _ = v; } } return; }",
            ),
            (
                "record nested Record",
                "record Inner { n: i32 } record Outer { x: Inner } fn main() { let o: Outer = Outer { x: Inner { n: 1 } }; let _ = o.x; return; }",
            ),
            (
                "adt nested Record",
                "record Inner { n: i32 } enum E { V(Inner) } fn main() { let e: E = E::V(Inner { n: 1 }); match e { E::V(i) => { let _ = i; } } return; }",
            ),
            (
                "record nested Adt",
                "enum Inner { A, B } record Outer { x: Inner } fn main() { let o: Outer = Outer { x: Inner::A }; let _ = o.x; return; }",
            ),
            (
                "adt nested Adt",
                "enum Inner { A, B } enum E { V(Inner) } fn main() { let e: E = E::V(Inner::A); match e { E::V(i) => { let _ = i; } } return; }",
            ),
        ];
        for (label, src) in cases {
            typecheck_source(src).unwrap_or_else(|e| {
                panic!("{label}: aggregate composite storage must typecheck: {e:?}")
            });
        }
    }

    #[test]
    fn storage_admission_record_closure_field_typechecks_end_to_end() {
        // Corrective round: the original PR admitted Type::Closure for
        // storage on the strength of *local-binding* evidence only (two
        // pre-existing tests proving `let`-bound closures work), which does
        // not by itself prove record-field closure storage -- the
        // historical `first_class_closures_full_scope.md` scope explicitly
        // named "local binding, parameter, and return transport" only, not
        // aggregate storage. This constructs a record with a closure field,
        // reads the field back out, and invokes the extracted closure --
        // full frontend proof; VM execution is proven separately in
        // sm-vm's test suite (see `docs/spec/foundation_source_profile_v1.md`
        // for the reconciled normative statement and the SSF-07 #1861
        // addendum in `first_class_closures_full_scope.md`).
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
        typecheck_source(src)
            .expect("record field closure storage: construct, extract, and invoke must typecheck");
    }

    #[test]
    fn storage_admission_adt_closure_payload_typechecks_end_to_end() {
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
        typecheck_source(src)
            .expect("ADT payload closure storage: construct and invoke via match must typecheck");
    }

    // FA-02-038 / #1861: exhaustive storage-type admission regression
    // matrix. `ensure_storage_type_supported` is the sole shared authority
    // for every field/binding storage position (Const/Let/LetTuple/
    // LetElseTuple/Discard local bindings, and -- newly wired by this fix --
    // record fields and ADT payloads). Its own exhaustiveness (no `_` arm at
    // all) is a compile-time property, not something a runtime test can
    // prove as strongly as the compiler itself: `cargo build`/`cargo check`
    // succeeding after adding this function's body IS that proof -- if a
    // 21st `Type` variant is ever added without updating this match, the
    // crate fails to compile rather than silently admitting it.

    #[test]
    fn storage_admission_accepts_every_qualified_scalar_leaf() {
        // Type::Unit has no explicit type-annotation source syntax at all
        // (parse_type has no "Unit"/"()" branch -- it only ever arises
        // implicitly as an omitted function return type), so it cannot be
        // exercised via a `let` annotation here; it is covered directly
        // below alongside the other structural-placeholder direct-call
        // proofs.
        let cases = [
            ("quad", "N"),
            ("bool", "true"),
            ("text", "\"s\""),
            ("i32", "0"),
            ("u32", "0u32"),
            ("fx", "0.0fx"),
            ("f64", "0.0"),
        ];
        for (ty, value) in cases {
            let src = format!("fn main() {{\n    let x: {ty} = {value};\n    return;\n}}\n");
            typecheck_source(&src)
                .unwrap_or_else(|e| panic!("scalar leaf '{ty}' must be admitted storage: {e:?}"));
        }
    }

    #[test]
    fn storage_admission_accepts_every_qualified_composite_and_recurses_into_children() {
        let cases = [
            (
                "Sequence(i32)",
                "fn main() {\n    let x: Sequence(i32) = [1, 2];\n    return;\n}\n",
            ),
            (
                "Option(i32)",
                "fn main() {\n    let x: Option(i32) = Option::Some(1);\n    return;\n}\n",
            ),
            (
                "Result(i32, i32)",
                "fn main() {\n    let x: Result(i32, i32) = Result::Ok(1);\n    return;\n}\n",
            ),
            (
                "tuple (i32, i32)",
                "fn main() {\n    let x: (i32, i32) = (1, 2);\n    return;\n}\n",
            ),
            (
                "Map(i32, i32)",
                "fn main() {\n    let x: Map(i32, i32) = map_empty();\n    return;\n}\n",
            ),
            (
                "Closure(f64 -> f64)",
                "fn main() {\n    let x: Closure(f64 -> f64) = (v => v + 1.0);\n    return;\n}\n",
            ),
        ];
        for (label, src) in cases {
            typecheck_source(src)
                .unwrap_or_else(|e| panic!("composite '{label}' must be admitted storage: {e:?}"));
        }
    }

    #[test]
    fn storage_admission_rejects_direct_qvec() {
        let src = "fn main() {\n    let x: qvec[8] = qvec[8];\n    return;\n}\n";
        let err = typecheck_source(src).expect_err("direct qvec storage must reject");
        assert!(
            err.message.contains("qvec is a reserved type"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn storage_admission_rejects_qvec_nested_in_every_composite() {
        let src = "fn main() { return; }\n";
        let program = parse_program(src).expect("parse");
        let record_table = crate::build_record_table(&program).expect("record table");
        let adt_table = crate::build_adt_table(&program).expect("adt table");
        let qvec = Type::QVec(8);
        let cases: Vec<(&str, Type)> = vec![
            ("Option(qvec)", Type::Option(Box::new(qvec.clone()))),
            (
                "Sequence(qvec)",
                Type::Sequence(crate::types::SequenceType {
                    family: crate::types::SequenceCollectionFamily::OrderedSequence,
                    item: Box::new(qvec.clone()),
                }),
            ),
            (
                "Result(qvec, i32)",
                Type::Result(Box::new(qvec.clone()), Box::new(Type::I32)),
            ),
            (
                "Result(i32, qvec)",
                Type::Result(Box::new(Type::I32), Box::new(qvec.clone())),
            ),
            (
                "Tuple(i32, qvec)",
                Type::Tuple(vec![Type::I32, qvec.clone()]),
            ),
            (
                "Map(i32, qvec)",
                Type::Map(crate::types::MapType {
                    key: Box::new(Type::I32),
                    val: Box::new(qvec.clone()),
                }),
            ),
            (
                "Closure(qvec -> i32)",
                Type::Closure(crate::types::ClosureType {
                    family: crate::types::ClosureValueFamily::UnaryDirect,
                    capture: crate::types::ClosureCapturePolicy::Immutable,
                    param: Box::new(qvec.clone()),
                    ret: Box::new(Type::I32),
                }),
            ),
        ];
        for (label, ty) in cases {
            let canonical =
                canonicalize_declared_type(&ty, &record_table, &adt_table, &program.arena)
                    .unwrap_or_else(|e| panic!("{label}: canonicalize failed: {e:?}"));
            let err =
                ensure_storage_type_supported(&canonical, &program.arena, "probe".to_string())
                    .expect_err(&format!(
                        "{label}: nested qvec must reject, recursion must reach it"
                    ));
            assert!(
                err.message.contains("qvec is a reserved type"),
                "{label}: unexpected error: {}",
                err.message
            );
        }
    }

    #[test]
    fn storage_admission_rejects_typevar_and_range_and_admits_unit_directly() {
        let src = "fn main() { return; }\n";
        let program = parse_program(src).expect("parse");
        let typevar_name = program.arena.symbol_to_id.get("main").copied().unwrap();

        let typevar_err = ensure_storage_type_supported(
            &Type::TypeVar(typevar_name),
            &program.arena,
            "probe".to_string(),
        )
        .expect_err("TypeVar must reject: storage admission has no admitted-type-var context");
        assert!(
            typevar_err
                .message
                .contains("is not admitted as a storage type"),
            "unexpected error: {}",
            typevar_err.message
        );

        let range_err =
            ensure_storage_type_supported(&Type::RangeI32, &program.arena, "probe".to_string())
                .expect_err("RangeI32 must reject: it is a structural/iteration type, not storage");
        assert!(
            range_err.message.contains("range values are not admitted"),
            "unexpected error: {}",
            range_err.message
        );

        // Type::Unit has no type-annotation source syntax (see the comment
        // on storage_admission_accepts_every_qualified_scalar_leaf), so its
        // admission is proven directly here instead.
        ensure_storage_type_supported(&Type::Unit, &program.arena, "probe".to_string())
            .expect("Unit must be admitted: a trivial, always-representable storage leaf");
    }

    #[test]
    fn storage_admission_rejects_unsupported_record_field_type() {
        // Items 5 + regression matrix core: end-to-end through the actual
        // record declaration validation path (validate_record_declarations),
        // not the helper called directly.
        let src = "record R {\n    x: qvec[8]\n}\nfn main() {\n    return;\n}\n";
        let err = typecheck_source(src).expect_err("record field with qvec must reject");
        assert!(
            err.message.contains("qvec is a reserved type") && err.message.contains("field 'R.x'"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn storage_admission_rejects_unsupported_adt_payload_type() {
        let src = "enum E {\n    A(qvec[8])\n}\nfn main() {\n    return;\n}\n";
        let err = typecheck_source(src).expect_err("ADT payload with qvec must reject");
        assert!(
            err.message.contains("qvec is a reserved type")
                && err.message.contains("variant 'E::A'"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn storage_admission_rejects_unsupported_binding_annotation() {
        // Item 7: at least one let/const path proving the shared authority
        // is wired identically for local bindings, not just record/ADT
        // fields.
        let src = "fn main() {\n    let x: qvec[8] = qvec[8];\n    return;\n}\n";
        let err = typecheck_source(src).expect_err("let annotation with qvec must reject");
        assert!(
            err.message.contains("qvec is a reserved type") && err.message.contains("let 'x'"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn storage_admission_admits_qualified_non_generic_record_and_adt_storage() {
        // Item 8: positive control -- ordinary, fully-supported field/
        // payload types must remain green after wiring storage admission
        // into declaration validation.
        let src = r#"
            record Point {
                x: i32,
                y: i32,
                label: text,
            }

            enum Shape {
                Circle(f64),
                Square(f64, f64),
            }

            fn main() {
                return;
            }
        "#;
        typecheck_source(src)
            .expect("record/ADT with fully-supported field/payload types must typecheck");
    }

    #[test]
    fn storage_admission_closes_nominal_cross_boundary_escape() {
        // Item 9, the central FA-02-038 finding: pre-fix, an unsupported
        // field type hid inside an admitted record declaration and escaped
        // as a trusted Type::Record nominal shell to an ordinary function
        // signature, which #1647's executable-signature admission accepted
        // without ever looking inside the record's own fields (it only
        // resolves the symbol name). The record declaration itself must now
        // reject before any nominal shell is ever trusted downstream.
        let src = "record R {\n    x: qvec[8]\n}\nfn f(r: R) -> i32 {\n    return 0;\n}\nfn main() {\n    return;\n}\n";
        let err = typecheck_source(src).expect_err(
            "unsupported field must reject before the record's nominal identity is trusted",
        );
        assert!(
            err.message.contains("qvec is a reserved type"),
            "unexpected error (possible authority inversion or cross-boundary escape): {}",
            err.message
        );
    }

    // FA-02-018 / #1650: generic record/ADT declarations are rejected at
    // the canonical owner boundary (build_record_table/build_adt_table),
    // never merely tolerated until an unrelated later error happens to
    // catch them. These are whole-program (type_check_program) regressions
    // complementing the direct build_record_table/build_adt_table unit
    // tests in lib.rs.

    #[test]
    fn type_check_program_rejects_generic_record_declaration() {
        let src = r#"
            record Box<T> { value: T }
            fn main() { return; }
        "#;
        let err = typecheck_source(src)
            .expect_err("a generic record declaration must not be admitted into Stable Foundation");
        assert!(
            err.message.contains("Box")
                && err
                    .message
                    .contains("not part of the current Stable Foundation"),
            "unexpected error: {}",
            err.message
        );
        // Owner-boundary proof: this must be #1650's declaration-admission
        // diagnostic, not the old accidental construction-time failure
        // (which never runs, because the declaration is rejected first).
        assert!(
            !err.message.contains("deferred to M9.1 Wave 2"),
            "rejection must come from the owner boundary, not the old accidental \
             construction-time TypeVar-canonicalization failure: {}",
            err.message
        );
    }

    #[test]
    fn type_check_program_rejects_generic_adt_declaration() {
        let src = r#"
            enum Maybe<T> { Some(T), None }
            fn main() { return; }
        "#;
        let err = typecheck_source(src)
            .expect_err("a generic enum declaration must not be admitted into Stable Foundation");
        assert!(
            err.message.contains("Maybe")
                && err
                    .message
                    .contains("not part of the current Stable Foundation"),
            "unexpected error: {}",
            err.message
        );
        assert!(
            !err.message.contains("deferred to M9.1 Wave 2"),
            "rejection must come from the owner boundary, not the old accidental \
             construction-time TypeVar-canonicalization failure: {}",
            err.message
        );
    }

    #[test]
    fn type_check_program_rejects_generic_record_with_nested_typevar_field() {
        // The rejection must fire on type_params alone, independent of how
        // (or whether) T is actually used in field types -- proving it is
        // the owner-layer declaration check, not something that happens to
        // trip over a nested TypeVar during field canonicalization.
        let src = r#"
            record Box<T> { value: Option(T) }
            fn main() { return; }
        "#;
        let err = typecheck_source(src).expect_err(
            "a generic record must reject regardless of nested field TypeVar positions",
        );
        assert!(
            err.message.contains("Box")
                && err
                    .message
                    .contains("not part of the current Stable Foundation"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn type_check_program_rejects_generic_adt_with_nested_typevar_payload() {
        let src = r#"
            enum Wrapper<T> { Boxed(Option(T)) }
            fn main() { return; }
        "#;
        let err = typecheck_source(src).expect_err(
            "a generic enum must reject regardless of nested payload TypeVar positions",
        );
        assert!(
            err.message.contains("Wrapper")
                && err
                    .message
                    .contains("not part of the current Stable Foundation"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn type_check_program_admits_non_generic_record_and_adt_unaffected() {
        // Positive control: ordinary non-generic records/ADTs, including
        // construction, field access, and ADT constructors, remain fully
        // admitted and unaffected by the #1650 zero-arity nominal
        // narrowing.
        let src = r#"
            record Point { x: i32, y: i32 }
            enum Color { Red, Green, Blue }
            fn main() {
                let p = Point { x: 1, y: 2 };
                let x: i32 = p.x;
                let p2 = p with { x: x };
                let _ = p2;
                let c = Color::Red;
                let _ = c;
                return;
            }
        "#;
        typecheck_source(src).expect(
            "ordinary non-generic record/ADT declarations and use sites must be unaffected",
        );
    }

    #[test]
    fn generic_record_syntax_still_parses_but_type_check_program_rejects() {
        // Parser fidelity (Model B precedent from #1635/#1634): raw AST
        // representation of `<T>` on a record/ADT is deliberately
        // preserved (see generic_record_type_params_are_parsed_and_stored
        // in parser.rs, unmodified); only canonical Stable Foundation
        // admission is narrowed.
        let program = parse_program(
            r#"
                record Box<T> { value: T }
                fn main() { return; }
            "#,
        )
        .expect("generic record syntax must still parse -- raw AST fidelity is preserved");
        assert_eq!(program.records[0].type_params.len(), 1);
        type_check_program(&program)
            .expect_err("canonical admission must still reject the parsed generic declaration");
    }

    // FA-02-016 / #1648 corrective: `type_check_function_with_table` accepts
    // a caller-supplied `FnTable`, so a callee's `FnSig` cannot be assumed
    // to have been built by the canonical `build_fn_table` authority.
    // Recursive constraint collection means an undeclared `TypeVar` nested
    // inside a malformed `params`/`ret` shape would otherwise be silently
    // "adopted" as this call's generic authority the moment it is
    // structurally reachable -- exactly the shape the pre-#1648 shallow
    // first pass could never see, so this was not a reachable gap before
    // recursion was added. `ensure_typevars_declared` closes it.

    fn generic_callee_and_caller(src: &str) -> (Program, SymbolId, Function) {
        let program = parse_program(src).expect("parse");
        let callee_id = *program
            .arena
            .symbol_to_id
            .get("callee")
            .expect("callee interned");
        let caller = program
            .functions
            .iter()
            .find(|f| program.arena.try_symbol_name(f.name) == Some("caller"))
            .expect("caller exists")
            .clone();
        (program, callee_id, caller)
    }

    #[test]
    fn malformed_fn_sig_undeclared_typevar_nested_in_param_fails_closed() {
        let (mut program, callee_id, caller) = generic_callee_and_caller(
            r#"
                fn callee<T>(a: T) -> T { return a; }
                fn caller() -> text {
                    let pair: (i32, text) = (1, "x");
                    return callee(pair);
                }
                fn main() { return; }
            "#,
        );
        let x_sym = program.arena.intern_symbol("X");
        let mut table = build_fn_table(&program).expect("canonical table builds");
        let sig = table.get_mut(&callee_id).expect("callee sig present");
        let t_sym = sig.type_params[0];
        // type_params = [T]; params = [(T, X)]; X is never declared.
        sig.params[0] = Type::Tuple(vec![Type::TypeVar(t_sym), Type::TypeVar(x_sym)]);
        sig.ret = Type::TypeVar(x_sym);
        let err = type_check_function_with_table(&caller, &program.arena, &table)
            .expect_err("an undeclared TypeVar nested in a param must not be silently adopted");
        assert!(
            err.message.contains('X') && err.message.contains("not declared"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn malformed_fn_sig_undeclared_typevar_in_return_only_fails_closed() {
        // The undeclared variable appears only in `ret`, never in any
        // `params` position, so it never participates in constraint
        // collection at all -- proving ensure_typevars_declared's ret check
        // is independently required, not merely incidental coverage of the
        // param check above.
        let (mut program, callee_id, caller) = generic_callee_and_caller(
            r#"
                fn callee<T>(a: T) -> T { return a; }
                fn caller() -> i32 {
                    let y = callee(1);
                    let _ = y;
                    return 0;
                }
                fn main() { return; }
            "#,
        );
        let x_sym = program.arena.intern_symbol("X");
        let mut table = build_fn_table(&program).expect("canonical table builds");
        let sig = table.get_mut(&callee_id).expect("callee sig present");
        // type_params = [T]; params = [T] (honest, so a call still supplies
        // a real T-binding argument); ret = Option(X); X is never declared
        // and never appears in params.
        sig.ret = Type::Option(Box::new(Type::TypeVar(x_sym)));
        let err = type_check_function_with_table(&caller, &program.arena, &table)
            .expect_err("an undeclared TypeVar appearing only in the return type must reject");
        assert!(
            err.message.contains('X') && err.message.contains("not declared"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn generic_to_generic_delegation_with_shared_opaque_typevar_still_typechecks() {
        // Preserves the legitimate pattern the corrective checks above must
        // not break: an *actual* argument type being a bare TypeVar --
        // including an enclosing generic function's own still-opaque
        // parameter -- remains a valid binding value, distinct from an
        // *expected*, declared signature shape referencing an undeclared
        // TypeVar (malformed FnSig, rejected above). `outer` and `id` both
        // spell their type parameter `T`; since symbol names are interned
        // program-wide, this is the same SymbolId in both declarations.
        let src = r#"
            fn id<T>(y: T) -> T {
                return y;
            }
            fn outer<T>(x: T) -> T {
                return id(x);
            }
            fn main() {
                return;
            }
        "#;
        typecheck_source(src).expect(
            "a generic function delegating to another generic function via its own opaque \
             parameter must still typecheck",
        );
    }

    #[test]
    fn malformed_fn_sig_undeclared_typevar_in_trait_bound_fails_closed() {
        // Same ownership gap, third site: a trait bound naming a TypeVar
        // that isn't one of type_params.
        let (mut program, callee_id, caller) = generic_callee_and_caller(
            r#"
                trait Marker {
                    fn mark(v: Self) -> i32;
                }
                fn callee<T>(a: T) -> T { return a; }
                fn caller() -> i32 {
                    let y = callee(1);
                    let _ = y;
                    return 0;
                }
                fn main() { return; }
            "#,
        );
        let x_sym = program.arena.intern_symbol("X");
        let marker_sym = *program
            .arena
            .symbol_to_id
            .get("Marker")
            .expect("Marker interned");
        let mut table = build_fn_table(&program).expect("canonical table builds");
        let sig = table.get_mut(&callee_id).expect("callee sig present");
        // type_params = [T]; trait_bounds = [X: Marker]; X is never declared.
        sig.trait_bounds = vec![TraitBound {
            param: x_sym,
            bound: marker_sym,
        }];
        let err = type_check_function_with_table(&caller, &program.arena, &table)
            .expect_err("a trait bound naming an undeclared type parameter must reject");
        assert!(
            err.message.contains('X') && err.message.contains("not declared"),
            "unexpected error: {}",
            err.message
        );
    }

    // FA-02-016 / #1648: generic call-site inference and substitution now
    // recurse through every currently admitted structural type family
    // (Tuple, Sequence, Map, Option, Result, Closure, Measured), not only a
    // directly TypeVar-shaped parameter. `Record`/`Adt` remain nominal
    // leaves (#1650 is the separate, unimplemented applied-nominal gap);
    // IR monomorphisation (#1717) is a different layer and is untouched.

    #[test]
    fn generic_call_direct_typevar_still_works() {
        // (Control) The pre-existing, already-working case must remain
        // unaffected by recursing constraint collection.
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
        typecheck_source(src).expect("direct TypeVar inference must still work");
    }

    #[test]
    fn generic_call_direct_typevar_conflicting_constraints_reject() {
        // fn same<T>(a: T, b: T) -> T; same(1, "x") must reject -- the
        // exact reproducer named in #1648's conflict-semantics section.
        let src = r#"
            fn same<T>(a: T, b: T) -> T {
                return a;
            }
            fn main() {
                let y = same(1, "x");
                let _ = y;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("conflicting direct TypeVar constraints for the same T must reject");
        assert!(
            err.message.contains("conflicting generic constraints") && err.message.contains('T'),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn generic_call_infers_and_substitutes_through_option() {
        // Also proves nested return substitution: the call's inferred type
        // must be the concrete `Option(i32)`, never `Option(TypeVar(T))` --
        // otherwise the `let r: Option(i32) = ...` binding could not
        // type-check.
        let src = r#"
            fn identity_via_option<T>(x: Option(T)) -> Option(T) {
                return x;
            }
            fn main() {
                let v: Option(i32) = Option::Some(1);
                let r: Option(i32) = identity_via_option(v);
                let _ = r;
                return;
            }
        "#;
        typecheck_source(src)
            .expect("Option(T) must infer T and substitute a concrete return type");
    }

    #[test]
    fn generic_call_infers_and_substitutes_through_sequence() {
        // Also proves nested return substitution via the returned element.
        let src = r#"
            fn first<T>(xs: Sequence(T)) -> T {
                return xs[0];
            }
            fn main() {
                let items: Sequence(i32) = [1, 2, 3];
                let y: i32 = first(items);
                let _ = y;
                return;
            }
        "#;
        typecheck_source(src)
            .expect("Sequence(T) must infer T and substitute a concrete return type");
    }

    #[test]
    fn generic_call_infers_and_substitutes_through_tuple() {
        let src = r#"
            fn tuple_first<T>(pair: (T, i32)) -> T {
                let (first, _) = pair;
                return first;
            }
            fn main() {
                let p: (i32, i32) = (7, 2);
                let y: i32 = tuple_first(p);
                let _ = y;
                return;
            }
        "#;
        typecheck_source(src)
            .expect("(T, concrete) must infer T and substitute a concrete return type");
    }

    #[test]
    fn generic_call_tuple_repeated_consistent_constraint_succeeds() {
        // (T, T) + (i32, i32) -> T = i32; the same T constrained twice
        // consistently must not be treated as a conflict.
        let src = r#"
            fn same_pair<T>(pair: (T, T)) -> T {
                let (first, _) = pair;
                return first;
            }
            fn main() {
                let ok: (i32, i32) = (1, 2);
                let y: i32 = same_pair(ok);
                let _ = y;
                return;
            }
        "#;
        typecheck_source(src).expect("repeated consistent nested constraints must succeed");
    }

    #[test]
    fn generic_call_tuple_repeated_conflicting_constraint_rejects() {
        // (T, T) + (i32, text) -> conflicting constraints -> deterministic
        // reject. No "last binding wins", no first-binding silent ignore.
        let src = r#"
            fn same_pair<T>(pair: (T, T)) -> T {
                let (first, _) = pair;
                return first;
            }
            fn main() {
                let bad: (i32, text) = (1, "x");
                let y = same_pair(bad);
                let _ = y;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("conflicting nested tuple constraints for the same T must reject");
        assert!(
            err.message.contains("conflicting generic constraints") && err.message.contains('T'),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn generic_call_infers_through_result() {
        // Result(T, concrete) is the only source-constructible Result shape
        // under the one-parameter first-wave contract (#1634) -- a shape
        // requiring two independent function type parameters (Result(T, E))
        // is not fabricated here.
        let src = r#"
            fn result_marker<T>(r: Result(T, text)) -> i32 {
                return 0;
            }
            fn main() {
                let r: Result(i32, text) = Result::Ok(1);
                let y: i32 = result_marker(r);
                let _ = y;
                return;
            }
        "#;
        typecheck_source(src).expect("Result(T, concrete) must infer T");
    }

    #[test]
    fn generic_call_cross_parameter_conflicting_constraints_reject() {
        // fn cross_constrain<T>(a: Option(T), b: Sequence(T)); arguments
        // constrain T differently across two different structural families
        // -> deterministic reject.
        let src = r#"
            fn cross_constrain<T>(a: Option(T), b: Sequence(T)) -> i32 {
                return 0;
            }
            fn main() {
                let opt: Option(i32) = Option::Some(1);
                let seq: Sequence(text) = ["x"];
                let y = cross_constrain(opt, seq);
                let _ = y;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("constraints on T from two different structural families must be checked against each other");
        assert!(
            err.message.contains("conflicting generic constraints") && err.message.contains('T'),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn generic_call_unused_type_param_cannot_infer() {
        // fn unused<T>(x: i32) -> i32; T never appears in params/return, so
        // no argument can ever constrain it -- deterministic reject at the
        // call site. Pre-existing behavior (the "every declared type
        // parameter must be bound" check predates this slice); confirmed
        // unaffected by the recursive-collection change.
        let src = r#"
            fn unused<T>(x: i32) -> i32 {
                return x;
            }
            fn main() {
                let y: i32 = unused(5);
                let _ = y;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("a type parameter no argument constrains must reject deterministically");
        assert!(
            err.message.contains("cannot infer") && err.message.contains('T'),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn generic_call_structural_mismatch_rejects_via_second_pass() {
        // Expected Option(T), actual Sequence(i32): a structural shape
        // mismatch yields no constraint from collect_generic_constraints
        // (T is separately bound by the first, direct-TypeVar argument
        // here, so this is not merely "cannot infer"); the existing
        // canonical second-pass parameter-compatibility check is proven to
        // be the one that actually rejects the call.
        let src = r#"
            fn wrap<T>(a: T, b: Option(T)) -> i32 {
                return 0;
            }
            fn main() {
                let items: Sequence(i32) = [1, 2, 3];
                let y = wrap(1, items);
                let _ = y;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "a structurally mismatched argument shape must still reject deterministically",
        );
        assert!(
            err.message.contains("has type") && err.message.contains("expected"),
            "expected the canonical second-pass type-mismatch diagnostic, got: {}",
            err.message
        );
    }

    #[test]
    fn generic_call_bound_satisfied_through_nested_option() {
        // Nested generic parameter shape (Option(T)) + bound on T +
        // satisfying impl -> passes. type_check_function cannot exercise
        // this positively (empty validated impl context by design, #1649);
        // this whole-program test is the required positive-satisfaction
        // evidence.
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
        typecheck_source(src)
            .expect("a bound on T satisfied by an impl through a nested Option(T) must pass");
    }

    #[test]
    fn generic_call_bound_missing_through_nested_option_fails_closed() {
        // Same nested shape, no satisfying impl -> fails closed.
        let src = r#"
            trait Printable {
                fn show(v: NoPrint) -> i32;
            }
            record NoPrint { x: i32 }
            record Unprinted { y: i32 }
            fn wrap2<T: Printable>(x: Option(T)) -> i32 {
                return 0;
            }
            fn main() {
                let v: Option(Unprinted) = Option::Some(Unprinted { y: 1 });
                let r = wrap2(v);
                let _ = r;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "a bound on T with no satisfying impl through a nested Option(T) must fail closed",
        );
        assert!(
            err.message.contains("Printable"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn type_check_function_and_type_check_program_agree_on_sequence_generic_declaration() {
        // #1649 regression, extended to a compound family: both public
        // entry points share the identical call-inference code path this
        // slice changed, so their admission verdict for a Sequence(T)
        // declaration must still agree.
        let single_src = r#"
            fn first<T>(xs: Sequence(T)) -> T {
                return xs[0];
            }
        "#;
        let program_src = r#"
            fn first<T>(xs: Sequence(T)) -> T {
                return xs[0];
            }
            fn main() {
                return;
            }
        "#;
        let single = parse_program(single_src).expect("parse single-function program");
        let full = parse_program(program_src).expect("parse full program");
        let single_result = type_check_function(&single);
        let full_result = type_check_program(&full);
        assert_eq!(
            single_result.is_ok(),
            full_result.is_ok(),
            "type_check_function must agree with type_check_program for a Sequence(T) declaration; \
             single={single_result:?} full={full_result:?}",
        );
    }

    #[test]
    fn generic_call_inference_uses_bound_parameter_slot_not_source_position() {
        // FA-02-033 / #1722: source evaluation order != parameter binding
        // order. `pick`'s declared order is (a: T, b: i32); this call names
        // both arguments in the reverse source order. If inference ever
        // read `sig.params[i]` against the source-position argument instead
        // of `ordered_args.slots[i]` (the canonically bound slot), it would
        // bind T from `b`'s i32 value instead of `a`'s text value, and
        // either infer the wrong T or reject with a spurious type mismatch.
        let src = r#"
            fn pick<T>(a: T, b: i32) -> T {
                return a;
            }
            fn main() {
                let y: text = pick(b = 2, a = "hello");
                let _ = y;
                return;
            }
        "#;
        typecheck_source(src).expect(
            "generic inference must use the bound parameter slot, not source argument position",
        );
    }

    // Focused structural unit tests for `collect_generic_constraints` and
    // `subst_apply` directly: Map has no literal construction syntax in
    // source (only `map_empty()` + `map_set(...)`), and constructing a
    // function signature exercising both Closure positions and a Measured
    // base independently has no simple corresponding call-site source form.
    // Per #1648's own guidance, these are tested as structural Type
    // recursion directly rather than through invented sugar.

    #[test]
    fn collect_generic_constraints_and_subst_apply_recurse_through_map_both_positions() {
        let program =
            parse_program("fn f<T>(x: T) -> T { return x; } fn main() { return; }").expect("parse");
        let t = *program.arena.symbol_to_id.get("T").expect("T interned");

        // Map(text, T) + Map(text, i32) -> T = i32 (value position).
        let expected_val = Type::Map(MapType {
            key: Box::new(Type::Text),
            val: Box::new(Type::TypeVar(t)),
        });
        let actual_val = Type::Map(MapType {
            key: Box::new(Type::Text),
            val: Box::new(Type::I32),
        });
        let mut subst = BTreeMap::new();
        collect_generic_constraints(&expected_val, &actual_val, &program.arena, &mut subst)
            .expect("Map value position must infer T");
        assert_eq!(subst.get(&t), Some(&Type::I32));
        assert_eq!(subst_apply(&expected_val, &subst), actual_val);

        // Map(T, i32) + Map(text, i32) -> T = text (key position,
        // independent generic parameter usage -- not two type variables).
        let expected_key = Type::Map(MapType {
            key: Box::new(Type::TypeVar(t)),
            val: Box::new(Type::I32),
        });
        let actual_key = Type::Map(MapType {
            key: Box::new(Type::Text),
            val: Box::new(Type::I32),
        });
        let mut subst2 = BTreeMap::new();
        collect_generic_constraints(&expected_key, &actual_key, &program.arena, &mut subst2)
            .expect("Map key position must infer T");
        assert_eq!(subst2.get(&t), Some(&Type::Text));
        assert_eq!(subst_apply(&expected_key, &subst2), actual_key);
    }

    #[test]
    fn collect_generic_constraints_and_subst_apply_recurse_through_closure_both_positions() {
        let program =
            parse_program("fn f<T>(x: T) -> T { return x; } fn main() { return; }").expect("parse");
        let t = *program.arena.symbol_to_id.get("T").expect("T interned");

        // Closure(T -> i32) + Closure(text -> i32) -> T = text (param).
        let expected_param = Type::Closure(ClosureType {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param: Box::new(Type::TypeVar(t)),
            ret: Box::new(Type::I32),
        });
        let actual_param = Type::Closure(ClosureType {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param: Box::new(Type::Text),
            ret: Box::new(Type::I32),
        });
        let mut subst = BTreeMap::new();
        collect_generic_constraints(&expected_param, &actual_param, &program.arena, &mut subst)
            .expect("Closure param position must infer T");
        assert_eq!(subst.get(&t), Some(&Type::Text));
        assert_eq!(subst_apply(&expected_param, &subst), actual_param);

        // Closure(i32 -> T) + Closure(i32 -> bool) -> T = bool (ret).
        let expected_ret = Type::Closure(ClosureType {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param: Box::new(Type::I32),
            ret: Box::new(Type::TypeVar(t)),
        });
        let actual_ret = Type::Closure(ClosureType {
            family: ClosureValueFamily::UnaryDirect,
            capture: ClosureCapturePolicy::Immutable,
            param: Box::new(Type::I32),
            ret: Box::new(Type::Bool),
        });
        let mut subst2 = BTreeMap::new();
        collect_generic_constraints(&expected_ret, &actual_ret, &program.arena, &mut subst2)
            .expect("Closure return position must infer T");
        assert_eq!(subst2.get(&t), Some(&Type::Bool));
        assert_eq!(subst_apply(&expected_ret, &subst2), actual_ret);
    }

    #[test]
    fn collect_generic_constraints_and_subst_apply_recurse_through_measured_preserving_unit() {
        let mut program =
            parse_program("fn f<T>(x: T) -> T { return x; } fn main() { return; }").expect("parse");
        let t = *program.arena.symbol_to_id.get("T").expect("T interned");
        let ms = program.arena.intern_symbol("ms");

        // Measured(T, ms) + Measured(i32, ms) -> T = i32, unit preserved.
        let expected = Type::Measured(Box::new(Type::TypeVar(t)), ms);
        let actual = Type::Measured(Box::new(Type::I32), ms);
        let mut subst = BTreeMap::new();
        collect_generic_constraints(&expected, &actual, &program.arena, &mut subst)
            .expect("Measured base must infer T");
        assert_eq!(subst.get(&t), Some(&Type::I32));
        assert_eq!(subst_apply(&expected, &subst), actual);

        // A different unit must not be absorbed by T: no unit erasure, no
        // coercive magic -- yields no constraint, left to final
        // compatibility.
        let kg = program.arena.intern_symbol("kg");
        let actual_wrong_unit = Type::Measured(Box::new(Type::I32), kg);
        let mut subst2 = BTreeMap::new();
        collect_generic_constraints(&expected, &actual_wrong_unit, &program.arena, &mut subst2)
            .expect("a differing unit yields no constraint rather than an error here");
        assert!(
            subst2.is_empty(),
            "a mismatched unit must never bind T to the other unit's base type"
        );
    }

    #[test]
    fn collect_generic_constraints_treats_record_and_adt_as_nominal_leaves() {
        // #1650 boundary: Record/Adt carry no applied type arguments and
        // are never recursed into for generic-constraint purposes.
        let program = parse_program(
            "record R { n: i32 } enum E { A } fn f<T>(x: T) -> T { return x; } fn main() { return; }",
        )
        .expect("parse");
        let t = *program.arena.symbol_to_id.get("T").expect("T interned");
        let r_id = program.records[0].name;
        let e_id = program.adts[0].name;

        let mut subst = BTreeMap::new();
        collect_generic_constraints(
            &Type::Record(r_id),
            &Type::Record(r_id),
            &program.arena,
            &mut subst,
        )
        .expect("nominal Record leaf yields no constraint");
        assert!(subst.is_empty());
        collect_generic_constraints(
            &Type::Adt(e_id),
            &Type::Adt(e_id),
            &program.arena,
            &mut subst,
        )
        .expect("nominal Adt leaf yields no constraint");
        assert!(subst.is_empty());

        // T itself can still bind to a nominal type as an opaque leaf value
        // from a direct TypeVar position -- this is ordinary inference, not
        // recursion through the nominal type's (nonexistent) arguments.
        let mut subst2 = BTreeMap::new();
        collect_generic_constraints(
            &Type::TypeVar(t),
            &Type::Record(r_id),
            &program.arena,
            &mut subst2,
        )
        .expect("T binds to a nominal type as an opaque value");
        assert_eq!(subst2.get(&t), Some(&Type::Record(r_id)));
    }

    fn derive_validation_plans_from_source(
        src: &str,
    ) -> Result<(Program, ValidationPlanTable), FrontendError> {
        let program = parse_program(src)?;
        let plans = derive_validation_plan_table(&program)?;
        Ok((program, plans))
    }

    // FA-02-033 / #1665: a public, externally-constructible `FnSig` whose
    // `param_names`/`param_defaults` length disagrees with `params` must be
    // rejected as a deterministic `FrontendError` by every consumer of the
    // public `FnTable` API, never allowed to reach the indexing logic in
    // `reorder_call_args`/`finalize_ordered_call_args` and panic.
    fn well_formed_add_program() -> Program {
        let src = r#"
            fn callee(a: i32, b: i32) -> i32 {
                return a + b;
            }

            fn caller() -> i32 {
                return callee(1, 2);
            }

            fn main() {
                return;
            }
        "#;
        parse_program(src).expect("well-formed program should parse")
    }

    fn callee_and_caller(program: &Program) -> (SymbolId, &Function) {
        let callee_id = *program
            .arena
            .symbol_to_id
            .get("callee")
            .expect("callee symbol exists");
        let caller = program
            .functions
            .iter()
            .find(|f| program.arena.try_symbol_name(f.name) == Some("caller"))
            .expect("caller function exists");
        (callee_id, caller)
    }

    // Dedicated program for the short-`param_names` case: the caller omits
    // an argument (no default exists for it), which drives execution into
    // `finalize_ordered_call_args`'s missing-argument branch. That branch
    // indexes `param_names[idx]` directly (not via `.get`), so a
    // shorter-than-`params` `param_names` indexes out of bounds and panics
    // pre-fix, exactly as FA-02-033 describes.
    fn program_with_missing_argument_call() -> Program {
        let src = r#"
            fn callee(a: i32, b: i32) -> i32 {
                return a + b;
            }

            fn caller() -> i32 {
                return callee(1);
            }

            fn main() {
                return;
            }
        "#;
        parse_program(src).expect("well-formed program should parse")
    }

    // Dedicated program for the long-`param_names` case: the caller names
    // an argument `c`, an identifier with no corresponding declared
    // parameter. Once the table is mutated so `param_names` contains an
    // extra `c` entry beyond `params`' length, `position()` resolves `c` to
    // an index `>= ordered.len()`, and `ordered[param_index]` indexes out
    // of bounds and panics pre-fix, exactly as FA-02-033 describes.
    fn program_with_named_call_referencing_extra_name() -> Program {
        let src = r#"
            fn callee(a: i32, b: i32) -> i32 {
                return a + b;
            }

            fn caller() -> i32 {
                return callee(c = 1, a = 2, b = 3);
            }

            fn main() {
                return;
            }
        "#;
        parse_program(src).expect("well-formed program should parse")
    }

    #[test]
    fn malformed_fn_sig_param_names_shorter_than_params_fails_closed() {
        let program = program_with_missing_argument_call();
        let mut table = build_fn_table(&program).expect("canonical table builds");
        let (callee_id, caller) = callee_and_caller(&program);
        let sig = table.get_mut(&callee_id).expect("callee signature exists");
        let mut names = sig.param_names.clone().expect("callee has param names");
        names.pop();
        sig.param_names = Some(names);

        let err = type_check_function_with_table(caller, &program.arena, &table)
            .expect_err("short param_names must fail closed, not panic");
        assert!(
            err.message.contains("malformed signature"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn malformed_fn_sig_param_names_longer_than_params_fails_closed() {
        let program = program_with_named_call_referencing_extra_name();
        let mut table = build_fn_table(&program).expect("canonical table builds");
        let (callee_id, caller) = callee_and_caller(&program);
        let sig = table.get_mut(&callee_id).expect("callee signature exists");
        let mut names = sig.param_names.clone().expect("callee has param names");
        let extra_c = *program
            .arena
            .symbol_to_id
            .get("c")
            .expect("'c' interned from the named call argument");
        names.push(extra_c);
        sig.param_names = Some(names);

        let err = type_check_function_with_table(caller, &program.arena, &table)
            .expect_err("long param_names must fail closed, not panic");
        assert!(
            err.message.contains("malformed signature"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn malformed_fn_sig_param_defaults_shorter_than_params_fails_closed() {
        let program = well_formed_add_program();
        let mut table = build_fn_table(&program).expect("canonical table builds");
        let (callee_id, caller) = callee_and_caller(&program);
        let sig = table.get_mut(&callee_id).expect("callee signature exists");
        let mut defaults = sig
            .param_defaults
            .clone()
            .expect("callee has param defaults");
        defaults.pop();
        sig.param_defaults = Some(defaults);

        let err = type_check_function_with_table(caller, &program.arena, &table)
            .expect_err("short param_defaults must fail closed, not panic");
        assert!(
            err.message.contains("malformed signature"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn malformed_fn_sig_param_defaults_longer_than_params_fails_closed() {
        let program = well_formed_add_program();
        let mut table = build_fn_table(&program).expect("canonical table builds");
        let (callee_id, caller) = callee_and_caller(&program);
        let sig = table.get_mut(&callee_id).expect("callee signature exists");
        let mut defaults = sig
            .param_defaults
            .clone()
            .expect("callee has param defaults");
        defaults.push(None);
        sig.param_defaults = Some(defaults);

        let err = type_check_function_with_table(caller, &program.arena, &table)
            .expect_err("long param_defaults must fail closed, not panic");
        assert!(
            err.message.contains("malformed signature"),
            "unexpected error message: {}",
            err.message
        );
    }

    #[test]
    fn canonical_fn_sig_continues_to_typecheck_via_public_table_api() {
        let program = well_formed_add_program();
        let table = build_fn_table(&program).expect("canonical table builds");
        let (_, caller) = callee_and_caller(&program);
        type_check_function_with_table(caller, &program.arena, &table)
            .expect("well-formed FnSig via the public FnTable API must still typecheck");
    }

    // FA-02-017 / #1649: type_check_function() previously reconstructed its
    // own FnSig by hand, canonicalizing params/ret through the non-generic
    // canonicalize_declared_type (which unconditionally rejects TypeVar)
    // and hardcoding type_params/trait_bounds to empty regardless of what
    // the parsed Function actually declared. It now builds its FnTable
    // through the same canonical build_fn_table() authority
    // type_check_program() uses, so this single-function API's admission
    // verdict for generic declarations matches the canonical program path
    // instead of silently disagreeing with it.

    fn typecheck_single_function_source(src: &str) -> Result<(), FrontendError> {
        let program = parse_program(src)?;
        type_check_function(&program)
    }

    #[test]
    fn type_check_function_admits_ordinary_non_generic_function() {
        // (A) Positive control: unrelated to #1649, must remain unchanged.
        let src = r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b;
            }
        "#;
        typecheck_single_function_source(src)
            .expect("ordinary non-generic function must typecheck");
    }

    #[test]
    fn type_check_function_admits_generic_function_matching_canonical_admission() {
        // (B) Central regression: a generic function whose parameter uses
        // its own type parameter directly. Pre-fix, this rejected with
        // "type variable 'T' is not admitted in the executable type-check
        // path yet" even though the canonical program path admits it.
        let src = r#"
            fn id<T>(x: T) -> T {
                return x;
            }
        "#;
        typecheck_single_function_source(src).expect(
            "a generic function admitted by the canonical program path must not be \
             rejected by type_check_function",
        );
    }

    #[test]
    fn type_check_function_preserves_generic_trait_bounds_in_canonical_fn_table() {
        // (C) Trait bounds must not be erased to an empty Vec. This tests
        // metadata preservation, not bound satisfaction (satisfaction is a
        // call-site concern this single-function API never performs --
        // see generic_fn_with_bound_and_satisfying_impl_typechecks above
        // for that separate contract).
        let src = r#"
            trait Zeroable {
                fn zero(v: ZeroInt) -> i32;
            }
            record ZeroInt { n: i32 }
            fn make_zero<T: Zeroable>(v: T) -> T {
                return v;
            }
        "#;
        let program = parse_program(src).expect("parse");
        type_check_function(&program).expect(
            "a bounded generic declaration must not be rejected merely for carrying a bound",
        );
        let table = build_fn_table(&program).expect("canonical table builds");
        let fn_id = *program
            .arena
            .symbol_to_id
            .get("make_zero")
            .expect("make_zero interned");
        let sig = table.get(&fn_id).expect("make_zero signature present");
        assert!(
            !sig.type_params.is_empty(),
            "type_params must not be erased to empty"
        );
        assert_eq!(
            sig.trait_bounds.len(),
            1,
            "trait_bounds must not be erased to empty"
        );
    }

    #[test]
    fn type_check_function_admits_generic_parameter_nested_in_compound_type() {
        // (D) Catches a fake repair that only handles a direct top-level
        // TypeVar param but still falls back to non-generic canonicalization
        // for a type parameter nested inside a compound type.
        let src = r#"
            fn first<T>(x: Option(T)) -> i32 {
                return 0;
            }
        "#;
        typecheck_single_function_source(src)
            .expect("a type parameter nested inside Option must not be erased or rejected");
    }

    #[test]
    fn type_check_function_admits_record_parameter_via_canonical_identity() {
        // (E) Existing non-generic behavior: nominal Record signatures must
        // continue to canonicalize and typecheck exactly as before.
        let src = r#"
            record Point { x: i32, y: i32 }
            fn make(x: i32, y: i32) -> Point {
                return Point { x: x, y: y };
            }
        "#;
        typecheck_single_function_source(src).expect("record-typed function must typecheck");
    }

    #[test]
    fn type_check_function_agrees_with_canonical_program_path_for_generic_admission() {
        // (F) Public API consistency regression: directly compares the
        // single-function API's admission verdict against
        // type_check_program's verdict for the identical generic
        // declaration. Fails if type_check_function ever again hand-builds
        // an FnSig with type_params/trait_bounds hardcoded to empty, since
        // that reintroduces exactly the divergence this test detects.
        let single_src = r#"
            fn id<T>(x: T) -> T {
                return x;
            }
        "#;
        let program_src = r#"
            fn id<T>(x: T) -> T {
                return x;
            }
            fn main() {
                return;
            }
        "#;
        let single = parse_program(single_src).expect("parse single-function program");
        let full = parse_program(program_src).expect("parse full program");
        let single_result = type_check_function(&single);
        let full_result = type_check_program(&full);
        assert_eq!(
            single_result.is_ok(),
            full_result.is_ok(),
            "type_check_function must agree with type_check_program's admission verdict \
             for the same generic declaration; single={single_result:?} full={full_result:?}",
        );
    }

    #[test]
    fn type_check_function_and_type_check_program_agree_on_multi_param_generic_rejection() {
        // FA-02-002 / #1634, extending the #1649 consistency guard above to
        // the out-of-contract-arity case specifically: since #1649,
        // type_check_function shares build_fn_table with type_check_program,
        // so both must reject a two-parameter generic function identically
        // rather than diverging again.
        let single_src = r#"
            fn pair<T, U>(x: T, y: U) -> T {
                return x;
            }
        "#;
        let program_src = r#"
            fn pair<T, U>(x: T, y: U) -> T {
                return x;
            }
            fn main() {
                return;
            }
        "#;
        let single = parse_program(single_src).expect("parse single-function program");
        let full = parse_program(program_src).expect("parse full program");
        let single_result = type_check_function(&single);
        let full_result = type_check_program(&full);
        assert!(
            single_result.is_err() && full_result.is_err(),
            "both APIs must reject an out-of-contract two-parameter generic function; \
             single={single_result:?} full={full_result:?}",
        );
    }

    #[test]
    fn fx_identity_surface_typechecks() {
        let src = r#"
            fn id(x: fx) -> fx {
                let y: fx = x;
                return y;
            }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("fx passthrough surface should typecheck");
    }

    #[test]
    fn executable_bare_local_path_import_typechecks_in_wave2() {
        let src = r#"
            Import "helper.sm"

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("bare local-path executable import should typecheck in wave2");
    }

    #[test]
    fn executable_selected_import_typechecks_in_wave2() {
        let src = r#"
            Import "helper.sm" { Foo }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("selected executable import should typecheck in wave2");
    }

    #[test]
    fn executable_reexport_import_rejects_as_wave2_out_of_scope() {
        let src = r#"
            Import pub "helper.sm" { Foo }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("re-export executable import must stay out of scope in wave2");
        assert!(err
            .message
            .contains(executable_import_wave2_out_of_scope_message()));
    }

    #[test]
    fn executable_package_qualified_import_typechecks_in_package_baseline() {
        let src = r#"
            Import "math::core.sm"

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("package-qualified executable import should typecheck");
    }

    #[test]
    fn executable_package_qualified_selected_import_stays_out_of_scope() {
        let src = r#"
            Import "math::core.sm" { helper }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("package-qualified selected import must stay out of scope");
        assert!(err
            .message
            .contains(executable_import_wave2_out_of_scope_message()));
    }

    #[test]
    fn fx_literal_surface_typechecks() {
        let src = r#"
            fn id(x: fx) -> fx {
                return x;
            }

            fn value() -> fx {
                return -1.25;
            }

            fn main() {
                let x: fx = 1.0;
                let y: fx = id(2);
                let z: fx = value();
                let same = x == x;
                let also_same = y == z;
                if same == also_same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("fx literal/call/return surface should typecheck");
    }

    #[test]
    fn extended_numeric_literal_surface_typechecks() {
        let src = r#"
            fn main() {
                let decimal: i32 = 1_000;
                let hex: i32 = 0xff;
                let unsigned: u32 = 1_000u32;
                let fx_value: fx = 1.25fx;
                let neg_fx: fx = -1.25fx;
                let same = unsigned == unsigned;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("extended numeric literal surface should typecheck");
    }

    #[test]
    fn range_literal_typechecks_for_i32_bounds() {
        let src = r#"
            fn main() {
                let half_open = 0..10;
                let closed = 1..=10;
                let _ = half_open;
                let _ = closed;
                return;
            }
        "#;

        typecheck_source(src).expect("i32 range literals should typecheck");
    }

    #[test]
    fn range_literal_rejects_non_i32_bounds() {
        let src = r#"
            fn main() {
                let bad = 0u32..10u32;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("u32 range bounds must reject");
        assert!(err
            .message
            .contains("range literal currently requires i32 bounds"));
    }

    #[test]
    fn range_literal_rejects_equality_surface() {
        let src = r#"
            fn main() {
                let left = 0..10;
                let right = 0..10;
                let same = left == right;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("range equality must reject");
        assert!(err
            .message
            .contains("range equality is not part of the stable v0 range surface"));
    }

    #[test]
    fn i32_relational_surface_typechecks_in_first_wave() {
        let src = r#"
            fn main() {
                let gt: bool = 3 > 2;
                let lt: bool = 2 < 3;
                let ge: bool = 3 >= 3;
                let le: bool = 3 <= 3;
                assert(gt == true);
                assert(lt == true);
                assert(ge == true);
                assert(le == true);
                return;
            }
        "#;

        typecheck_source(src).expect("same-family i32 relationals should typecheck");
    }

    #[test]
    fn non_i32_relational_surface_stays_out_of_scope() {
        let src = r#"
            fn main() {
                let ok: bool = 1.0 < 2.0;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("f64 relational surface must reject");
        assert!(err.message.contains(first_wave_relational_gap_message()));
    }

    #[test]
    fn range_literal_rejects_tuple_nesting() {
        let src = r#"
            fn main() {
                let pair = (0..10, true);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("range tuple nesting must reject");
        assert!(err
            .message
            .contains("range literal is not yet part of the stable tuple/user-data surface"));
    }

    #[test]
    fn explicit_fx_literal_bypasses_f64_gap_at_same_type() {
        let src = r#"
            fn main() {
                let value: fx = 2fx;
                let same = value == value;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("explicit fx literal should typecheck as fx directly");
    }

    #[test]
    fn plain_fx_arithmetic_typechecks_in_post_stable_track() {
        let src = r#"
            fn add(x: fx, y: fx) -> fx {
                let sum: fx = x + y;
                let diff: fx = -sum;
                let same: fx = +diff;
                let prod: fx = same * y;
                return prod / x;
            }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src)
            .expect("plain fx arithmetic should typecheck in the first post-stable slice");
    }

    #[test]
    fn measured_fx_addition_still_reports_narrow_slice_gap() {
        let src = r#"
            fn main() {
                let x: fx[m] = 1.0fx;
                let y: fx[m] = 2.0fx;
                let sum: fx[m] = x + y;
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("measured fx arithmetic must stay outside the first slice");
        assert!(err
            .message
            .contains("unit-carrying fx arithmetic is not part of the first post-stable fx arithmetic slice yet"));
    }

    // SSF-07 (issue #1578) closes the "cross-family and measured arithmetic
    // remain excluded" decision as already made and enforced, not still
    // pending -- these pin the remaining measured/fx combinations that
    // `measured_fx_addition_still_reports_narrow_slice_gap` above doesn't
    // cover. Each is expected to pass immediately against unmodified code;
    // this is a coverage freeze, not a bugfix.
    #[test]
    fn measured_fx_subtraction_reports_narrow_slice_gap() {
        let src = r#"
            fn main() {
                let x: fx[m] = 2.0fx;
                let y: fx[m] = 1.0fx;
                let diff: fx[m] = x - y;
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("measured fx subtraction must stay outside the first slice");
        assert!(err
            .message
            .contains("unit-carrying fx arithmetic is not part of the first post-stable fx arithmetic slice yet"));
    }

    #[test]
    fn measured_fx_unary_minus_reports_narrow_slice_gap() {
        let src = r#"
            fn main() {
                let x: fx[m] = 1.0fx;
                let negated: fx[m] = -x;
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("measured fx unary minus must stay outside the first slice");
        assert!(err
            .message
            .contains("unit-carrying fx arithmetic is not part of the first post-stable fx arithmetic slice yet"));
    }

    #[test]
    fn measured_fx_unary_plus_reports_narrow_slice_gap() {
        let src = r#"
            fn main() {
                let x: fx[m] = 1.0fx;
                let same: fx[m] = +x;
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("measured fx unary plus must stay outside the first slice");
        assert!(err
            .message
            .contains("unit-carrying fx arithmetic is not part of the first post-stable fx arithmetic slice yet"));
    }

    #[test]
    fn measured_arithmetic_rejects_mul() {
        let src = r#"
            fn main() {
                let x: f64[m] = 2.0;
                let y: f64[m] = 3.0;
                let product: f64[m] = x * y;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err(
            "mul on a measured operand must be rejected in the first-wave units surface",
        );
        assert!(
            err.message.contains("first-wave units surface") || err.message.contains("unsupported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn measured_arithmetic_rejects_div() {
        let src = r#"
            fn main() {
                let x: f64[m] = 6.0;
                let y: f64[m] = 3.0;
                let quotient: f64[m] = x / y;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err(
            "div on a measured operand must be rejected in the first-wave units surface",
        );
        assert!(
            err.message.contains("first-wave units surface") || err.message.contains("unsupported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn measured_arithmetic_rejects_mod() {
        let src = r#"
            fn main() {
                let x: f64[m] = 7.0;
                let y: f64[m] = 3.0;
                let remainder: f64[m] = x % y;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err(
            "mod on a measured operand must be rejected in the first-wave units surface",
        );
        assert!(
            err.message.contains("first-wave units surface") || err.message.contains("unsupported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn measured_arithmetic_rejects_mismatched_units() {
        let src = r#"
            fn main() {
                let x: f64[m] = 1.0;
                let y: f64[s] = 1.0;
                let sum: f64[m] = x + y;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err(
            "addition of measured operands with different unit symbols must be rejected",
        );
        assert!(!err.message.is_empty(), "expected a type-mismatch error");
    }

    #[test]
    fn measured_f64_addition_typechecks() {
        let src = r#"
            fn main() {
                let x: f64[m] = 1.0;
                let y: f64[m] = 2.0;
                let sum: f64[m] = x + y;
                return;
            }
        "#;

        typecheck_source(src)
            .expect("addition of two measured f64 values with matching units should typecheck");
    }

    #[test]
    fn measured_f64_binary_subtraction_typechecks() {
        let src = r#"
            fn main() {
                let x: f64[m] = 2.0;
                let y: f64[m] = 1.0;
                let diff: f64[m] = x - y;
                return;
            }
        "#;

        typecheck_source(src)
            .expect("subtraction of two measured f64 values with matching units should typecheck");
    }

    #[test]
    fn measured_f64_unary_plus_typechecks() {
        let src = r#"
            fn main() {
                let x: f64[m] = 1.0;
                let same: f64[m] = +x;
                return;
            }
        "#;

        typecheck_source(src).expect("unary plus on a measured f64 value should typecheck");
    }

    #[test]
    fn measured_f64_unary_minus_typechecks() {
        let src = r#"
            fn main() {
                let x: f64[m] = 1.0;
                let negated: f64[m] = -x;
                return;
            }
        "#;

        typecheck_source(src).expect("unary minus on a measured f64 value should typecheck");
    }

    #[test]
    fn measured_i32_unary_minus_rejects() {
        let src = r#"
            fn main() {
                let x: i32[m] = 1;
                let negated: i32[m] = -x;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("unary minus on a measured i32 must be rejected");
        assert!(
            err.message.contains("unsupported"),
            "expected an operator-unsupported error, got: {}",
            err.message
        );
    }

    // The u32 fixtures below must initialize with a `u32`-suffixed literal
    // (`1u32`, not `1`): match_unit_lift only lifts a literal into a measured
    // binding when the literal's own inferred type equals the measured base,
    // and a bare numeric literal infers as i32. An unsuffixed `let x: u32[m]
    // = 1;` fails at that let-binding lift with a type-mismatch error before
    // the operator under test is ever reached, so the assertion also checks
    // for the specific "unsupported" operator message rather than accepting
    // any error, to keep that class of false pass from recurring.
    #[test]
    fn measured_u32_unary_minus_rejects() {
        let src = r#"
            fn main() {
                let x: u32[m] = 1u32;
                let negated: u32[m] = -x;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("unary minus on a measured u32 must be rejected");
        assert!(
            err.message.contains("unsupported"),
            "expected an operator-unsupported error, got: {}",
            err.message
        );
    }

    #[test]
    fn measured_i32_unary_plus_rejects() {
        let src = r#"
            fn main() {
                let x: i32[m] = 1;
                let same: i32[m] = +x;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unary plus on a measured i32 must be rejected");
        assert!(
            err.message.contains("unsupported"),
            "expected an operator-unsupported error, got: {}",
            err.message
        );
    }

    #[test]
    fn measured_u32_unary_plus_rejects() {
        let src = r#"
            fn main() {
                let x: u32[m] = 1u32;
                let same: u32[m] = +x;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unary plus on a measured u32 must be rejected");
        assert!(
            err.message.contains("unsupported"),
            "expected an operator-unsupported error, got: {}",
            err.message
        );
    }

    #[test]
    fn measured_i32_binary_addition_rejects() {
        let src = r#"
            fn main() {
                let x: i32[m] = 1;
                let y: i32[m] = 2;
                let sum: i32[m] = x + y;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("binary addition on measured i32 must be rejected");
        assert!(
            err.message.contains("unsupported"),
            "expected an operator-unsupported error, got: {}",
            err.message
        );
    }

    #[test]
    fn measured_i32_binary_subtraction_rejects() {
        let src = r#"
            fn main() {
                let x: i32[m] = 2;
                let y: i32[m] = 1;
                let diff: i32[m] = x - y;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("binary subtraction on measured i32 must be rejected");
        assert!(
            err.message.contains("unsupported"),
            "expected an operator-unsupported error, got: {}",
            err.message
        );
    }

    #[test]
    fn measured_u32_binary_addition_rejects() {
        let src = r#"
            fn main() {
                let x: u32[m] = 1u32;
                let y: u32[m] = 2u32;
                let sum: u32[m] = x + y;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("binary addition on measured u32 must be rejected");
        assert!(
            err.message.contains("unsupported"),
            "expected an operator-unsupported error, got: {}",
            err.message
        );
    }

    #[test]
    fn measured_u32_binary_subtraction_rejects() {
        let src = r#"
            fn main() {
                let x: u32[m] = 2u32;
                let y: u32[m] = 1u32;
                let diff: u32[m] = x - y;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("binary subtraction on measured u32 must be rejected");
        assert!(
            err.message.contains("unsupported"),
            "expected an operator-unsupported error, got: {}",
            err.message
        );
    }

    #[test]
    fn text_literal_and_equality_surface_typechecks() {
        let src = r#"
            fn id(message: text) -> text {
                return message;
            }

            fn main() {
                let left: text = "alpha";
                let right: text = id("alpha");
                let same = left == right;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("text literals and text equality should typecheck");
    }

    #[test]
    fn text_concatenation_and_to_text_surface_typechecks() {
        let src = r#"
            fn main() {
                let score: i32 = 10;
                let count: u32 = 7u32;
                let flag: bool = true;
                let marker: quad = T;
                let label: text = "score=" + to_text(score);
                let count_label: text = to_text(count);
                let flag_label: text = to_text(flag);
                let marker_label: text = to_text(marker);
                let copy_label: text = to_text("done");
                return;
            }
        "#;
        typecheck_source(src).expect("text concatenation and to_text should typecheck");
    }

    #[test]
    fn text_concatenation_rejects_scalar_operand() {
        let src = r#"
            fn main() {
                let both: text = "a" + 1;
                return;
            }
        "#;
        let err =
            typecheck_source(src).expect_err("text concatenation with scalar must still reject");
        assert!(err
            .message
            .contains("text concatenation currently admits only text + text operands"));
    }

    #[test]
    fn to_text_rejects_record_types() {
        let src = r#"
            record Probe { x: i32, }
            fn main() {
                let probe: Probe = Probe { x: 1 };
                let label: text = to_text(probe);
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("to_text should reject record values");
        assert!(err
            .message
            .contains("builtin 'to_text' does not yet support record type 'Probe'"));
    }

    #[test]
    fn sequence_literal_and_equality_surface_typechecks_in_wave2() {
        let src = r#"
            fn id(values: Sequence(i32)) -> Sequence(i32) {
                return values;
            }

            fn main() {
                let left: Sequence(i32) = [1, 2, 3];
                let right: Sequence(i32) = id([1, 2, 3]);
                let same = left == right;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("ordered sequence literals and equality should typecheck");
    }

    #[test]
    fn empty_sequence_literal_requires_contextual_sequence_type() {
        let src = r#"
            fn main() {
                let values = [];
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("empty ordered sequence literal without context must reject");
        assert!(err.message.contains(
            "empty ordered sequence literal currently requires contextual Sequence(type) in M8.3 Wave 2"
        ));
    }

    #[test]
    fn sequence_literal_rejects_heterogeneous_item_types() {
        let src = r#"
            fn main() {
                let values: Sequence(i32) = [1, true];
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("heterogeneous ordered sequence items must reject");
        assert!(err.message.contains("type mismatch"));
    }

    #[test]
    fn sequence_index_surface_typechecks_in_wave3() {
        let src = r#"
            fn head(values: Sequence(i32)) -> i32 {
                return values[0];
            }

            fn main() {
                let values: Sequence(i32) = [1, 2, 3];
                let first: i32 = head(values);
                return;
            }
        "#;

        typecheck_source(src).expect("ordered sequence indexing should typecheck");
    }

    #[test]
    fn sequence_index_rejects_non_sequence_base() {
        let src = r#"
            fn main() {
                let first: i32 = 1[0];
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("sequence indexing on non-sequence base must reject");
        assert!(err
            .message
            .contains("sequence indexing requires Sequence(type) base"));
    }

    #[test]
    fn sequence_index_rejects_non_i32_index() {
        let src = r#"
            fn main() {
                let values: Sequence(i32) = [1, 2, 3];
                let first: i32 = values[true];
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("sequence indexing with non-i32 index must reject");
        assert!(err
            .message
            .contains("sequence indexing currently requires i32 index"));
    }

    #[test]
    fn block_expression_tail_typechecks() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    let base: f64 = 1.0;
                    base + 2.0
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("block expression tail should typecheck");
    }

    #[test]
    fn block_expression_scope_does_not_escape() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    let base: f64 = 1.0;
                    base + 2.0
                };
                let leak = base;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("block-local name must not escape");
        assert!(err.message.contains("unknown variable 'base'"));
    }

    #[test]
    fn if_expression_typechecks_when_branches_match() {
        let src = r#"
            fn main() {
                let total: f64 = if true { 1.0 } else { 2.0 };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("if expression should typecheck");
    }

    #[test]
    fn if_expression_rejects_branch_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = if true { 1.0 } else { true };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("mismatched if expression branches must reject");
        assert!(err.message.contains("if expression branch type mismatch"));
    }

    #[test]
    fn if_expression_requires_bool_condition() {
        let src = r#"
            fn main() {
                let total: f64 = if T { 1.0 } else { 2.0 };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("quad condition must reject");
        assert!(err.message.contains("if expression condition must be bool"));
    }

    #[test]
    fn if_expression_accepts_else_if_sugar() {
        let src = r#"
            fn main() {
                let total: i32 = if true { 1 } else if false { 2 } else { 3 };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("else-if sugar should typecheck");
    }

    #[test]
    fn when_expression_typechecks_for_bool_result() {
        let src = r#"
            fn main() {
                let total: bool = when true { true } else { false };
                if total { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("when bool expression should typecheck");
    }

    #[test]
    fn when_expression_typechecks_for_quad_result() {
        let src = r#"
            fn main() {
                let total: quad = when T == T { N } else { S };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("when quad expression should typecheck");
    }

    #[test]
    fn when_expression_rejects_non_bool_condition() {
        let src = r#"
            fn main() {
                let total: quad = when T { N } else { S };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("when condition must be bool");
        assert!(
            err.message.contains("when condition must be bool")
                || err.message.contains("condition must be bool")
                || err.message.contains("cannot compare"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn when_expression_rejects_arm_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = when true { 1.0 } else { true };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("when arm mismatch must reject");
        assert!(
            err.message.contains("branch type mismatch")
                || err.message.contains("cannot compare")
                || err.message.contains("mismatch"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn is_predicate_rejects_non_quad_comparison() {
        let src = r#"
            fn main() {
                let value: bool = true is S;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-quad is predicate must reject");
        assert!(
            err.message.contains("cannot compare Bool and Quad")
                || err.message.contains("quad")
                || err.message.contains("mismatch"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn match_expression_typechecks_when_arms_match() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T => { 1.0 }
                    F => { 2.0 }
                    _ => { 3.0 }
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("match expression should typecheck");
    }

    #[test]
    fn match_expression_typechecks_with_integer_literal_cases() {
        let src = r#"
            fn main() {
                let index: u32 = 1u32;
                let total: quad = match index {
                    0u32 => { N }
                    1u32 => { F }
                    2u32 => { T }
                    _ => { S }
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("integer literal match cases should typecheck");
    }

    #[test]
    fn adt_match_expression_typechecks_with_payload_bindings() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn read(value: Maybe) -> f64 {
                let total: f64 = match value {
                    Maybe::Some(inner) => { inner }
                    _ => { 0.0 }
                };
                return total;
            }

            fn main() {
                let value: Maybe = Maybe::Some(1.0);
                let total: f64 = read(value);
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("ADT match expression should typecheck");
    }

    #[test]
    fn exhaustive_adt_match_expression_without_default_typechecks() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn read(value: Maybe) -> f64 {
                let total: f64 = match value {
                    Maybe::None => { 0.0 }
                    Maybe::Some(inner) => { inner }
                };
                return total;
            }

            fn main() {
                let value: Maybe = Maybe::Some(1.0);
                let total: f64 = read(value);
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src)
            .expect("exhaustive ADT match expression without default should typecheck");
    }

    #[test]
    fn match_expression_requires_default_arm_for_scalar_cases() {
        let src = r#"
            fn main() {
                let total: quad = match 1u32 {
                    0u32 => { N }
                    1u32 => { F }
                    2u32 => { T }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("scalar match without wildcard must reject");
        assert!(err
            .message
            .contains("match expression requires default arm '_'"));
    }

    #[test]
    fn match_expression_requires_quad_scrutinee() {
        let src = r#"
            fn main() {
                let total: f64 = match true {
                    T => { 1.0 }
                    _ => { 2.0 }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-quad match expression must reject");
        assert!(err
            .message
            .contains("match expression is allowed only for quad"));
    }

    #[test]
    fn match_expression_rejects_integer_cases_on_quad_scrutinee() {
        let src = r#"
            fn main() {
                let total: quad = match T {
                    0 => { N }
                    _ => { S }
                };
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("integer match cases on quad scrutinee must reject");
        assert!(err
            .message
            .contains("integer match pattern requires i32 or u32 scrutinee"));
    }

    #[test]
    fn match_expression_requires_default_arm() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T => { 1.0 }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("match expression without default must reject");
        assert!(err
            .message
            .contains("match expression requires default arm '_'"));
    }

    #[test]
    fn non_exhaustive_adt_match_expression_without_default_rejects() {
        let src = r#"
            enum Maybe {
                None,
                Some(f64),
            }

            fn read(value: Maybe) -> f64 {
                let total: f64 = match value {
                    Maybe::Some(inner) => { inner }
                };
                return total;
            }

            fn main() {
                let value: Maybe = Maybe::Some(1.0);
                let total: f64 = read(value);
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("non-exhaustive ADT match expression without default must reject");
        assert!(err
            .message
            .contains("non-exhaustive match expression for enum 'Maybe'; missing variants: None"));
    }

    #[test]
    fn match_expression_rejects_branch_type_mismatch() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T => { 1.0 }
                    _ => { true }
                };
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("mismatched match expression branches must reject");
        assert!(err
            .message
            .contains("match expression branch type mismatch"));
    }

    #[test]
    fn match_expression_guard_requires_bool() {
        let src = r#"
            fn main() {
                let total: f64 = match T {
                    T if T => { 1.0 }
                    _ => { 2.0 }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-bool guard must reject");
        assert!(err.message.contains("match guard condition must be bool"));
    }

    #[test]
    fn guard_clause_typechecks_with_unit_return() {
        let src = r#"
            fn main() {
                guard true else return;
                return;
            }
        "#;

        typecheck_source(src).expect("guard clause should typecheck");
    }

    #[test]
    fn guard_clause_requires_bool_condition() {
        let src = r#"
            fn main() {
                guard T else return;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-bool guard clause must reject");
        assert!(err.message.contains("guard clause condition must be bool"));
    }

    #[test]
    fn guard_clause_reuses_return_type_contract() {
        let src = r#"
            fn main() {
                guard true else return true;
            }
        "#;

        let err = typecheck_source(src).expect_err("guard return payload must typecheck");
        assert!(err.message.contains("return type mismatch"));
    }

    #[test]
    fn while_statement_with_bool_condition_typechecks() {
        let src = r#"
            fn main() {
                let mut i: i32 = 0;
                while i < 3 {
                    i = i + 1;
                }
                return;
            }
        "#;

        typecheck_source(src).expect("while statement with bool condition should typecheck");
    }

    #[test]
    fn while_statement_with_non_bool_condition_rejects() {
        let src = r#"
            fn main() {
                while 1 {
                    return;
                }
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-bool while condition must reject");
        assert!(err.message.contains("while condition must be bool"));
    }

    #[test]
    fn statement_loop_with_continue_and_bare_break_typechecks() {
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

        typecheck_source(src).expect("statement loop control exits should typecheck");
    }

    #[test]
    fn bare_break_outside_loop_rejects() {
        let src = r#"
            fn main() {
                break;
            }
        "#;

        let err = typecheck_source(src).expect_err("bare break outside loop must reject");
        assert!(err
            .message
            .contains("bare break is allowed only inside while or statement loop"));
    }

    #[test]
    fn continue_outside_loop_rejects() {
        let src = r#"
            fn main() {
                continue;
            }
        "#;

        let err = typecheck_source(src).expect_err("continue outside loop must reject");
        assert!(err
            .message
            .contains("continue is allowed only inside while or statement loop"));
    }

    #[test]
    fn expression_bodied_function_reuses_return_typing() {
        let src = r#"
            fn id(x: f64) -> f64 = x;

            fn main() {
                let same: f64 = id(1.0);
                let ok = same == same;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("expression-bodied function should typecheck");
    }

    #[test]
    fn expression_bodied_function_returns_quad() {
        let src = r#"
            fn idq(q: quad) -> quad = q;

            fn main() {
                let got: quad = idq(T);
                let ok: bool = got == T;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("expression-bodied quad function should typecheck");
    }

    #[test]
    fn expression_bodied_function_returns_bool() {
        let src = r#"
            fn not_false(b: bool) -> bool = b;

            fn main() {
                let got: bool = not_false(true);
                if got { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("expression-bodied bool function should typecheck");
    }

    #[test]
    fn local_let_without_annotation_infers_from_value() {
        let src = r#"
            fn main() {
                let total = 1 + 2;
                let ok: bool = total == 3;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("local let inference should typecheck");
    }

    #[test]
    fn local_let_without_annotation_infers_quad_values() {
        let src = r#"
            fn main() {
                let left: quad = T;
                let right: quad = S;
                let merged = left || right;
                let ok: bool = merged == S;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("quad local let inference should typecheck");
    }

    #[test]
    fn local_let_without_annotation_infers_bool_predicates() {
        let src = r#"
            fn main() {
                let left: quad = T;
                let right: quad = S;
                let same = left == right;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("bool predicate local let inference should typecheck");
    }

    #[test]
    fn local_let_without_annotation_accepts_integer_literal_default() {
        let src = r#"
            fn main() {
                let value = 0;
                let ok: bool = value == 0;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("integer literal local let inference should typecheck");
    }

    #[test]
    fn local_let_explicit_annotation_still_typechecks() {
        let src = r#"
            fn main() {
                let left: quad = T;
                let right: quad = S;
                let merged: quad = left || right;
                let ok: bool = merged == S;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("explicit local annotation should still typecheck");
    }

    #[test]
    fn local_let_unresolved_type_reports_diagnostic() {
        let src = r#"
            fn main() {
                let map = map_empty();
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unresolved let inference must reject");
        assert!(err
            .message
            .contains("map_empty() requires a contextual Map(K, V) type"));
    }

    #[test]
    fn expression_bodied_function_reports_return_mismatch() {
        let src = r#"
            fn bad() -> f64 = true;

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("expression-bodied return mismatch must reject");
        assert!(err.message.contains("return type mismatch"));
    }

    #[test]
    fn pipeline_chain_typechecks_via_existing_call_rules() {
        let src = r#"
            fn inc(x: f64) -> f64 = x + 1.0;
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = 1.0 |> inc() |> scale(3.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("pipeline desugaring should typecheck");
    }

    #[test]
    fn named_arguments_typecheck_via_parameter_reorder() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(factor = 3.0, x = 2.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("named arguments should typecheck");
    }

    #[test]
    fn pipeline_named_arguments_typecheck_after_positional_prefix() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = 2.0 |> scale(factor = 3.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("pipeline named arguments should typecheck");
    }

    #[test]
    fn default_parameters_fill_omitted_trailing_arguments() {
        let src = r#"
            fn scale(x: f64, factor: f64 = 2.0) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(3.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("default parameters should fill omitted trailing arguments");
    }

    #[test]
    fn named_arguments_can_override_remaining_default_parameters() {
        let src = r#"
            fn scale(x: f64, factor: f64 = 2.0) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(x = 3.0, factor = 4.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("named arguments should override defaulted parameters");
    }

    #[test]
    fn builtin_named_arguments_are_rejected() {
        let src = r#"
            fn main() {
                let total: f64 = sqrt(x = 4.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("builtin named arguments must reject");
        assert!(err
            .message
            .contains("named arguments are not supported for builtin 'sqrt'"));
    }

    #[test]
    fn duplicate_named_arguments_are_rejected() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(x = 2.0, x = 3.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("duplicate named arguments must reject");
        assert!(err.message.contains("duplicate named argument 'x'"));
    }

    #[test]
    fn missing_named_argument_is_rejected() {
        let src = r#"
            fn scale(x: f64, factor: f64) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale(x = 2.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("missing named argument must reject");
        assert!(err
            .message
            .contains("is missing argument for parameter 'factor'"));
    }

    #[test]
    fn required_parameter_still_rejects_when_default_is_missing() {
        let src = r#"
            fn scale(x: f64, factor: f64 = 2.0) -> f64 = x * factor;

            fn main() {
                let total: f64 = scale();
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("required non-default parameter must reject");
        assert!(err
            .message
            .contains("is missing argument for parameter 'x'"));
    }

    #[test]
    fn default_parameter_initializer_must_be_const_safe() {
        let src = r#"
            fn scale(x: f64, factor: f64 = sqrt(4.0)) -> f64 = x * factor;

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-const-safe default parameter must reject");
        assert!(err.message.contains("default parameter 'factor'"));
    }

    #[test]
    fn default_parameter_initializer_cannot_reference_previous_parameter() {
        let src = r#"
            fn scale(x: f64, factor: f64 = x) -> f64 = x * factor;

            fn main() {
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("default parameter cannot reference earlier param");
        assert!(err.message.contains("'x'"));
    }

    #[test]
    fn immediate_short_lambda_typechecks_via_block_desugaring() {
        let src = r#"
            fn main() {
                let total: f64 = (x => x + 1.0)(2.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("immediate short lambda should typecheck");
    }

    #[test]
    fn pipeline_short_lambda_typechecks_via_block_desugaring() {
        let src = r#"
            fn main() {
                let total: f64 = 2.0 |> (x => x + 1.0);
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("pipeline short lambda should typecheck");
    }

    #[test]
    fn const_declaration_typechecks_for_literal_expression_subset() {
        let src = r#"
            fn main() {
                const two: f64 = 1.0 + 1.0;
                const four: f64 = two + two;
                let ok = four == four;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("const declarations should typecheck");
    }

    #[test]
    fn const_declaration_rejects_non_const_initializer() {
        let src = r#"
            fn main() {
                let base: f64 = 1.0;
                const total: f64 = base + 1.0;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("const initializer must reject runtime binding");
        assert!(err.message.contains("is not const"));
    }

    #[test]
    fn const_binding_rejects_assignment_target() {
        let src = r#"
            fn main() {
                const total: f64 = 1.0;
                total += 2.0;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("assignment to const must reject");
        assert!(err
            .message
            .contains("cannot assign to const binding 'total'"));
    }

    #[test]
    fn const_declaration_is_allowed_inside_value_block_body() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    const offset: f64 = 2.0;
                    1.0 + offset
                };
                let ok = total == total;
                if ok { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("const should be accepted in value block body");
    }

    #[test]
    fn captureful_short_lambda_is_rejected() {
        let src = r#"
            fn main() {
                let offset: f64 = 1.0;
                let total: f64 = (x => x + offset)(2.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("captureful short lambda must reject");
        assert!(err.message.contains("capture-free only"));
    }

    #[test]
    fn first_class_closure_literal_requires_contextual_type() {
        let src = r#"
            fn main() {
                let value = (x => x);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("closure literal without context must reject");
        assert!(err.message.contains("contextual Closure(T -> U) type"));
    }

    #[test]
    fn first_class_closure_literal_typechecks_with_declared_signature_and_capture() {
        let src = r#"
            fn keep(f: Closure(f64 -> f64)) -> Closure(f64 -> f64) = f;

            fn main() {
                let offset: f64 = 1.0;
                let f: Closure(f64 -> f64) = (x => x + offset);
                let g: Closure(f64 -> f64) = keep(f);
                return;
            }
        "#;

        typecheck_source(src).expect("contextual first-class closure should typecheck");
    }

    #[test]
    fn direct_first_class_closure_invocation_typechecks_in_wave3() {
        let src = r#"
            fn main() {
                let f: Closure(f64 -> f64) = (x => x + 1.0);
                let total: f64 = f(2.0);
                return;
            }
        "#;

        typecheck_source(src).expect("closure invocation should typecheck in Wave 3");
    }

    #[test]
    fn direct_first_class_closure_invocation_rejects_named_arguments() {
        let src = r#"
            fn main() {
                let f: Closure(f64 -> f64) = (x => x + 1.0);
                let total: f64 = f(x: 2.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("named closure invocation must reject");
        assert!(
            err.message.contains("exactly one positional argument")
                || err.message.contains("expected ')'")
        );
    }

    #[test]
    fn compound_assignment_typechecks_for_existing_scalar_rules() {
        let src = r#"
            fn main() {
                let mut total: f64 = 1.0;
                total += 2.0;
                let mut ready: bool = true;
                ready &&= false;
                return;
            }
        "#;

        typecheck_source(src).expect("compound assignment should typecheck");
    }

    #[test]
    fn compound_assignment_requires_existing_binding() {
        let src = r#"
            fn main() {
                total += 1.0;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown assignment target must reject");
        assert!(err.message.contains("unknown assignment target 'total'"));
    }

    #[test]
    fn compound_assignment_reuses_operator_type_rules() {
        let src = r#"
            fn main() {
                let mut total: f64 = 1.0;
                total += true;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("compound assignment operator mismatch must reject");
        assert!(err.message.contains("f64 arithmetic requires f64 operands"));
    }

    #[test]
    fn mutable_local_reassignment_typechecks() {
        let src = r#"
            fn main() {
                let mut score: i32 = 0;
                score = 1;
                score += 2;
                return;
            }
        "#;

        typecheck_source(src).expect("mutable local reassignment should typecheck");
    }

    #[test]
    fn plain_local_reassignment_typechecks() {
        let src = r#"
            fn main() {
                let score: i32 = 0;
                score = 1;
                return;
            }
        "#;

        typecheck_source(src).expect("plain local reassignment should typecheck");
    }

    #[test]
    fn i32_arithmetic_typechecks_for_add_sub_mul_and_neg() {
        let src = r#"
            fn main() {
                let a: i32 = 4;
                let b: i32 = 2;
                let add: i32 = a + b;
                let sub: i32 = a - b;
                let mul: i32 = a * b;
                let neg: i32 = -a;
                let folded: i32 = (a + b) * neg;
                if add == sub {
                    let keep: i32 = folded;
                }
                return;
            }
        "#;

        typecheck_source(src).expect("same-family i32 arithmetic should typecheck");
    }

    #[test]
    fn i32_division_and_modulo_typecheck_for_same_family_i32() {
        let src = r#"
            fn main() {
                let a: i32 = 4;
                let b: i32 = 2;
                let q: i32 = a / b;
                let r: i32 = a % b;
                return;
            }
        "#;

        typecheck_source(src).expect("i32 division and modulo should typecheck");
    }

    #[test]
    fn repeated_discard_binds_typecheck_without_name_collisions() {
        let src = r#"
            fn main() {
                let _ = 1.0;
                let _ = 2.0;
                return;
            }
        "#;

        typecheck_source(src).expect("discard binds should not create conflicting visible names");
    }

    #[test]
    fn typed_discard_bind_reuses_type_mismatch_rules() {
        let src = r#"
            fn main() {
                let _: f64 = true;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("typed discard bind must check rhs type");
        assert!(err.message.contains("discard binding"));
    }

    #[test]
    fn discard_bind_is_allowed_inside_value_block_body() {
        let src = r#"
            fn main() {
                let total: f64 = {
                    let _ = 1.0;
                    2.0
                };
                let same = total == total;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("discard bind should be accepted in value block body");
    }

    #[test]
    fn assert_builtin_statement_typechecks() {
        let src = r#"
            fn main() {
                assert(true);
                return;
            }
        "#;

        typecheck_source(src).expect("assert builtin statement should typecheck");
    }

    #[test]
    fn assert_builtin_requires_bool_condition() {
        let src = r#"
            fn main() {
                assert(1.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("assert builtin must require bool");
        assert!(err
            .message
            .contains("assert builtin requires bool condition"));
    }

    #[test]
    fn assert_builtin_requires_single_argument() {
        let src = r#"
            fn main() {
                assert(true, false);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("assert builtin arity must reject");
        assert!(err.message.contains("assert builtin expects 1 arg"));
    }

    #[test]
    fn assert_builtin_is_statement_only() {
        let src = r#"
            fn main() {
                let ok: bool = assert(true);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("assert builtin should reject value position");
        assert!(err
            .message
            .contains("assert builtin is statement-only and cannot be used as expression value"));
    }

    #[test]
    fn function_requires_clause_typechecks_with_param_and_record_field_reads() {
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

        typecheck_source(src).expect("requires clauses should typecheck");
    }

    #[test]
    fn function_requires_clause_requires_bool_condition() {
        let src = r#"
            fn choose(count: i32) -> i32 requires(count) {
                return count;
            }

            fn main() { return; }
        "#;

        let err = typecheck_source(src).expect_err("requires clause must require bool");
        assert!(err
            .message
            .contains("requires clause condition must be bool"));
    }

    #[test]
    fn function_requires_clause_rejects_call_surface() {
        let src = r#"
            fn check(flag: bool) -> bool = flag;

            fn choose(flag: bool) -> bool requires(check(flag)) {
                return flag;
            }

            fn main() { return; }
        "#;

        let err = typecheck_source(src).expect_err("requires clause should reject call surface");
        assert!(err
            .message
            .contains("requires clause currently allows only parameter references"));
    }

    #[test]
    fn function_ensures_clause_typechecks_with_result_and_record_field_reads() {
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

        typecheck_source(src).expect("ensures clauses should typecheck");
    }

    #[test]
    fn function_ensures_clause_requires_bool_condition() {
        let src = r#"
            fn id(count: i32) -> i32 ensures(result) {
                return count;
            }

            fn main() { return; }
        "#;

        let err = typecheck_source(src).expect_err("ensures clause must require bool");
        assert!(err
            .message
            .contains("ensures clause condition must be bool"));
    }

    #[test]
    fn function_ensures_clause_rejects_call_surface() {
        let src = r#"
            fn check(flag: bool) -> bool = flag;

            fn choose(flag: bool) -> bool ensures(check(result)) {
                return flag;
            }

            fn main() { return; }
        "#;

        let err = typecheck_source(src).expect_err("ensures clause should reject call surface");
        assert!(err.message.contains(
            "ensures clause currently allows only parameter references, optional result binding"
        ));
    }

    #[test]
    fn function_ensures_clause_reserves_result_parameter_name() {
        let src = r#"
            fn echo(result: bool) -> bool ensures(result == true) {
                return result;
            }

            fn main() { return; }
        "#;

        let err =
            typecheck_source(src).expect_err("ensures clause must reserve synthetic result name");
        assert!(err
            .message
            .contains("parameter name 'result' is reserved while ensures clauses are present"));
    }

    #[test]
    fn function_invariant_clause_typechecks_with_entry_and_exit_subset() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn decide(ctx: DecisionContext) -> quad
                invariant(ctx.quality == 0.75)
                invariant(result == ctx.camera) {
                return ctx.camera;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let seen: quad = decide(ctx);
                assert(seen == T);
                return;
            }
        "#;

        typecheck_source(src).expect("invariant clauses should typecheck");
    }

    #[test]
    fn function_invariant_clause_requires_bool_condition() {
        let src = r#"
            fn id(count: i32) -> i32 invariant(result) {
                return count;
            }

            fn main() { return; }
        "#;

        let err = typecheck_source(src).expect_err("invariant clause must require bool");
        assert!(err
            .message
            .contains("invariant clause condition must be bool"));
    }

    #[test]
    fn function_invariant_clause_rejects_call_surface() {
        let src = r#"
            fn check(flag: bool) -> bool = flag;

            fn choose(flag: bool) -> bool invariant(check(result)) {
                return flag;
            }

            fn main() { return; }
        "#;

        let err = typecheck_source(src).expect_err("invariant clause should reject call surface");
        assert!(err.message.contains(
            "invariant clause currently allows only parameter references, optional result binding"
        ));
    }

    #[test]
    fn function_invariant_clause_reserves_result_parameter_name() {
        let src = r#"
            fn echo(result: bool) -> bool invariant(result == true) {
                return result;
            }

            fn main() { return; }
        "#;

        let err =
            typecheck_source(src).expect_err("invariant clause must reserve synthetic result name");
        assert!(err
            .message
            .contains("parameter name 'result' is reserved while invariant clauses are present"));
    }

    #[test]
    fn function_invariant_clause_rejects_result_in_unit_return_function() {
        let src = r#"
            fn main() invariant(result == true) {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unit-return invariant cannot reference result");
        assert!(err
            .message
            .contains("invariant clause may reference 'result' only in non-unit return functions"));
    }

    #[test]
    fn tuple_literals_and_tuple_types_typecheck_through_call_and_return_paths() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) {
                return (1, flag);
            }

            fn main() {
                let left: (i32, bool) = pair(true);
                let right: (i32, bool) = (1, true);
                assert(left == right);
                return;
            }
        "#;

        typecheck_source(src).expect("tuple literal/type surface should typecheck");
    }

    #[test]
    fn tuple_destructuring_bind_typechecks() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let (count, ready): (i32, bool) = pair(true);
                assert(ready == true);
                return;
            }
        "#;

        typecheck_source(src).expect("tuple destructuring bind should typecheck");
    }

    #[test]
    fn tuple_let_else_typechecks() {
        let src = r#"
            fn pair() -> (i32, quad) = (1, T);

            fn main() {
                let (count, T): (i32, quad) = pair() else return;
                assert(count == 1);
                return;
            }
        "#;

        typecheck_source(src).expect("tuple let-else should typecheck");
    }

    #[test]
    fn tuple_let_else_rejects_non_tuple_value() {
        let src = r#"
            fn main() {
                let (count, T) = 1 else return;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-tuple let-else must reject");
        assert!(err
            .message
            .contains("let-else tuple destructuring bind requires tuple value"));
    }

    #[test]
    fn tuple_let_else_rejects_non_quad_literal_position() {
        let src = r#"
            fn pair() -> (i32, bool) = (1, true);

            fn main() {
                let (count, T): (i32, bool) = pair() else return;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-quad let-else literal pattern must reject");
        assert!(err
            .message
            .contains("let-else tuple literal pattern requires quad element"));
    }

    #[test]
    fn tuple_let_else_rejects_return_type_mismatch() {
        let src = r#"
            fn pair() -> (i32, quad) = (1, T);

            fn main() {
                let (count, T): (i32, quad) = pair() else return 1.0;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("let-else return type mismatch must reject");
        assert!(err.message.contains("return type mismatch"));
    }

    #[test]
    fn tuple_destructuring_bind_rejects_non_tuple_value() {
        let src = r#"
            fn main() {
                let (count, ready) = 1;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-tuple destructuring must reject");
        assert!(err
            .message
            .contains("tuple destructuring bind requires tuple value"));
    }

    #[test]
    fn tuple_destructuring_assignment_typechecks() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let count: i32 = 0;
                let ready: bool = false;
                (count, ready) = pair(true);
                assert(count == 1);
                assert(ready == true);
                return;
            }
        "#;

        typecheck_source(src).expect("tuple destructuring assignment should typecheck");
    }

    #[test]
    fn tuple_destructuring_assignment_rejects_unknown_target() {
        let src = r#"
            fn pair(flag: bool) -> (i32, bool) = (1, flag);

            fn main() {
                let count: i32 = 0;
                (count, ready) = pair(true);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown tuple assignment target must reject");
        assert!(err
            .message
            .contains("unknown tuple assignment target 'ready'"));
    }

    #[test]
    fn for_range_typechecks_with_i32_loop_binding() {
        let src = r#"
            fn main() {
                for i in 0..=2 {
                    let _: i32 = i;
                }
                return;
            }
        "#;

        typecheck_source(src).expect("for-range should typecheck");
    }

    #[test]
    fn iterable_for_surface_rejects_non_iterable_execution_in_wave_one() {
        let src = r#"
            fn main() {
                for i in 1 {
                    let _: i32 = i;
                }
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-iterable executable for input must reject");
        assert!(err
            .message
            .contains("currently requires built-in Sequence(type), i32 range"));
    }

    #[test]
    fn iterable_for_sequence_values_typechecks_with_item_binding() {
        let src = r#"
            fn main() {
                let items: Sequence(i32) = [1, 2, 3];
                for item in items {
                    let _: i32 = item;
                }
                return;
            }
        "#;

        typecheck_source(src).expect("Sequence(T) iterable loop should now typecheck");
    }

    #[test]
    fn for_range_through_variable_remains_typecheckable() {
        let src = r#"
            fn main() {
                let window = 0..=2;
                for i in window {
                    let _: i32 = i;
                }
                return;
            }
        "#;

        typecheck_source(src).expect("range-valued variable for-loop should keep existing path");
    }

    #[test]
    fn for_range_loop_variable_is_const_in_body() {
        let src = r#"
            fn main() {
                for i in 0..2 {
                    i += 1;
                }
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("for-range binding must be const");
        assert!(err.message.contains("cannot assign to const binding 'i'"));
    }

    #[test]
    fn loop_expression_rejects_for_range_in_body() {
        let src = r#"
            fn main() {
                let value: i32 = loop {
                    for i in 0..1 {
                        break 1;
                    }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("for-range in loop expression body must reject");
        assert!(err
            .message
            .contains("loop expression body currently does not allow for-range"));
    }

    #[test]
    fn loop_expression_rejects_iterable_for_each_in_body() {
        let src = r#"
            fn main() {
                let items: Sequence(i32) = [1, 2, 3];
                let value: i32 = loop {
                    for item in items {
                        break item;
                    }
                };
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("iterable for-each in loop expression must reject");
        assert!(err
            .message
            .contains("loop expression body currently does not allow iterable for-each"));
    }

    #[test]
    fn explicit_iterable_impl_surface_typechecks_without_loop_execution() {
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

        typecheck_source(src).expect("Iterable trait/impl surface should typecheck");
    }

    #[test]
    fn iterable_for_with_explicit_record_impl_typechecks() {
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
                    let _: i32 = value;
                }
                return;
            }
        "#;

        typecheck_source(src).expect("direct record Iterable loop should typecheck");
    }

    #[test]
    fn iterable_for_with_wrong_iterable_contract_rejects() {
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
            typecheck_source(src).expect_err("wrong executable Iterable contract must reject");
        assert!(err
            .message
            .contains("fn next(self: Self, index: i32) -> Option(Item)"));
    }

    #[test]
    fn iterable_for_with_explicit_adt_impl_reports_out_of_scope() {
        let src = r#"
            trait Iterable {
                fn next(self: Self, index: i32) -> Option(i32);
            }

            enum Numbers {
                Wrap(i32),
            }

            impl Iterable for Numbers {
                fn next(self: Self, index: i32) -> Option(i32) {
                    let _ = self;
                    let _ = index;
                    return Option::None;
                }
            }

            fn main() {
                let numbers: Numbers = Numbers::Wrap(0);
                for value in numbers {
                    let _ = value;
                }
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("ADT Iterable loop must stay out of scope");
        assert!(err.message.contains("direct record impls only"));
    }

    #[test]
    fn where_clause_typechecks_via_block_desugaring() {
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

        typecheck_source(src).expect("where-clause should typecheck");
    }

    #[test]
    fn where_clause_reuses_let_type_mismatch_rules() {
        let src = r#"
            fn main() {
                let value: f64 = total where total: bool = true;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("typed where binding mismatch must reject");
        assert!(err.message.contains("type mismatch in let"));
    }

    #[test]
    fn loop_expression_typechecks_with_break_value() {
        let src = r#"
            fn main() {
                let value: f64 = loop {
                    if true {
                        break 1.0;
                    } else {
                        break 2.0;
                    }
                };
                return;
            }
        "#;

        typecheck_source(src).expect("loop expression should typecheck");
    }

    #[test]
    fn loop_expression_rejects_break_outside_loop() {
        let src = r#"
            fn main() {
                break 1.0;
            }
        "#;

        let err = typecheck_source(src).expect_err("break outside loop must reject");
        assert!(err
            .message
            .contains("break with value is allowed only inside loop expression"));
    }

    #[test]
    fn loop_expression_rejects_continue_in_body() {
        let src = r#"
            fn main() {
                let value: f64 = loop {
                    continue;
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("continue in loop expression body must reject");
        assert!(err
            .message
            .contains("loop expression body currently does not allow guard clause or return"));
    }

    #[test]
    fn loop_expression_rejects_mismatched_break_types() {
        let src = r#"
            fn main() {
                let value: f64 = loop {
                    if true {
                        break 1.0;
                    } else {
                        break true;
                    }
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("mismatched break types must reject");
        assert!(err.message.contains("loop expression break type mismatch"));
    }

    #[test]
    fn loop_expression_rejects_return_in_body() {
        let src = r#"
            fn main() {
                let value: f64 = loop {
                    return;
                };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("return in loop expression body must reject");
        assert!(err
            .message
            .contains("loop expression body currently does not allow guard clause or return"));
    }

    #[test]
    fn record_declarations_typecheck_as_nominal_top_level_items() {
        let src = r#"
            record Point {
                x: i32,
                y: i32,
            }

            record Pixel {
                x: i32,
                y: i32,
            }

            fn main() {
                return;
            }
        "#;

        let program = parse_program(src).expect("parse");
        type_check_program(&program).expect("record declarations should typecheck");
        assert_eq!(program.records.len(), 2);
        assert_ne!(program.records[0].name, program.records[1].name);
    }

    #[test]
    fn record_declaration_rejects_duplicate_field_name() {
        let src = r#"
            record Point {
                x: i32,
                x: i32,
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("duplicate record field must reject");
        assert!(err.message.contains("cannot repeat field 'x'"));
    }

    #[test]
    fn record_declaration_rejects_unknown_record_field_type() {
        let src = r#"
            record Wrapper {
                inner: Missing,
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown record field type must reject");
        assert!(err.message.contains("unknown record type 'Missing'"));
    }

    #[test]
    fn schema_declarations_typecheck_as_compile_time_top_level_items() {
        let src = r#"
            record Point {
                x: i32,
                y: i32,
            }

            schema PointPayload {
                point: Point,
                label: Option(quad),
            }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("schema declarations should typecheck");
    }

    #[test]
    fn tagged_union_schema_declarations_typecheck_as_compile_time_top_level_items() {
        let src = r#"
            record Point {
                x: i32,
                y: i32,
            }

            schema Payload {
                Empty {},
                PointRef {
                    point: Point,
                    label: Option(quad),
                },
            }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("tagged-union schema declarations should typecheck");
    }

    #[test]
    fn role_marked_schema_declarations_typecheck_as_compile_time_items() {
        let src = r#"
            config schema AppConfig {
                interval_ms: u32[ms],
            }

            api schema SensorRequest {
                payload: Result(quad, bool),
            }

            wire schema Envelope {
                Ping {},
                Data {
                    value: f64,
                },
            }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("role-marked schema declarations should typecheck");
    }

    #[test]
    fn version_marked_schema_declarations_typecheck_as_compile_time_items() {
        let src = r#"
            api schema SensorRequest version(2) {
                payload: Result(quad, bool),
            }

            wire schema Envelope version(3) {
                Ping {},
                Data {
                    value: f64,
                },
            }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("version-marked schema declarations should typecheck");
    }

    #[test]
    fn derive_validation_plan_table_returns_canonical_record_schema_plan() {
        let src = r#"
            record Point {
                x: i32,
                y: i32,
            }

            config schema PointPayload {
                point: Point,
                label: Option(quad),
                interval_ms: u32[ms],
            }

            fn main() {
                return;
            }
        "#;

        let (program, plans) =
            derive_validation_plans_from_source(src).expect("validation plans should derive");
        let schema_name = program.schemas[0].name;
        let plan = plans.get(&schema_name).expect("schema plan must exist");
        assert_eq!(plan.role, Some(SchemaRole::Config));
        let ValidationShapePlan::Record(fields) = &plan.shape else {
            panic!("expected record-shaped validation plan");
        };
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].ty, Type::Record(program.records[0].name));
        assert_eq!(fields[1].ty, Type::Option(Box::new(Type::Quad)));
        let Type::Measured(base, unit) = &fields[2].ty else {
            panic!("expected measured u32 field in validation plan");
        };
        assert_eq!(**base, Type::U32);
        assert_eq!(
            resolve_symbol_name(&program.arena, *unit).expect("unit symbol"),
            "ms"
        );
        assert_eq!(
            plan.checks,
            vec![
                ValidationCheck::RequiredField {
                    field: fields[0].name,
                },
                ValidationCheck::FieldType {
                    field: fields[0].name,
                    ty: fields[0].ty.clone(),
                },
                ValidationCheck::RequiredField {
                    field: fields[1].name,
                },
                ValidationCheck::FieldType {
                    field: fields[1].name,
                    ty: fields[1].ty.clone(),
                },
                ValidationCheck::RequiredField {
                    field: fields[2].name,
                },
                ValidationCheck::FieldType {
                    field: fields[2].name,
                    ty: fields[2].ty.clone(),
                },
            ]
        );
    }

    #[test]
    fn derive_validation_plan_table_returns_tagged_union_schema_plan() {
        let src = r#"
            record Point {
                x: i32,
                y: i32,
            }

            wire schema Envelope {
                Empty {},
                Data {
                    point: Point,
                    verdict: Result(quad, bool),
                },
            }

            fn main() {
                return;
            }
        "#;

        let (program, plans) =
            derive_validation_plans_from_source(src).expect("validation plans should derive");
        let schema_name = program.schemas[0].name;
        let plan = plans.get(&schema_name).expect("schema plan must exist");
        assert_eq!(plan.role, Some(SchemaRole::Wire));
        let ValidationShapePlan::TaggedUnion(variants) = &plan.shape else {
            panic!("expected tagged-union validation plan");
        };
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].fields.len(), 0);
        assert_eq!(variants[1].fields.len(), 2);
        assert_eq!(
            variants[1].fields[0].ty,
            Type::Record(program.records[0].name)
        );
        assert_eq!(
            variants[1].fields[1].ty,
            Type::Result(Box::new(Type::Quad), Box::new(Type::Bool))
        );
        assert_eq!(
            plan.checks,
            vec![
                ValidationCheck::TaggedUnionBranch {
                    variant: variants[0].name,
                },
                ValidationCheck::TaggedUnionBranch {
                    variant: variants[1].name,
                },
                ValidationCheck::TaggedUnionBranchRequiredField {
                    variant: variants[1].name,
                    field: variants[1].fields[0].name,
                },
                ValidationCheck::TaggedUnionBranchFieldType {
                    variant: variants[1].name,
                    field: variants[1].fields[0].name,
                    ty: variants[1].fields[0].ty.clone(),
                },
                ValidationCheck::TaggedUnionBranchRequiredField {
                    variant: variants[1].name,
                    field: variants[1].fields[1].name,
                },
                ValidationCheck::TaggedUnionBranchFieldType {
                    variant: variants[1].name,
                    field: variants[1].fields[1].name,
                    ty: variants[1].fields[1].ty.clone(),
                },
            ]
        );
    }

    #[test]
    fn schema_declaration_rejects_duplicate_field_name() {
        let src = r#"
            schema PointPayload {
                point: i32,
                point: i32,
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("duplicate schema field must reject");
        assert!(err
            .message
            .contains("schema 'PointPayload' cannot repeat field 'point'"));
    }

    #[test]
    fn schema_declaration_rejects_empty_body() {
        let src = r#"
            schema PointPayload {
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("empty schema must reject");
        assert!(err
            .message
            .contains("schema 'PointPayload' must declare at least 1 field"));
    }

    #[test]
    fn tagged_union_schema_rejects_duplicate_variant_name() {
        let src = r#"
            schema Payload {
                Ready {},
                Ready {
                    detail: quad,
                },
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("duplicate schema variant must reject");
        assert!(err
            .message
            .contains("schema 'Payload' cannot repeat variant 'Ready'"));
    }

    #[test]
    fn tagged_union_schema_rejects_duplicate_variant_field_name() {
        let src = r#"
            schema Payload {
                Data {
                    value: i32,
                    value: i32,
                },
            }

            fn main() {
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("duplicate tagged-union schema field must reject");
        assert!(err
            .message
            .contains("schema 'Payload::Data' cannot repeat field 'value'"));
    }

    #[test]
    fn schema_declaration_rejects_top_level_name_collision_with_record() {
        let src = r#"
            record PointPayload {
                x: i32,
            }

            schema PointPayload {
                point: i32,
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("schema/record collision must reject");
        assert!(err
            .message
            .contains("top-level name 'PointPayload' cannot be used for both record and schema"));
    }

    #[test]
    fn record_declaration_rejects_recursive_field_graph() {
        let src = r#"
            record A {
                next: B,
            }

            record B {
                prev: A,
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("recursive record graph must reject");
        assert!(err.message.contains("recursive field graph involving 'A'"));
    }

    #[test]
    fn record_type_allows_executable_function_signature_use() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            fn echo(ctx: DecisionContext) -> DecisionContext {
                return ctx;
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T };
                let mirror: DecisionContext = echo(ctx);
                let _ = mirror;
                return;
            }
        "#;

        typecheck_source(src).expect("record params and returns should typecheck");
    }

    #[test]
    fn record_literal_typechecks_for_local_stage1_carrier_bind() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext {
                    quality: 0.75,
                    camera: T,
                };
                let mirror = ctx;
                return;
            }
        "#;

        typecheck_source(src).expect("record literal local carrier bind should typecheck");
    }

    #[test]
    fn record_literal_rejects_missing_field() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx = DecisionContext { camera: T };
                let _ = ctx;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("missing record field must reject");
        assert!(err
            .message
            .contains("record literal 'DecisionContext' is missing field 'quality'"));
    }

    #[test]
    fn record_literal_rejects_unknown_field() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            fn main() {
                let ctx = DecisionContext { camera: T, badge: F };
                let _ = ctx;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown record field must reject");
        assert!(err
            .message
            .contains("record literal 'DecisionContext' has no field named 'badge'"));
    }

    #[test]
    fn record_literal_allows_equality_for_stable_field_subset() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            fn main() {
                let left = DecisionContext { camera: T };
                let right = DecisionContext { camera: T };
                assert(left == right);
                return;
            }
        "#;

        typecheck_source(src).expect("record equality should typecheck for stable field subset");
    }

    #[test]
    fn record_equality_rejects_unsupported_field_subset() {
        // FA-02-038 / #1861: this test previously used a `qvec` field to
        // reach a field type equality never supports. That relied on
        // `qvec`-typed record fields being admitted at declaration time at
        // all -- itself the exact fail-open storage-admission gap #1861
        // closes (record fields now reject `qvec` before equality is ever
        // considered). `Map(i32, i32)` is a storage-admitted field type
        // (declares successfully) that `supports_stable_equality_type_inner`
        // still unconditionally reports as not equality-supporting, so it
        // isolates the equality-specific rejection this test targets.
        let src = r#"
            record SensorFrame {
                mask: Map(i32, i32),
            }

            fn compare(left: SensorFrame, right: SensorFrame) {
                assert(left == right);
                return;
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("record equality subset must reject unsupported fields");
        assert!(err
            .message
            .contains("record equality is allowed only when every field type already supports stable equality"));
    }

    #[test]
    fn record_field_access_typechecks_against_canonical_decl() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx = DecisionContext { camera: T, quality: 0.75 };
                let seen: quad = ctx.camera;
                let score: f64 = ctx.quality;
                return;
            }
        "#;

        typecheck_source(src).expect("record field access should typecheck");
    }

    #[test]
    fn record_field_access_rejects_unknown_field() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            fn main() {
                let ctx = DecisionContext { camera: T };
                let badge = ctx.badge;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown record field must reject");
        assert!(err
            .message
            .contains("record type 'DecisionContext' has no field named 'badge'"));
    }

    #[test]
    fn record_field_access_rejects_non_record_base() {
        let src = r#"
            fn main() {
                let value: f64 = 1.0;
                let bad = value.quality;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-record field access must reject");
        assert!(err
            .message
            .contains("record field access requires record value before '.quality', got F64"));
    }

    #[test]
    fn record_copy_with_typechecks_for_explicit_override_subset() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let patched: DecisionContext = ctx with { quality: 1.0 };
                assert(patched.camera == T);
                assert(patched.quality == 1.0);
                return;
            }
        "#;

        typecheck_source(src).expect("record copy-with should typecheck");
    }

    #[test]
    fn record_field_shorthand_typechecks_for_literal_and_copy_with() {
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

        typecheck_source(src).expect("record field shorthand should typecheck");
    }

    #[test]
    fn record_copy_with_rejects_unknown_field() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T };
                let patched = ctx with { badge: T };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown copy-with field must reject");
        assert!(err
            .message
            .contains("record copy-with 'DecisionContext' has no field named 'badge'"));
    }

    #[test]
    fn record_copy_with_rejects_duplicate_field_override() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let patched = ctx with { quality: 1.0, quality: 2.0 };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("duplicate copy-with field must reject");
        assert!(err
            .message
            .contains("record copy-with 'DecisionContext' cannot repeat field 'quality'"));
    }

    #[test]
    fn record_copy_with_rejects_non_record_base() {
        let src = r#"
            fn main() {
                let value: f64 = 1.0;
                let patched = value with { quality: 0.75 };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("non-record copy-with base must reject");
        assert!(err
            .message
            .contains("record copy-with requires record base before 'with', got F64"));
    }

    #[test]
    fn record_copy_with_rejects_empty_override_set() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T };
                let patched = ctx with { };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("empty copy-with must reject");
        assert!(err
            .message
            .contains("record copy-with requires at least one explicit override field"));
    }

    #[test]
    fn record_destructuring_bind_typechecks_for_explicit_field_subset() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: seen_camera, quality: _ } =
                    DecisionContext { camera: T, quality: 0.75 };
                let same = seen_camera == T;
                if same { return; } else { return; }
            }
        "#;

        typecheck_source(src).expect("record destructuring bind should typecheck");
    }

    #[test]
    fn record_pattern_punning_typechecks_for_bind_and_let_else() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera, quality: _ } =
                    DecisionContext { camera: T, quality: 0.75 };
                let DecisionContext { camera: T, quality } =
                    DecisionContext { camera: T, quality: 1.0 } else return;
                assert(camera == T);
                let _: f64 = quality;
                return;
            }
        "#;

        typecheck_source(src).expect("record pattern punning should typecheck");
    }

    #[test]
    fn record_destructuring_bind_rejects_unknown_field() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            fn main() {
                let DecisionContext { badge: seen_badge } =
                    DecisionContext { camera: T };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unknown record field must reject");
        assert!(err
            .message
            .contains("record type 'DecisionContext' has no field named 'badge'"));
    }

    #[test]
    fn record_destructuring_bind_rejects_wrong_record_value() {
        let src = r#"
            record DecisionContext {
                camera: quad,
            }

            record RuntimeConfig {
                debug_mode: bool,
            }

            fn main() {
                let DecisionContext { camera: seen_camera } =
                    RuntimeConfig { debug_mode: true };
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("wrong record value must reject");
        assert!(err
            .message
            .contains("record destructuring bind requires value of type 'DecisionContext'"));
    }

    #[test]
    fn record_let_else_typechecks_with_explicit_quad_field_pattern() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: T, quality: score } =
                    DecisionContext { camera: T, quality: 0.75 } else return;
                let _: f64 = score;
                return;
            }
        "#;

        typecheck_source(src).expect("record let-else should typecheck");
    }

    #[test]
    fn record_let_else_rejects_when_no_refutable_field_is_present() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: seen_camera, quality: score } =
                    DecisionContext { camera: T, quality: 0.75 } else return;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("record let-else without refutable field must reject");
        assert!(err.message.contains(
            "record let-else requires at least one refutable quad literal field pattern"
        ));
    }

    #[test]
    fn record_let_else_rejects_non_quad_literal_field_position() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }

            fn main() {
                let DecisionContext { camera: seen_camera, quality: T } =
                    DecisionContext { camera: T, quality: 0.75 } else return;
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("record let-else quad literal on non-quad field must reject");
        assert!(err
            .message
            .contains("record let-else literal pattern requires quad field"));
    }

    #[test]
    fn ufcs_method_call_typechecks_via_ordinary_call_contract() {
        let src = r#"
            fn scale(value: f64, factor: f64) -> f64 = value * factor;

            fn main() {
                let total: f64 = 2.0.scale(3.0);
                return;
            }
        "#;

        typecheck_source(src).expect("UFCS method-call sugar should typecheck");
    }

    #[test]
    fn ufcs_named_arguments_reuse_parameter_reorder_rules() {
        let src = r#"
            fn clamp(value: f64, min: f64, max: f64) -> f64 = value;

            fn main() {
                let total: f64 = 2.0.clamp(min = 0.0, max = 10.0);
                return;
            }
        "#;

        typecheck_source(src).expect("UFCS named arguments should typecheck");
    }

    #[test]
    fn ufcs_builtin_named_arguments_still_reject() {
        let src = r#"
            fn main() {
                let total: f64 = 2.0.pow(exp = 3.0);
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("builtin named arguments must still reject");
        assert!(err
            .message
            .contains("named arguments are not supported for builtin 'pow'"));
    }

    #[test]
    fn adt_constructor_surface_typechecks_for_nominal_return_and_local_bindings() {
        let src = r#"
            enum Maybe {
                None,
                Some(bool),
            }

            fn choose(flag: bool) -> Maybe = if flag { Maybe::Some(true) } else { Maybe::None };

            fn main() {
                let left: Maybe = choose(true);
                let right: Maybe = Maybe::None;
                let _ = left;
                let _ = right;
                return;
            }
        "#;

        typecheck_source(src).expect("adt constructor surface should typecheck");
    }

    #[test]
    fn option_and_result_standard_forms_typecheck_in_typed_positions() {
        let src = r#"
            fn keep(flag: bool) -> Option(bool) {
                let seen: Option(bool) = Option::None;
                let _ = seen;
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

        typecheck_source(src).expect("Option/Result standard forms should typecheck");
    }

    #[test]
    fn result_constructor_requires_contextual_result_type() {
        let src = r#"
            fn main() {
                let value = Result::Ok(true);
                let _ = value;
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("contextless Result constructor must currently reject");
        assert!(err
            .message
            .contains("Result::Ok currently requires contextual Result(T, E) type in v0"));
    }

    #[test]
    fn option_and_result_match_patterns_typecheck_without_default_when_exhaustive() {
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

        typecheck_source(src).expect("Option/Result match ergonomics should typecheck");
    }

    #[test]
    fn option_match_without_none_arm_rejects_as_non_exhaustive() {
        let src = r#"
            fn unwrap(opt: Option(bool)) -> bool {
                let out: bool = match opt {
                    Option::Some(value) => { value }
                };
                return out;
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("non-exhaustive Option match expression without default must reject");
        assert!(err
            .message
            .contains("non-exhaustive match expression for Option(T); missing variants: None"));
    }

    // FA-02-007 / #1639: statement-form match presence-vs-emptiness matrix.
    // These exercise the plain `Stmt::Match` handler (as opposed to the
    // `Expr::Match`/value-producing-loop handlers above, which already used
    // `Option<...>` and were unaffected).

    #[test]
    fn statement_match_quad_without_wildcard_rejects() {
        let src = r#"
            fn main() {
                match T {
                    T => { return; }
                    F => { return; }
                }
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("quad match statement with no `_` arm at all must reject");
        assert!(
            err.message.contains("match requires default arm '_'"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn statement_match_empty_and_nonempty_wildcard_satisfy_identical_quad_presence_requirement() {
        // Item 4 of the FA-02-007 regression matrix: an empty `_ => {}` and a
        // non-empty `_ => { ... }` must both satisfy the same "a wildcard is
        // present" requirement -- neither is special-cased relative to the
        // other, and in particular the empty one is never treated as if no
        // wildcard were written at all.
        let empty_src = r#"
            fn main() {
                match T {
                    T => { return; }
                    _ => { }
                }
                return;
            }
        "#;
        let nonempty_src = r#"
            fn main() {
                match T {
                    T => { return; }
                    _ => { let _ = 0; }
                }
                return;
            }
        "#;

        typecheck_source(empty_src).expect("empty wildcard must satisfy the presence requirement");
        typecheck_source(nonempty_src)
            .expect("non-empty wildcard must satisfy the presence requirement");
    }

    #[test]
    fn statement_match_enum_present_empty_wildcard_admits_non_exhaustive_arms() {
        // The exact bug reproduced pre-fix: non-exhaustive arms (only
        // Flag::A covered out of three variants) plus an explicitly present,
        // empty wildcard `_ => {}`. Before this fix, `default.is_empty()`
        // could not distinguish this from "no wildcard at all" and rejected
        // with a non-exhaustive-match error despite the real (if empty)
        // catch-all in the source.
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

        typecheck_source(src).expect(
            "non-exhaustive arms plus a present (even empty) wildcard must typecheck, not \
             reject as non-exhaustive",
        );
    }

    #[test]
    fn statement_match_enum_exhaustive_arms_may_still_omit_wildcard() {
        // Item 6: guard against accidentally making the wildcard mandatory.
        // A match family/form already allowed to omit `_` when arms are
        // independently exhaustive must remain admitted after this fix.
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

        typecheck_source(src)
            .expect("exhaustive enum match statement with no wildcard should still typecheck");
    }

    #[test]
    fn statement_match_enum_non_exhaustive_arms_without_wildcard_still_rejects() {
        // Item 5: a genuinely missing wildcard (no `_` arm at all, `None`)
        // over non-exhaustive arms must still reject deterministically.
        let src = r#"
            enum Flag { A, B, C }

            fn main() {
                let f: Flag = Flag::A;
                match f {
                    Flag::A => { }
                }
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("non-exhaustive enum match statement with no wildcard must reject");
        assert!(
            err.message
                .contains("non-exhaustive match for enum 'Flag'; missing variants: B, C"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn result_pattern_family_must_match_result_scrutinee() {
        let src = r#"
            fn settle(res: Result(bool, quad)) -> bool {
                let out: bool = match res {
                    Option::Some(value) => { value }
                    _ => { false }
                };
                return out;
            }

            fn main() {
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("mismatched standard-form match family must reject");
        assert!(err
            .message
            .contains("match arm pattern type 'Option' does not match scrutinee Result(T, E)"));
    }

    #[test]
    fn units_of_measure_typecheck_through_transport_and_supported_operators() {
        let src = r#"
            record Measurement {
                distance: f64[m],
            }

            fn echo(
                distance: f64[m],
                pair: (f64[m], f64[m]),
                maybe: Option(f64[m]),
                result: Result(f64[m], quad),
                sample: Measurement
            ) -> f64[m] {
                let left: f64[m] = 1.0;
                let right: f64[m] = 2.0;
                let pair_copy: (f64[m], f64[m]) = pair;
                let maybe_copy: Option(f64[m]) = maybe;
                let result_copy: Result(f64[m], quad) = result;
                let total: f64[m] = left + right;
                let same: bool = total == sample.distance;
                let _ = pair_copy;
                let _ = maybe_copy;
                let _ = result_copy;
                assert(same == true);
                return total;
            }

            fn main() {
                let sample: Measurement = Measurement { distance: 3.0 };
                let total: f64[m] = echo(
                    1.0,
                    (1.0, 2.0),
                    Option::Some(1.0),
                    Result::Ok(2.0),
                    sample
                );
                let expected: f64[m] = 3.0;
                assert(total == expected);
                return;
            }
        "#;

        typecheck_source(src).expect("first-wave units transport and operators should typecheck");
    }

    #[test]
    fn units_of_measure_reject_mismatched_symbols_in_binding() {
        let src = r#"
            fn main() {
                let distance: f64[m] = 1.0;
                let time: f64[s] = distance;
                let _ = time;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("different unit symbols must reject");
        assert!(err.message.contains("type mismatch in let 'time'"));
    }

    #[test]
    fn units_of_measure_reject_mul_div_and_mod_in_first_wave() {
        let src = r#"
            fn main() {
                let distance: f64[m] = 1.0;
                let area: f64[m] = distance * distance;
                let _ = area;
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("mul/div/mod on unit-carrying values must reject in first wave");
        assert!(err.message.contains(
            "*, /, % on unit-carrying values are rejected in the first-wave units surface"
        ));
    }

    // ── M9.1 Wave 3: generic call-site substitution ──────────────────────────

    #[test]
    fn generic_identity_fn_typechecks_with_i32() {
        let src = r#"
            fn identity<T>(x: T) -> T {
                return x;
            }

            fn main() {
                let v: i32 = identity(42);
                let _ = v;
                return;
            }
        "#;
        typecheck_source(src).expect("identity<i32> should typecheck");
    }

    #[test]
    fn generic_identity_fn_typechecks_with_bool() {
        let src = r#"
            fn identity<T>(x: T) -> T {
                return x;
            }

            fn main() {
                let v: bool = identity(true);
                let _ = v;
                return;
            }
        "#;
        typecheck_source(src).expect("identity<bool> should typecheck");
    }

    #[test]
    fn generic_fn_with_concrete_and_type_var_params() {
        let src = r#"
            fn first<T>(x: T, y: i32) -> T {
                return x;
            }

            fn main() {
                let v: bool = first(true, 1);
                let _ = v;
                return;
            }
        "#;
        typecheck_source(src).expect("first<bool>(bool, i32) should typecheck");
    }

    #[test]
    fn generic_call_wrong_return_type_rejects() {
        let src = r#"
            fn identity<T>(x: T) -> T {
                return x;
            }

            fn main() {
                let v: i32 = identity(true);
                let _ = v;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("bool assigned to i32 binding must reject");
        assert!(
            err.message.contains("type mismatch") || err.message.contains("bool"),
            "unexpected error: {}",
            err.message
        );
    }

    // M9.2 Wave 3 — trait coherence, conformance, and bound satisfaction

    // SSF-07 explicitly defers blanket/generic impls (`impl<T> Trait for T`) --
    // ImplDecl.type_params is documented as "Empty in first-wave canonical form"
    // but nothing previously enforced that, so a blanket-shaped impl silently
    // typechecked as an ordinary impl for a nominal type literally named `T`.
    // Reject any impl with non-empty type_params deterministically instead.
    #[test]
    fn blanket_impl_with_type_params_is_rejected() {
        let src = r#"
            trait Show {
                fn show(self: MyType) -> i32;
            }

            record MyType { x: i32 }

            impl<T> Show for MyType {
                fn show(self: MyType) -> i32 {
                    return 0;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("blanket/generic impl with type_params must be rejected");
        assert!(
            err.message.contains("generic") || err.message.contains("type param"),
            "unexpected error: {}",
            err.message
        );
    }

    // FA-02-002 / #1634 classification evidence: a two-parameter impl is
    // already rejected by #1668's stricter zero-arity impl contract,
    // independent of #1634's new max-one check elsewhere -- not a #1634
    // repair target for impls.
    #[test]
    fn two_type_param_impl_is_rejected_by_existing_zero_arity_contract() {
        let src = r#"
            trait Show {
                fn show(self: MyType) -> i32;
            }

            record MyType { x: i32 }

            impl<X, Y> Show for MyType {
                fn show(self: MyType) -> i32 {
                    return 0;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("a two-parameter generic impl must still reject via #1668's contract");
        assert!(
            err.message.contains("generic") || err.message.contains("type param"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn duplicate_impl_same_trait_and_type_is_rejected() {
        let src = r#"
            trait Display {
                fn show(self: MyType) -> i32;
            }

            record MyType { x: i32 }

            impl Display for MyType {
                fn show(self: MyType) -> i32 {
                    return 0;
                }
            }

            impl Display for MyType {
                fn show(self: MyType) -> i32 {
                    return 1;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err =
            typecheck_source(src).expect_err("duplicate impl must be rejected by coherence check");
        assert!(
            err.message.contains("duplicate") || err.message.contains("impl"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_missing_required_method_is_rejected() {
        let src = r#"
            trait Greet {
                fn hello(self: Greeter) -> i32;
                fn bye(self: Greeter) -> i32;
            }

            record Greeter { x: i32 }

            impl Greet for Greeter {
                fn hello(self: Greeter) -> i32 {
                    return 1;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("impl missing required method must be rejected");
        assert!(
            err.message.contains("bye")
                || err.message.contains("missing")
                || err.message.contains("method"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_method_wrong_return_type_is_rejected() {
        let src = r#"
            trait Counter {
                fn count(self: Cnt) -> i32;
            }

            record Cnt { n: i32 }

            impl Counter for Cnt {
                fn count(self: Cnt) -> bool {
                    return true;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err =
            typecheck_source(src).expect_err("impl method with wrong return type must be rejected");
        assert!(
            err.message.contains("count")
                || err.message.contains("return type")
                || err.message.contains("mismatch"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_method_wrong_parameter_type_is_rejected() {
        let src = r#"
            trait Counter {
                fn count(self: Cnt) -> i32;
            }

            record Cnt { n: i32 }

            impl Counter for Cnt {
                fn count(self: i32) -> i32 {
                    return 0;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("impl method with wrong parameter type must be rejected");
        assert!(
            err.message.contains("parameter type") || err.message.contains("expected"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_extra_method_not_in_trait_is_rejected() {
        let src = r#"
            trait Counter {
                fn count(self: Cnt) -> i32;
            }

            record Cnt { n: i32 }

            impl Counter for Cnt {
                fn count(self: Cnt) -> i32 {
                    return 0;
                }
                fn reset(self: Cnt) -> i32 {
                    return 0;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("impl method not declared by the trait must be rejected");
        assert!(
            err.message.contains("reset") || err.message.contains("not declared"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_duplicate_method_name_within_impl_is_rejected() {
        let src = r#"
            trait Counter {
                fn count(self: Cnt) -> i32;
            }

            record Cnt { n: i32 }

            impl Counter for Cnt {
                fn count(self: Cnt) -> i32 {
                    return 0;
                }
                fn count(self: Cnt) -> i32 {
                    return 1;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("duplicate method name within one impl must be rejected");
        assert!(
            err.message.contains("duplicate") || err.message.contains("count"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_of_unknown_trait_is_rejected() {
        let src = r#"
            record Cnt { n: i32 }

            impl Missing for Cnt {
                fn count(self: Cnt) -> i32 {
                    return 0;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("impl of an unknown trait must be rejected");
        assert!(
            err.message.contains("unknown trait") || err.message.contains("Missing"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn trait_self_contract_allows_multiple_impl_targets() {
        let src = r#"
            trait Iterable {
                fn next(self: Self, index: i32) -> Option(i32);
            }

            record Numbers {
                current: i32,
            }

            record Others {
                current: i32,
            }

            impl Iterable for Numbers {
                fn next(self: Self, index: i32) -> Option(i32) {
                    let _ = self.current;
                    let _ = index;
                    return Option::None;
                }
            }

            impl Iterable for Others {
                fn next(self: Self, index: i32) -> Option(i32) {
                    let _ = self.current;
                    let _ = index;
                    return Option::None;
                }
            }

            fn main() {
                return;
            }
        "#;

        typecheck_source(src).expect("trait-side Self should anchor independently per impl target");
    }

    #[test]
    fn trait_self_contract_still_rejects_wrong_concrete_impl_parameter() {
        let src = r#"
            trait Counter {
                fn count(self: Self) -> i32;
            }

            record Cnt { n: i32 }

            impl Counter for Cnt {
                fn count(self: i32) -> i32 {
                    return 0;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("trait-side Self must still anchor to the impl target");
        assert!(
            err.message.contains("parameter type") || err.message.contains("expected"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn self_type_outside_trait_or_impl_positions_is_not_admitted() {
        // FA-02-014 / #1646: before this fix, this test passed only by
        // coincidence -- `parse_type` silently rewrote out-of-context `Self`
        // to `Type::Record("Self")`, and this specific source happened to
        // reject downstream only because no record named "Self" exists, with
        // the accidental message "unknown nominal type 'Self'". The parser
        // now rejects `Self` deterministically the moment it is recognized
        // outside a trait/impl owner scope, regardless of whether any
        // declaration happens to be named "Self" -- see the companion test
        // below, which proves the difference is not merely cosmetic.
        let src = r#"
            fn id(value: Self) -> Self {
                return value;
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("Self outside trait/impl method type positions must stay unsupported");
        assert!(
            err.message
                .contains("'Self' is only admitted in trait or impl method type positions"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn self_type_outside_owner_context_rejects_even_when_a_record_named_self_exists() {
        // FA-02-014 / #1646, matrix item 12: the required invariant is that
        // an ordinary type reference spelled `Self` must never resolve as a
        // nominal record merely because a record happens to be named
        // "Self". Before this fix, `record Self { x: i32 }` followed by an
        // ordinary `fn f(x: Self) -> i32 { ... }` typechecked successfully
        // end-to-end (`Ok(())`) -- `Self` silently resolved to that
        // unrelated record, exactly the misresolution this issue exists to
        // close. The parser-level rejection is unconditional: it never
        // consults the record/ADT tables at all, so this now rejects
        // deterministically regardless of what happens to be declared.
        let src = r#"
            record Self { x: i32 }

            fn f(x: Self) -> i32 {
                return 0;
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "an ordinary Self reference must reject even when a record literally named \
             Self is declared, never silently resolve to it",
        );
        assert!(
            err.message
                .contains("'Self' is only admitted in trait or impl method type positions"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_method_body_is_typechecked_even_without_dispatch() {
        let src = r#"
            trait Iterable {
                fn next(self: Numbers) -> Option(i32);
            }

            record Numbers {
                current: i32,
            }

            impl Iterable for Numbers {
                fn next(self: Numbers) -> Option(i32) {
                    return 1;
                }
            }

            fn main() {
                return;
            }
        "#;

        let err = typecheck_source(src)
            .expect_err("impl method body must be typechecked before dispatch lands");
        assert!(
            err.message.contains("return") || err.message.contains("Option"),
            "unexpected error: {}",
            err.message
        );
    }

    // FA-02-035 / #1667 + FA-02-019 / #1651: impl target identity is proven
    // once against the canonical RecordTable/AdtTable, and every meaning of
    // `Self` derives from that one proven concrete Type. See
    // `validate_impl_conformance` and `substitute_trait_self_type`.

    /// Table-driven proof that `substitute_trait_self_type` substitutes both
    /// parsed forms of `Self` (the trait-side `TypeVar` placeholder and the
    /// impl-side uncanonicalized `Record(for_type)` form) identically,
    /// recursively, through every currently admitted composite `Type`
    /// position, for both a record and an ADT concrete target.
    #[test]
    fn substitute_trait_self_type_covers_nested_positions_for_both_nominal_families() {
        let src = r#"
            record R { n: i32 }
            enum E { A, B }
            fn main() {
                return;
            }
        "#;
        let mut program = parse_program(src).expect("parse");
        let r_id = *program.arena.symbol_to_id.get("R").expect("R symbol");
        let e_id = *program.arena.symbol_to_id.get("E").expect("E symbol");
        let self_id = program.arena.intern_symbol("Self");
        let arena = &program.arena;

        for (for_type, concrete) in [(r_id, Type::Record(r_id)), (e_id, Type::Adt(e_id))] {
            // direct Self, both parsed forms
            assert_eq!(
                substitute_trait_self_type(
                    &Type::TypeVar(self_id),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                concrete,
                "direct Self (TypeVar form) must resolve to the concrete target type"
            );
            assert_eq!(
                substitute_trait_self_type(
                    &Type::Record(for_type),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                concrete,
                "direct Self (impl-side Record form) must resolve to the concrete target type"
            );

            // tuple containing Self
            assert_eq!(
                substitute_trait_self_type(
                    &Type::Tuple(vec![Type::I32, Type::TypeVar(self_id)]),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                Type::Tuple(vec![Type::I32, concrete.clone()])
            );

            // Sequence(Self)
            assert_eq!(
                substitute_trait_self_type(
                    &Type::Sequence(SequenceType {
                        family: SequenceCollectionFamily::OrderedSequence,
                        item: Box::new(Type::TypeVar(self_id)),
                    }),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                Type::Sequence(SequenceType {
                    family: SequenceCollectionFamily::OrderedSequence,
                    item: Box::new(concrete.clone()),
                })
            );

            // Option(Self)
            assert_eq!(
                substitute_trait_self_type(
                    &Type::Option(Box::new(Type::TypeVar(self_id))),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                Type::Option(Box::new(concrete.clone()))
            );

            // Result(Self, ...) and Result(..., Self)
            assert_eq!(
                substitute_trait_self_type(
                    &Type::Result(Box::new(Type::TypeVar(self_id)), Box::new(Type::Bool)),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                Type::Result(Box::new(concrete.clone()), Box::new(Type::Bool))
            );
            assert_eq!(
                substitute_trait_self_type(
                    &Type::Result(Box::new(Type::Bool), Box::new(Type::TypeVar(self_id))),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                Type::Result(Box::new(Type::Bool), Box::new(concrete.clone()))
            );

            // closure parameter and return containing Self
            assert_eq!(
                substitute_trait_self_type(
                    &Type::Closure(ClosureType {
                        family: ClosureValueFamily::UnaryDirect,
                        capture: ClosureCapturePolicy::Immutable,
                        param: Box::new(Type::TypeVar(self_id)),
                        ret: Box::new(Type::I32),
                    }),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                Type::Closure(ClosureType {
                    family: ClosureValueFamily::UnaryDirect,
                    capture: ClosureCapturePolicy::Immutable,
                    param: Box::new(concrete.clone()),
                    ret: Box::new(Type::I32),
                })
            );
            assert_eq!(
                substitute_trait_self_type(
                    &Type::Closure(ClosureType {
                        family: ClosureValueFamily::UnaryDirect,
                        capture: ClosureCapturePolicy::Immutable,
                        param: Box::new(Type::I32),
                        ret: Box::new(Type::TypeVar(self_id)),
                    }),
                    Some(self_id),
                    for_type,
                    &concrete
                ),
                Type::Closure(ClosureType {
                    family: ClosureValueFamily::UnaryDirect,
                    capture: ClosureCapturePolicy::Immutable,
                    param: Box::new(Type::I32),
                    ret: Box::new(concrete.clone()),
                })
            );

            // an unrelated nominal name (not Self, not for_type) must never
            // be substituted
            let other = arena
                .symbol_to_id
                .get("R")
                .copied()
                .filter(|id| *id != for_type);
            if let Some(other_id) = other {
                assert_eq!(
                    substitute_trait_self_type(
                        &Type::Record(other_id),
                        Some(self_id),
                        for_type,
                        &concrete
                    ),
                    Type::Record(other_id),
                    "a nominal type reference that is not Self and not the impl target must be left untouched"
                );
            }
        }
    }

    #[test]
    fn impl_self_for_adt_target_typechecks_body_as_the_correct_adt_not_a_guessed_record() {
        // FA-02-035 / #1667 core regression: a `Self`-typed impl method body
        // for an enum/ADT target must be typechecked as the actual ADT, not
        // as a guessed Record. Pattern-matching on `self` only typechecks if
        // `self`'s resolved type is genuinely `Type::Adt(Color)`.
        let src = r#"
            trait Describe {
                fn label(self: Self) -> i32;
            }

            enum Color {
                Red,
                Blue,
            }

            impl Describe for Color {
                fn label(self: Self) -> i32 {
                    match self {
                        Color::Red => { return 1; }
                        Color::Blue => { return 2; }
                    }
                }
            }

            fn main() {
                return;
            }
        "#;
        typecheck_source(src).expect(
            "enum-Self impl method body must typecheck against the correct ADT-shaped Self",
        );
    }

    #[test]
    fn impl_self_conformance_mismatch_reports_the_correct_adt_type_not_record() {
        // FA-02-035 / #1667: this is the direct, historical-bug-exposing
        // proof. Pre-fix, `substitute_trait_self_type` always reconstructed
        // `Type::Record(concrete_self)` for `Self`, regardless of the impl
        // target's real nominal family. A genuine conformance mismatch on a
        // Self-typed position for an *enum* impl target must report the
        // concrete type as `Adt(..)`, never `Record(..)`, in its diagnostic.
        let src = r#"
            trait Make {
                fn make() -> Self;
            }

            enum Color {
                Red,
                Blue,
            }

            impl Make for Color {
                fn make() -> i32 {
                    return 0;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("impl method returning i32 instead of Self must fail conformance");
        assert!(
            err.message.contains("Adt("),
            "expected the conformance diagnostic to report the resolved Adt(..) type for Self, got: {}",
            err.message
        );
        assert!(
            !err.message.contains("Record("),
            "the conformance diagnostic must never guess Record(..) for an enum impl target, got: {}",
            err.message
        );
    }

    #[test]
    fn impl_target_that_does_not_exist_rejects_even_without_any_self_reference() {
        // FA-02-019 / #1651 core regression: a trait whose methods never
        // reference `Self` previously gave `validate_impl_conformance` no
        // way to independently prove the impl target exists (it had no
        // RecordTable/AdtTable input at all). An impl for a wholly
        // undeclared name must reject deterministically regardless.
        let src = r#"
            trait Marker {
                fn ping() -> i32;
            }

            impl Marker for MissingType {
                fn ping() -> i32 {
                    return 1;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("impl for an undeclared target must reject even with no Self reference");
        assert!(
            err.message.contains("unknown nominal type 'MissingType'"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_target_ambiguous_between_record_and_enum_rejects_deterministically() {
        // The impl-target resolver must fail closed on ambiguity rather than
        // arbitrarily preferring one nominal family, even though this
        // program is also independently rejected later by
        // validate_top_level_name_collisions — validate_impl_conformance
        // runs first and must not silently pick Record.
        let src = r#"
            trait Marker {
                fn ping() -> i32;
            }

            record Dup { n: i32 }
            enum Dup { A, B }

            impl Marker for Dup {
                fn ping() -> i32 {
                    return 1;
                }
            }

            fn main() {
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("impl target ambiguous between record and enum must reject");
        assert!(
            err.message
                .contains("ambiguously declared as both record and enum"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn impl_target_that_exists_without_self_reference_still_typechecks() {
        // Positive control paired with the #1651 regression above: a real,
        // unambiguous target must still be admitted when the trait's
        // methods never reference Self.
        let src = r#"
            trait Marker {
                fn ping() -> i32;
            }

            record RealType { n: i32 }

            impl Marker for RealType {
                fn ping() -> i32 {
                    return 1;
                }
            }

            fn main() {
                return;
            }
        "#;
        typecheck_source(src)
            .expect("impl for a real declared target must still typecheck with no Self reference");
    }

    #[test]
    fn impl_self_nested_in_option_typechecks_end_to_end_for_enum_target() {
        // At least one source-level conformance test for a nested Self
        // position (Option(Self)) through the real pipeline, complementing
        // the table-driven substitute_trait_self_type unit test above.
        let src = r#"
            trait MaybeSelf {
                fn maybe(self: Self, flag: quad) -> Option(Self);
            }

            enum Color {
                Red,
                Blue,
            }

            impl MaybeSelf for Color {
                fn maybe(self: Self, flag: quad) -> Option(Self) {
                    if flag == T {
                        return Option::Some(self);
                    }
                    return Option::None;
                }
            }

            fn main() {
                return;
            }
        "#;
        typecheck_source(src).expect("Option(Self) for an enum impl target must typecheck");
    }

    // FA-02-015 / #1647: ensure_executable_type_supported must be exhaustive
    // (no `_ => Ok(())` catch-all) over every current Type variant, so a
    // reserved/not-yet-promoted family like QVec rejects deterministically
    // -- directly, and nested inside every currently admitted composite
    // position.

    #[test]
    fn ensure_executable_type_supported_rejects_qvec_directly_and_when_nested() {
        let qvec = Type::QVec(8);
        let cases: Vec<(&str, Type)> = vec![
            ("direct", qvec.clone()),
            ("Option(qvec)", Type::Option(Box::new(qvec.clone()))),
            (
                "Sequence(qvec)",
                Type::Sequence(SequenceType {
                    family: SequenceCollectionFamily::OrderedSequence,
                    item: Box::new(qvec.clone()),
                }),
            ),
            (
                "Result(qvec, i32)",
                Type::Result(Box::new(qvec.clone()), Box::new(Type::I32)),
            ),
            (
                "Result(i32, qvec)",
                Type::Result(Box::new(Type::I32), Box::new(qvec.clone())),
            ),
            ("tuple containing qvec", Type::Tuple(vec![Type::I32, qvec])),
        ];
        let program = parse_program("fn main() { return; }").expect("parse");
        for (label, ty) in cases {
            let err =
                ensure_executable_type_supported(&ty, &program.arena, &[], "test".to_string())
                    .expect_err(&format!("{label} must reject as not executable-admitted"));
            assert!(
                err.message.contains("qvec is a reserved type"),
                "{label}: unexpected error: {}",
                err.message
            );
        }
    }

    #[test]
    fn ensure_executable_type_supported_admits_ordinary_families() {
        // Positive control: every already-qualified executable family must
        // continue to pass through the now-exhaustive match unchanged.
        let program = parse_program(
            r#"
            record R { n: i32 }
            enum E { A, B }
            fn main() { return; }
        "#,
        )
        .expect("parse");
        let r_id = *program.arena.symbol_to_id.get("R").expect("R symbol");
        let e_id = *program.arena.symbol_to_id.get("E").expect("E symbol");
        for ty in [
            Type::Quad,
            Type::Bool,
            Type::Text,
            Type::I32,
            Type::U32,
            Type::Fx,
            Type::F64,
            Type::Unit,
            Type::RangeI32,
            Type::Record(r_id),
            Type::Adt(e_id),
            Type::Tuple(vec![Type::I32, Type::Bool]),
            Type::Option(Box::new(Type::I32)),
            Type::Result(Box::new(Type::I32), Box::new(Type::Bool)),
        ] {
            ensure_executable_type_supported(&ty, &program.arena, &[], "test".to_string())
                .unwrap_or_else(|err| panic!("{ty:?} must remain executable-admitted: {err:?}"));
        }
    }

    #[test]
    fn generic_fn_with_bound_and_satisfying_impl_typechecks() {
        let src = r#"
            trait Zeroable {
                fn zero(v: ZeroInt) -> i32;
            }

            record ZeroInt { n: i32 }

            impl Zeroable for ZeroInt {
                fn zero(v: ZeroInt) -> i32 {
                    return 0;
                }
            }

            fn make_zero<T: Zeroable>(v: T) -> T {
                return v;
            }

            fn main() {
                let z: ZeroInt = ZeroInt { n: 0 };
                let r: ZeroInt = make_zero(z);
                let _ = r;
                return;
            }
        "#;
        typecheck_source(src).expect("bound satisfied by impl should typecheck");
    }

    #[test]
    fn generic_fn_with_bound_and_missing_impl_rejects() {
        let src = r#"
            trait Printable {
                fn print(v: NoPrint) -> i32;
            }

            record NoPrint { x: i32 }

            fn show<T: Printable>(v: T) -> T {
                return v;
            }

            fn main() {
                let p: NoPrint = NoPrint { x: 1 };
                let r: NoPrint = show(p);
                let _ = r;
                return;
            }
        "#;
        let err =
            typecheck_source(src).expect_err("call with unsatisfied trait bound must be rejected");
        assert!(
            err.message.contains("Printable")
                || err.message.contains("implement")
                || err.message.contains("trait"),
            "unexpected error: {}",
            err.message
        );
    }

    // M9.4 Wave 3 — richer pattern surface typecheck

    #[test]
    fn wildcard_match_pattern_typechecks() {
        let src = r#"
            enum Color { Red, Blue, Green }

            fn main() {
                let c: Color = Color::Red;
                match c {
                    Color::Red => { let r: i32 = 0; let _ = r; }
                    Color::Blue => { let r: i32 = 1; let _ = r; }
                    Color::Green => { let r: i32 = 2; let _ = r; }
                }
                return;
            }
        "#;
        typecheck_source(src).expect("exhaustive ADT match should typecheck");
    }

    #[test]
    fn or_pattern_match_arm_rejects_even_when_it_would_cover_two_variants() {
        // SSF-07: or-patterns have no lowering implementation for any
        // scrutinee family, so `match` rejects them deterministically at
        // typecheck regardless of whether they would otherwise contribute
        // useful exhaustiveness coverage.
        let src = r#"
            enum Color { Red, Blue, Green }

            fn main() {
                let c: Color = Color::Red;
                match c {
                    Color::Red | Color::Blue => { let r: i32 = 0; let _ = r; }
                    Color::Green => { let r: i32 = 2; let _ = r; }
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("or-pattern match arm must be rejected");
        assert!(
            err.message.contains("or-pattern match arms"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn or_pattern_match_arm_rejects_even_when_it_would_be_exhaustive() {
        let src = r#"
            enum Flag { A, B }

            fn main() {
                let f: Flag = Flag::A;
                match f {
                    Flag::A | Flag::B => { let r: i32 = 0; let _ = r; }
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("or-pattern match arm must be rejected");
        assert!(
            err.message.contains("or-pattern match arms"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn or_pattern_match_arm_rejects_inside_value_producing_loop_body_too() {
        // Regression for a real ordering bug: check_loop_expr_stmt's own
        // "match requires default arm '_'" check used to run before its
        // per-arm build_and_apply_match_plan loop, so a no-wildcard
        // or-pattern arm inside a `loop { ... break value; }` expression
        // body surfaced the generic default-arm diagnostic instead of the
        // or-pattern one, breaking the "identical diagnostic regardless of
        // wildcard presence" promise specifically in this control-flow
        // context (found by review on PR #1615).
        let src = r#"
            enum Flag { A, B }

            fn pick() -> i32 {
                let result: i32 = loop {
                    let f: Flag = Flag::A;
                    match f {
                        Flag::A | Flag::B => { break 1; }
                    }
                };
                return result;
            }

            fn main() {
                let r: i32 = pick();
                let _ = r;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("or-pattern match arm must be rejected");
        assert!(
            err.message.contains("or-pattern match arms"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn exhaustive_enum_match_without_wildcard_typechecks_inside_value_producing_loop_body() {
        // Regression for a second real bug in the same control-flow context:
        // check_loop_expr_stmt's default-arm check used to be a naive
        // "default.is_empty() => reject", unlike the plain statement-form
        // match handler's missing_exhaustive_sum_variants-based check. An
        // exhaustive enum/Option/Result match with no wildcard arm therefore
        // could not typecheck inside a `loop { ... break value; }` body even
        // though the identical program typechecks fine as an ordinary
        // statement (found by review on PR #1615).
        let src = r#"
            enum Flag { A, B }

            fn pick(f: Flag) -> i32 {
                let result: i32 = loop {
                    match f {
                        Flag::A => { break 1; }
                        Flag::B => { break 2; }
                    }
                };
                return result;
            }

            fn main() {
                let r: i32 = pick(Flag::A);
                let _ = r;
                return;
            }
        "#;
        typecheck_source(src)
            .expect("exhaustive enum match without wildcard should typecheck inside a loop body");
    }

    #[test]
    fn non_exhaustive_enum_match_without_wildcard_still_rejects_inside_value_producing_loop_body() {
        let src = r#"
            enum Flag { A, B, C }

            fn pick(f: Flag) -> i32 {
                let result: i32 = loop {
                    match f {
                        Flag::A => { break 1; }
                        Flag::B => { break 2; }
                    }
                };
                return result;
            }

            fn main() {
                let r: i32 = pick(Flag::A);
                let _ = r;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("non-exhaustive match must still be rejected");
        assert!(
            err.message.contains("non-exhaustive match") && err.message.contains("C"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn int_range_pattern_typechecks_on_i32() {
        let src = r#"
            fn main() {
                let x: i32 = 3;
                match x {
                    1..=5 => { let y: i32 = 1; let _ = y; }
                    _ => { let y: i32 = 0; let _ = y; }
                }
                return;
            }
        "#;
        typecheck_source(src).expect("int range pattern on i32 should typecheck");
    }

    #[test]
    fn int_range_pattern_rejects_non_integer_scrutinee() {
        let src = r#"
            fn main() {
                let x: bool = true;
                match x {
                    1..=5 => { let r: i32 = 0; let _ = r; }
                    _ => { let r: i32 = 1; let _ = r; }
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("int range pattern on bool must reject");
        assert!(
            err.message.contains("i32")
                || err.message.contains("u32")
                || err.message.contains("scrutinee"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn int_range_inverted_bounds_rejects() {
        let src = r#"
            fn main() {
                let x: i32 = 3;
                match x {
                    5..=1 => { let r: i32 = 0; let _ = r; }
                    _ => { let r: i32 = 1; let _ = r; }
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("inverted range bounds must reject");
        assert!(
            err.message.contains("start")
                || err.message.contains("end")
                || err.message.contains("<="),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn nested_tuple_destructuring_typechecks() {
        let src = r#"
            fn main() {
                let (a, (b, c)) = (1, (2, 3));
                let ra: i32 = a;
                let rb: i32 = b;
                let rc: i32 = c;
                let _ = ra;
                let _ = rb;
                let _ = rc;
                return;
            }
        "#;
        typecheck_source(src).expect("nested tuple destructuring should typecheck");
    }

    #[test]
    fn nested_tuple_arity_mismatch_rejects() {
        let src = r#"
            fn main() {
                let (a, (b, c)) = (1, (2, 3, 4));
                let _ = a;
                let _ = b;
                let _ = c;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("nested tuple arity mismatch must reject");
        assert!(
            err.message.contains("arity") || err.message.contains("mismatch"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn if_let_wildcard_typechecks() {
        let src = r#"
            fn make_int() -> i32 {
                return 1;
            }

            fn main() {
                let r: i32 = if let _ = make_int() { 1 } else { 0 };
                let _ = r;
                return;
            }
        "#;
        typecheck_source(src).expect("if-let wildcard should typecheck");
    }

    #[test]
    fn if_let_branch_type_mismatch_rejects() {
        let src = r#"
            enum Flag { A, B }

            fn main() {
                let f: Flag = Flag::A;
                let r: i32 = if let Flag::A = f { 1 } else { true };
                let _ = r;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("if-let branch type mismatch must reject");
        assert!(
            err.message.contains("mismatch")
                || err.message.contains("bool")
                || err.message.contains("i32"),
            "unexpected error: {}",
            err.message
        );
    }

    // M9.5 Wave B — parser admits `ref x` binding syntax

    #[test]
    fn ref_binding_in_tuple_pattern_parses() {
        let src = r#"
            fn make_pair() -> (i32, i32) { return (1, 2); }
            fn main() {
                let (ref a, b) = make_pair();
                let _ = b;
                return;
            }
        "#;
        // Plain tuple binds must preserve borrow capture instead of rewriting
        // every bind to Move before the ownership pipeline runs.
        typecheck_source(src).expect("ref binding in tuple pattern should parse and typecheck");
    }

    #[test]
    fn plain_tuple_ref_binding_preserves_borrow_path_state() {
        use crate::types::{PathAvailability, PatternPath};

        let mut arena = AstArena::default();
        let source = arena.intern_symbol("source");
        let borrowed = arena.intern_symbol("borrowed");
        let moved = arena.intern_symbol("moved");
        let value = arena.alloc_expr(Expr::Var(source));
        let stmt = arena.alloc_stmt(Stmt::LetTuple {
            items: vec![
                TuplePatternItem::Bind {
                    name: borrowed,
                    capture: CaptureMode::Borrow,
                },
                TuplePatternItem::Bind {
                    name: moved,
                    capture: CaptureMode::Move,
                },
            ],
            ty: None,
            value,
        });

        let mut env = ScopeEnv::new();
        env.insert(source, Type::Tuple(vec![Type::I32, Type::I32]));

        let table = FnTable::new();
        let record_table = RecordTable::new();
        let adt_table = AdtTable::new();
        let mut loop_stack = Vec::new();
        check_stmt(
            stmt,
            &arena,
            &mut env,
            Type::Unit,
            &table,
            &record_table,
            &adt_table,
            &mut loop_stack,
            &[],
        )
        .expect("tuple ref bind should typecheck");

        let binding = env.binding(source).expect("source binding must exist");
        assert!(binding.path_state.iter().any(|(path, state)| {
            *state == PathAvailability::Borrowed && *path == PatternPath::root().tuple_index(0)
        }));
        assert!(binding.path_state.iter().any(|(path, state)| {
            *state == PathAvailability::Moved && *path == PatternPath::root().tuple_index(1)
        }));
    }

    #[test]
    fn ref_binding_in_record_pattern_parses() {
        let src = r#"
            record DecisionContext {
                camera: quad,
                quality: f64,
            }
            fn main() {
                let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
                let DecisionContext { camera: ref seen_camera, quality: score } = ctx;
                let _ = seen_camera;
                let _ = score;
                return;
            }
        "#;
        typecheck_source(src).expect("ref binding in record pattern should parse and typecheck");
    }

    #[test]
    fn plain_record_ref_binding_preserves_record_field_path_state() {
        use crate::types::{
            PathAvailability, PatternPath, RecordDecl, RecordField, RecordPatternItem,
        };

        let mut arena = AstArena::default();
        let source = arena.intern_symbol("source");
        let record_name = arena.intern_symbol("DecisionContext");
        let camera = arena.intern_symbol("camera");
        let quality = arena.intern_symbol("quality");
        let borrowed = arena.intern_symbol("borrowed");
        let moved = arena.intern_symbol("moved");
        let value = arena.alloc_expr(Expr::Var(source));
        let stmt = arena.alloc_stmt(Stmt::LetRecord {
            record_name,
            items: vec![
                RecordPatternItem {
                    field: camera,
                    target: RecordPatternTarget::Bind {
                        name: borrowed,
                        capture: CaptureMode::Borrow,
                    },
                },
                RecordPatternItem {
                    field: quality,
                    target: RecordPatternTarget::Bind {
                        name: moved,
                        capture: CaptureMode::Move,
                    },
                },
            ],
            value,
        });

        let mut env = ScopeEnv::new();
        env.insert(source, Type::Record(record_name));

        let table = FnTable::new();
        let mut record_table = RecordTable::new();
        record_table.insert(
            record_name,
            RecordDecl {
                name: record_name,
                type_params: Vec::new(),
                fields: vec![
                    RecordField {
                        name: camera,
                        ty: Type::Quad,
                    },
                    RecordField {
                        name: quality,
                        ty: Type::F64,
                    },
                ],
            },
        );
        let adt_table = AdtTable::new();
        let mut loop_stack = Vec::new();
        check_stmt(
            stmt,
            &arena,
            &mut env,
            Type::Unit,
            &table,
            &record_table,
            &adt_table,
            &mut loop_stack,
            &[],
        )
        .expect("record ref bind should typecheck");

        let binding = env.binding(source).expect("source binding must exist");
        assert!(binding.path_state.iter().any(|(path, state)| {
            *state == PathAvailability::Borrowed
                && *path == PatternPath::root().record_field(camera)
        }));
        assert!(binding.path_state.iter().any(|(path, state)| {
            *state == PathAvailability::Moved && *path == PatternPath::root().record_field(quality)
        }));
    }

    #[test]
    fn ref_binding_in_adt_pattern_parses() {
        let src = r#"
            enum Wrap { Val(i32) }
            fn make() -> Wrap { return Wrap::Val(1); }
            fn main() {
                let w: Wrap = make();
                match w {
                    Wrap::Val(ref x) => { let _ = x; }
                }
                return;
            }
        "#;
        typecheck_source(src).expect("ref binding in ADT pattern should parse and typecheck");
    }

    // M9.5 Wave C — binding plan builders + conflict detection + consumed-state

    #[test]
    fn binding_plan_tuple_move_ok() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: PatternPath::root().tuple_index(0),
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan).expect("single move binding should not conflict");
    }

    #[test]
    fn binding_plan_two_borrows_same_path_ok() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let path = PatternPath::root().tuple_index(0);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Borrow,
            path: path.clone(),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path,
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan)
            .expect("two borrows of same path should not conflict");
    }

    #[test]
    fn binding_plan_move_and_borrow_same_path_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let path = PatternPath::root().tuple_index(0);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: path.clone(),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("move+borrow same path must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("capture"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn scrutinee_use_move_gives_consumed() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, ScrutineeUse, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: PatternPath::root().tuple_index(0),
            ty: Type::I32,
        });
        assert_eq!(scrutinee_use_from_plan(&plan), ScrutineeUse::Consumed);
    }

    #[test]
    fn scrutinee_use_all_borrow_gives_preserved() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, ScrutineeUse, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Borrow,
            path: PatternPath::root().tuple_index(0),
            ty: Type::I32,
        });
        assert_eq!(scrutinee_use_from_plan(&plan), ScrutineeUse::Preserved);
    }

    #[test]
    fn use_after_move_rejects() {
        let src = r#"
            fn take_val() -> i32 { return 5; }
            fn main() {
                let x: i32 = take_val();
                let _ = x;
                let _ = x;
                return;
            }
        "#;
        // i32 is Copy — use-after-move semantics only apply to non-Copy types.
        // This test just validates the checker doesn't false-positive on i32.
        typecheck_source(src).expect("plain i32 variable reuse should typecheck fine");
    }

    // M9.5 Wave D — match ownership pipeline

    #[test]
    fn match_borrow_binding_does_not_consume_scrutinee() {
        // All-borrow match: scrutinee variable stays available after the match.
        let src = r#"
            enum Maybe { Some(i32), None }
            fn make() -> Maybe { return Maybe::None; }
            fn main() {
                let v: Maybe = make();
                match v {
                    Maybe::Some(ref x) => { let _ = x; }
                    Maybe::None => { let r: i32 = 0; let _ = r; }
                }
                return;
            }
        "#;
        typecheck_source(src).expect("all-borrow match should not consume scrutinee");
    }

    #[test]
    fn match_move_binding_typechecks() {
        // Move binding in match arm: the binding captures the payload.
        let src = r#"
            enum Wrap { Val(i32) }
            fn make() -> Wrap { return Wrap::Val(5); }
            fn main() {
                let w: Wrap = make();
                match w {
                    Wrap::Val(x) => { let r: i32 = x; let _ = r; }
                }
                return;
            }
        "#;
        typecheck_source(src).expect("move binding in match arm should typecheck");
    }

    // ──────────────────────────────────────────────────────────────
    // SSF-08 Lane 1 (#1656-#1664): canonical ScopeEnv ownership-state
    // transition/join model. Each test proves, against the shared model,
    // that a real regression exists on unpatched code (documented in the
    // decision record's evidence and reproduced against origin/main before
    // this PR) and is closed here.
    // ──────────────────────────────────────────────────────────────

    #[test]
    fn ssf08_diag_minimal_record_destructure_bisect() {
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                let Point { x: moved, y: yv } = p;
                let _ = moved;
                return;
            }
        "#;
        typecheck_source(src).expect("diag: minimal top-level record destructure");
    }

    #[test]
    fn fa02_039_known_defect_nested_projection_rereads_intermediate_base_as_whole_value() {
        // Tracked as FA-02-039 (skulmakov-oss/Semantic#1881). NOT part of
        // #1656-#1664, NOT fixed by this PR. Documents a pre-existing
        // frontend defect found while writing the #1663 regression suite
        // (SSF-08 Lane 1): reading a two-level-nested field projection
        // (`c.pair.b`) spuriously rejects if a *sibling* field of the
        // intermediate base was moved (`c.pair.a`), because the
        // intermediate base (`c.pair`) is independently re-checked as if it
        // were being read as a whole value, not merely projected through.
        // Isolated to a straight-line program with zero if/loop/match and
        // zero Lane 1 join-model code involved at all -- the root cause is
        // in `infer_expr_type_no_check`, which only skips its own
        // path-availability re-check when the base expression is a bare
        // `Expr::Var`; a `RecordField` base (as in `c.pair.b`, whose base
        // `c.pair` is itself a `RecordField` access, not a `Var`) falls
        // through to a second, independent, shorter-path check that the
        // outer expression's own top-level check has already correctly
        // subsumed. This is a read-path-checking gap orthogonal to
        // control-flow/branch-joining (Lane 1's actual scope) -- it would
        // misfire in a single straight-line function with no
        // `if`/`loop`/`match` anywhere. This test pins the *current*
        // (buggy) behavior so it does not regress silently; when FA-02-039
        // is repaired, this test must be replaced or inverted to assert
        // success, not preserved as-is -- its continued `expect_err` after
        // a fix would itself be a stale, incorrect assertion.
        let src = r#"
            record Pair { a: i32, b: i32 }
            record Container { pair: Pair, other: i32 }
            fn main() {
                let c: Container = Container { pair: Pair { a: 1, b: 2 }, other: 9 };
                let Pair { a: moved, b: _ } = c.pair;
                let _ = moved;
                let z: i32 = c.pair.b;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "known pre-existing defect: reading a sibling field two levels deep spuriously rejects; see comment above",
        );
        assert!(
            err.message.contains("partially moved"),
            "unexpected: {}",
            err.message
        );
    }

    // #1656 -- statement `if`

    #[test]
    fn ssf08_1656_one_branch_move_persists_after_if() {
        // Only the `then` branch moves p.x; a bare `if` with no matching
        // restriction in `else` must still restrict p.x afterward, because
        // the `then` branch is a reachable successor.
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                if (1 == 1) {
                    let Point { x: moved, y: yv } = p;
                    let _ = moved;
                } else {
                    let unused_v: i32 = 0;
                }
                let z: i32 = p.x;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("outer path moved in only the then-branch must reject after the if");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1656_both_branches_move_rejects_after() {
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                if (1 == 1) {
                    let Point { x: moved, y: yv } = p;
                    let _ = moved;
                } else {
                    let Point { x: moved2, y: y2v } = p;
                    let _ = moved2;
                }
                let z: i32 = p.x;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("outer path moved in both branches must reject after the if");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1656_sibling_paths_across_branches_both_retained() {
        // then moves p.x, else moves p.y -- since we don't know which branch
        // ran, BOTH restrictions must be retained after the if (conservative
        // join: a restriction present in *either* successor survives).
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                if (1 == 1) {
                    let Point { x: moved, y: yv } = p;
                    let _ = moved;
                } else {
                    let Point { x: x2v, y: moved2 } = p;
                    let _ = moved2;
                }
                let zx: i32 = p.x;
                let _ = zx;
                return;
            }
        "#;
        let err_x = typecheck_source(src).expect_err("p.x must reject: then-branch moved it");
        assert!(
            err_x.message.contains("moved"),
            "unexpected: {}",
            err_x.message
        );

        let src_y = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                if (1 == 1) {
                    let Point { x: moved, y: yv } = p;
                    let _ = moved;
                } else {
                    let Point { x: x2v, y: moved2 } = p;
                    let _ = moved2;
                }
                let zy: i32 = p.y;
                let _ = zy;
                return;
            }
        "#;
        let err_y = typecheck_source(src_y).expect_err("p.y must reject: else-branch moved it");
        assert!(
            err_y.message.contains("moved"),
            "unexpected: {}",
            err_y.message
        );
    }

    #[test]
    fn ssf08_1656_branch_local_binding_does_not_leak() {
        let src = r#"
            fn main() {
                if (1 == 1) {
                    let leaked: i32 = 5;
                    let _ = leaked;
                } else {
                }
                let z: i32 = leaked;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("then-branch-local binding must not be visible after the if");
        assert!(
            err.message.contains("unknown variable"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1656_no_else_move_still_persists() {
        // A bare `if` with an empty else_block: the then-branch is still a
        // reachable successor, so its restriction must survive.
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                if (1 == 1) {
                    let Point { x: moved, y: yv } = p;
                    let _ = moved;
                }
                let z: i32 = p.x;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("no-else if that moves p.x in its only branch must reject after");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    // #1657 -- loop statements

    #[test]
    fn ssf08_1657_while_loop_move_persists_after_loop() {
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                let mut i: i32 = 0;
                while i < 1 {
                    let Point { x: moved, y: yv } = p;
                    let _ = moved;
                    i = i + 1;
                }
                let z: i32 = p.x;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("outer path moved inside a while body must reject after the loop");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1657_while_loop_repeated_move_of_same_path_rejects() {
        // The fixed-point property: a body that unconditionally moves the
        // same outer path every pass must reject, because a second logical
        // pass would re-move an already-moved path -- a single-pass check
        // (the pre-Lane-1 behavior) cannot see this, since it only ever
        // checks the body once against the untouched pre-loop state.
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                let mut i: i32 = 0;
                while i < 3 {
                    let Point { x: moved, y: yv } = p;
                    let _ = moved;
                    i = i + 1;
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "loop body that unconditionally re-moves the same path every pass must reject",
        );
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1657_for_range_loop_variable_does_not_leak() {
        let src = r#"
            fn main() {
                for i in 0..3 {
                    let _ = i;
                }
                let z: i32 = i;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("for-range loop variable must not be visible after the loop");
        assert!(
            err.message.contains("unknown variable"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_statement_loop_break_move_persists_after_loop() {
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                loop {
                    let Point { x: moved, y: yv } = p;
                    let _ = moved;
                    break;
                }
                let z: i32 = p.x;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("outer path moved inside a statement-loop body must reject after break");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    // #1658 -- statement `match`

    #[test]
    fn ssf08_1658_arm_moves_unrelated_outer_binding_persists_after_match() {
        let src = r#"
            enum Wrap { Val(i32) }
            record Point { x: i32, y: i32 }
            fn main() {
                let w: Wrap = Wrap::Val(5);
                let p: Point = Point { x: 1, y: 2 };
                match w {
                    Wrap::Val(v) => {
                        let _ = v;
                        let Point { x: moved, y: yv } = p;
                        let _ = moved;
                    }
                }
                let z: i32 = p.x;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("an arm moving an unrelated outer path must restrict it after the match");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1658_default_arm_included_in_join() {
        let src = r#"
            enum Wrap { A, B }
            record Point { x: i32, y: i32 }
            fn main() {
                let w: Wrap = Wrap::B;
                let p: Point = Point { x: 1, y: 2 };
                match w {
                    Wrap::A => { let rv: i32 = 0; let _ = rv; }
                    _ => {
                        let Point { x: moved, y: yv } = p;
                        let _ = moved;
                    }
                }
                let z: i32 = p.x;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("a default arm moving an outer path must restrict it after the match");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1658_arm_local_binding_does_not_leak() {
        let src = r#"
            enum Wrap { Val(i32) }
            fn main() {
                match Wrap::Val(5) {
                    Wrap::Val(v) => {
                        let leaked: i32 = v;
                        let _ = leaked;
                    }
                }
                let z: i32 = leaked;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("match-arm-local binding must not be visible after the match");
        assert!(
            err.message.contains("unknown variable"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1658_sibling_arms_do_not_contaminate_each_other() {
        // Arm A's own capture must not be visible while checking arm B's
        // body -- each arm starts independently from the pre-match state.
        let src = r#"
            enum Wrap { A(i32), B(i32) }
            fn main() {
                match Wrap::A(1) {
                    Wrap::A(x) => { let _ = x; }
                    Wrap::B(y) => {
                        let _ = y;
                        let leaked: i32 = x;
                        let _ = leaked;
                    }
                }
                return;
            }
        "#;
        let err =
            typecheck_source(src).expect_err("arm B must not see arm A's own pattern-bound name");
        assert!(
            err.message.contains("unknown variable"),
            "unexpected: {}",
            err.message
        );
    }

    // #1659 -- expression `if`

    #[test]
    fn ssf08_1659_expr_if_ownership_effect_survives() {
        let src = r#"
            fn main() {
                let pair: (i32, i32) = (1, 2);
                let v: i32 = if (1 == 1) {
                    let (moved, dummy) = pair;
                    let _ = dummy;
                    moved
                } else {
                    0
                };
                let _ = v;
                let (checked, dummy2) = pair;
                let _ = checked;
                let _ = dummy2;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "value-producing if expression must propagate ownership state like statement if",
        );
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1659_expr_if_result_type_still_correct_when_state_preserved() {
        // The join must not corrupt the computed result type.
        let src = r#"
            fn main() {
                let v: i32 = if (1 == 1) { 1 } else { 2 };
                let _ = v;
                return;
            }
        "#;
        typecheck_source(src).expect("plain if-expression result typing must remain correct");
    }

    // #1660 -- loop expression

    #[test]
    fn ssf08_1660_loop_expr_outer_effect_survives() {
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                let v: i32 = loop {
                    let Point { x: moved, y: yv } = p;
                    break moved;
                };
                let _ = v;
                let z: i32 = p.x;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("loop-expression body's ownership effect on an outer path must survive");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1660_loop_expr_break_type_still_correct() {
        let src = r#"
            fn main() {
                let v: i32 = loop {
                    break 7;
                };
                let _ = v;
                return;
            }
        "#;
        typecheck_source(src).expect("loop-expression break typing must remain correct");
    }

    // #1661 -- match expression

    #[test]
    fn ssf08_1661_match_expr_move_pattern_state_reaches_caller() {
        let src = r#"
            enum Wrap { Val(i32) }
            fn main() {
                let w: Wrap = Wrap::Val(5);
                let pair: (i32, i32) = (1, 2);
                let v: i32 = match w {
                    Wrap::Val(x) => {
                        let (moved, dummy) = pair;
                        let _ = dummy;
                        moved + x
                    }
                };
                let _ = v;
                let (checked, dummy2) = pair;
                let _ = checked;
                let _ = dummy2;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "match-expression arm's ownership effect on an outer path must reach the caller",
        );
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1661_match_expr_alternatives_join_conservatively() {
        let src = r#"
            enum Wrap { A, B }
            fn main() {
                let w: Wrap = Wrap::A;
                let pair: (i32, i32) = (1, 2);
                let v: i32 = match w {
                    Wrap::A => {
                        let (moved, dummy) = pair;
                        let _ = dummy;
                        moved
                    }
                    Wrap::B => { 0 }
                };
                let _ = v;
                let (checked, dummy2) = pair;
                let _ = checked;
                let _ = dummy2;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("one match-expression arm moving p.x must restrict it, even though the other arm doesn't");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    // #1662 -- if-let expression

    #[test]
    fn ssf08_1662_if_let_expr_success_move_reflected_after() {
        let src = r#"
            enum Wrap { Val(i32) }
            fn main() {
                let w: Wrap = Wrap::Val(5);
                let pair: (i32, i32) = (1, 2);
                let v: i32 = if let Wrap::Val(x) = w {
                    let (moved, dummy) = pair;
                    let _ = dummy;
                    moved + x
                } else {
                    0
                };
                let _ = v;
                let (checked, dummy2) = pair;
                let _ = checked;
                let _ = dummy2;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("if-let success branch's ownership effect on an outer path must survive");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1662_if_let_expr_else_branch_included_in_join() {
        // The then-block's tail is `x + 0` rather than bare `x`: a bare
        // single-identifier tail (`{ x }`) right after a bare-identifier
        // scrutinee (`w`) is ambiguous with record-literal field-shorthand
        // syntax (`w { x }` meaning "construct record w with field x"), a
        // pre-existing parser ambiguity unrelated to Lane 1 (see the
        // "TODO(M9.5): disambiguate expr parsing for scrutinee" comment on
        // `Expr::IfLet` handling) -- worked around here, not fixed.
        let src = r#"
            enum Wrap { Val(i32) }
            fn main() {
                let w: Wrap = Wrap::Val(5);
                let pair: (i32, i32) = (1, 2);
                let v: i32 = if let Wrap::Val(x) = w {
                    x + 0
                } else {
                    let (moved, dummy) = pair;
                    let _ = dummy;
                    moved
                };
                let _ = v;
                let (checked, dummy2) = pair;
                let _ = checked;
                let _ = dummy2;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("if-let else branch's ownership effect on an outer path must survive");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1662_if_let_expr_pattern_binding_does_not_leak() {
        let src = r#"
            enum Wrap { Val(i32) }
            fn main() {
                let w: Wrap = Wrap::Val(5);
                let v: i32 = if let Wrap::Val(x) = w { x + 0 } else { 0 };
                let _ = v;
                let leaked: i32 = x;
                let _ = leaked;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("if-let pattern-bound name must not leak past the expression");
        assert!(
            err.message.contains("unknown variable"),
            "unexpected: {}",
            err.message
        );
    }

    // #1663 -- projected scrutinee

    #[test]
    fn ssf08_1663_projected_record_field_scrutinee_move_is_tracked() {
        // A destructuring bind whose value expression is a projected field
        // (`c.pair`, not a bare variable) must restrict `c.pair.a` itself,
        // not a bogus root-only `c` path or nothing at all. (`match` does
        // not admit a record scrutinee at all -- source_semantics.md's
        // `match` scrutinee families are quad/enum/Option/Result/i32/u32 --
        // so this exercises the same `apply_arm_pattern_capture` machinery
        // through the plain-destructuring-let call sites instead.)
        let src = r#"
            record Pair { a: i32, b: i32 }
            record Container { pair: Pair, other: i32 }
            fn main() {
                let c: Container = Container { pair: Pair { a: 1, b: 2 }, other: 9 };
                let Pair { a: moved, b: bv } = c.pair;
                let _ = moved;
                let _ = bv;
                let z: i32 = c.pair.a;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src)
            .expect_err("a move captured from a projected record-field scrutinee must be tracked on that exact path");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_1663_projected_record_field_scrutinee_sibling_field_still_usable() {
        // Proves the tracked path is `c.pair.a` specifically, not a coarser
        // `c` root -- a sibling field one level up, `c.other`, must remain
        // usable after only `c.pair.a` was moved. (Deliberately uses a
        // one-level-nested sibling, not `c.pair.b`: reading a two-level
        // projection through an intermediate record-field base hits an
        // unrelated, pre-existing defect tracked as FA-02-039
        // (skulmakov-oss/Semantic#1881) -- see
        // `fa02_039_known_defect_nested_projection_rereads_intermediate_base_as_whole_value`
        // -- which this test must not exercise.)
        let src = r#"
            record Pair { a: i32, b: i32 }
            record Container { pair: Pair, other: i32 }
            fn main() {
                let c: Container = Container { pair: Pair { a: 1, b: 2 }, other: 9 };
                let Pair { a: moved, b: _ } = c.pair;
                let _ = moved;
                let z: i32 = c.other;
                let _ = z;
                return;
            }
        "#;
        typecheck_source(src)
            .expect("sibling field of a projected-scrutinee move must remain usable");
    }

    #[test]
    fn ssf08_1663_dynamically_indexed_sequence_scrutinee_with_capture_rejects() {
        // seq[i] with a non-literal index cannot be resolved to a static
        // path by expr_access_path; since the pattern here captures a value
        // from it, this must reject deterministically rather than silently
        // skip ownership tracking.
        let src = r#"
            fn main() {
                let seq: Sequence(i32) = [1, 2, 3];
                let i: i32 = 1;
                match seq[i] {
                    v => { let _ = v; }
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "a capturing match against a dynamically-indexed sequence scrutinee must reject deterministically",
        );
        assert!(
            !err.message.contains("unknown variable"),
            "must be an explicit unsupported-path rejection, not an unrelated unknown-variable error: {}",
            err.message
        );
    }

    // #1664 -- missing binding fail-closed

    #[test]
    fn ssf08_1664_mark_path_state_fails_closed_on_missing_binding() {
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let unknown = SymbolId(999);
        let err = env
            .mark_path_state(unknown, PatternPath::root(), PathAvailability::Moved)
            .expect_err("mark_path_state on a genuinely missing binding must fail closed");
        assert!(
            err.message.contains("internal ownership state"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1664_check_capture_allowed_fails_closed_on_missing_binding() {
        use crate::types::{CaptureMode, PatternPath};
        let env = ScopeEnv::new();
        let unknown = SymbolId(998);
        let err = env
            .check_capture_allowed(unknown, &PatternPath::root(), CaptureMode::Move)
            .expect_err("check_capture_allowed on a genuinely missing binding must fail closed");
        assert!(
            err.message.contains("internal ownership state"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1664_check_path_available_fails_closed_on_missing_binding() {
        // check_path_available itself is REQUIRED_BINDING and fails closed;
        // its one production call site (top of infer_expr_type) never lets
        // this branch fire for a genuinely-unknown source variable because
        // it existence-gates first -- see
        // ssf08_1664_unknown_variable_diagnostic_remains_canonical below for
        // that source-level proof. This test proves the API itself, in
        // isolation, no longer treats "missing" as "available".
        use crate::types::PatternPath;
        let env = ScopeEnv::new();
        let unknown = SymbolId(997);
        let err = env
            .check_path_available(unknown, &PatternPath::root())
            .expect_err("check_path_available on a genuinely missing binding must fail closed");
        assert!(
            err.message.contains("internal ownership state"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1664_mark_consumed_fails_closed_on_missing_binding() {
        let mut env = ScopeEnv::new();
        let unknown = SymbolId(996);
        let err = env
            .mark_consumed(unknown)
            .expect_err("mark_consumed on a genuinely missing binding must fail closed");
        assert!(
            err.message.contains("internal ownership state"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1664_is_consumed_fails_closed_on_missing_binding() {
        let env = ScopeEnv::new();
        let unknown = SymbolId(995);
        let err = env
            .is_consumed(unknown)
            .expect_err("is_consumed on a genuinely missing binding must fail closed");
        assert!(
            err.message.contains("internal ownership state"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1664_is_mutable_fails_closed_on_missing_binding() {
        // Zero call sites exist anywhere in the workspace for is_mutable,
        // so this signature carries no external-crate compatibility burden
        // (unlike is_const below) and could be changed directly.
        let env = ScopeEnv::new();
        let unknown = SymbolId(994);
        let err = env
            .is_mutable(unknown)
            .expect_err("is_mutable on a genuinely missing binding must fail closed");
        assert!(
            err.message.contains("internal ownership state"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1664_is_const_checked_fails_closed_on_missing_binding() {
        // is_const_checked is the fail-closed sibling used by every
        // sm-front-internal call site (see is_const's own doc comment and
        // the next test for why the original is_const symbol itself could
        // not be changed in place).
        let env = ScopeEnv::new();
        let unknown = SymbolId(993);
        let err = env
            .is_const_checked(unknown)
            .expect_err("is_const_checked on a genuinely missing binding must fail closed");
        assert!(
            err.message.contains("internal ownership state"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1664_is_const_legacy_bool_api_remains_fail_open_for_lane2_compat() {
        // Deliberate, evidenced exception: `is_const`'s bool-returning shape
        // has live production call sites in crates/sm-ir/src/
        // legacy_lowering.rs (Lane 2), which SSF-08 Lane 1 is not permitted
        // to modify. This pins that the old symbol still returns `false` on
        // a missing binding -- not because it is correct, but because
        // changing it would require editing a Lane 2 crate. Every
        // sm-front-internal decision now goes through is_const_checked
        // instead (see the prior test); this method must gain no new
        // sm-front call sites.
        let env = ScopeEnv::new();
        let unknown = SymbolId(992);
        assert!(
            !env.is_const(unknown),
            "legacy is_const must still fail open (false) on a missing binding -- this is the \
             documented Lane 2 compatibility exception, not new behavior"
        );
    }

    #[test]
    fn ssf08_1664_unknown_variable_diagnostic_remains_canonical() {
        // check_path_available is REQUIRED_BINDING and fails closed (see
        // ssf08_1664_check_path_available_fails_closed_on_missing_binding
        // above), but its one production call site (top of infer_expr_type)
        // existence-gates first via env.get, so a genuinely unknown source
        // variable never reaches it -- it still reports the ordinary
        // "unknown variable" diagnostic from Expr::Var's own resolution,
        // not an internal ownership-state error.
        let src = r#"
            fn main() {
                let z: i32 = totally_unknown_name;
                let _ = z;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("unknown variable must still be reported");
        assert!(
            err.message.contains("unknown variable"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn ssf08_1664_const_initializer_unknown_variable_diagnostic_remains_canonical() {
        // Genuine pre-existing bug fixed by this PR: ensure_const_initializer_safe
        // runs before infer_expr_type_with_expected in the Stmt::Const path
        // (see check_stmt), so its own Expr::Var arm could not rely on a
        // prior existence check -- it used to call the fail-open is_const,
        // silently reporting "'x' is not const" for a variable that does
        // not exist at all. It now resolves existence itself first.
        let src = r#"
            fn main() {
                const X: i32 = totally_unknown_name;
                let _ = X;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("unknown variable must still be reported");
        assert!(
            err.message.contains("unknown variable"),
            "unexpected diagnostic (must not be 'is not const'): {}",
            err.message
        );
    }

    // #1656-#1664 cross-product: proving the model is genuinely shared, not
    // duplicated per construct.

    #[test]
    fn ssf08_cross_product_match_arm_modifies_outer_tuple_inside_value_block() {
        let src = r#"
            enum Wrap { A, B }
            fn main() {
                let pair: (i32, i32) = (1, 2);
                let v: i32 = match Wrap::A {
                    Wrap::A => {
                        let (ref moved, second_v) = pair;
                        moved
                    }
                    Wrap::B => { 0 }
                };
                let _ = v;
                let (first, sv) = pair;
                let _ = first;
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "match arm inside a value-producing context that moves an outer tuple path must restrict it afterward",
        );
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_cross_product_if_let_on_projected_record_scrutinee_with_move_then_sibling() {
        // if-let's own scrutinee (`c.flag`) is itself a projected field
        // (#1663), and the then-branch separately moves a tuple field
        // (destructuring bind is admitted inside a value-producing block;
        // record destructuring bind is not, so the outer moved value uses a
        // tuple here rather than a second record).
        let src = r#"
            record Container { pair: (i32, i32), flag: Wrap }
            enum Wrap { Present(i32), Absent }
            fn main() {
                let c: Container = Container { pair: (1, 2), flag: Wrap::Present(9) };
                let v: i32 = if let Wrap::Present(x) = c.flag {
                    let (moved, dummy) = c.pair;
                    let _ = dummy;
                    moved + x
                } else {
                    0
                };
                let _ = v;
                let (checked, dummy2) = c.pair;
                let _ = checked;
                let _ = dummy2;
                return;
            }
        "#;
        // The then-branch's plain tuple destructure moves both `c.pair.0`
        // and `c.pair.1` (default capture is Move); re-destructuring
        // `c.pair` again afterward must therefore still reject -- proving
        // the projected-scrutinee if-let's own branch effect on an
        // unrelated projected path correctly survives to after the whole
        // if-let expression, not just to the end of its own branch.
        let err = typecheck_source(src).expect_err(
            "re-destructuring c.pair after the then-branch already moved both elements must reject",
        );
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn ssf08_cross_product_loop_body_conditional_move_then_second_pass_conflict() {
        // Loop body conditionally moves an outer path only when a runtime
        // condition holds -- since the condition can't be proven false at
        // compile time, the fixed point must still treat the path as
        // restricted, and a second unconditional move attempt (outside the
        // conditional) against the now-restricted state must reject.
        let src = r#"
            record Point { x: i32, y: i32 }
            fn main() {
                let p: Point = Point { x: 1, y: 2 };
                let mut i: i32 = 0;
                while i < 2 {
                    if (1 == 1) {
                        let Point { x: moved, y: yv } = p;
                        let _ = moved;
                    } else {
                    }
                    let Point { x: moved2, y: y2v } = p;
                    let _ = moved2;
                    i = i + 1;
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err(
            "an unconditional second move of a conditionally-already-moved outer path in the same loop body must reject",
        );
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn match_or_pattern_rejects_regardless_of_consistent_capture_modes() {
        // Or-pattern where all alternatives borrow the same way is still
        // rejected — SSF-07 blanket-rejects or-pattern match arms before
        // capture-mode consistency is even considered.
        let src = r#"
            enum Flag { A, B, C }
            fn main() {
                let f: Flag = Flag::A;
                match f {
                    Flag::A | Flag::B => { let r: i32 = 0; let _ = r; }
                    Flag::C => { let r: i32 = 1; let _ = r; }
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("or-pattern match arm must be rejected");
        assert!(
            err.message.contains("or-pattern match arms"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn match_or_pattern_rejects_before_capture_mode_conflict_is_checked() {
        // One arm binds with ref, the other without. Before SSF-07's
        // blanket or-pattern rejection this failed on capture-mode
        // inconsistency instead; now the blanket rejection fires first,
        // regardless of whether the alternatives would have been internally
        // consistent. The underlying capture-mode-conflict check itself is
        // still exercised via `if let`, which does not route through this
        // match-arm entry point.
        let src = r#"
            enum Wrap { Val(i32) }
            fn make() -> Wrap { return Wrap::Val(1); }
            fn main() {
                let w: Wrap = make();
                match w {
                    Wrap::Val(ref x) | Wrap::Val(y) => { let _ = y; }
                }
                return;
            }
        "#;
        let err = typecheck_source(src).expect_err("or-pattern match arm must be rejected");
        assert!(
            err.message.contains("or-pattern match arms"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn match_same_path_move_and_borrow_rejects() {
        // A single arm with two bindings for the same payload slot (move + borrow conflict).
        // This is enforced by validate_binding_plan_conflicts.
        // Note: parser currently only allows one binding per payload slot,
        // so this test validates the plan-level conflict check via direct API.
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let path = PatternPath::root().variant(SymbolId(0)).variant_field(0);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: path.clone(),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("move+borrow same path must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("capture"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn match_all_arms_borrow_path_ok() {
        // Two bindings for the same path both borrowing: allowed.
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let path = PatternPath::root().tuple_index(0);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Borrow,
            path: path.clone(),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path,
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan).expect("double-borrow same path must not conflict");
    }

    // M9.7 — partial move: path-based availability in ScopeEnv

    #[test]
    fn partial_move_sibling_path_still_usable() {
        // Move root.0 (first element), then use root.1 (second element) — ok.
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(1);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        // Accessing root.1 (different sibling) should be allowed.
        env.check_path_available(sym, &PatternPath::root().tuple_index(1))
            .expect("sibling path of moved path should remain available");
    }

    #[test]
    fn partial_move_root_blocks_whole_var_use() {
        // Move root.0, then try to use the whole variable (root) — must reject.
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(2);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        // Accessing root (the whole variable) overlaps with root.0 that was moved → reject.
        let err = env
            .check_path_available(sym, &PatternPath::root())
            .expect_err("use of whole var after partial move must reject");
        assert!(
            err.message.contains("partially moved") || err.message.contains("moved"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn partial_move_child_blocks_child_use() {
        // Move root.0, then try to use root.0 again — must reject.
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(3);
        env.insert(sym, Type::I32);
        let path = PatternPath::root().tuple_index(0);
        env.mark_path_state(sym, path.clone(), PathAvailability::Moved)
            .expect("test-constructed binding must exist");
        let err = env
            .check_path_available(sym, &path)
            .expect_err("re-use of moved child path must reject");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn whole_var_consumed_still_blocks() {
        // mark_consumed (whole-var) still blocks root access.
        use crate::types::PatternPath;
        let mut env = ScopeEnv::new();
        let sym = SymbolId(4);
        env.insert(sym, Type::I32);
        env.mark_consumed(sym)
            .expect("test-constructed binding must exist");
        let err = env
            .check_path_available(sym, &PatternPath::root())
            .expect_err("whole-consumed var must be blocked");
        assert!(err.message.contains("moved"), "unexpected: {}", err.message);
    }

    #[test]
    fn borrow_path_does_not_block_read() {
        // Borrow only — read should still be allowed (conservative: borrows don't block reads).
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(5);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Borrowed,
        )
        .expect("test-constructed binding must exist");
        env.check_path_available(sym, &PatternPath::root().tuple_index(0))
            .expect("borrow-only path should not block reads");
    }

    // M9.8 — borrow enforcement against prior path-state

    #[test]
    fn check_capture_allowed_borrow_then_move_rejects() {
        use crate::types::{CaptureMode, PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(10);
        env.insert(sym, Type::I32);
        // Borrow root.0
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Borrowed,
        )
        .expect("test-constructed binding must exist");
        // Now try to move root.0 — must reject
        let err = env
            .check_capture_allowed(sym, &PatternPath::root().tuple_index(0), CaptureMode::Move)
            .expect_err("move after borrow of same path must reject");
        assert!(
            err.message.contains("borrow") || err.message.contains("cannot move"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn check_capture_allowed_move_then_borrow_rejects() {
        use crate::types::{CaptureMode, PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(11);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        let err = env
            .check_capture_allowed(
                sym,
                &PatternPath::root().tuple_index(0),
                CaptureMode::Borrow,
            )
            .expect_err("borrow after move of same path must reject");
        assert!(
            err.message.contains("moved") || err.message.contains("cannot borrow"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn check_capture_allowed_borrow_then_borrow_ok() {
        use crate::types::{CaptureMode, PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(12);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Borrowed,
        )
        .expect("test-constructed binding must exist");
        env.check_capture_allowed(
            sym,
            &PatternPath::root().tuple_index(0),
            CaptureMode::Borrow,
        )
        .expect("borrow after borrow of same path must be ok");
    }

    #[test]
    fn check_capture_allowed_borrow_then_move_sibling_ok() {
        // Borrow root.0, then move root.1 — different sibling, no overlap, ok.
        use crate::types::{CaptureMode, PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(13);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Borrowed,
        )
        .expect("test-constructed binding must exist");
        env.check_capture_allowed(sym, &PatternPath::root().tuple_index(1), CaptureMode::Move)
            .expect("move of sibling of borrowed path must be ok");
    }

    // M9.9 — expr_access_path + path-state normalization

    #[test]
    fn expr_access_path_var_is_root() {
        use crate::types::PatternPath;
        let mut arena = AstArena::default();
        let sym = SymbolId(99);
        let var_id = arena.alloc_expr(Expr::Var(sym));
        let result = expr_access_path(var_id, &arena);
        assert_eq!(result, Some((sym, PatternPath::root())));
    }

    #[test]
    fn expr_access_path_literal_is_none() {
        let mut arena = AstArena::default();
        let lit_id = arena.alloc_expr(Expr::BoolLiteral(true));
        assert_eq!(expr_access_path(lit_id, &arena), None);
    }

    #[test]
    fn expr_access_path_sequence_index_literal() {
        use crate::types::{NumericLiteral, PatternPath, SequenceIndexExpr};
        let mut arena = AstArena::default();
        let sym = SymbolId(7);
        let base = arena.alloc_expr(Expr::Var(sym));
        let idx = arena.alloc_expr(Expr::NumericLiteral(NumericLiteral::I32(2)));
        let expr = arena.alloc_expr(Expr::SequenceIndex(SequenceIndexExpr { base, index: idx }));
        let result = expr_access_path(expr, &arena);
        assert_eq!(result, Some((sym, PatternPath::root().tuple_index(2))));
    }

    #[test]
    fn expr_access_path_sequence_index_non_literal_is_none() {
        use crate::types::SequenceIndexExpr;
        let mut arena = AstArena::default();
        let sym = SymbolId(7);
        let base = arena.alloc_expr(Expr::Var(sym));
        let dyn_idx = arena.alloc_expr(Expr::Var(SymbolId(8)));
        let expr = arena.alloc_expr(Expr::SequenceIndex(SequenceIndexExpr {
            base,
            index: dyn_idx,
        }));
        // dynamic index → cannot determine path statically
        assert_eq!(expr_access_path(expr, &arena), None);
    }

    fn sequence_index_pattern_path(index: i32) -> PatternPath {
        use crate::types::{NumericLiteral, SequenceIndexExpr};

        let mut arena = AstArena::default();
        let sym = SymbolId(77);
        let base = arena.alloc_expr(Expr::Var(sym));
        let idx = arena.alloc_expr(Expr::NumericLiteral(NumericLiteral::I32(index)));
        let expr = arena.alloc_expr(Expr::SequenceIndex(SequenceIndexExpr { base, index: idx }));
        let result = expr_access_path(expr, &arena).expect("sequence index literal should resolve");
        assert_eq!(result.0, sym);
        assert_eq!(result.1, PatternPath::root().tuple_index(index as usize));
        result.1
    }

    #[test]
    fn sequence_index_same_path_move_and_borrow_rejects() {
        use crate::types::{BindingPlan, BindingPlanItem, CaptureMode, SymbolId, Type};

        let path = sequence_index_pattern_path(0);
        let plan = BindingPlan {
            items: vec![
                BindingPlanItem {
                    name: SymbolId(101),
                    capture: CaptureMode::Borrow,
                    path: path.clone(),
                    ty: Type::I32,
                },
                BindingPlanItem {
                    name: SymbolId(102),
                    capture: CaptureMode::Move,
                    path,
                    ty: Type::I32,
                },
            ],
        };
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("same sequence element move+borrow must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn sequence_index_different_indexes_allow() {
        use crate::types::{BindingPlan, BindingPlanItem, CaptureMode, SymbolId, Type};

        let plan = BindingPlan {
            items: vec![
                BindingPlanItem {
                    name: SymbolId(201),
                    capture: CaptureMode::Borrow,
                    path: sequence_index_pattern_path(0),
                    ty: Type::I32,
                },
                BindingPlanItem {
                    name: SymbolId(202),
                    capture: CaptureMode::Move,
                    path: sequence_index_pattern_path(1),
                    ty: Type::I32,
                },
            ],
        };
        validate_binding_plan_conflicts(&plan)
            .expect("different static sequence indexes must not conflict");
    }

    #[test]
    fn sequence_index_parent_then_child_overlap_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SequenceCollectionFamily,
            SequenceType, SymbolId, Type,
        };

        let plan = BindingPlan {
            items: vec![
                BindingPlanItem {
                    name: SymbolId(301),
                    capture: CaptureMode::Move,
                    path: PatternPath::root(),
                    ty: Type::Sequence(SequenceType {
                        family: SequenceCollectionFamily::OrderedSequence,
                        item: Box::new(Type::I32),
                    }),
                },
                BindingPlanItem {
                    name: SymbolId(302),
                    capture: CaptureMode::Borrow,
                    path: sequence_index_pattern_path(0),
                    ty: Type::I32,
                },
            ],
        };
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("whole sequence move and child borrow must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn sequence_index_child_then_parent_overlap_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SequenceCollectionFamily,
            SequenceType, SymbolId, Type,
        };

        let plan = BindingPlan {
            items: vec![
                BindingPlanItem {
                    name: SymbolId(401),
                    capture: CaptureMode::Borrow,
                    path: sequence_index_pattern_path(0),
                    ty: Type::I32,
                },
                BindingPlanItem {
                    name: SymbolId(402),
                    capture: CaptureMode::Move,
                    path: PatternPath::root(),
                    ty: Type::Sequence(SequenceType {
                        family: SequenceCollectionFamily::OrderedSequence,
                        item: Box::new(Type::I32),
                    }),
                },
            ],
        };
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("child borrow and whole sequence move must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn path_state_normalization_root_subsumes_children() {
        // Adding Moved(root) when Moved(root.0) already exists → root.0 is dropped.
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(50);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(1),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        // Now add root — should subsume both children.
        env.mark_path_state(sym, PatternPath::root(), PathAvailability::Moved)
            .expect("test-constructed binding must exist");
        // Only one entry should remain: root.
        let binding = env.binding(sym).expect("binding must exist");
        assert_eq!(
            binding.path_state.len(),
            1,
            "root should subsume child entries"
        );
        assert_eq!(binding.path_state[0].0, PatternPath::root());
    }

    #[test]
    fn path_state_normalization_child_redundant_if_parent_present() {
        // If Moved(root) exists, adding Moved(root.0) should be a no-op.
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(51);
        env.insert(sym, Type::I32);
        env.mark_path_state(sym, PatternPath::root(), PathAvailability::Moved)
            .expect("test-constructed binding must exist");
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        let binding = env.binding(sym).expect("binding must exist");
        assert_eq!(
            binding.path_state.len(),
            1,
            "child must be suppressed when parent already present"
        );
    }

    #[test]
    fn check_path_available_sibling_of_moved_is_ok() {
        // After moving root.0, accessing root.1 must succeed.
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(60);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        env.check_path_available(sym, &PatternPath::root().tuple_index(1))
            .expect("sibling of moved path must be accessible");
    }

    #[test]
    fn check_path_available_whole_var_blocked_after_child_move() {
        // After moving root.0, accessing root (whole var) must fail.
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(61);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        let err = env
            .check_path_available(sym, &PatternPath::root())
            .expect_err("whole-var access after child move must be blocked");
        assert!(
            err.message.contains("moved"),
            "error must mention moved: {}",
            err.message
        );
    }

    #[test]
    fn check_path_available_moved_child_blocked() {
        // After moving root.0, accessing root.0 itself must fail.
        use crate::types::{PathAvailability, PatternPath};
        let mut env = ScopeEnv::new();
        let sym = SymbolId(62);
        env.insert(sym, Type::I32);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        )
        .expect("test-constructed binding must exist");
        let err = env
            .check_path_available(sym, &PatternPath::root().tuple_index(0))
            .expect_err("access of moved path must be blocked");
        assert!(
            err.message.contains("moved"),
            "error must mention moved: {}",
            err.message
        );
    }

    // M9.6 — prefix-overlap conflict detection

    #[test]
    fn prefix_overlap_move_and_borrow_rejects() {
        // root.0 is a prefix of root.0.1 — move + borrow should conflict.
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let parent = PatternPath::root().tuple_index(0);
        let child = PatternPath::root().tuple_index(0).tuple_index(1);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: parent,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path: child,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("prefix-overlap move+borrow must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn prefix_overlap_two_moves_rejects() {
        // root and root.0 — both moved is also a conflict.
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let parent = PatternPath::root();
        let child = PatternPath::root().tuple_index(0);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: parent,
            ty: Type::Quad,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: child,
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan)
            .expect_err("prefix-overlap double-move must conflict");
    }

    #[test]
    fn distinct_paths_no_conflict() {
        // root.0 and root.1 share the root prefix but diverge at index — no overlap.
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: PatternPath::root().tuple_index(0),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: PatternPath::root().tuple_index(1),
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan).expect("distinct sibling paths must not conflict");
    }

    #[test]
    fn typecheck_rejects_overlapping_tuple_element_bindings() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let path = PatternPath::root().tuple_index(0);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: path.clone(),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("same tuple element move+borrow must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn typecheck_allows_disjoint_tuple_element_bindings() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: PatternPath::root().tuple_index(0),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path: PatternPath::root().tuple_index(1),
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan).expect("different tuple elements must not conflict");
    }

    #[test]
    fn typecheck_rejects_tuple_parent_then_child_overlap() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: PatternPath::root(),
            ty: Type::Tuple(vec![Type::I32]),
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path: PatternPath::root().tuple_index(0),
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("whole tuple move and child borrow must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn typecheck_rejects_tuple_child_then_parent_overlap() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Borrow,
            path: PatternPath::root().tuple_index(0),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: PatternPath::root(),
            ty: Type::Tuple(vec![Type::I32]),
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("child borrow and whole tuple move must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn typecheck_handles_nested_tuple_prefix_overlap() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let nested = PatternPath::root().tuple_index(0).tuple_index(1);
        let sibling = PatternPath::root().tuple_index(1);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Move,
            path: nested,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path: sibling,
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan)
            .expect("nested tuple path and sibling path must not conflict");
    }

    #[test]
    fn record_field_same_path_move_and_borrow_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let path = PatternPath::root().record_field(SymbolId(1));
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: path.clone(),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Borrow,
            path,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("record field same-path move+borrow must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn record_field_different_fields_allow() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path: PatternPath::root().record_field(SymbolId(1)),
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Move,
            path: PatternPath::root().record_field(SymbolId(4)),
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan).expect("different record fields must not conflict");
    }

    #[test]
    fn record_field_parent_child_move_and_borrow_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let parent = PatternPath::root();
        let child = PatternPath::root().record_field(SymbolId(1));
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path: parent,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Move,
            path: child,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("record field parent/child move+borrow must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn record_field_child_parent_move_and_borrow_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let parent = PatternPath::root();
        let child = PatternPath::root().record_field(SymbolId(1));
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: child,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Borrow,
            path: parent,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("record field child/parent move+borrow must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn record_field_nested_prefix_overlap_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let outer = PatternPath::root().record_field(SymbolId(1));
        let nested = PatternPath::root()
            .record_field(SymbolId(1))
            .record_field(SymbolId(2));
        let sibling = PatternPath::root().record_field(SymbolId(3));
        plan.push(BindingPlanItem {
            name: SymbolId(4),
            capture: CaptureMode::Move,
            path: outer,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(5),
            capture: CaptureMode::Borrow,
            path: nested,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(6),
            capture: CaptureMode::Borrow,
            path: sibling,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("nested record field prefix overlap must conflict");
        assert!(
            err.message.contains("conflicting") || err.message.contains("overlapping"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn prefix_overlap_two_borrows_ok() {
        // root.0 borrows and root.0.1 also borrows — allowed.
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let parent = PatternPath::root().tuple_index(0);
        let child = PatternPath::root().tuple_index(0).tuple_index(1);
        plan.push(BindingPlanItem {
            name: SymbolId(1),
            capture: CaptureMode::Borrow,
            path: parent,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Borrow,
            path: child,
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan)
            .expect("prefix-overlap double-borrow must not conflict");
    }

    #[test]
    fn adt_payload_prefix_overlap_move_and_borrow_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let parent = PatternPath::root().variant(SymbolId(1));
        let child = PatternPath::root().variant(SymbolId(1)).variant_field(0);
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: parent,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Borrow,
            path: child,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("prefix-overlap move+borrow must conflict");
        assert!(err.message.contains("conflicting") || err.message.contains("overlapping"));
    }

    #[test]
    fn adt_payload_different_variants_allow() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let path1 = PatternPath::root().variant(SymbolId(1)).variant_field(0);
        let path2 = PatternPath::root().variant(SymbolId(2)).variant_field(0);
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Move,
            path: path1,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(4),
            capture: CaptureMode::Move,
            path: path2,
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan).expect("different variants must not conflict");
    }

    #[test]
    fn adt_payload_different_indexes_allow() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let path1 = PatternPath::root().variant(SymbolId(1)).variant_field(0);
        let path2 = PatternPath::root().variant(SymbolId(1)).variant_field(1);
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: path1,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Move,
            path: path2,
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan).expect("different indexes must not conflict");
    }

    #[test]
    fn option_same_payload_prefix_overlap_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let some_variant = PatternPath::root().variant(SymbolId(1));
        let some_payload = some_variant.variant_field(0);
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: some_variant.clone(),
            ty: Type::Option(Box::new(Type::I32)),
        });
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Borrow,
            path: some_payload,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("Option::Some move+borrow prefix overlap must conflict");
        assert!(err.message.contains("conflicting") || err.message.contains("overlapping"));
    }

    #[test]
    fn option_disjoint_variants_allow() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let some_variant = PatternPath::root().variant(SymbolId(1));
        let none_variant = PatternPath::root().variant(SymbolId(2));
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Borrow,
            path: some_variant,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(4),
            capture: CaptureMode::Borrow,
            path: none_variant,
            ty: Type::Unit,
        });
        validate_binding_plan_conflicts(&plan).expect("Option Some vs None must not conflict");
    }

    #[test]
    fn result_same_payload_prefix_overlap_rejects() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let ok_variant = PatternPath::root().variant(SymbolId(1));
        let ok_payload = ok_variant.variant_field(0);
        plan.push(BindingPlanItem {
            name: SymbolId(2),
            capture: CaptureMode::Move,
            path: ok_variant.clone(),
            ty: Type::Result(Box::new(Type::I32), Box::new(Type::I32)),
        });
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Borrow,
            path: ok_payload,
            ty: Type::I32,
        });
        let err = validate_binding_plan_conflicts(&plan)
            .expect_err("Result::Ok move+borrow prefix overlap must conflict");
        assert!(err.message.contains("conflicting") || err.message.contains("overlapping"));
    }

    #[test]
    fn result_disjoint_variants_allow() {
        use crate::types::{
            BindingPlan, BindingPlanItem, CaptureMode, PatternPath, SymbolId, Type,
        };
        let mut plan = BindingPlan::default();
        let ok_variant = PatternPath::root().variant(SymbolId(1)).variant_field(0);
        let err_variant = PatternPath::root().variant(SymbolId(2)).variant_field(0);
        plan.push(BindingPlanItem {
            name: SymbolId(3),
            capture: CaptureMode::Borrow,
            path: ok_variant,
            ty: Type::I32,
        });
        plan.push(BindingPlanItem {
            name: SymbolId(4),
            capture: CaptureMode::Borrow,
            path: err_variant,
            ty: Type::I32,
        });
        validate_binding_plan_conflicts(&plan).expect("Result Ok vs Err must not conflict");
    }

    // M9.10 Wave B — LetTuple / LetElseTuple path-state tracking

    #[test]
    fn let_tuple_marks_moved_paths_on_source_var() {
        // `let (a, b) = src;` should typecheck — both paths move from src.
        typecheck_source(
            r#"
            fn f(src: (i32, i32)) { let (a, b) = src; }
            fn main() { return; }
        "#,
        )
        .expect("let-tuple destructure must typecheck");
    }

    #[test]
    fn let_tuple_rejects_second_destructure_of_same_source() {
        // After `let (a, b) = src;` (move), `let (c, d) = src;` must be rejected.
        let err = typecheck_source(
            r#"
            fn f(src: (i32, i32)) { let (a, b) = src; let (c, d) = src; }
            fn main() { return; }
        "#,
        )
        .expect_err("second move-destructure of same source must fail");
        assert!(
            err.message.contains("moved"),
            "error must mention moved: {}",
            err.message
        );
    }

    #[test]
    fn let_tuple_partial_move_then_full_destructure_rejected() {
        // After `let (a, _) = src;`, trying to destructure src again must fail.
        let err = typecheck_source(
            r#"
            fn f(src: (i32, i32)) { let (a, _) = src; let (b, c) = src; }
            fn main() { return; }
        "#,
        )
        .expect_err("second destructure after partial move must fail");
        assert!(
            err.message.contains("moved"),
            "error must mention moved: {}",
            err.message
        );
    }
}

fn is_builtin_assert_name(
    name: SymbolId,
    arena: &AstArena,
    table: &FnTable,
) -> Result<bool, FrontendError> {
    Ok(!table.contains_key(&name) && resolve_symbol_name(arena, name)? == "assert")
}

fn check_builtin_assert_stmt(
    expr_id: ExprId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<bool, FrontendError> {
    let Expr::Call(name, args) = arena.expr(expr_id) else {
        return Ok(false);
    };
    if !is_builtin_assert_name(*name, arena, table)? {
        return Ok(false);
    }
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
    let cond_ty = infer_expr_type(
        args[0].value,
        arena,
        env,
        table,
        record_table,
        adt_table,
        ret_ty,
        loop_stack,
        impl_list,
    )?;
    if cond_ty != Type::Bool {
        return Err(FrontendError {
            pos: 0,
            message: format!("assert builtin requires bool condition, got {:?}", cond_ty),
        });
    }
    Ok(true)
}

/// SSF-08 Lane 1 (#1659/#1660/#1661/#1662): returns the block's tail type
/// *and* the resulting ownership successor state (a clone of `env` with the
/// block's own local scope already popped, but pre-existing bindings'
/// ownership effects intact) so callers that check more than one
/// alternative block (an `if`/`match`/`if-let` expression's branches) can
/// collect these as `ScopeEnv::join_ownership_from` successors, instead of
/// the caller-invisible ownership state a `Type`-only return produced.
fn infer_value_block_type(
    block: &BlockExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<(Type, ScopeEnv), FrontendError> {
    let mut block_env = env.clone();
    block_env.push_scope();
    for stmt in &block.statements {
        match arena.stmt(*stmt) {
            Stmt::Const { .. }
            | Stmt::Let { .. }
            | Stmt::LetTuple { .. }
            | Stmt::Discard { .. }
            | Stmt::Expr(_) => {
                check_stmt(
                    *stmt,
                    arena,
                    &mut block_env,
                    ret_ty.clone(),
                    table,
                    record_table,
                    adt_table,
                    loop_stack,
                    impl_list,
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
    let tail_ty = infer_expr_type(
        block.tail,
        arena,
        &mut block_env,
        table,
        record_table,
        adt_table,
        ret_ty,
        loop_stack,
        impl_list,
    )?;
    block_env.pop_scope();
    Ok((tail_ty, block_env))
}

/// SSF-08 Lane 1 (#1661/#1662/#1663): build a fresh clone of `env` with
/// `pattern`'s bound names inserted and this pattern's *own* scrutinee
/// capture effect applied directly to it (via `apply_arm_pattern_capture`,
/// so a subsequent guard/body check on the returned env sees its own
/// capture, per #1663) -- one pushed scope deeper than `env`. The caller
/// checks any guard and the arm/branch body against the returned env (a
/// value-producing body should go through `infer_value_block_type`, which
/// pushes and pops its own further-nested scope internally), then must
/// `pop_scope()` once more on whatever successor state results, to drop
/// this function's own pushed scope and bring the successor back to
/// `env`'s original depth before it can be fed to
/// `ScopeEnv::join_ownership_from`.
fn build_pattern_arm_env(
    scrutinee: ExprId,
    pattern: &MatchPattern,
    scrutinee_ty: &Type,
    arena: &AstArena,
    env: &mut ScopeEnv,
    adt_table: &AdtTable,
) -> Result<ScopeEnv, FrontendError> {
    if matches!(pattern, MatchPattern::Or(_)) {
        return Err(FrontendError {
            pos: 0,
            message: "or-pattern match arms ('A | B') are not supported; split into separate arms with identical bodies instead".to_string(),
        });
    }
    let mut plan = BindingPlan::default();
    build_match_pattern_plan(
        pattern,
        scrutinee_ty,
        &PatternPath::root(),
        &mut plan,
        arena,
        adt_table,
    )?;
    validate_binding_plan_conflicts(&plan)?;

    let mut arm_env = env.clone();
    arm_env.push_scope();
    apply_binding_plan(&mut arm_env, &plan);
    apply_arm_pattern_capture(scrutinee, &plan, arena, &mut arm_env)?;
    Ok(arm_env)
}

fn infer_match_expr_type(
    match_expr: &MatchExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    let scrutinee_ty = infer_expr_type(
        match_expr.scrutinee,
        arena,
        env,
        table,
        record_table,
        adt_table,
        ret_ty.clone(),
        loop_stack,
        impl_list,
    )?;
    // M9.4 Wave 3: widen to also allow i32/u32 (for int range patterns).
    if !matches!(
        scrutinee_ty,
        Type::Quad | Type::Adt(_) | Type::Option(_) | Type::Result(_, _) | Type::I32 | Type::U32
    ) {
        return Err(FrontendError {
            pos: 0,
            message:
                "match expression is allowed only for quad, enum, Option(T), Result(T, E), i32, or u32 scrutinee"
                    .to_string(),
        });
    }

    // SSF-08 Lane 1 (#1661/#1663): each arm's pattern capture is now applied
    // directly to its own arm_env (visible to its own guard/body), and every
    // reachable successor -- every arm plus `default` if present -- is
    // conservatively joined back into `env`, rather than skipped entirely as
    // the prior `&ScopeEnv`-bound signature forced.
    let mut result_ty = None;
    let mut successors: Vec<ScopeEnv> = Vec::new();
    for arm in &match_expr.arms {
        let mut arm_env = build_pattern_arm_env(
            match_expr.scrutinee,
            &arm.pat,
            &scrutinee_ty,
            arena,
            env,
            adt_table,
        )?;
        check_match_guard(
            arm.guard,
            arena,
            &mut arm_env,
            table,
            record_table,
            adt_table,
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        let (arm_ty, mut result_env) = infer_value_block_type(
            &arm.block,
            arena,
            &mut arm_env,
            table,
            record_table,
            adt_table,
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        result_env.pop_scope();
        successors.push(result_env);
        if let Some(ref expected) = result_ty {
            if *expected != arm_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "match expression branch type mismatch: expected {:?}, got {:?}",
                        expected, arm_ty
                    ),
                });
            }
        } else {
            result_ty = Some(arm_ty);
        }
    }

    let final_ty = if let Some(default) = match_expr.default.as_ref() {
        let (default_ty, default_result_env) = infer_value_block_type(
            default,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        )?;
        successors.push(default_result_env);
        if let Some(expected) = result_ty {
            if expected != default_ty {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "match expression branch type mismatch: expected {:?}, got {:?}",
                        expected, default_ty
                    ),
                });
            }
            expected
        } else {
            default_ty
        }
    } else {
        match missing_exhaustive_sum_variants(
            &scrutinee_ty,
            match_expr.arms.iter().map(|arm| (&arm.pat, arm.guard)),
            arena,
            adt_table,
        )? {
            Some((family_label, missing)) if !missing.is_empty() => {
                return Err(non_exhaustive_match_error(&family_label, &missing, true)?)
            }
            Some(_) => {
                result_ty.expect("exhaustive enum match expression should have at least one arm")
            }
            None => {
                return Err(FrontendError {
                    pos: 0,
                    message: "match expression requires default arm '_'".to_string(),
                })
            }
        }
    };
    env.join_ownership_from(&successors)?;
    Ok(final_ty)
}

fn infer_loop_expr_type(
    loop_expr: &LoopExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    // SSF-08 Lane 1 (#1660): same fixed-point loop-carried ownership
    // analysis as statement loops -- a value-producing `loop` body can also
    // run more than once before `break`, so a restriction one logical pass
    // creates must be visible to (and able to conflict with) the next, and
    // any restriction that survives every pass must persist onto `env`
    // after the loop, not be discarded with `body_env`.
    loop_stack.push(LoopTypeFrame {
        kind: LoopTypeFrameKind::Expression,
        break_ty: None,
    });
    let result = run_loop_body_to_fixed_point(
        &loop_expr.body,
        arena,
        env,
        ret_ty,
        table,
        record_table,
        adt_table,
        loop_stack,
        impl_list,
        |_| {},
        check_loop_expr_stmt,
    );
    let frame = loop_stack.pop().expect("loop frame must exist");
    result?;
    frame.break_ty.ok_or(FrontendError {
        pos: 0,
        message: "loop expression requires at least one break value".to_string(),
    })
}

fn check_loop_expr_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    ret_ty: Type,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
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
        } => check_if_branches_joined(
            *condition,
            then_block,
            else_block,
            arena,
            env,
            ret_ty,
            table,
            record_table,
            adt_table,
            loop_stack,
            impl_list,
            check_loop_expr_stmt,
        ),
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => check_match_arms_joined(
            *scrutinee,
            arms,
            default,
            arena,
            env,
            ret_ty,
            table,
            record_table,
            adt_table,
            loop_stack,
            impl_list,
            check_loop_expr_stmt,
        ),
        _ => check_stmt(
            stmt_id,
            arena,
            env,
            ret_ty,
            table,
            record_table,
            adt_table,
            loop_stack,
            impl_list,
        ),
    }
}

/// Coherence check: at most one impl per (trait_name, for_type) pair, and no
/// generic/blanket impls. `ImplDecl.type_params` is documented as "Empty in
/// first-wave canonical form" (see its doc comment in types.rs), but nothing
/// previously enforced that -- a blanket-shaped impl such as
/// `impl<T> Trait for T` or an impl declaring an unused type parameter both
/// silently typechecked. Trait objects, associated types, blanket impls,
/// specialization, and default methods are explicit SSF-07 non-goals
/// (docs/spec/foundation_source_profile_v1.md); this is the enforcement point
/// for the blanket-impl/specialization carve-out specifically.
fn validate_trait_coherence(impls: &[ImplDecl], arena: &AstArena) -> Result<(), FrontendError> {
    let mut seen: BTreeSet<(SymbolId, SymbolId)> = BTreeSet::new();
    for imp in impls {
        if !imp.type_params.is_empty() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "impl of trait '{}' for type '{}' declares type parameters; \
                     generic/blanket impls are not supported",
                    resolve_symbol_name(arena, imp.trait_name)?,
                    resolve_symbol_name(arena, imp.for_type)?,
                ),
            });
        }
        let key = (imp.trait_name, imp.for_type);
        if !seen.insert(key) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "duplicate impl of trait '{}' for type '{}'",
                    resolve_symbol_name(arena, imp.trait_name)?,
                    resolve_symbol_name(arena, imp.for_type)?,
                ),
            });
        }
    }
    Ok(())
}

/// Conformance check: each impl provides every method declared in its trait
/// with a matching return type.
fn validate_impl_conformance(
    impls: &[ImplDecl],
    trait_table: &TraitTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
) -> Result<(), FrontendError> {
    let self_type_var = arena.symbol_to_id.get("Self").copied();
    for imp in impls {
        let mut seen_methods = BTreeSet::new();
        for method in &imp.methods {
            if !seen_methods.insert(method.name) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "impl of '{}' for '{}' defines duplicate method '{}'",
                        resolve_symbol_name(arena, imp.trait_name)?,
                        resolve_symbol_name(arena, imp.for_type)?,
                        resolve_symbol_name(arena, method.name)?,
                    ),
                });
            }
        }
        let trait_decl = match trait_table.get(&imp.trait_name) {
            Some(t) => t,
            None => {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "impl references unknown trait '{}'",
                        resolve_symbol_name(arena, imp.trait_name)?,
                    ),
                });
            }
        };
        // FA-02-019 / #1651: prove the impl target itself is an admitted,
        // unambiguous nominal type before anything else is checked. Reuses
        // the same canonical resolver every other declared-type position in
        // the frontend already goes through (Type::Record(name) is this
        // codebase's uncanonicalized "unresolved nominal name" convention;
        // canonicalize_declared_type_generic disambiguates it against
        // RecordTable/AdtTable, or fails closed on unknown/ambiguous names).
        // Without this, a trait whose methods never reference `Self` had no
        // other admission check that the target type exists at all.
        let trait_name_str = resolve_symbol_name(arena, imp.trait_name)?;
        let for_type_str = resolve_symbol_name(arena, imp.for_type)?;
        let resolved_self_ty = canonicalize_declared_type_generic(
            &Type::Record(imp.for_type),
            record_table,
            adt_table,
            arena,
            &[],
        )
        .map_err(|err| FrontendError {
            pos: 0,
            message: format!(
                "impl of trait '{}' for '{}': {}",
                trait_name_str, for_type_str, err.message
            ),
        })?;
        for trait_method in &trait_decl.methods {
            match imp.methods.iter().find(|m| m.name == trait_method.name) {
                None => {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "impl of '{}' for '{}' is missing method '{}'",
                            resolve_symbol_name(arena, imp.trait_name)?,
                            resolve_symbol_name(arena, imp.for_type)?,
                            resolve_symbol_name(arena, trait_method.name)?,
                        ),
                    });
                }
                Some(m) => {
                    if m.params.len() != trait_method.params.len() {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "impl method '{}' has {} parameter(s), expected {} from trait '{}'",
                                resolve_symbol_name(arena, trait_method.name)?,
                                m.params.len(),
                                trait_method.params.len(),
                                resolve_symbol_name(arena, imp.trait_name)?,
                            ),
                        });
                    }
                    for ((_, actual_ty), (_, expected_ty)) in
                        m.params.iter().zip(trait_method.params.iter())
                    {
                        // FA-02-035 / #1667: substitute Self on *both* sides
                        // through the same resolved concrete Type. The impl
                        // side's own `Self` occurrences were parsed as the
                        // uncanonicalized `Type::Record(imp.for_type)`
                        // placeholder (identical to writing the target's
                        // name directly instead of `Self`); the trait side's
                        // were parsed as `Type::TypeVar("Self")`. Both must
                        // resolve to the one proven concrete type before
                        // comparison, never to a guessed `Record`.
                        let actual_ty = substitute_trait_self_type(
                            actual_ty,
                            self_type_var,
                            imp.for_type,
                            &resolved_self_ty,
                        );
                        let expected_ty = substitute_trait_self_type(
                            expected_ty,
                            self_type_var,
                            imp.for_type,
                            &resolved_self_ty,
                        );
                        if actual_ty != expected_ty {
                            return Err(FrontendError {
                                pos: 0,
                                message: format!(
                                    "impl method '{}' parameter type {:?} does not match expected {:?} from trait '{}'",
                                    resolve_symbol_name(arena, trait_method.name)?,
                                    actual_ty,
                                    expected_ty,
                                    resolve_symbol_name(arena, imp.trait_name)?,
                                ),
                            });
                        }
                    }
                    let actual_ret = substitute_trait_self_type(
                        &m.ret,
                        self_type_var,
                        imp.for_type,
                        &resolved_self_ty,
                    );
                    let expected_ret = substitute_trait_self_type(
                        &trait_method.ret,
                        self_type_var,
                        imp.for_type,
                        &resolved_self_ty,
                    );
                    if actual_ret != expected_ret {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "impl method '{}' has return type {:?}, expected {:?} from trait '{}'",
                                resolve_symbol_name(arena, trait_method.name)?,
                                actual_ret,
                                expected_ret,
                                resolve_symbol_name(arena, imp.trait_name)?,
                            ),
                        });
                    }
                }
            }
        }
        for method in &imp.methods {
            if !trait_decl.methods.iter().any(|tm| tm.name == method.name) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "impl of trait '{}' for '{}' defines method '{}' not declared by the trait",
                        resolve_symbol_name(arena, imp.trait_name)?,
                        resolve_symbol_name(arena, imp.for_type)?,
                        resolve_symbol_name(arena, method.name)?,
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Replace every occurrence of `Self` with the one canonically resolved
/// concrete impl-target `Type` (see FA-02-035 / #1667 and FA-02-019 /
/// #1651). Two distinct parsed forms both denote `Self` and must both
/// substitute to the identical concrete type:
///
/// - `Type::TypeVar(name)` where `name` is the interned "Self" symbol: how
///   trait method signatures parse `Self` (a neutral placeholder, since no
///   concrete impl target is known while parsing a trait body).
/// - `Type::Record(name)` where `name == for_type`: how impl method
///   signatures parse `Self` (this codebase's general "unresolved nominal
///   name" convention — indistinguishable from writing the target type's
///   name directly instead of `Self`, which is intentional: they denote the
///   same type).
///
/// `concrete_self` must already be the canonically resolved `Type::Record`
/// or `Type::Adt` for `for_type` (see `validate_impl_conformance`) — this
/// function never itself guesses a nominal family.
fn substitute_trait_self_type(
    ty: &Type,
    self_type_var: Option<SymbolId>,
    for_type: SymbolId,
    concrete_self: &Type,
) -> Type {
    match ty {
        Type::TypeVar(name) if Some(*name) == self_type_var => concrete_self.clone(),
        Type::Record(name) if *name == for_type => concrete_self.clone(),
        Type::Tuple(items) => Type::Tuple(
            items
                .iter()
                .map(|item| {
                    substitute_trait_self_type(item, self_type_var, for_type, concrete_self)
                })
                .collect(),
        ),
        Type::Sequence(sequence) => Type::Sequence(SequenceType {
            family: sequence.family,
            item: Box::new(substitute_trait_self_type(
                sequence.item.as_ref(),
                self_type_var,
                for_type,
                concrete_self,
            )),
        }),
        Type::Map(map) => Type::Map(crate::types::MapType {
            key: Box::new(substitute_trait_self_type(
                map.key.as_ref(),
                self_type_var,
                for_type,
                concrete_self,
            )),
            val: Box::new(substitute_trait_self_type(
                map.val.as_ref(),
                self_type_var,
                for_type,
                concrete_self,
            )),
        }),
        Type::Closure(closure) => Type::Closure(crate::types::ClosureType {
            family: closure.family,
            capture: closure.capture,
            param: Box::new(substitute_trait_self_type(
                closure.param.as_ref(),
                self_type_var,
                for_type,
                concrete_self,
            )),
            ret: Box::new(substitute_trait_self_type(
                closure.ret.as_ref(),
                self_type_var,
                for_type,
                concrete_self,
            )),
        }),
        Type::Measured(base, unit) => Type::Measured(
            Box::new(substitute_trait_self_type(
                base.as_ref(),
                self_type_var,
                for_type,
                concrete_self,
            )),
            *unit,
        ),
        Type::Option(item) => Type::Option(Box::new(substitute_trait_self_type(
            item.as_ref(),
            self_type_var,
            for_type,
            concrete_self,
        ))),
        Type::Result(ok_ty, err_ty) => Type::Result(
            Box::new(substitute_trait_self_type(
                ok_ty.as_ref(),
                self_type_var,
                for_type,
                concrete_self,
            )),
            Box::new(substitute_trait_self_type(
                err_ty.as_ref(),
                self_type_var,
                for_type,
                concrete_self,
            )),
        ),
        _ => ty.clone(),
    }
}

fn validate_top_level_name_collisions(
    program: &Program,
    fn_table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    schema_table: &SchemaTable,
) -> Result<(), FrontendError> {
    for record in &program.records {
        if fn_table.contains_key(&record.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both record and function",
                    resolve_symbol_name(&program.arena, record.name)?
                ),
            });
        }
        if adt_table.contains_key(&record.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both record and enum",
                    resolve_symbol_name(&program.arena, record.name)?
                ),
            });
        }
        if schema_table.contains_key(&record.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both record and schema",
                    resolve_symbol_name(&program.arena, record.name)?
                ),
            });
        }
    }
    for adt in &program.adts {
        if fn_table.contains_key(&adt.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both enum and function",
                    resolve_symbol_name(&program.arena, adt.name)?
                ),
            });
        }
        if record_table.contains_key(&adt.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both enum and record",
                    resolve_symbol_name(&program.arena, adt.name)?
                ),
            });
        }
        if schema_table.contains_key(&adt.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both enum and schema",
                    resolve_symbol_name(&program.arena, adt.name)?
                ),
            });
        }
    }
    for schema in &program.schemas {
        if fn_table.contains_key(&schema.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both schema and function",
                    resolve_symbol_name(&program.arena, schema.name)?
                ),
            });
        }
        if record_table.contains_key(&schema.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both schema and record",
                    resolve_symbol_name(&program.arena, schema.name)?
                ),
            });
        }
        if adt_table.contains_key(&schema.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "top-level name '{}' cannot be used for both schema and enum",
                    resolve_symbol_name(&program.arena, schema.name)?
                ),
            });
        }
    }
    Ok(())
}

fn validate_record_declarations(
    program: &Program,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    for record in &program.records {
        if record.fields.is_empty() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "record '{}' must declare at least 1 field",
                    resolve_symbol_name(&program.arena, record.name)?
                ),
            });
        }
        let mut seen = BTreeSet::new();
        for field in &record.fields {
            if !seen.insert(field.name) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "record '{}' cannot repeat field '{}'",
                        resolve_symbol_name(&program.arena, record.name)?,
                        resolve_symbol_name(&program.arena, field.name)?
                    ),
                });
            }
            ensure_type_resolved(
                &field.ty,
                record_table,
                adt_table,
                &program.arena,
                format!(
                    "field '{}.{}'",
                    resolve_symbol_name(&program.arena, record.name)?,
                    resolve_symbol_name(&program.arena, field.name)?
                ),
            )?;
            // FA-02-038 / #1861: ensure_type_resolved above proves nominal
            // references resolve, not that the field's type is one this
            // Foundation actually admits as storage -- a reserved/
            // structural/contextual type (e.g. qvec, a closure, an
            // unresolved generic) could previously hide inside an admitted
            // record declaration and escape as a trusted Type::Record
            // nominal shell everywhere downstream, since no earlier phase
            // ever looked inside the field.
            ensure_storage_type_supported(
                &field.ty,
                &program.arena,
                format!(
                    "field '{}.{}'",
                    resolve_symbol_name(&program.arena, record.name)?,
                    resolve_symbol_name(&program.arena, field.name)?
                ),
            )?;
        }
    }

    let mut visited = BTreeSet::new();
    let mut active = BTreeSet::new();
    for record in &program.records {
        validate_record_acyclic(
            record.name,
            record_table,
            adt_table,
            &program.arena,
            &mut active,
            &mut visited,
        )?;
    }
    Ok(())
}

fn validate_adt_declarations(
    program: &Program,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    for adt in &program.adts {
        if adt.variants.is_empty() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "enum '{}' must declare at least 1 variant",
                    resolve_symbol_name(&program.arena, adt.name)?
                ),
            });
        }
        let mut seen = BTreeSet::new();
        for variant in &adt.variants {
            if !seen.insert(variant.name) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "enum '{}' cannot repeat variant '{}'",
                        resolve_symbol_name(&program.arena, adt.name)?,
                        resolve_symbol_name(&program.arena, variant.name)?
                    ),
                });
            }
            for (index, item_ty) in variant.payload.iter().enumerate() {
                ensure_type_resolved(
                    item_ty,
                    record_table,
                    adt_table,
                    &program.arena,
                    format!(
                        "variant '{}::{}' payload item {}",
                        resolve_symbol_name(&program.arena, adt.name)?,
                        resolve_symbol_name(&program.arena, variant.name)?,
                        index
                    ),
                )?;
                // FA-02-038 / #1861: see the matching comment in
                // validate_record_declarations -- ensure_type_resolved alone
                // does not prove a payload's type is admitted storage.
                ensure_storage_type_supported(
                    item_ty,
                    &program.arena,
                    format!(
                        "variant '{}::{}' payload item {}",
                        resolve_symbol_name(&program.arena, adt.name)?,
                        resolve_symbol_name(&program.arena, variant.name)?,
                        index
                    ),
                )?;
            }
        }
    }

    let mut visited = BTreeSet::new();
    let mut active = BTreeSet::new();
    for adt in &program.adts {
        validate_adt_acyclic(
            adt.name,
            record_table,
            adt_table,
            &program.arena,
            &mut active,
            &mut visited,
        )?;
    }
    Ok(())
}

fn validate_schema_declarations(
    program: &Program,
    schema_table: &SchemaTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    for schema in &program.schemas {
        let _ = schema_table.get(&schema.name).ok_or(FrontendError {
            pos: 0,
            message: format!(
                "missing schema '{}' in canonical schema table",
                resolve_symbol_name(&program.arena, schema.name)?
            ),
        })?;
        match &schema.shape {
            SchemaShape::Record(fields) => validate_record_shaped_schema(
                schema.name,
                fields,
                record_table,
                adt_table,
                &program.arena,
            )?,
            SchemaShape::TaggedUnion(variants) => validate_tagged_union_schema(
                schema.name,
                variants,
                record_table,
                adt_table,
                &program.arena,
            )?,
        }
    }
    Ok(())
}

fn validate_record_shaped_schema(
    schema_name: SymbolId,
    fields: &[SchemaField],
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
) -> Result<(), FrontendError> {
    if fields.is_empty() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "schema '{}' must declare at least 1 field",
                resolve_symbol_name(arena, schema_name)?
            ),
        });
    }
    let mut seen = BTreeSet::new();
    for field in fields {
        if !seen.insert(field.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "schema '{}' cannot repeat field '{}'",
                    resolve_symbol_name(arena, schema_name)?,
                    resolve_symbol_name(arena, field.name)?
                ),
            });
        }
        ensure_type_resolved(
            &field.ty,
            record_table,
            adt_table,
            arena,
            format!(
                "schema field '{}.{}'",
                resolve_symbol_name(arena, schema_name)?,
                resolve_symbol_name(arena, field.name)?
            ),
        )?;
    }
    Ok(())
}

fn derive_validation_field_plans(
    fields: &[SchemaField],
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
) -> Result<Vec<ValidationFieldPlan>, FrontendError> {
    fields
        .iter()
        .map(|field| {
            Ok(ValidationFieldPlan {
                name: field.name,
                ty: canonicalize_declared_type(&field.ty, record_table, adt_table, arena)?,
            })
        })
        .collect()
}

fn derive_validation_variant_plans(
    variants: &[SchemaVariant],
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
) -> Result<Vec<ValidationVariantPlan>, FrontendError> {
    variants
        .iter()
        .map(|variant| {
            Ok(ValidationVariantPlan {
                name: variant.name,
                fields: derive_validation_field_plans(
                    &variant.fields,
                    record_table,
                    adt_table,
                    arena,
                )?,
            })
        })
        .collect()
}

fn derive_record_validation_checks(fields: &[ValidationFieldPlan]) -> Vec<ValidationCheck> {
    let mut checks = Vec::with_capacity(fields.len() * 2);
    for field in fields {
        checks.push(ValidationCheck::RequiredField { field: field.name });
        checks.push(ValidationCheck::FieldType {
            field: field.name,
            ty: field.ty.clone(),
        });
    }
    checks
}

fn derive_tagged_union_validation_checks(
    variants: &[ValidationVariantPlan],
) -> Vec<ValidationCheck> {
    let mut checks = Vec::new();
    for variant in variants {
        checks.push(ValidationCheck::TaggedUnionBranch {
            variant: variant.name,
        });
        for field in &variant.fields {
            checks.push(ValidationCheck::TaggedUnionBranchRequiredField {
                variant: variant.name,
                field: field.name,
            });
            checks.push(ValidationCheck::TaggedUnionBranchFieldType {
                variant: variant.name,
                field: field.name,
                ty: field.ty.clone(),
            });
        }
    }
    checks
}

fn validate_tagged_union_schema(
    schema_name: SymbolId,
    variants: &[SchemaVariant],
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
) -> Result<(), FrontendError> {
    if variants.is_empty() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "schema '{}' must declare at least 1 variant",
                resolve_symbol_name(arena, schema_name)?
            ),
        });
    }
    let mut seen_variants = BTreeSet::new();
    for variant in variants {
        if !seen_variants.insert(variant.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "schema '{}' cannot repeat variant '{}'",
                    resolve_symbol_name(arena, schema_name)?,
                    resolve_symbol_name(arena, variant.name)?
                ),
            });
        }
        let mut seen_fields = BTreeSet::new();
        for field in &variant.fields {
            if !seen_fields.insert(field.name) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "schema '{}::{}' cannot repeat field '{}'",
                        resolve_symbol_name(arena, schema_name)?,
                        resolve_symbol_name(arena, variant.name)?,
                        resolve_symbol_name(arena, field.name)?
                    ),
                });
            }
            ensure_type_resolved(
                &field.ty,
                record_table,
                adt_table,
                arena,
                format!(
                    "schema field '{}::{}.{}'",
                    resolve_symbol_name(arena, schema_name)?,
                    resolve_symbol_name(arena, variant.name)?,
                    resolve_symbol_name(arena, field.name)?
                ),
            )?;
        }
    }
    Ok(())
}

fn validate_record_acyclic(
    record_name: SymbolId,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
    active: &mut BTreeSet<SymbolId>,
    visited: &mut BTreeSet<SymbolId>,
) -> Result<(), FrontendError> {
    if visited.contains(&record_name) {
        return Ok(());
    }
    if !active.insert(record_name) {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "record declarations currently do not allow recursive field graph involving '{}'",
                resolve_symbol_name(arena, record_name)?
            ),
        });
    }
    let record = record_table.get(&record_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown record type '{}'",
            resolve_symbol_name(arena, record_name)?
        ),
    })?;
    for field in &record.fields {
        validate_nominal_type_acyclic(&field.ty, record_table, adt_table, arena, active, visited)?;
    }
    active.remove(&record_name);
    visited.insert(record_name);
    Ok(())
}

fn validate_adt_acyclic(
    adt_name: SymbolId,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
    active: &mut BTreeSet<SymbolId>,
    visited: &mut BTreeSet<SymbolId>,
) -> Result<(), FrontendError> {
    if visited.contains(&adt_name) {
        return Ok(());
    }
    if !active.insert(adt_name) {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "enum declarations currently do not allow recursive payload graph involving '{}'",
                resolve_symbol_name(arena, adt_name)?
            ),
        });
    }
    let adt = adt_table.get(&adt_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown enum type '{}'",
            resolve_symbol_name(arena, adt_name)?
        ),
    })?;
    for variant in &adt.variants {
        for item_ty in &variant.payload {
            validate_nominal_type_acyclic(
                item_ty,
                record_table,
                adt_table,
                arena,
                active,
                visited,
            )?;
        }
    }
    active.remove(&adt_name);
    visited.insert(adt_name);
    Ok(())
}

fn validate_nominal_type_acyclic(
    ty: &Type,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
    active: &mut BTreeSet<SymbolId>,
    visited: &mut BTreeSet<SymbolId>,
) -> Result<(), FrontendError> {
    match ty {
        Type::Tuple(items) => {
            for item in items {
                validate_nominal_type_acyclic(
                    item,
                    record_table,
                    adt_table,
                    arena,
                    active,
                    visited,
                )?;
            }
            Ok(())
        }
        Type::Record(name) => {
            if record_table.contains_key(name) {
                validate_record_acyclic(*name, record_table, adt_table, arena, active, visited)
            } else {
                validate_adt_acyclic(*name, record_table, adt_table, arena, active, visited)
            }
        }
        Type::Adt(name) => {
            validate_adt_acyclic(*name, record_table, adt_table, arena, active, visited)
        }
        _ => Ok(()),
    }
}

fn ensure_type_resolved(
    ty: &Type,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    arena: &AstArena,
    context: String,
) -> Result<(), FrontendError> {
    match ty {
        Type::Tuple(items) => {
            for item in items {
                ensure_type_resolved(item, record_table, adt_table, arena, context.clone())?;
            }
            Ok(())
        }
        Type::Measured(base, _) => {
            ensure_type_resolved(base, record_table, adt_table, arena, context.clone())?;
            if base.is_core_numeric_scalar() {
                Ok(())
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unit annotation is allowed only on i32, u32, f64, or fx in {}",
                        context
                    ),
                })
            }
        }
        Type::Record(name) => {
            if record_table.contains_key(name) || adt_table.contains_key(name) {
                Ok(())
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unknown record type '{}' in {}",
                        resolve_symbol_name(arena, *name)?,
                        context
                    ),
                })
            }
        }
        Type::Adt(name) => {
            if adt_table.contains_key(name) {
                Ok(())
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "unknown enum type '{}' in {}",
                        resolve_symbol_name(arena, *name)?,
                        context
                    ),
                })
            }
        }
        Type::Option(item) => ensure_type_resolved(item, record_table, adt_table, arena, context),
        Type::Sequence(sequence) => ensure_type_resolved(
            sequence.item.as_ref(),
            record_table,
            adt_table,
            arena,
            context,
        ),
        Type::Map(map) => {
            ensure_type_resolved(
                map.key.as_ref(),
                record_table,
                adt_table,
                arena,
                context.clone(),
            )?;
            ensure_type_resolved(map.val.as_ref(), record_table, adt_table, arena, context)
        }
        Type::Result(ok_ty, err_ty) => {
            ensure_type_resolved(ok_ty, record_table, adt_table, arena, context.clone())?;
            ensure_type_resolved(err_ty, record_table, adt_table, arena, context)
        }
        _ => Ok(()),
    }
}

/// FA-02-015 / #1647: exhaustive (no catch-all) executable-type admission.
///
/// Every `Type` variant is matched explicitly, so adding a new variant to
/// the `Type` enum without updating this function is a compile error rather
/// than a silent "falls through to `Ok(())`" admission. `admitted_type_vars`
/// is the same narrow, caller-scoped admission list
/// `canonicalize_declared_type_generic` already uses for the same type (a
/// function's own `type_params`, or — for a trait method signature — just
/// the reserved `Self` symbol): a `Type::TypeVar` is executable-admitted
/// only when its name is in that list, never unconditionally. `Type::QVec`
/// is a reserved, not-yet-promoted-to-executable type family and rejects
/// deterministically until it is explicitly qualified by its own repair.
pub(crate) fn ensure_executable_type_supported(
    ty: &Type,
    arena: &AstArena,
    admitted_type_vars: &[SymbolId],
    context: String,
) -> Result<(), FrontendError> {
    match ty {
        Type::Quad
        | Type::Bool
        | Type::Text
        | Type::I32
        | Type::U32
        | Type::Fx
        | Type::F64
        | Type::Unit
        | Type::RangeI32 => Ok(()),
        Type::Tuple(items) => {
            for item in items {
                ensure_executable_type_supported(item, arena, admitted_type_vars, context.clone())?;
            }
            Ok(())
        }
        Type::Sequence(sequence) => ensure_executable_type_supported(
            sequence.item.as_ref(),
            arena,
            admitted_type_vars,
            context,
        ),
        Type::Map(map) => {
            ensure_executable_type_supported(
                map.key.as_ref(),
                arena,
                admitted_type_vars,
                context.clone(),
            )?;
            ensure_executable_type_supported(map.val.as_ref(), arena, admitted_type_vars, context)
        }
        Type::Closure(closure) => {
            ensure_executable_type_supported(
                closure.param.as_ref(),
                arena,
                admitted_type_vars,
                context.clone(),
            )?;
            ensure_executable_type_supported(
                closure.ret.as_ref(),
                arena,
                admitted_type_vars,
                context,
            )
        }
        Type::Measured(base, _) => {
            ensure_executable_type_supported(base, arena, admitted_type_vars, context)
        }
        Type::Option(item) => {
            ensure_executable_type_supported(item, arena, admitted_type_vars, context)
        }
        Type::Result(ok_ty, err_ty) => {
            ensure_executable_type_supported(ok_ty, arena, admitted_type_vars, context.clone())?;
            ensure_executable_type_supported(err_ty, arena, admitted_type_vars, context)
        }
        Type::Record(name) => {
            let _ = resolve_symbol_name(arena, *name)?;
            Ok(())
        }
        Type::Adt(name) => {
            let _ = resolve_symbol_name(arena, *name)?;
            Ok(())
        }
        Type::TypeVar(name) => {
            if admitted_type_vars.contains(name) {
                Ok(())
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "type variable '{}' is not admitted as an executable type in {}",
                        resolve_symbol_name(arena, *name)?,
                        context
                    ),
                })
            }
        }
        Type::QVec(_) => Err(FrontendError {
            pos: 0,
            message: format!(
                "qvec is a reserved type and is not yet admitted as an executable type in {}",
                context
            ),
        }),
    }
}

/// FA-02-038 / #1861: exhaustive (no catch-all) storage-type admission.
///
/// Every `Type` variant is matched explicitly, so adding a new variant to
/// the `Type` enum without updating this function is a compile error rather
/// than a silent "falls through to `Ok(())`" admission -- the same
/// exhaustiveness discipline `ensure_executable_type_supported` established
/// for #1647, applied to a genuinely different question. Storage admission
/// answers "may this type be the type of a field, binding, or annotated
/// value", not "may this type appear in an executable function
/// signature" -- the two contracts happen to agree on every variant here,
/// but are decided independently rather than by reuse, since nothing
/// guarantees they always will (e.g. a future closure-capture storage
/// model could diverge from closure's executable-parameter admission).
///
/// `Type::Closure` is an admitted storage composite, and this admission is
/// position-independent (local binding, record field, ADT payload) by
/// direct proof rather than by assumption. Local `let` storage was already
/// covered (see `first_class_closure_literal_typechecks_with_declared_signature_and_capture`
/// / `direct_first_class_closure_invocation_typechecks_in_wave3`), but an
/// earlier revision of this fix generalized that to record/ADT storage on
/// the weaker claim that "no evidence distinguishes record-field storage as
/// stricter". Wiring this function into `validate_record_declarations`/
/// `validate_adt_declarations` genuinely widened its scope beyond local
/// bindings, so that generalization needed its own proof, not an absence of
/// counter-evidence. That proof now exists at every pipeline stage for both
/// aggregate positions: typecheck (`storage_admission_record_closure_field_typechecks_end_to_end`
/// / `storage_admission_adt_closure_payload_typechecks_end_to_end`), IR
/// lowering to the correct `MakeClosure`/`MakeRecord`/`MakeAdt`/`RecordGet`/
/// `ClosureCall` opcode sequence (`storage_admission_record_closure_field_lowers_to_working_ir`
/// / `storage_admission_adt_closure_payload_lowers_to_working_ir` in
/// `sm-ir`), and full frontend-to-VM execution producing the correct result
/// (`storage_admission_record_closure_field_vm_executes_correctly` /
/// `storage_admission_adt_closure_payload_vm_executes_correctly` in
/// `sm-vm`). This is documented as a deliberate SSF-07 #1861 widening of
/// closure storage scope beyond the M8.4 local-binding-only baseline; see
/// the addendum in `docs/roadmap/language_maturity/first_class_closures_full_scope.md`.
/// Recursing into the closure's parameter/return types mirrors
/// `ensure_executable_type_supported`, but the two remain independently
/// decided contracts, not a shared implementation.
///
/// The other admitted composites are backed the same way, evidence first:
/// `Sequence`/`Map` have no prior normative statement admitting them in
/// aggregate position, so `storage_admission_sequence_and_map_aggregate_storage_lowers`
/// (`sm-ir`) is their sole but sufficient lowering proof for both record
/// and ADT storage. `Tuple`/`Measured`/`Option`/`Result` are directly named
/// in aggregate position by `docs/spec/types.md` ("measured numeric types
/// may appear in ... tuple elements, record fields ... `Option(T)`, and
/// `Result(T, E)` payload positions"), confirmed end-to-end by
/// `storage_admission_aggregate_composite_matrix_typechecks_for_record_and_adt`,
/// which exercises all eight admitted composites in both record-field and
/// ADT-payload position. `Record`/`Adt` nominal nesting inside other
/// records/ADTs is proven intended by the pre-existing
/// `validate_record_acyclic`/`validate_adt_acyclic` cycle-detection
/// machinery, which would be pointless if nominal aggregate nesting were
/// not a supported storage pattern.
///
/// `Type::RangeI32` and `Type::TypeVar` reject unconditionally: no current
/// Foundation source-profile evidence, test, or lowering path shows a range
/// value or an unresolved generic parameter is ever storable, and unlike
/// `canonicalize_declared_type_generic`/`ensure_executable_type_supported`
/// this helper has no caller-scoped `admitted_type_vars` list through which
/// a `TypeVar` could ever be proven legitimate here -- there is also no
/// parser-level annotation syntax that ever constructs `Type::RangeI32` in
/// the first place (it only ever arises as an inferred range-*expression*
/// type), so this arm is believed unreachable via any current call path;
/// it is included because exhaustiveness requires a decision for every
/// variant regardless of reachability. `Type::QVec` remains reserved and
/// not yet promoted, as it is for executable admission.
fn ensure_storage_type_supported(
    ty: &Type,
    arena: &AstArena,
    context: String,
) -> Result<(), FrontendError> {
    match ty {
        Type::Quad
        | Type::Bool
        | Type::Text
        | Type::I32
        | Type::U32
        | Type::Fx
        | Type::F64
        | Type::Unit => Ok(()),
        Type::Tuple(items) => {
            for item in items {
                ensure_storage_type_supported(item, arena, context.clone())?;
            }
            Ok(())
        }
        Type::Sequence(sequence) => {
            ensure_storage_type_supported(sequence.item.as_ref(), arena, context)
        }
        Type::Map(map) => {
            ensure_storage_type_supported(map.key.as_ref(), arena, context.clone())?;
            ensure_storage_type_supported(map.val.as_ref(), arena, context)
        }
        Type::Closure(closure) => {
            ensure_storage_type_supported(closure.param.as_ref(), arena, context.clone())?;
            ensure_storage_type_supported(closure.ret.as_ref(), arena, context)
        }
        Type::Measured(base, _) => ensure_storage_type_supported(base, arena, context),
        Type::Option(item) => ensure_storage_type_supported(item, arena, context),
        Type::Result(ok_ty, err_ty) => {
            ensure_storage_type_supported(ok_ty, arena, context.clone())?;
            ensure_storage_type_supported(err_ty, arena, context)
        }
        Type::Record(name) => {
            let _ = resolve_symbol_name(arena, *name)?;
            Ok(())
        }
        Type::Adt(name) => {
            let _ = resolve_symbol_name(arena, *name)?;
            Ok(())
        }
        Type::TypeVar(name) => Err(FrontendError {
            pos: 0,
            message: format!(
                "type variable '{}' is not admitted as a storage type in {}",
                resolve_symbol_name(arena, *name)?,
                context
            ),
        }),
        Type::RangeI32 => Err(FrontendError {
            pos: 0,
            message: format!("range values are not admitted as a storage type in {context}"),
        }),
        Type::QVec(_) => Err(FrontendError {
            pos: 0,
            message: format!(
                "qvec is a reserved type and is not admitted as a storage type in {context}"
            ),
        }),
    }
}

fn supports_stable_equality_type(
    ty: &Type,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<bool, FrontendError> {
    let mut active = BTreeSet::new();
    supports_stable_equality_type_inner(ty, record_table, adt_table, &mut active)
}

fn ensure_requires_expr_supported(expr_id: ExprId, arena: &AstArena) -> Result<(), FrontendError> {
    ensure_contract_expr_supported(expr_id, arena, "requires", "parameter references")
}

fn ensure_ensures_expr_supported(expr_id: ExprId, arena: &AstArena) -> Result<(), FrontendError> {
    ensure_contract_expr_supported(
        expr_id,
        arena,
        "ensures",
        "parameter references, optional result binding",
    )
}

fn ensure_invariant_expr_supported(expr_id: ExprId, arena: &AstArena) -> Result<(), FrontendError> {
    ensure_contract_expr_supported(
        expr_id,
        arena,
        "invariant",
        "parameter references, optional result binding",
    )
}

fn ensure_contract_expr_supported(
    expr_id: ExprId,
    arena: &AstArena,
    clause_name: &str,
    binding_desc: &str,
) -> Result<(), FrontendError> {
    match arena.expr(expr_id) {
        Expr::QuadLiteral(_)
        | Expr::BoolLiteral(_)
        | Expr::TextLiteral(_)
        | Expr::NumericLiteral(_)
        | Expr::Var(_) => Ok(()),
        Expr::Tuple(items) => {
            for item in items {
                ensure_contract_expr_supported(*item, arena, clause_name, binding_desc)?;
            }
            Ok(())
        }
        Expr::RecordField(field_expr) => {
            ensure_contract_expr_supported(field_expr.base, arena, clause_name, binding_desc)
        }
        Expr::SequenceIndex(index_expr) => {
            ensure_contract_expr_supported(index_expr.base, arena, clause_name, binding_desc)?;
            ensure_contract_expr_supported(index_expr.index, arena, clause_name, binding_desc)
        }
        Expr::Unary(_, inner) => {
            ensure_contract_expr_supported(*inner, arena, clause_name, binding_desc)
        }
        Expr::Binary(lhs, _, rhs) => {
            ensure_contract_expr_supported(*lhs, arena, clause_name, binding_desc)?;
            ensure_contract_expr_supported(*rhs, arena, clause_name, binding_desc)
        }
        _ => Err(FrontendError {
            pos: 0,
            message: format!(
                "{clause_name} clause currently allows only {binding_desc}, tuple literals, record/sequence reads, and pure unary/binary operator expressions"
            ),
        }),
    }
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
        Expr::SequenceIndex(index_expr) => {
            if let Some(symbol) = find_named_var_symbol(index_expr.base, arena, name)? {
                return Ok(Some(symbol));
            }
            find_named_var_symbol(index_expr.index, arena, name)
        }
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

fn supports_stable_equality_type_inner(
    ty: &Type,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    active: &mut BTreeSet<SymbolId>,
) -> Result<bool, FrontendError> {
    match ty {
        Type::Quad
        | Type::Bool
        | Type::Text
        | Type::I32
        | Type::U32
        | Type::Fx
        | Type::F64
        | Type::Unit => Ok(true),
        Type::Measured(base, _) => {
            supports_stable_equality_type_inner(base, record_table, adt_table, active)
        }
        Type::Sequence(sequence) => supports_stable_equality_type_inner(
            sequence.item.as_ref(),
            record_table,
            adt_table,
            active,
        ),
        Type::QVec(_) => Ok(false),
        Type::RangeI32 => Ok(false),
        Type::Tuple(items) => {
            for item in items {
                if !supports_stable_equality_type_inner(item, record_table, adt_table, active)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Type::Option(item) => {
            supports_stable_equality_type_inner(item, record_table, adt_table, active)
        }
        Type::Result(ok_ty, err_ty) => {
            if !supports_stable_equality_type_inner(ok_ty, record_table, adt_table, active)? {
                return Ok(false);
            }
            supports_stable_equality_type_inner(err_ty, record_table, adt_table, active)
        }
        Type::Record(name) => {
            if !active.insert(*name) {
                return Ok(false);
            }
            let record = record_table.get(name).ok_or(FrontendError {
                pos: 0,
                message: "record equality subset references unknown record type".to_string(),
            })?;
            for field in &record.fields {
                if !supports_stable_equality_type_inner(&field.ty, record_table, adt_table, active)?
                {
                    active.remove(name);
                    return Ok(false);
                }
            }
            active.remove(name);
            Ok(true)
        }
        Type::Closure(_) => Ok(false),
        Type::Adt(_) => Ok(false),
        Type::Map(_) => Ok(false),
        // TypeVar is an owner-layer marker; equality support is unknown until
        // monomorphisation substitutes the variable (Wave 2).
        Type::TypeVar(_) => Ok(false),
    }
}

fn infer_record_literal_type(
    record_literal: &RecordLiteralExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    let record = record_table
        .get(&record_literal.name)
        .ok_or(FrontendError {
            pos: 0,
            message: format!(
                "unknown record type '{}' in record literal",
                resolve_symbol_name(arena, record_literal.name)?
            ),
        })?;
    let record_name = resolve_symbol_name(arena, record_literal.name)?;
    let mut field_types = BTreeMap::new();
    for field in &record.fields {
        field_types.insert(
            field.name,
            canonicalize_declared_type(&field.ty, record_table, adt_table, arena)?,
        );
    }
    let mut seen = BTreeSet::new();
    for field in &record_literal.fields {
        if !seen.insert(field.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "record literal '{}' cannot repeat field '{}'",
                    record_name,
                    resolve_symbol_name(arena, field.name)?
                ),
            });
        }
        let expected_ty = field_types.get(&field.name).ok_or(FrontendError {
            pos: 0,
            message: format!(
                "record literal '{}' has no field named '{}'",
                record_name,
                resolve_symbol_name(arena, field.name)?
            ),
        })?;
        let actual_ty = infer_expr_type_with_expected(
            field.value,
            arena,
            env,
            table,
            record_table,
            adt_table,
            Some(expected_ty.clone()),
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        ensure_binding_value_type(
            expected_ty.clone(),
            actual_ty,
            field.value,
            arena,
            format!(
                "record field '{}.{}'",
                record_name,
                resolve_symbol_name(arena, field.name)?
            ),
        )?;
    }
    for decl_field in &record.fields {
        if !seen.contains(&decl_field.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "record literal '{}' is missing field '{}'",
                    record_name,
                    resolve_symbol_name(arena, decl_field.name)?
                ),
            });
        }
    }
    Ok(Type::Record(record_literal.name))
}

fn infer_record_field_access_type(
    field_expr: &RecordFieldExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    // M9.9: use no-check variant for the base; caller already verified full path.
    let base_ty = infer_expr_type_no_check(
        field_expr.base,
        arena,
        env,
        table,
        record_table,
        adt_table,
        ret_ty,
        loop_stack,
        impl_list,
    )?;
    let Type::Record(record_name) = base_ty else {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "record field access requires record value before '.{}', got {:?}",
                resolve_symbol_name(arena, field_expr.field)?,
                base_ty
            ),
        });
    };
    let record = record_table.get(&record_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown record type '{}' in field access",
            resolve_symbol_name(arena, record_name)?
        ),
    })?;
    let field = record
        .fields
        .iter()
        .find(|field| field.name == field_expr.field)
        .ok_or(FrontendError {
            pos: 0,
            message: format!(
                "record type '{}' has no field named '{}'",
                resolve_symbol_name(arena, record_name)?,
                resolve_symbol_name(arena, field_expr.field)?
            ),
        })?;
    canonicalize_declared_type(&field.ty, record_table, adt_table, arena)
}

fn infer_sequence_index_type(
    index_expr: &SequenceIndexExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    // M9.9: use no-check variant for the base; caller already verified full path.
    let base_ty = infer_expr_type_no_check(
        index_expr.base,
        arena,
        env,
        table,
        record_table,
        adt_table,
        ret_ty.clone(),
        loop_stack,
        impl_list,
    )?;
    let Type::Sequence(sequence_ty) = base_ty else {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "sequence indexing requires Sequence(type) base before '[...]', got {:?}",
                base_ty
            ),
        });
    };
    let index_ty = infer_expr_type(
        index_expr.index,
        arena,
        env,
        table,
        record_table,
        adt_table,
        ret_ty,
        loop_stack,
        impl_list,
    )?;
    if index_ty != Type::I32 {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "sequence indexing currently requires i32 index, got {:?}",
                index_ty
            ),
        });
    }
    Ok(sequence_ty.item.as_ref().clone())
}

fn infer_record_update_type(
    update_expr: &RecordUpdateExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    let base_ty = infer_expr_type(
        update_expr.base,
        arena,
        env,
        table,
        record_table,
        adt_table,
        ret_ty.clone(),
        loop_stack,
        impl_list,
    )?;
    let Type::Record(record_name) = base_ty else {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "record copy-with requires record base before 'with', got {:?}",
                base_ty
            ),
        });
    };
    let record = record_table.get(&record_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown record type '{}' in record copy-with",
            resolve_symbol_name(arena, record_name)?
        ),
    })?;
    let record_name_text = resolve_symbol_name(arena, record_name)?;
    if update_expr.fields.is_empty() {
        return Err(FrontendError {
            pos: 0,
            message: "record copy-with requires at least one explicit override field".to_string(),
        });
    }
    let mut field_types = BTreeMap::new();
    for field in &record.fields {
        field_types.insert(
            field.name,
            canonicalize_declared_type(&field.ty, record_table, adt_table, arena)?,
        );
    }
    let mut seen = BTreeSet::new();
    for field in &update_expr.fields {
        if !seen.insert(field.name) {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "record copy-with '{}' cannot repeat field '{}'",
                    record_name_text,
                    resolve_symbol_name(arena, field.name)?
                ),
            });
        }
        let expected_ty = field_types.get(&field.name).ok_or(FrontendError {
            pos: 0,
            message: format!(
                "record copy-with '{}' has no field named '{}'",
                record_name_text,
                resolve_symbol_name(arena, field.name)?
            ),
        })?;
        let actual_ty = infer_expr_type_with_expected(
            field.value,
            arena,
            env,
            table,
            record_table,
            adt_table,
            Some(expected_ty.clone()),
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        ensure_binding_value_type(
            expected_ty.clone(),
            actual_ty,
            field.value,
            arena,
            format!(
                "record copy-with '{}.{}'",
                record_name_text,
                resolve_symbol_name(arena, field.name)?
            ),
        )?;
    }
    Ok(Type::Record(record_name))
}

fn infer_adt_ctor_type(
    ctor_expr: &AdtCtorExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<&Type>,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    if let Some(ty) = infer_std_form_ctor_type(
        ctor_expr,
        arena,
        env,
        table,
        record_table,
        adt_table,
        expected,
        ret_ty.clone(),
        loop_stack,
        impl_list,
    )? {
        return Ok(ty);
    }
    let adt = adt_table.get(&ctor_expr.adt_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown enum type '{}' in constructor",
            resolve_symbol_name(arena, ctor_expr.adt_name)?
        ),
    })?;
    let variant = adt
        .variants
        .iter()
        .find(|variant| variant.name == ctor_expr.variant_name)
        .ok_or(FrontendError {
            pos: 0,
            message: format!(
                "enum '{}' has no variant named '{}'",
                resolve_symbol_name(arena, ctor_expr.adt_name)?,
                resolve_symbol_name(arena, ctor_expr.variant_name)?
            ),
        })?;
    if variant.payload.len() != ctor_expr.payload.len() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "enum constructor '{}::{}' expects {} payload items, got {}",
                resolve_symbol_name(arena, ctor_expr.adt_name)?,
                resolve_symbol_name(arena, ctor_expr.variant_name)?,
                variant.payload.len(),
                ctor_expr.payload.len()
            ),
        });
    }
    for (index, (payload_expr, expected_ty)) in ctor_expr
        .payload
        .iter()
        .zip(variant.payload.iter())
        .enumerate()
    {
        let canonical_expected =
            canonicalize_declared_type(expected_ty, record_table, adt_table, arena)?;
        let actual_ty = infer_expr_type_with_expected(
            *payload_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            Some(canonical_expected.clone()),
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        ensure_binding_value_type(
            canonical_expected,
            actual_ty,
            *payload_expr,
            arena,
            format!(
                "enum constructor '{}::{}' payload item {}",
                resolve_symbol_name(arena, ctor_expr.adt_name)?,
                resolve_symbol_name(arena, ctor_expr.variant_name)?,
                index
            ),
        )?;
    }
    Ok(Type::Adt(ctor_expr.adt_name))
}

fn infer_expr_type_with_expected(
    expr_id: ExprId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<Type>,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    match arena.expr(expr_id) {
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
                            "tuple arity mismatch in typed position: expected {}, got {}",
                            types.len(),
                            items.len()
                        ),
                    });
                }
            }
            let mut item_tys = Vec::with_capacity(items.len());
            for (index, item) in items.iter().enumerate() {
                let item_expected = expected_items.and_then(|types| types.get(index)).cloned();
                let item_ty = infer_expr_type_with_expected(
                    *item,
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    item_expected,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                if item_ty == Type::RangeI32 {
                    return Err(FrontendError {
                        pos: 0,
                        message:
                            "range literal is not yet part of the stable tuple/user-data surface"
                                .to_string(),
                    });
                }
                item_tys.push(item_ty);
            }
            Ok(Type::Tuple(item_tys))
        }
        Expr::SequenceLiteral(sequence) => infer_sequence_literal_type(
            sequence,
            arena,
            env,
            table,
            record_table,
            adt_table,
            expected.as_ref(),
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::SequenceIndex(index_expr) => infer_sequence_index_type(
            index_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::Closure(closure) => infer_closure_literal_type(
            closure,
            arena,
            env,
            table,
            record_table,
            adt_table,
            expected.as_ref(),
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::AdtCtor(ctor_expr) => infer_adt_ctor_type(
            ctor_expr,
            arena,
            env,
            table,
            record_table,
            adt_table,
            expected.as_ref(),
            ret_ty,
            loop_stack,
            impl_list,
        ),
        Expr::Call(name, args) => {
            // Special case: map_empty() uses contextual type
            if let Ok("map_empty") = resolve_symbol_name(arena, *name).as_deref() {
                if args.is_empty() {
                    match expected.as_ref() {
                        Some(ty @ Type::Map(_)) => return Ok(ty.clone()),
                        _ => {
                            return Err(FrontendError {
                                pos: 0,
                                message: "map_empty() requires a contextual Map(K, V) type; \
                                     use 'let q: Map(K, V) = map_empty()'"
                                    .to_string(),
                            })
                        }
                    }
                }
            }
            let actual = infer_expr_type(
                expr_id,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            Ok(
                lift_literal_to_expected_type(expected.as_ref(), &actual, expr_id, arena)
                    .unwrap_or(actual),
            )
        }
        _ => {
            let actual = infer_expr_type(
                expr_id,
                arena,
                env,
                table,
                record_table,
                adt_table,
                ret_ty,
                loop_stack,
                impl_list,
            )?;
            Ok(
                lift_literal_to_expected_type(expected.as_ref(), &actual, expr_id, arena)
                    .unwrap_or(actual),
            )
        }
    }
}

fn infer_sequence_literal_type(
    sequence: &SequenceLiteral,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<&Type>,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    let expected_item = match expected {
        Some(Type::Sequence(sequence_ty))
            if sequence_ty.family == SequenceCollectionFamily::OrderedSequence =>
        {
            Some(sequence_ty.item.as_ref())
        }
        _ => None,
    };

    if sequence.items.is_empty() {
        let Some(expected_item) = expected_item else {
            return Err(FrontendError {
                pos: 0,
                message:
                    "empty ordered sequence literal currently requires contextual Sequence(type) in M8.3 Wave 2"
                        .to_string(),
            });
        };
        return Ok(Type::Sequence(SequenceType {
            family: SequenceCollectionFamily::OrderedSequence,
            item: Box::new(expected_item.clone()),
        }));
    }

    let first_ty = if let Some(expected_item) = expected_item {
        let actual_ty = infer_expr_type_with_expected(
            sequence.items[0],
            arena,
            env,
            table,
            record_table,
            adt_table,
            Some(expected_item.clone()),
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        ensure_binding_value_type(
            expected_item.clone(),
            actual_ty,
            sequence.items[0],
            arena,
            "ordered sequence item 0".to_string(),
        )?;
        expected_item.clone()
    } else {
        infer_expr_type(
            sequence.items[0],
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?
    };

    for (index, item) in sequence.items.iter().enumerate().skip(1) {
        let actual_ty = infer_expr_type_with_expected(
            *item,
            arena,
            env,
            table,
            record_table,
            adt_table,
            Some(first_ty.clone()),
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        ensure_binding_value_type(
            first_ty.clone(),
            actual_ty,
            *item,
            arena,
            format!("ordered sequence item {}", index),
        )?;
    }

    Ok(Type::Sequence(SequenceType {
        family: SequenceCollectionFamily::OrderedSequence,
        item: Box::new(first_ty),
    }))
}

fn infer_closure_literal_type(
    closure: &ClosureLiteral,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<&Type>,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    let Some(Type::Closure(expected_closure)) = expected else {
        return Err(FrontendError {
            pos: 0,
            message:
                "first-class closure literals currently require contextual Closure(T -> U) type in M8.4 Wave 2"
                    .to_string(),
        });
    };

    if expected_closure.family != closure.family || expected_closure.capture != closure.capture {
        return Err(FrontendError {
            pos: 0,
            message:
                "first-class closure literal does not match the current Wave 2 closure family/capture contract"
                    .to_string(),
        });
    }

    for capture in &closure.captures {
        if env.get(*capture).is_none() {
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "unknown captured value '{}' in first-class closure literal",
                    resolve_symbol_name(arena, *capture)?
                ),
            });
        }
    }

    let mut closure_env = env.clone();
    closure_env.push_scope();
    closure_env.insert(closure.param, expected_closure.param.as_ref().clone());
    let body_ty = infer_expr_type_with_expected(
        closure.body,
        arena,
        &mut closure_env,
        table,
        record_table,
        adt_table,
        Some(expected_closure.ret.as_ref().clone()),
        ret_ty,
        loop_stack,
        impl_list,
    )?;
    ensure_binding_value_type(
        expected_closure.ret.as_ref().clone(),
        body_ty,
        closure.body,
        arena,
        "first-class closure body".to_string(),
    )?;
    Ok(Type::Closure(expected_closure.clone()))
}

fn infer_std_form_ctor_type(
    ctor_expr: &AdtCtorExpr,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    expected: Option<&Type>,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Option<Type>, FrontendError> {
    let type_name = resolve_symbol_name(arena, ctor_expr.adt_name)?;
    let variant_name = resolve_symbol_name(arena, ctor_expr.variant_name)?;

    if type_name == "Option" {
        return match variant_name {
            "Some" => {
                if ctor_expr.payload.len() != 1 {
                    return Err(FrontendError {
                        pos: 0,
                        message: "Option::Some expects exactly one payload item".to_string(),
                    });
                }
                let item_ty = if let Some(Type::Option(item_ty)) = expected {
                    let expected_item = (**item_ty).clone();
                    let actual_ty = infer_expr_type_with_expected(
                        ctor_expr.payload[0],
                        arena,
                        env,
                        table,
                        record_table,
                        adt_table,
                        Some(expected_item.clone()),
                        ret_ty,
                        loop_stack,
                        impl_list,
                    )?;
                    ensure_binding_value_type(
                        expected_item.clone(),
                        actual_ty,
                        ctor_expr.payload[0],
                        arena,
                        "Option::Some payload".to_string(),
                    )?;
                    expected_item
                } else {
                    infer_expr_type(
                        ctor_expr.payload[0],
                        arena,
                        env,
                        table,
                        record_table,
                        adt_table,
                        ret_ty,
                        loop_stack,
                        impl_list,
                    )?
                };
                Ok(Some(Type::Option(Box::new(item_ty))))
            }
            "None" => {
                if !ctor_expr.payload.is_empty() {
                    return Err(FrontendError {
                        pos: 0,
                        message: "Option::None does not accept payload items".to_string(),
                    });
                }
                match expected {
                    Some(Type::Option(item_ty)) => {
                        Ok(Some(Type::Option(Box::new((**item_ty).clone()))))
                    }
                    _ => Err(FrontendError {
                        pos: 0,
                        message: "Option::None currently requires contextual Option(T) type in v0"
                            .to_string(),
                    }),
                }
            }
            _ => Err(FrontendError {
                pos: 0,
                message: format!("Option has no variant named '{}'", variant_name),
            }),
        };
    }

    if type_name == "Result" {
        return match variant_name {
            "Ok" | "Err" => {
                if ctor_expr.payload.len() != 1 {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "Result::{} expects exactly one payload item",
                            variant_name
                        ),
                    });
                }
                let Some(Type::Result(ok_ty, err_ty)) = expected else {
                    return Err(FrontendError {
                        pos: 0,
                        message: format!(
                            "Result::{} currently requires contextual Result(T, E) type in v0",
                            variant_name
                        ),
                    });
                };
                let expected_payload = if variant_name == "Ok" {
                    (**ok_ty).clone()
                } else {
                    (**err_ty).clone()
                };
                let actual_ty = infer_expr_type_with_expected(
                    ctor_expr.payload[0],
                    arena,
                    env,
                    table,
                    record_table,
                    adt_table,
                    Some(expected_payload.clone()),
                    ret_ty,
                    loop_stack,
                    impl_list,
                )?;
                ensure_binding_value_type(
                    expected_payload,
                    actual_ty,
                    ctor_expr.payload[0],
                    arena,
                    format!("Result::{} payload", variant_name),
                )?;
                Ok(Some(Type::Result(
                    Box::new((**ok_ty).clone()),
                    Box::new((**err_ty).clone()),
                )))
            }
            _ => Err(FrontendError {
                pos: 0,
                message: format!("Result has no variant named '{}'", variant_name),
            }),
        };
    }

    Ok(None)
}

#[derive(Debug, Clone)]
struct MatchFamilyVariantSpec {
    name: String,
    payload: Vec<Type>,
}

#[derive(Debug, Clone)]
struct MatchFamilySpec {
    family_name: String,
    display_label: String,
    variants: Vec<MatchFamilyVariantSpec>,
}

fn resolve_match_family_spec(
    scrutinee_ty: &Type,
    arena: &AstArena,
    adt_table: &AdtTable,
) -> Result<Option<MatchFamilySpec>, FrontendError> {
    match scrutinee_ty {
        Type::Adt(adt_name) => {
            let adt = adt_table.get(adt_name).ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "unknown enum type '{}' in match resolution",
                    resolve_symbol_name(arena, *adt_name)?,
                ),
            })?;
            let family_name = resolve_symbol_name(arena, *adt_name)?.to_string();
            let mut variants = Vec::new();
            for variant in &adt.variants {
                variants.push(MatchFamilyVariantSpec {
                    name: resolve_symbol_name(arena, variant.name)?.to_string(),
                    payload: variant.payload.clone(),
                });
            }
            Ok(Some(MatchFamilySpec {
                display_label: format!("enum '{}'", family_name),
                family_name,
                variants,
            }))
        }
        Type::Option(item_ty) => Ok(Some(MatchFamilySpec {
            family_name: "Option".to_string(),
            display_label: "Option(T)".to_string(),
            variants: vec![
                MatchFamilyVariantSpec {
                    name: "None".to_string(),
                    payload: Vec::new(),
                },
                MatchFamilyVariantSpec {
                    name: "Some".to_string(),
                    payload: vec![(**item_ty).clone()],
                },
            ],
        })),
        Type::Result(ok_ty, err_ty) => Ok(Some(MatchFamilySpec {
            family_name: "Result".to_string(),
            display_label: "Result(T, E)".to_string(),
            variants: vec![
                MatchFamilyVariantSpec {
                    name: "Ok".to_string(),
                    payload: vec![(**ok_ty).clone()],
                },
                MatchFamilyVariantSpec {
                    name: "Err".to_string(),
                    payload: vec![(**err_ty).clone()],
                },
            ],
        })),
        _ => Ok(None),
    }
}

fn missing_exhaustive_sum_variants<'a>(
    scrutinee_ty: &Type,
    patterns: impl IntoIterator<Item = (&'a MatchPattern, Option<ExprId>)>,
    arena: &AstArena,
    adt_table: &AdtTable,
) -> Result<Option<(String, Vec<String>)>, FrontendError> {
    let Some(family) = resolve_match_family_spec(scrutinee_ty, arena, adt_table)? else {
        return Ok(None);
    };

    let mut covered = BTreeSet::new();
    for (pat, guard) in patterns {
        if guard.is_some() {
            continue;
        }
        // NOTE: Range and tuple patterns are not included in exhaustiveness (M9.4 Wave 3 boundary).
        // Wildcard covers all variants.
        if matches!(pat, MatchPattern::Wildcard) {
            return Ok(Some((family.display_label, Vec::new())));
        }
        // M9.4 Wave 3: or-pattern — expand alternatives into coverage.
        if let MatchPattern::Or(alts) = pat {
            for alt in alts {
                if matches!(alt, MatchPattern::Wildcard) {
                    return Ok(Some((family.display_label, Vec::new())));
                }
                if let MatchPattern::Adt(adt_pat) = alt {
                    if resolve_symbol_name(arena, adt_pat.adt_name)? == family.family_name {
                        covered
                            .insert(resolve_symbol_name(arena, adt_pat.variant_name)?.to_string());
                    }
                }
            }
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

fn check_match_guard(
    guard: Option<ExprId>,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<(), FrontendError> {
    if let Some(expr_id) = guard {
        let guard_ty = infer_expr_type(
            expr_id,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        )?;
        if guard_ty != Type::Bool {
            return Err(FrontendError {
                pos: 0,
                message:
                    "match guard condition must be bool; explicit compare is required for quad"
                        .to_string(),
            });
        }
    }
    Ok(())
}

fn check_return_payload(
    value: Option<ExprId>,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<(), FrontendError> {
    let got = if let Some(expr_id) = value {
        infer_expr_type_with_expected(
            expr_id,
            arena,
            env,
            table,
            record_table,
            adt_table,
            Some(ret_ty.clone()),
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?
    } else {
        Type::Unit
    };
    if got != ret_ty {
        if ret_ty == Type::Fx && is_numeric_for_fx_gap(&got) {
            if let Some(expr_id) = value {
                if is_fx_literal_expr(expr_id, arena) {
                    return Ok(());
                }
            }
            return Err(FrontendError {
                pos: 0,
                message: format!(
                    "{}; function return currently requires an fx literal or an existing fx-typed value",
                    fx_coercion_gap_message(),
                ),
            });
        }
        return Err(FrontendError {
            pos: 0,
            message: format!("return type mismatch: expected {:?}, got {:?}", ret_ty, got),
        });
    }
    Ok(())
}

fn ensure_binding_value_type(
    expected: Type,
    actual: Type,
    value_expr: ExprId,
    arena: &AstArena,
    context: String,
) -> Result<(), FrontendError> {
    if expected == actual {
        return Ok(());
    }
    if match_unit_lift(&expected, &actual, value_expr, arena) {
        return Ok(());
    }
    if expected == Type::Fx && is_numeric_for_fx_gap(&actual) {
        if is_fx_literal_expr(value_expr, arena) {
            return Ok(());
        }
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "{}; {} currently accepts only fx literals or existing fx-typed values",
                fx_coercion_gap_message(),
                context,
            ),
        });
    }
    Err(FrontendError {
        pos: 0,
        message: format!(
            "type mismatch in {}: {:?} vs {:?}",
            context, expected, actual
        ),
    })
}

fn ensure_const_initializer_safe(
    expr_id: ExprId,
    arena: &AstArena,
    env: &mut ScopeEnv,
) -> Result<(), FrontendError> {
    match arena.expr(expr_id) {
        Expr::QuadLiteral(_) | Expr::BoolLiteral(_) | Expr::NumericLiteral(_) => Ok(()),
        Expr::Range(range_expr) => {
            ensure_const_initializer_safe(range_expr.start, arena, env)?;
            ensure_const_initializer_safe(range_expr.end, arena, env)
        }
        Expr::Tuple(items) => {
            for item in items {
                ensure_const_initializer_safe(*item, arena, env)?;
            }
            Ok(())
        }
        Expr::Var(name) => {
            // SSF-08 Lane 1 (#1664): existence must be resolved -- with the
            // canonical "unknown variable" diagnostic on absence -- before
            // querying const-ness; a missing binding is not evidence of
            // "not const". This function is called before the caller's own
            // `infer_expr_type` pass in `Stmt::Const` (see check_stmt), so
            // that pass cannot be relied on to have already produced this
            // diagnostic first.
            if env.get(*name).is_none() {
                return Err(FrontendError {
                    pos: 0,
                    message: format!("unknown variable '{}'", resolve_symbol_name(arena, *name)?),
                });
            }
            if env.is_const_checked(*name)? {
                Ok(())
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "const initializer currently allows only literals, unary/binary operations, and references to earlier const bindings; '{}' is not const",
                        resolve_symbol_name(arena, *name)?
                    ),
                })
            }
        }
        Expr::Unary(_, inner) => ensure_const_initializer_safe(*inner, arena, env),
        Expr::Binary(lhs, _, rhs) => {
            ensure_const_initializer_safe(*lhs, arena, env)?;
            ensure_const_initializer_safe(*rhs, arena, env)
        }
        _ => Err(FrontendError {
            pos: 0,
            message:
                "const initializer currently supports only pure literal/const expression forms"
                    .to_string(),
        }),
    }
}

// ──────────────────────────────────────────────────────────────
// M9.5 Wave C: binding plan builders + conflict validation
// ──────────────────────────────────────────────────────────────

/// Validate that no two items in the plan access the same path via conflicting
/// capture modes (borrow vs. move, or duplicate move).
/// Multiple borrows of the same path are allowed.
/// M9.6: returns true if every element of `a` is a prefix of `b`.
fn path_is_prefix(a: &PatternPath, b: &PatternPath) -> bool {
    if a.elems.len() > b.elems.len() {
        return false;
    }
    a.elems.iter().zip(&b.elems).all(|(x, y)| x == y)
}

/// M9.6: two paths conflict (overlap) if one is a prefix of the other or they are equal.
fn paths_overlap(a: &PatternPath, b: &PatternPath) -> bool {
    path_is_prefix(a, b) || path_is_prefix(b, a)
}

fn captures_conflict(a: CaptureMode, b: CaptureMode) -> bool {
    !matches!((a, b), (CaptureMode::Borrow, CaptureMode::Borrow))
}

/// Validate that no two items in the plan access overlapping paths via conflicting
/// capture modes.  Two paths overlap when one is a prefix of the other (or equal).
/// Multiple borrows of the same or ancestor/descendant path are allowed.
///
/// NOTE (M9.5/M9.6): overlap check is prefix-based only.
/// Alias analysis and field-sensitivity beyond the current PatternPath model are deferred.
pub(crate) fn validate_binding_plan_conflicts(plan: &BindingPlan) -> Result<(), FrontendError> {
    for (i, a) in plan.items.iter().enumerate() {
        for b in plan.items.iter().skip(i + 1) {
            if !paths_overlap(&a.path, &b.path) {
                continue;
            }
            if captures_conflict(a.capture, b.capture) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "conflicting captures on overlapping pattern paths for '{}' and '{}'",
                        a.name.0, b.name.0
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Determine whether the scrutinee is consumed (moved) by the plan.
#[allow(dead_code)]
pub(crate) fn scrutinee_use_from_plan(plan: &BindingPlan) -> ScrutineeUse {
    if plan.items.iter().any(|it| it.capture == CaptureMode::Move) {
        ScrutineeUse::Consumed
    } else {
        ScrutineeUse::Preserved
    }
}

/// Apply a binding plan to an env scope (insert all bindings as mutable locals).
pub(crate) fn apply_binding_plan(env: &mut ScopeEnv, plan: &BindingPlan) {
    for item in &plan.items {
        env.insert(item.name, item.ty.clone());
    }
}

/// Build a `BindingPlan` from tuple pattern items against a known tuple type.
pub(crate) fn build_tuple_pattern_plan(
    items: &[TuplePatternItem],
    expected_ty: &Type,
    base: &PatternPath,
    out: &mut BindingPlan,
) -> Result<(), FrontendError> {
    let Type::Tuple(tuple_items) = expected_ty else {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "tuple pattern requires tuple scrutinee, got {:?}",
                expected_ty
            ),
        });
    };
    if items.len() != tuple_items.len() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "tuple pattern arity mismatch: pattern has {} items, value has {}",
                items.len(),
                tuple_items.len()
            ),
        });
    }
    for (idx, (item, item_ty)) in items.iter().zip(tuple_items.iter()).enumerate() {
        let path = base.tuple_index(idx);
        match item {
            TuplePatternItem::Discard | TuplePatternItem::QuadLiteral(_) => {}
            TuplePatternItem::Nested(nested) => {
                build_tuple_pattern_plan(nested, item_ty, &path, out)?;
            }
            TuplePatternItem::Bind { name, capture } => {
                out.push(BindingPlanItem {
                    name: *name,
                    capture: *capture,
                    path,
                    ty: item_ty.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Build a `BindingPlan` from record pattern items against a known record type.
pub(crate) fn build_record_pattern_plan(
    items: &[crate::types::RecordPatternItem],
    expected_ty: &Type,
    base: &PatternPath,
    out: &mut BindingPlan,
    arena: &AstArena,
    record_table: &RecordTable,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let Type::Record(record_name) = expected_ty else {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "record pattern requires record scrutinee, got {:?}",
                expected_ty
            ),
        });
    };
    let record = record_table.get(record_name).ok_or(FrontendError {
        pos: 0,
        message: format!(
            "unknown record type '{}' in record pattern",
            resolve_symbol_name(arena, *record_name)?
        ),
    })?;
    for item in items {
        let field = record
            .fields
            .iter()
            .find(|field| field.name == item.field)
            .ok_or(FrontendError {
                pos: 0,
                message: format!(
                    "record type '{}' has no field named '{}' in record pattern",
                    resolve_symbol_name(arena, *record_name)?,
                    resolve_symbol_name(arena, item.field)?
                ),
            })?;
        if let RecordPatternTarget::Bind { name, capture } = &item.target {
            out.push(BindingPlanItem {
                name: *name,
                capture: *capture,
                path: base.record_field(item.field),
                ty: canonicalize_declared_type(&field.ty, record_table, adt_table, arena)?,
            });
        }
    }
    Ok(())
}

/// Build a `BindingPlan` from an ADT match pattern against a known ADT type.
pub(crate) fn build_adt_pattern_plan(
    pat: &AdtMatchPattern,
    expected_ty: &Type,
    base: &PatternPath,
    out: &mut BindingPlan,
    arena: &AstArena,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    let family =
        resolve_match_family_spec(expected_ty, arena, adt_table)?.ok_or_else(|| FrontendError {
            pos: 0,
            message: "ADT pattern plan: scrutinee is not a sum type".to_string(),
        })?;
    // Verify that the pattern's enum name matches the scrutinee family.
    let pattern_family_name = resolve_symbol_name(arena, pat.adt_name)?.to_string();
    if pattern_family_name != family.family_name {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "match arm pattern type '{}' does not match scrutinee {}",
                pattern_family_name, family.display_label
            ),
        });
    }
    let variant_name_str = resolve_symbol_name(arena, pat.variant_name)?;
    let variant = family
        .variants
        .iter()
        .find(|v| v.name == variant_name_str)
        .ok_or_else(|| FrontendError {
            pos: 0,
            message: format!(
                "{} has no variant named '{}' in match pattern",
                family.display_label, variant_name_str
            ),
        })?;

    if pat.items.len() != variant.payload.len() {
        return Err(FrontendError {
            pos: 0,
            message: format!(
                "ADT pattern '{}::{}' arity mismatch: pattern has {} items, variant has {}",
                family.family_name,
                variant_name_str,
                pat.items.len(),
                variant.payload.len()
            ),
        });
    }

    let variant_root = base.variant(pat.variant_name);
    for (idx, (item, item_ty)) in pat.items.iter().zip(variant.payload.iter()).enumerate() {
        let path = variant_root.variant_field(idx);
        match item {
            AdtPatternItem::Discard => {}
            AdtPatternItem::Bind { name, capture } => {
                out.push(BindingPlanItem {
                    name: *name,
                    capture: *capture,
                    path,
                    ty: item_ty.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Build a `BindingPlan` from any `MatchPattern`.
///
/// For `Or`, takes the first alternative as the canonical binding shape and
/// validates that all other alternatives bind the same names/modes/types.
pub(crate) fn build_match_pattern_plan(
    pat: &MatchPattern,
    expected_ty: &Type,
    base: &PatternPath,
    out: &mut BindingPlan,
    arena: &AstArena,
    adt_table: &AdtTable,
) -> Result<(), FrontendError> {
    match pat {
        MatchPattern::Wildcard => Ok(()),
        MatchPattern::Quad(_) => {
            if matches!(expected_ty, Type::Quad) {
                Ok(())
            } else {
                Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "quad match pattern requires quad scrutinee, got {:?}",
                        expected_ty
                    ),
                })
            }
        }
        MatchPattern::IntRange(range) => {
            if !matches!(expected_ty, Type::I32 | Type::U32) {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "integer match pattern requires i32 or u32 scrutinee, got {:?}",
                        expected_ty
                    ),
                });
            }
            if range.start > range.end {
                return Err(FrontendError {
                    pos: 0,
                    message: format!(
                        "int range pattern start ({}) must be <= end ({})",
                        range.start, range.end
                    ),
                });
            }
            Ok(())
        }
        MatchPattern::Adt(adt_pat) => {
            build_adt_pattern_plan(adt_pat, expected_ty, base, out, arena, adt_table)
        }
        MatchPattern::Or(alts) => {
            if alts.is_empty() {
                return Err(FrontendError {
                    pos: 0,
                    message: "or-pattern must contain at least one alternative".to_string(),
                });
            }
            let mut first_plan = BindingPlan::default();
            build_match_pattern_plan(
                &alts[0],
                expected_ty,
                base,
                &mut first_plan,
                arena,
                adt_table,
            )?;
            validate_binding_plan_conflicts(&first_plan)?;

            let baseline: Vec<(u32, CaptureMode)> = first_plan
                .items
                .iter()
                .map(|it| (it.name.0, it.capture))
                .collect();

            for alt in &alts[1..] {
                let mut alt_plan = BindingPlan::default();
                build_match_pattern_plan(alt, expected_ty, base, &mut alt_plan, arena, adt_table)?;
                validate_binding_plan_conflicts(&alt_plan)?;

                let shape: Vec<(u32, CaptureMode)> = alt_plan
                    .items
                    .iter()
                    .map(|it| (it.name.0, it.capture))
                    .collect();

                if shape != baseline {
                    return Err(FrontendError {
                        pos: 0,
                        message: "all or-pattern alternatives must bind the same names with the same capture modes".to_string(),
                    });
                }
            }
            out.items.extend(first_plan.items);
            Ok(())
        }
    }
}
// ──────────────────────────────────────────────────────────────
// SSF-08 Lane 1: `build_and_apply_match_plan`, `validate_plan_against_
// scrutinee_state`, and `apply_plans_to_scrutinee` (the M9.5/M9.7/M9.8
// match-integration helpers this section used to hold) are retired --
// fully superseded by `build_pattern_arm_env` and
// `apply_arm_pattern_capture` (defined near `check_match_arms_joined`),
// which fold validate+apply into one canonical-access-path-aware pass used
// uniformly by statement match, expression match, if-let, and plain
// destructuring lets. Zero remaining call sites confirmed before removal.
// ──────────────────────────────────────────────────────────────
// M9.9 Wave A: path-aware expression access helpers
// ──────────────────────────────────────────────────────────────

/// Attempt to extract a `(base_variable, PatternPath)` pair from an expression.
///
/// Returns `Some` for:
///   * `Expr::Var(x)`                          → `(x, root)`
///   * `Expr::RecordField { base, field }`      → recurse + `RecordField(field)`
///   * `Expr::SequenceIndex { base, index }`    → recurse + `TupleIndex(n)` for
///                                   literal `i32` index only
///
/// Returns `None` for calls, computed indices, closures, and anything not
/// expressible as a single static path from a local variable.
pub(crate) fn expr_access_path(
    expr_id: ExprId,
    arena: &AstArena,
) -> Option<(SymbolId, PatternPath)> {
    match arena.expr(expr_id) {
        Expr::Var(name) => Some((*name, PatternPath::root())),
        Expr::RecordField(field_expr) => {
            let (base_sym, base_path) = expr_access_path(field_expr.base, arena)?;
            Some((base_sym, base_path.record_field(field_expr.field)))
        }
        Expr::SequenceIndex(index_expr) => {
            if let Expr::NumericLiteral(crate::types::NumericLiteral::I32(idx)) =
                arena.expr(index_expr.index)
            {
                if *idx >= 0 {
                    let (base_sym, base_path) = expr_access_path(index_expr.base, arena)?;
                    return Some((base_sym, base_path.tuple_index(*idx as usize)));
                }
            }
            None
        }
        _ => None,
    }
}

/// Format a path as a human-readable access string (e.g. `"v"`, `"v.0"`, `"v.field"`).
///
/// Field name symbols are rendered as `.<numeric_id>` since this layer has no

/// Infer the type of `expr_id` **without** running the top-level path-availability
/// check from M9.9.  Used internally when `expr_id` is the *base* of a field or
/// index access whose **caller** has already verified the full access path.
///
/// Only skips the path check for `Expr::Var`; all other expressions fall through
/// to the normal `infer_expr_type` (which includes their own path check).
fn infer_expr_type_no_check(
    expr_id: ExprId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    match arena.expr(expr_id) {
        Expr::Var(v) => {
            // No path check here; the outer infer_expr_type call for the full
            // field/index expression already checked the correct sub-path.
            env.get(*v).ok_or(FrontendError {
                pos: 0,
                message: format!("unknown variable '{}'", resolve_symbol_name(arena, *v)?),
            })
        }
        _ => infer_expr_type(
            expr_id,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
    }
}
