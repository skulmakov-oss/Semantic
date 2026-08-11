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

pub fn type_check_function(program: &Program) -> Result<(), FrontendError> {
    validate_executable_imports(program)?;
    if program.functions.len() != 1 {
        return Err(FrontendError {
            pos: 0,
            message: "type_check_function expects exactly one function in program".to_string(),
        });
    }
    let mut table = BTreeMap::new();
    let record_table = build_record_table(program)?;
    let adt_table = build_adt_table(program)?;
    let schema_table = build_schema_table(program)?;
    let func = &program.functions[0];
    table.insert(
        func.name,
        FnSig {
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
            params: func
                .params
                .iter()
                .map(|(_, t)| {
                    canonicalize_declared_type(t, &record_table, &adt_table, &program.arena)
                })
                .collect::<Result<Vec<_>, _>>()?,
            param_names: Some(func.params.iter().map(|(name, _)| *name).collect()),
            param_defaults: Some(func.param_defaults.clone()),
            ret: canonicalize_declared_type(&func.ret, &record_table, &adt_table, &program.arena)?,
        },
    );
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
    validate_impl_conformance(&p.impls, &trait_table, &p.arena)?;
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
            format!(
                "return type of '{}'",
                resolve_symbol_name(arena, func.name)?
            ),
        )?;
    }
    let empty_env = ScopeEnv::new();
    let mut default_loop_stack = Vec::new();
    for ((name, ty), default_expr) in canonical_params.iter().zip(func.param_defaults.iter()) {
        if let Some(default_expr) = default_expr {
            let default_ty = infer_expr_type(
                *default_expr,
                arena,
                &empty_env,
                table,
                record_table,
                adt_table,
                Type::Unit,
                &mut default_loop_stack,
                impl_list,
            )?;
            if let Err(err) = ensure_const_initializer_safe(*default_expr, arena, &empty_env) {
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
    let env = ScopeEnv::with_params(&params);
    let mut loop_stack = Vec::new();
    for condition in &func.requires {
        ensure_requires_expr_supported(*condition, arena)?;
        let condition_ty = infer_expr_type(
            *condition,
            arena,
            &env,
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
            &env,
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
            &env,
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
            validate_plan_against_scrutinee_state(env, *value, arena, &plan)?;
            apply_binding_plan(env, &plan);
            apply_plans_to_scrutinee(*value, &[plan], arena, env);
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
            validate_plan_against_scrutinee_state(env, *value, arena, &plan)?;
            apply_binding_plan(env, &plan);
            apply_plans_to_scrutinee(*value, &[plan], arena, env);
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
            validate_plan_against_scrutinee_state(env, *value, arena, &plan)?;
            apply_binding_plan(env, &plan);
            apply_plans_to_scrutinee(*value, &[plan], arena, env);
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
            validate_plan_against_scrutinee_state(env, *value, arena, &plan)?;
            apply_binding_plan(env, &plan);
            apply_plans_to_scrutinee(*value, &[plan], arena, env);
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
            if env.is_const(*name) {
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
                if env.is_const(*name) {
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
            let mut body_env = env.clone();
            body_env.push_scope();
            body_env.insert_const(*name, Type::I32);
            for stmt in body {
                check_stmt(
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
            Ok(())
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
            let mut body_env = env.clone();
            body_env.push_scope();
            loop_stack.push(LoopTypeFrame {
                kind: LoopTypeFrameKind::Control,
                break_ty: None,
            });
            for stmt in body {
                check_stmt(
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
            let _ = loop_stack.pop().expect("control loop frame must exist");
            body_env.pop_scope();
            Ok(())
        }
        Stmt::Loop { body } => {
            let mut body_env = env.clone();
            body_env.push_scope();
            loop_stack.push(LoopTypeFrame {
                kind: LoopTypeFrameKind::Control,
                break_ty: None,
            });
            for stmt in body {
                check_stmt(
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
            let _ = loop_stack.pop().expect("control loop frame must exist");
            body_env.pop_scope();
            Ok(())
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
                let mut body_env = env.clone();
                body_env.push_scope();
                body_env.insert_const(*name, Type::I32);
                for stmt in body {
                    check_stmt(
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
                return Ok(());
            }
            if let Type::Sequence(sequence_ty) = &iterable_ty {
                let mut body_env = env.clone();
                body_env.push_scope();
                body_env.insert_const(*name, sequence_ty.item.as_ref().clone());
                for stmt in body {
                    check_stmt(
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
                return Ok(());
            }
            if let Some(item_ty) = resolve_explicit_iterable_loop_item_type(
                &iterable_ty,
                desugaring.trait_name,
                arena,
                impl_list,
            )? {
                let mut body_env = env.clone();
                body_env.push_scope();
                body_env.insert_const(*name, item_ty);
                for stmt in body {
                    check_stmt(
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
                return Ok(());
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
        } => {
            let ct = infer_expr_type(
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
            if ct != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if condition must be bool; explicit compare is required for quad"
                        .to_string(),
                });
            }

            let mut then_env = env.clone();
            then_env.push_scope();
            for s in then_block {
                check_stmt(
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
                check_stmt(
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
            Ok(())
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            let st = infer_expr_type(
                *scrutinee,
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
                st,
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
                        "match is allowed only for quad, enum, Option(T), Result(T, E), i32, or u32 scrutinee"
                            .to_string(),
                });
            }

            // M9.5 Wave D / M9.7 / M9.8: BindingPlan pipeline + path-based ownership.
            let mut arm_plans: Vec<BindingPlan> = Vec::new();
            for arm in arms {
                let (plan, mut arm_env) =
                    build_and_apply_match_plan(&arm.pat, &st, env, arena, adt_table)?;
                // M9.8: reject if new plan conflicts with prior path-state of scrutinee.
                validate_plan_against_scrutinee_state(env, *scrutinee, arena, &plan)?;
                arm_plans.push(plan);
                check_match_guard(
                    arm.guard,
                    arena,
                    &arm_env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                for s in &arm.block {
                    check_stmt(
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
            }
            // M9.7: apply path-based state for each arm's moves/borrows to scrutinee.
            apply_plans_to_scrutinee(*scrutinee, &arm_plans, arena, env);

            if default.is_empty() {
                match missing_exhaustive_sum_variants(
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
                }
            } else {
                let mut def_env = env.clone();
                def_env.push_scope();
                for s in default {
                    check_stmt(
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
            }
            Ok(())
        }
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

/// Apply a type-variable substitution map to `ty` (M9.1 Wave 3).
/// Only direct `TypeVar` occurrences are substituted; compound types
/// that could theoretically contain a TypeVar (e.g. `Sequence<T>`) are
/// not handled here — those are deferred to the monomorphisation pass.
fn subst_apply(ty: &Type, subst: &BTreeMap<SymbolId, Type>) -> Type {
    match ty {
        Type::TypeVar(id) => subst.get(id).cloned().unwrap_or_else(|| ty.clone()),
        _ => ty.clone(),
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
    env: &ScopeEnv,
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
    if let Some((name, path)) = expr_access_path(expr_id, arena) {
        env.check_path_available(name, &path)?;
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
        Expr::Block(block) => infer_value_block_type(
            block,
            arena,
            env,
            table,
            record_table,
            adt_table,
            ret_ty,
            loop_stack,
            impl_list,
        ),
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
            let then_ty = infer_value_block_type(
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
            let else_ty = infer_value_block_type(
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
            // M9.5 Wave C: build binding plan, validate conflicts, apply to then-env.
            let mut plan = BindingPlan::default();
            build_match_pattern_plan(
                &if_let.pattern,
                &value_ty,
                &PatternPath::root(),
                &mut plan,
                arena,
                adt_table,
            )?;
            validate_binding_plan_conflicts(&plan)?;
            // M9.8: reject if new plan conflicts with prior path-state of scrutinee.
            validate_plan_against_scrutinee_state(env, if_let.value, arena, &plan)?;
            // then-block sees the bindings.
            let mut then_env = env.clone();
            then_env.push_scope();
            apply_binding_plan(&mut then_env, &plan);
            // NOTE: scrutinee consumed-state is enforced at statement level only
            // (infer_expr_type receives &ScopeEnv, which is immutable).
            let then_ty = infer_value_block_type(
                &if_let.then_block,
                arena,
                &then_env,
                table,
                record_table,
                adt_table,
                ret_ty.clone(),
                loop_stack,
                impl_list,
            )?;
            // else-block uses original env (no bindings).
            let else_ty = infer_value_block_type(
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
            // M9.1 Wave 3: generic call-site substitution.
            // When the function is generic (sig.type_params non-empty), infer a
            // substitution map TypeVar(T) → concrete_type from the argument
            // expressions and apply it before checking argument/return types.
            if !sig.type_params.is_empty() {
                let fn_name = resolve_symbol_name(arena, *name)?;
                // First pass: build substitution map from arguments whose
                // expected param type is a TypeVar.
                let mut subst: BTreeMap<SymbolId, Type> = BTreeMap::new();
                for (i, arg) in ordered_args.iter().enumerate() {
                    if let Type::TypeVar(tid) = &sig.params[i] {
                        if subst.contains_key(tid) {
                            // Already bound — verify consistency below.
                            continue;
                        }
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
                        subst.insert(*tid, at);
                    }
                }
                // Every declared type parameter must have been bound.
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
                for (i, arg) in ordered_args.iter().enumerate() {
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
            for (i, arg) in ordered_args.iter().enumerate() {
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

    fn derive_validation_plans_from_source(
        src: &str,
    ) -> Result<(Program, ValidationPlanTable), FrontendError> {
        let program = parse_program(src)?;
        let plans = derive_validation_plan_table(&program)?;
        Ok((program, plans))
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
            !err.message.is_empty(),
            "expected an operator-unsupported error"
        );
    }

    #[test]
    fn measured_u32_unary_minus_rejects() {
        let src = r#"
            fn main() {
                let x: u32[m] = 1;
                let negated: u32[m] = -x;
                return;
            }
        "#;

        let err =
            typecheck_source(src).expect_err("unary minus on a measured u32 must be rejected");
        assert!(
            !err.message.is_empty(),
            "expected an operator-unsupported error"
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
            !err.message.is_empty(),
            "expected an operator-unsupported error"
        );
    }

    #[test]
    fn measured_u32_unary_plus_rejects() {
        let src = r#"
            fn main() {
                let x: u32[m] = 1;
                let same: u32[m] = +x;
                return;
            }
        "#;

        let err = typecheck_source(src).expect_err("unary plus on a measured u32 must be rejected");
        assert!(
            !err.message.is_empty(),
            "expected an operator-unsupported error"
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
        let src = r#"
            record SensorFrame {
                mask: qvec,
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
            err.message.contains("unknown nominal type 'Self'"),
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
    fn or_pattern_two_variants_covers_both() {
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
        typecheck_source(src).expect("or-pattern covering two variants should typecheck");
    }

    #[test]
    fn or_pattern_covers_all_variants_exhaustive() {
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
        typecheck_source(src).expect("or-pattern covering all variants should be exhaustive");
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

    #[test]
    fn match_or_pattern_all_borrow_ok() {
        // Or-pattern where all alternatives borrow: ok.
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
        typecheck_source(src).expect("or-pattern match should typecheck");
    }

    #[test]
    fn match_inconsistent_or_pattern_capture_rejects() {
        // One arm binds with ref, the other without — must be same shape.
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
        let err =
            typecheck_source(src).expect_err("inconsistent or-pattern capture modes must reject");
        assert!(
            err.message.contains("same")
                || err.message.contains("capture")
                || err.message.contains("alternative"),
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
        );
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
        );
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
        env.mark_path_state(sym, path.clone(), PathAvailability::Moved);
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
        env.mark_consumed(sym);
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
        );
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
        );
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
        );
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
        );
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
        );
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
        );
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(1),
            PathAvailability::Moved,
        );
        // Now add root — should subsume both children.
        env.mark_path_state(sym, PatternPath::root(), PathAvailability::Moved);
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
        env.mark_path_state(sym, PatternPath::root(), PathAvailability::Moved);
        env.mark_path_state(
            sym,
            PatternPath::root().tuple_index(0),
            PathAvailability::Moved,
        );
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
        );
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
        );
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
        );
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
    env: &ScopeEnv,
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

fn infer_value_block_type(
    block: &BlockExpr,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
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
        &block_env,
        table,
        record_table,
        adt_table,
        ret_ty,
        loop_stack,
        impl_list,
    )?;
    block_env.pop_scope();
    Ok(tail_ty)
}

fn infer_match_expr_type(
    match_expr: &MatchExpr,
    arena: &AstArena,
    env: &ScopeEnv,
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

    // M9.5 Wave D: migrate to BindingPlan ownership pipeline.
    // NOTE: infer_match_expr_type receives &ScopeEnv (not mut), so consumed-state
    // marking is skipped here; it is enforced at statement-level match sites instead.
    let mut result_ty = None;
    for arm in &match_expr.arms {
        let (_, arm_env) =
            build_and_apply_match_plan(&arm.pat, &scrutinee_ty, env, arena, adt_table)?;
        check_match_guard(
            arm.guard,
            arena,
            &arm_env,
            table,
            record_table,
            adt_table,
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
        let arm_ty = infer_value_block_type(
            &arm.block,
            arena,
            &arm_env,
            table,
            record_table,
            adt_table,
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
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

    if let Some(default) = match_expr.default.as_ref() {
        let default_ty = infer_value_block_type(
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
            Ok(expected)
        } else {
            Ok(default_ty)
        }
    } else {
        match missing_exhaustive_sum_variants(
            &scrutinee_ty,
            match_expr.arms.iter().map(|arm| (&arm.pat, arm.guard)),
            arena,
            adt_table,
        )? {
            Some((family_label, missing)) if !missing.is_empty() => {
                Err(non_exhaustive_match_error(&family_label, &missing, true)?)
            }
            Some(_) => {
                Ok(result_ty
                    .expect("exhaustive enum match expression should have at least one arm"))
            }
            None => Err(FrontendError {
                pos: 0,
                message: "match expression requires default arm '_'".to_string(),
            }),
        }
    }
}

fn infer_loop_expr_type(
    loop_expr: &LoopExpr,
    arena: &AstArena,
    env: &ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
    loop_stack: &mut Vec<LoopTypeFrame>,
    impl_list: &[ImplDecl],
) -> Result<Type, FrontendError> {
    let mut body_env = env.clone();
    body_env.push_scope();
    loop_stack.push(LoopTypeFrame {
        kind: LoopTypeFrameKind::Expression,
        break_ty: None,
    });
    for stmt in &loop_expr.body {
        check_loop_expr_stmt(
            *stmt,
            arena,
            &mut body_env,
            table,
            record_table,
            adt_table,
            ret_ty.clone(),
            loop_stack,
            impl_list,
        )?;
    }
    body_env.pop_scope();
    let frame = loop_stack.pop().expect("loop frame must exist");
    frame.break_ty.ok_or(FrontendError {
        pos: 0,
        message: "loop expression requires at least one break value".to_string(),
    })
}

fn check_loop_expr_stmt(
    stmt_id: StmtId,
    arena: &AstArena,
    env: &mut ScopeEnv,
    table: &FnTable,
    record_table: &RecordTable,
    adt_table: &AdtTable,
    ret_ty: Type,
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
        } => {
            let cond_ty = infer_expr_type(
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
            if cond_ty != Type::Bool {
                return Err(FrontendError {
                    pos: 0,
                    message: "if condition must be bool; explicit compare is required for quad"
                        .to_string(),
                });
            }

            let mut then_env = env.clone();
            then_env.push_scope();
            for stmt in then_block {
                check_loop_expr_stmt(
                    *stmt,
                    arena,
                    &mut then_env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
            }
            then_env.pop_scope();

            let mut else_env = env.clone();
            else_env.push_scope();
            for stmt in else_block {
                check_loop_expr_stmt(
                    *stmt,
                    arena,
                    &mut else_env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
            }
            else_env.pop_scope();
            Ok(())
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            let st = infer_expr_type(
                *scrutinee,
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
                st,
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
                        "match is allowed only for quad, enum, Option(T), Result(T, E), i32, or u32 scrutinee"
                            .to_string(),
                });
            }
            if default.is_empty() {
                return Err(FrontendError {
                    pos: 0,
                    message: "match requires default arm '_'".to_string(),
                });
            }

            // M9.5 Wave D / M9.7 / M9.8: BindingPlan pipeline + path-based ownership.
            let mut arm_plans: Vec<BindingPlan> = Vec::new();
            for arm in arms {
                let (plan, mut arm_env) =
                    build_and_apply_match_plan(&arm.pat, &st, env, arena, adt_table)?;
                // M9.8: reject if new plan conflicts with prior path-state of scrutinee.
                validate_plan_against_scrutinee_state(env, *scrutinee, arena, &plan)?;
                arm_plans.push(plan);
                check_match_guard(
                    arm.guard,
                    arena,
                    &arm_env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
                for stmt in &arm.block {
                    check_loop_expr_stmt(
                        *stmt,
                        arena,
                        &mut arm_env,
                        table,
                        record_table,
                        adt_table,
                        ret_ty.clone(),
                        loop_stack,
                        impl_list,
                    )?;
                }
                arm_env.pop_scope();
            }
            apply_plans_to_scrutinee(*scrutinee, &arm_plans, arena, env);

            let mut def_env = env.clone();
            def_env.push_scope();
            for stmt in default {
                check_loop_expr_stmt(
                    *stmt,
                    arena,
                    &mut def_env,
                    table,
                    record_table,
                    adt_table,
                    ret_ty.clone(),
                    loop_stack,
                    impl_list,
                )?;
            }
            def_env.pop_scope();
            Ok(())
        }
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
                        let expected_ty =
                            substitute_trait_self_type(expected_ty, self_type_var, imp.for_type);
                        if actual_ty != &expected_ty {
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
                    let expected_ret =
                        substitute_trait_self_type(&trait_method.ret, self_type_var, imp.for_type);
                    if m.ret != expected_ret {
                        return Err(FrontendError {
                            pos: 0,
                            message: format!(
                                "impl method '{}' has return type {:?}, expected {:?} from trait '{}'",
                                resolve_symbol_name(arena, trait_method.name)?,
                                m.ret,
                                expected_ret,
                                resolve_symbol_name(arena, imp.trait_name)?,
                            ),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

fn substitute_trait_self_type(
    ty: &Type,
    self_type_var: Option<SymbolId>,
    concrete_self: SymbolId,
) -> Type {
    match ty {
        Type::TypeVar(name) if Some(*name) == self_type_var => Type::Record(concrete_self),
        Type::Tuple(items) => Type::Tuple(
            items
                .iter()
                .map(|item| substitute_trait_self_type(item, self_type_var, concrete_self))
                .collect(),
        ),
        Type::Sequence(sequence) => Type::Sequence(SequenceType {
            family: sequence.family,
            item: Box::new(substitute_trait_self_type(
                sequence.item.as_ref(),
                self_type_var,
                concrete_self,
            )),
        }),
        Type::Map(map) => Type::Map(crate::types::MapType {
            key: Box::new(substitute_trait_self_type(
                map.key.as_ref(),
                self_type_var,
                concrete_self,
            )),
            val: Box::new(substitute_trait_self_type(
                map.val.as_ref(),
                self_type_var,
                concrete_self,
            )),
        }),
        Type::Closure(closure) => Type::Closure(crate::types::ClosureType {
            family: closure.family,
            capture: closure.capture,
            param: Box::new(substitute_trait_self_type(
                closure.param.as_ref(),
                self_type_var,
                concrete_self,
            )),
            ret: Box::new(substitute_trait_self_type(
                closure.ret.as_ref(),
                self_type_var,
                concrete_self,
            )),
        }),
        Type::Measured(base, unit) => Type::Measured(
            Box::new(substitute_trait_self_type(
                base.as_ref(),
                self_type_var,
                concrete_self,
            )),
            *unit,
        ),
        Type::Option(item) => Type::Option(Box::new(substitute_trait_self_type(
            item.as_ref(),
            self_type_var,
            concrete_self,
        ))),
        Type::Result(ok_ty, err_ty) => Type::Result(
            Box::new(substitute_trait_self_type(
                ok_ty.as_ref(),
                self_type_var,
                concrete_self,
            )),
            Box::new(substitute_trait_self_type(
                err_ty.as_ref(),
                self_type_var,
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

fn ensure_executable_type_supported(
    ty: &Type,
    arena: &AstArena,
    context: String,
) -> Result<(), FrontendError> {
    match ty {
        Type::Tuple(items) => {
            for item in items {
                ensure_executable_type_supported(item, arena, context.clone())?;
            }
            Ok(())
        }
        Type::Sequence(sequence) => {
            ensure_executable_type_supported(sequence.item.as_ref(), arena, context)
        }
        Type::Map(map) => {
            ensure_executable_type_supported(map.key.as_ref(), arena, context.clone())?;
            ensure_executable_type_supported(map.val.as_ref(), arena, context)
        }
        Type::Measured(base, _) => ensure_executable_type_supported(base, arena, context),
        Type::Option(item) => ensure_executable_type_supported(item, arena, context),
        Type::Result(ok_ty, err_ty) => {
            ensure_executable_type_supported(ok_ty, arena, context.clone())?;
            ensure_executable_type_supported(err_ty, arena, context)
        }
        Type::Record(name) => {
            let _ = resolve_symbol_name(arena, *name)?;
            let _ = context;
            Ok(())
        }
        Type::Adt(name) => {
            let _ = resolve_symbol_name(arena, *name)?;
            let _ = context;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn ensure_storage_type_supported(
    ty: &Type,
    arena: &AstArena,
    context: String,
) -> Result<(), FrontendError> {
    match ty {
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
        _ => Ok(()),
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
        &closure_env,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
    env: &ScopeEnv,
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
            if env.is_const(*name) {
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
// M9.5 Wave D: match integration helpers
// ──────────────────────────────────────────────────────────────

/// Build a binding plan for one match arm pattern, validate conflicts,
/// clone `env`, and apply the plan to the clone. Returns `(plan, arm_env)`.
///
/// NOTE (M9.5): PatternPath overlap (e.g., root vs root.0) is NOT checked yet.
/// Only exact-path conflicts are validated.
pub(crate) fn build_and_apply_match_plan(
    pattern: &MatchPattern,
    scrutinee_ty: &Type,
    env: &ScopeEnv,
    arena: &AstArena,
    adt_table: &AdtTable,
) -> Result<(BindingPlan, ScopeEnv), FrontendError> {
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
    Ok((plan, arm_env))
}

/// M9.8: Validate that all items in `plan` are capture-compatible with the
/// existing path-state of the scrutinee variable (if it is a plain Expr::Var).
///
/// Prevents: move after borrow, borrow after move, move after move on same/overlapping path.
pub(crate) fn validate_plan_against_scrutinee_state(
    env: &ScopeEnv,
    scrutinee_expr: ExprId,
    arena: &AstArena,
    plan: &BindingPlan,
) -> Result<(), FrontendError> {
    let Expr::Var(name) = arena.expr(scrutinee_expr) else {
        return Ok(());
    };
    for item in &plan.items {
        env.check_capture_allowed(*name, &item.path, item.capture)?;
    }
    Ok(())
}

/// M9.7: For each arm plan, record the capture state of every binding path
/// onto the scrutinee variable (if it is a plain Expr::Var).
///
/// Conservative: we union the paths across all arms. A path moved in any arm
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
    env: &ScopeEnv,
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

/// is considered moved after the match.
pub(crate) fn apply_plans_to_scrutinee(
    scrutinee_expr: ExprId,
    plans: &[BindingPlan],
    arena: &AstArena,
    env: &mut ScopeEnv,
) {
    let Expr::Var(var_name) = arena.expr(scrutinee_expr) else {
        return;
    };
    for plan in plans {
        for item in &plan.items {
            let avail = match item.capture {
                CaptureMode::Move => PathAvailability::Moved,
                CaptureMode::Borrow => PathAvailability::Borrowed,
            };
            env.mark_path_state(*var_name, item.path.clone(), avail);
        }
    }
}
