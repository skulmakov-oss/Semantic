use crate::package_manifest::{admit_package_entry_module, resolve_package_import_path};
use sm_front::types::{
    AstArena, ExecutableImport, Expr, ExprId, Function, Stmt, StmtId, SymbolId, TokenKind, Type,
};
use sm_front::{lex, parse_program_with_profile, ParserProfile};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) fn executable_import_wave2_out_of_scope_message() -> &'static str {
    "top-level executable Import admits direct local-path and package-qualified helper modules plus selected local imports; alias, wildcard, and re-export forms remain out of scope"
}

pub(crate) fn read_source_with_package_admission(path: &Path) -> Result<String, String> {
    admit_package_entry_module(path)
        .map(|_| ())
        .map_err(|e| e.to_string())?;
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("failed to resolve '{}': {}", path.display(), e))?;
    let source = std::fs::read_to_string(&canonical)
        .map_err(|e| format!("failed to read '{}': {}", canonical.display(), e))?;
    let parser_profile = ParserProfile::foundation_default();
    let program = match parse_program_with_profile(&source, &parser_profile) {
        Ok(program) => program,
        Err(_) => return Ok(source),
    };
    if program.imports.is_empty() {
        return Ok(source);
    }
    let mut visiting = Vec::<PathBuf>::new();
    let mut planned = BTreeMap::<PathBuf, ExecutableBundleModule>::new();
    let mut order = Vec::<PathBuf>::new();
    collect_executable_bundle_plan(
        &canonical,
        ExecutableBundleMode::Full,
        &parser_profile,
        &mut visiting,
        &mut planned,
        &mut order,
    )?;
    let mut bundle = String::new();
    for (idx, module_path) in order.into_iter().enumerate() {
        let module = planned.get(&module_path).ok_or_else(|| {
            format!(
                "missing executable bundle module '{}'",
                module_path.display()
            )
        })?;
        let module_source = render_executable_bundle_module(module)?;
        if idx > 0 {
            bundle.push_str("\n\n");
        }
        bundle.push_str(&module_source);
        if !module_source.ends_with('\n') {
            bundle.push('\n');
        }
    }
    Ok(bundle)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct SelectedExecutableBindings {
    by_symbol: BTreeMap<String, BTreeSet<String>>,
}

impl SelectedExecutableBindings {
    fn insert(&mut self, original: String, public_name: String) {
        self.by_symbol
            .entry(original)
            .or_default()
            .insert(public_name);
    }

    fn merge_from(&mut self, other: &SelectedExecutableBindings) {
        for (original, public_names) in &other.by_symbol {
            for public_name in public_names {
                self.insert(original.clone(), public_name.clone());
            }
        }
    }

    fn selected_names(&self) -> BTreeSet<String> {
        self.by_symbol.keys().cloned().collect()
    }

    fn public_bindings(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (original, public_names) in &self.by_symbol {
            for public_name in public_names {
                out.push((original.clone(), public_name.clone()));
            }
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExecutableBundleMode {
    Full,
    Selected(SelectedExecutableBindings),
}

impl ExecutableBundleMode {
    fn merge(&mut self, requested: ExecutableBundleMode) {
        match requested {
            ExecutableBundleMode::Full => *self = ExecutableBundleMode::Full,
            ExecutableBundleMode::Selected(bindings) => match self {
                ExecutableBundleMode::Full => {}
                ExecutableBundleMode::Selected(existing) => existing.merge_from(&bindings),
            },
        }
    }
}

#[derive(Debug, Clone)]
struct ExecutableBundleModule {
    path: PathBuf,
    source: String,
    program: sm_front::Program,
    mode: ExecutableBundleMode,
}

fn collect_executable_bundle_plan(
    path: &Path,
    requested_mode: ExecutableBundleMode,
    parser_profile: &ParserProfile,
    visiting: &mut Vec<PathBuf>,
    planned: &mut BTreeMap<PathBuf, ExecutableBundleModule>,
    order: &mut Vec<PathBuf>,
) -> Result<(), String> {
    admit_package_entry_module(path)
        .map(|_| ())
        .map_err(|e| e.to_string())?;
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("failed to resolve '{}': {}", path.display(), e))?;
    if let Some(existing) = planned.get_mut(&canonical) {
        existing.mode.merge(requested_mode);
        return Ok(());
    }
    if let Some(pos) = visiting.iter().position(|entry| entry == &canonical) {
        let mut chain = visiting[pos..]
            .iter()
            .map(|entry| entry.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>();
        chain.push(canonical.to_string_lossy().replace('\\', "/"));
        return Err(format!(
            "cyclic executable helper import detected: {}",
            chain.join(" -> ")
        ));
    }
    let source = std::fs::read_to_string(&canonical)
        .map_err(|e| format!("failed to read '{}': {}", canonical.display(), e))?;
    let program = parse_program_with_profile(&source, parser_profile).map_err(|e| {
        format!(
            "executable helper module '{}' must parse on the Rust-like source path: {}",
            canonical.display(),
            e
        )
    })?;
    visiting.push(canonical.clone());
    for import in &program.imports {
        validate_executable_bundle_import(&canonical, import)?;
        let child = resolve_package_import_path(&canonical, &import.spec)
            .map_err(|e| format!("{}: {}", canonical.display(), e))?;
        let child_mode = requested_bundle_mode_for_import(&program.arena, import);
        collect_executable_bundle_plan(
            &child,
            child_mode,
            parser_profile,
            visiting,
            planned,
            order,
        )?;
    }
    let _ = visiting.pop();
    planned.insert(
        canonical.clone(),
        ExecutableBundleModule {
            path: canonical.clone(),
            source,
            program,
            mode: requested_mode,
        },
    );
    order.push(canonical);
    Ok(())
}

fn requested_bundle_mode_for_import(
    arena: &AstArena,
    import: &ExecutableImport,
) -> ExecutableBundleMode {
    if import.select_items.is_empty() {
        return ExecutableBundleMode::Full;
    }
    let mut bindings = SelectedExecutableBindings::default();
    for item in &import.select_items {
        let original = arena.symbol_name(item.name).to_string();
        let public_name = item
            .alias
            .map(|sym| arena.symbol_name(sym).to_string())
            .unwrap_or_else(|| original.clone());
        bindings.insert(original, public_name);
    }
    ExecutableBundleMode::Selected(bindings)
}

fn validate_executable_bundle_import(
    importer: &Path,
    import: &ExecutableImport,
) -> Result<(), String> {
    if import.reexport || import.wildcard || import.alias.is_some() {
        return Err(format!(
            "{} in '{}'",
            executable_import_wave2_out_of_scope_message(),
            importer.display()
        ));
    }
    Ok(())
}

fn render_executable_bundle_module(module: &ExecutableBundleModule) -> Result<String, String> {
    match &module.mode {
        ExecutableBundleMode::Full => Ok(module.source.clone()),
        ExecutableBundleMode::Selected(bindings) => {
            synthesize_selected_executable_module(module, bindings)
        }
    }
}

fn selected_executable_import_function_only_message(importer: &Path, spec: &str) -> String {
    format!(
        "selected executable helper import '{}' in '{}' currently supports function-only helper modules; records, enums, schemas, traits, and impls remain out of scope",
        spec,
        importer.display()
    )
}

fn selected_executable_import_missing_symbol_message(
    importer: &Path,
    spec: &str,
    symbol: &str,
) -> String {
    format!(
        "selected executable helper import '{}' in '{}' is missing symbol '{}'",
        spec,
        importer.display(),
        symbol
    )
}

fn selected_executable_import_public_collision_message(
    importer: &Path,
    spec: &str,
    public_name: &str,
) -> String {
    format!(
        "selected executable helper import '{}' in '{}' binds duplicate public symbol '{}'",
        spec,
        importer.display(),
        public_name
    )
}

fn selected_executable_import_wrapper_gap_message(
    importer: &Path,
    spec: &str,
    symbol: &str,
) -> String {
    format!(
        "selected executable helper import '{}' in '{}' currently supports helper functions without defaulted parameters; '{}' stays out of scope",
        spec,
        importer.display(),
        symbol
    )
}

fn synthesize_selected_executable_module(
    module: &ExecutableBundleModule,
    bindings: &SelectedExecutableBindings,
) -> Result<String, String> {
    if !module.program.records.is_empty()
        || !module.program.adts.is_empty()
        || !module.program.schemas.is_empty()
        || !module.program.traits.is_empty()
        || !module.program.impls.is_empty()
    {
        return Err(selected_executable_import_function_only_message(
            &module.path,
            &module
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
        ));
    }

    let function_sources = extract_top_level_function_sources(&module.source)?;
    let mut functions_by_name = BTreeMap::<String, &Function>::new();
    for func in &module.program.functions {
        let name = module.program.arena.symbol_name(func.name).to_string();
        functions_by_name.insert(name, func);
    }

    for selected in bindings.selected_names() {
        if !functions_by_name.contains_key(&selected) {
            return Err(selected_executable_import_missing_symbol_message(
                &module.path,
                &module
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                &selected,
            ));
        }
    }

    let required = collect_required_local_functions(
        &module.program.arena,
        &functions_by_name,
        &bindings.selected_names(),
    );
    let prefix = format!(
        "execsel_{:016x}_",
        fnv1a64(module.path.to_string_lossy().as_bytes())
    );
    let mut rename_map = BTreeMap::<String, String>::new();
    for name in &required {
        rename_map.insert(name.clone(), format!("{prefix}{name}"));
    }

    let mut public_names = BTreeSet::new();
    for (_original, public_name) in bindings.public_bindings() {
        if !public_names.insert(public_name.clone()) {
            return Err(selected_executable_import_public_collision_message(
                &module.path,
                &module
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                &public_name,
            ));
        }
    }

    let mut pieces = Vec::<String>::new();
    for (name, source) in &function_sources {
        if !required.contains(name) {
            continue;
        }
        pieces.push(rewrite_selected_function_source(source, &rename_map)?);
    }
    for (original, public_name) in bindings.public_bindings() {
        let func = functions_by_name.get(&original).ok_or_else(|| {
            selected_executable_import_missing_symbol_message(
                &module.path,
                &module
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                &original,
            )
        })?;
        let internal = rename_map.get(&original).ok_or_else(|| {
            format!(
                "missing internal executable selected-import binding for '{}' in '{}'",
                original,
                module.path.display()
            )
        })?;
        pieces.push(render_selected_import_wrapper(
            &module.path,
            &module.program.arena,
            func,
            &module
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            &public_name,
            internal,
        )?);
    }

    Ok(pieces.join("\n\n"))
}

fn extract_top_level_function_sources(source: &str) -> Result<Vec<(String, String)>, String> {
    let tokens = lex(source).map_err(|e| e.to_string())?;
    if tokens.is_empty() {
        return Ok(Vec::new());
    }
    let mut starts = Vec::<usize>::new();
    let mut brace_depth = 0i32;
    for (idx, token) in tokens.iter().enumerate() {
        if brace_depth == 0
            && matches!(
                token.kind,
                TokenKind::KwImport
                    | TokenKind::KwFn
                    | TokenKind::KwRecord
                    | TokenKind::KwSchema
                    | TokenKind::KwEnum
                    | TokenKind::KwTrait
                    | TokenKind::KwImpl
            )
        {
            starts.push(idx);
        }
        match token.kind {
            TokenKind::LBrace => brace_depth += 1,
            TokenKind::RBrace => brace_depth -= 1,
            _ => {}
        }
    }

    let mut out = Vec::<(String, String)>::new();
    for (pos, start_idx) in starts.iter().enumerate() {
        let token = &tokens[*start_idx];
        if token.kind != TokenKind::KwFn {
            continue;
        }
        let end_pos = if let Some(next_idx) = starts.get(pos + 1) {
            tokens[*next_idx].pos
        } else {
            source.len()
        };
        let name_token = tokens
            .iter()
            .enumerate()
            .skip(*start_idx + 1)
            .find_map(|(_idx, tok)| (tok.kind == TokenKind::Ident).then_some(tok))
            .ok_or_else(|| "malformed top-level function declaration".to_string())?;
        let snippet = source[token.pos..end_pos].trim().to_string();
        out.push((name_token.text.clone(), snippet));
    }
    Ok(out)
}

fn rewrite_selected_function_source(
    source: &str,
    rename_map: &BTreeMap<String, String>,
) -> Result<String, String> {
    let tokens = lex(source).map_err(|e| e.to_string())?;
    let fn_idx = tokens
        .iter()
        .position(|tok| tok.kind == TokenKind::KwFn)
        .ok_or_else(|| "expected function token in selected helper source".to_string())?;
    let name_idx = tokens
        .iter()
        .enumerate()
        .skip(fn_idx + 1)
        .find_map(|(idx, tok)| (tok.kind == TokenKind::Ident).then_some(idx))
        .ok_or_else(|| "expected function name in selected helper source".to_string())?;

    let mut replacements = Vec::<(usize, usize, String)>::new();
    let decl = &tokens[name_idx];
    if let Some(new_name) = rename_map.get(&decl.text) {
        replacements.push((decl.pos, decl.pos + decl.text.len(), new_name.clone()));
    }

    for idx in 0..tokens.len() {
        let token = &tokens[idx];
        if token.kind != TokenKind::Ident || idx == name_idx {
            continue;
        }
        let Some(new_name) = rename_map.get(&token.text) else {
            continue;
        };
        let next_kind = tokens.get(idx + 1).map(|tok| tok.kind);
        let prev_kind = idx
            .checked_sub(1)
            .and_then(|pos| tokens.get(pos))
            .map(|tok| tok.kind);
        if next_kind == Some(TokenKind::LParen) && prev_kind != Some(TokenKind::Dot) {
            replacements.push((token.pos, token.pos + token.text.len(), new_name.clone()));
        }
    }

    replacements.sort_by_key(|replacement| core::cmp::Reverse(replacement.0));
    let mut out = source.to_string();
    for (start, end, replacement) in replacements {
        out.replace_range(start..end, &replacement);
    }
    Ok(out)
}

fn render_selected_import_wrapper(
    importer: &Path,
    arena: &AstArena,
    func: &Function,
    spec: &str,
    public_name: &str,
    internal_name: &str,
) -> Result<String, String> {
    if func.param_defaults.iter().any(|default| default.is_some()) {
        return Err(selected_executable_import_wrapper_gap_message(
            importer,
            spec,
            arena.symbol_name(func.name),
        ));
    }
    let type_params = render_type_params_with_bounds(arena, &func.type_params, &func.trait_bounds);
    let params = func
        .params
        .iter()
        .map(|(name, ty)| format!("{}: {}", arena.symbol_name(*name), render_type(arena, ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let args = func
        .params
        .iter()
        .map(|(name, _)| arena.symbol_name(*name).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    if func.ret == Type::Unit {
        Ok(format!(
            "fn {public_name}{type_params}({params}) {{\n    {internal_name}({args});\n    return;\n}}"
        ))
    } else {
        Ok(format!(
            "fn {public_name}{type_params}({params}) -> {} {{\n    return {internal_name}({args});\n}}",
            render_type(arena, &func.ret)
        ))
    }
}

fn render_type_params_with_bounds(
    arena: &AstArena,
    type_params: &[SymbolId],
    trait_bounds: &[sm_front::types::TraitBound],
) -> String {
    if type_params.is_empty() {
        return String::new();
    }
    let mut rendered = Vec::<String>::new();
    for param in type_params {
        let mut item = arena.symbol_name(*param).to_string();
        let bounds = trait_bounds
            .iter()
            .filter(|bound| bound.param == *param)
            .map(|bound| arena.symbol_name(bound.bound).to_string())
            .collect::<Vec<_>>();
        if !bounds.is_empty() {
            item.push_str(": ");
            item.push_str(&bounds.join(" + "));
        }
        rendered.push(item);
    }
    format!("<{}>", rendered.join(", "))
}

fn render_type(arena: &AstArena, ty: &Type) -> String {
    match ty {
        Type::Quad => "quad".to_string(),
        Type::QVec(width) => format!("qvec{}", width),
        Type::Bool => "bool".to_string(),
        Type::Text => "text".to_string(),
        Type::Sequence(sequence) => format!("Sequence({})", render_type(arena, &sequence.item)),
        Type::Map(map) => format!(
            "Map({}, {})",
            render_type(arena, &map.key),
            render_type(arena, &map.val)
        ),
        Type::Closure(closure) => format!(
            "fn({}) -> {}",
            render_type(arena, &closure.param),
            render_type(arena, &closure.ret)
        ),
        Type::I32 => "i32".to_string(),
        Type::U32 => "u32".to_string(),
        Type::Fx => "fx".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Measured(base, unit) => {
            format!("{}[{}]", render_type(arena, base), arena.symbol_name(*unit))
        }
        Type::RangeI32 => "RangeI32".to_string(),
        Type::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(|item| render_type(arena, item))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Option(inner) => format!("Option({})", render_type(arena, inner)),
        Type::Result(ok, err) => format!(
            "Result({}, {})",
            render_type(arena, ok),
            render_type(arena, err)
        ),
        Type::Record(name) | Type::Adt(name) | Type::TypeVar(name) => {
            arena.symbol_name(*name).to_string()
        }
        Type::Unit => "()".to_string(),
    }
}

fn collect_required_local_functions(
    arena: &AstArena,
    functions_by_name: &BTreeMap<String, &Function>,
    selected: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut required = BTreeSet::<String>::new();
    let mut queue: Vec<String> = selected.iter().cloned().collect();
    while let Some(name) = queue.pop() {
        if !required.insert(name.clone()) {
            continue;
        }
        let Some(func) = functions_by_name.get(&name) else {
            continue;
        };
        let mut deps = BTreeSet::<String>::new();
        collect_local_calls_from_function(arena, func, functions_by_name, &mut deps);
        for dep in deps {
            if !required.contains(&dep) {
                queue.push(dep);
            }
        }
    }
    required
}

fn collect_local_calls_from_function(
    arena: &AstArena,
    func: &Function,
    functions_by_name: &BTreeMap<String, &Function>,
    out: &mut BTreeSet<String>,
) {
    for expr in func.param_defaults.iter().flatten() {
        collect_local_calls_from_expr(arena, *expr, functions_by_name, out);
    }
    for expr in &func.requires {
        collect_local_calls_from_expr(arena, *expr, functions_by_name, out);
    }
    for expr in &func.ensures {
        collect_local_calls_from_expr(arena, *expr, functions_by_name, out);
    }
    for expr in &func.invariants {
        collect_local_calls_from_expr(arena, *expr, functions_by_name, out);
    }
    for stmt in &func.body {
        collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
    }
}

fn collect_local_calls_from_stmt(
    arena: &AstArena,
    stmt_id: StmtId,
    functions_by_name: &BTreeMap<String, &Function>,
    out: &mut BTreeSet<String>,
) {
    match arena.stmt(stmt_id) {
        Stmt::Const { value, .. }
        | Stmt::Let { value, .. }
        | Stmt::Discard { value, .. }
        | Stmt::Assign { value, .. }
        | Stmt::AssignTuple { value, .. } => {
            collect_local_calls_from_expr(arena, *value, functions_by_name, out)
        }
        Stmt::Break(Some(value)) => {
            collect_local_calls_from_expr(arena, *value, functions_by_name, out)
        }
        Stmt::Break(None) | Stmt::Continue => {}
        Stmt::LetTuple { value, .. } | Stmt::LetRecord { value, .. } => {
            collect_local_calls_from_expr(arena, *value, functions_by_name, out);
        }
        Stmt::LetElseRecord {
            value, else_return, ..
        }
        | Stmt::LetElseTuple {
            value, else_return, ..
        } => {
            collect_local_calls_from_expr(arena, *value, functions_by_name, out);
            if let Some(expr) = else_return {
                collect_local_calls_from_expr(arena, *expr, functions_by_name, out);
            }
        }
        Stmt::ForEach { iterable, body, .. } => {
            collect_local_calls_from_expr(arena, *iterable, functions_by_name, out);
            for stmt in body {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
        }
        Stmt::ForRange { range, body, .. } => {
            collect_local_calls_from_expr(arena, *range, functions_by_name, out);
            for stmt in body {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
        }
        Stmt::While { condition, body } => {
            collect_local_calls_from_expr(arena, *condition, functions_by_name, out);
            for stmt in body {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
        }
        Stmt::Loop { body } => {
            for stmt in body {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
        }
        Stmt::Guard {
            condition,
            else_return,
        } => {
            collect_local_calls_from_expr(arena, *condition, functions_by_name, out);
            if let Some(expr) = else_return {
                collect_local_calls_from_expr(arena, *expr, functions_by_name, out);
            }
        }
        Stmt::If {
            condition,
            then_block,
            else_block,
        } => {
            collect_local_calls_from_expr(arena, *condition, functions_by_name, out);
            for stmt in then_block {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
            for stmt in else_block {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
        }
        Stmt::Match {
            scrutinee,
            arms,
            default,
        } => {
            collect_local_calls_from_expr(arena, *scrutinee, functions_by_name, out);
            for arm in arms {
                if let Some(guard) = arm.guard {
                    collect_local_calls_from_expr(arena, guard, functions_by_name, out);
                }
                for stmt in &arm.block {
                    collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
                }
            }
            for stmt in default {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
        }
        Stmt::Return(expr) => {
            if let Some(expr) = expr {
                collect_local_calls_from_expr(arena, *expr, functions_by_name, out);
            }
        }
        Stmt::Expr(expr) => collect_local_calls_from_expr(arena, *expr, functions_by_name, out),
    }
}

fn collect_local_calls_from_expr(
    arena: &AstArena,
    expr_id: ExprId,
    functions_by_name: &BTreeMap<String, &Function>,
    out: &mut BTreeSet<String>,
) {
    match arena.expr(expr_id) {
        Expr::SequenceLiteral(sequence) => {
            for item in &sequence.items {
                collect_local_calls_from_expr(arena, *item, functions_by_name, out);
            }
        }
        Expr::Tuple(items) => {
            for item in items {
                collect_local_calls_from_expr(arena, *item, functions_by_name, out);
            }
        }
        Expr::RecordLiteral(record) => {
            for field in &record.fields {
                collect_local_calls_from_expr(arena, field.value, functions_by_name, out);
            }
        }
        Expr::RecordField(record) => {
            collect_local_calls_from_expr(arena, record.base, functions_by_name, out);
        }
        Expr::SequenceIndex(index) => {
            collect_local_calls_from_expr(arena, index.base, functions_by_name, out);
            collect_local_calls_from_expr(arena, index.index, functions_by_name, out);
        }
        Expr::Closure(closure) => {
            collect_local_calls_from_expr(arena, closure.body, functions_by_name, out);
        }
        Expr::RecordUpdate(update) => {
            collect_local_calls_from_expr(arena, update.base, functions_by_name, out);
            for field in &update.fields {
                collect_local_calls_from_expr(arena, field.value, functions_by_name, out);
            }
        }
        Expr::AdtCtor(ctor) => {
            for payload in &ctor.payload {
                collect_local_calls_from_expr(arena, *payload, functions_by_name, out);
            }
        }
        Expr::Call(callee, args) => {
            let callee_name = arena.symbol_name(*callee).to_string();
            if functions_by_name.contains_key(&callee_name) {
                out.insert(callee_name);
            }
            for arg in args {
                collect_local_calls_from_expr(arena, arg.value, functions_by_name, out);
            }
        }
        Expr::Unary(_, inner) => {
            collect_local_calls_from_expr(arena, *inner, functions_by_name, out)
        }
        Expr::Binary(left, _, right) => {
            collect_local_calls_from_expr(arena, *left, functions_by_name, out);
            collect_local_calls_from_expr(arena, *right, functions_by_name, out);
        }
        Expr::Range(range) => {
            collect_local_calls_from_expr(arena, range.start, functions_by_name, out);
            collect_local_calls_from_expr(arena, range.end, functions_by_name, out);
        }
        Expr::Block(block) => {
            for stmt in &block.statements {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
            collect_local_calls_from_expr(arena, block.tail, functions_by_name, out);
        }
        Expr::If(expr) => {
            collect_local_calls_from_expr(arena, expr.condition, functions_by_name, out);
            collect_local_calls_from_block(arena, &expr.then_block, functions_by_name, out);
            collect_local_calls_from_block(arena, &expr.else_block, functions_by_name, out);
        }
        Expr::IfLet(expr) => {
            collect_local_calls_from_expr(arena, expr.value, functions_by_name, out);
            collect_local_calls_from_block(arena, &expr.then_block, functions_by_name, out);
            collect_local_calls_from_block(arena, &expr.else_block, functions_by_name, out);
        }
        Expr::Match(expr) => {
            collect_local_calls_from_expr(arena, expr.scrutinee, functions_by_name, out);
            for arm in &expr.arms {
                if let Some(guard) = arm.guard {
                    collect_local_calls_from_expr(arena, guard, functions_by_name, out);
                }
                collect_local_calls_from_block(arena, &arm.block, functions_by_name, out);
            }
            if let Some(default) = &expr.default {
                collect_local_calls_from_block(arena, default, functions_by_name, out);
            }
        }
        Expr::Loop(loop_expr) => {
            for stmt in &loop_expr.body {
                collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
            }
        }
        Expr::QuadLiteral(_)
        | Expr::BoolLiteral(_)
        | Expr::TextLiteral(_)
        | Expr::NumericLiteral(_)
        | Expr::Var(_) => {}
    }
}

fn collect_local_calls_from_block(
    arena: &AstArena,
    block: &sm_front::types::BlockExpr,
    functions_by_name: &BTreeMap<String, &Function>,
    out: &mut BTreeSet<String>,
) {
    for stmt in &block.statements {
        collect_local_calls_from_stmt(arena, *stmt, functions_by_name, out);
    }
    collect_local_calls_from_expr(arena, block.tail, functions_by_name, out);
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
