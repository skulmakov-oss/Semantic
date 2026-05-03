use sm_front::{
    build_record_table, build_schema_table, derive_validation_plan_table, parse_program,
    resolve_symbol_name, FrontendError, Program, QuadVal, RecordDecl, RecordTable, SchemaDecl,
    SchemaRole, SchemaTable, Type, ValidationFieldPlan, ValidationPlanTable, ValidationShapePlan,
    ValidationVariantPlan,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigDocument {
    pub fields: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntry {
    pub key: String,
    pub value: ConfigValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigNumberKind {
    Integer,
    Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigNumber {
    pub raw: String,
    pub kind: ConfigNumberKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValue {
    Object(Vec<ConfigEntry>),
    String(String),
    Bool(bool),
    Quad(QuadVal),
    Number(ConfigNumber),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigParseError {
    pub pos: usize,
    pub message: String,
}

impl fmt::Display for ConfigParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "config parse error at {}: {}", self.pos, self.message)
    }
}

impl Error for ConfigParseError {}

pub fn parse_config_document(src: &str) -> Result<ConfigDocument, ConfigParseError> {
    let mut parser = ConfigParser::new(src);
    parser.skip_ws();
    if parser.peek() != Some(b'{') {
        return Err(parser.error("config document must start with '{'"));
    }
    let fields = parser.parse_object_entries()?;
    parser.skip_ws();
    if !parser.is_eof() {
        return Err(parser.error("unexpected trailing input after config document"));
    }
    Ok(ConfigDocument { fields })
}

#[derive(Debug, Clone)]
pub struct ConfigContract {
    program: Program,
    record_table: RecordTable,
    schema_table: SchemaTable,
    validation_plans: ValidationPlanTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigContractBuildError {
    pub message: String,
}

impl fmt::Display for ConfigContractBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "config contract build error: {}", self.message)
    }
}

impl Error for ConfigContractBuildError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigValidationDiagnostic {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigValidationError {
    pub schema_name: String,
    pub diagnostics: Vec<ConfigValidationDiagnostic>,
}

impl fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "config validation failed for schema '{}': {} diagnostic(s)",
            self.schema_name,
            self.diagnostics.len()
        )
    }
}

impl Error for ConfigValidationError {}

pub fn build_config_contract(src: &str) -> Result<ConfigContract, ConfigContractBuildError> {
    let program = parse_program(src).map_err(config_contract_build_error)?;
    let record_table = build_record_table(&program).map_err(config_contract_build_error)?;
    let schema_table = build_schema_table(&program).map_err(config_contract_build_error)?;
    let validation_plans =
        derive_validation_plan_table(&program).map_err(config_contract_build_error)?;
    Ok(ConfigContract {
        program,
        record_table,
        schema_table,
        validation_plans,
    })
}

pub fn validate_config_document(
    contract: &ConfigContract,
    schema_name: &str,
    document: &ConfigDocument,
) -> Result<(), ConfigValidationError> {
    let Some((schema_symbol, schema_decl)) = contract.find_schema_decl(schema_name) else {
        return Err(ConfigValidationError {
            schema_name: schema_name.to_string(),
            diagnostics: vec![ConfigValidationDiagnostic {
                path: "<root>".to_string(),
                message: format!("unknown config schema '{}'", schema_name),
            }],
        });
    };

    if schema_decl.role != Some(SchemaRole::Config) {
        return Err(ConfigValidationError {
            schema_name: schema_name.to_string(),
            diagnostics: vec![ConfigValidationDiagnostic {
                path: "<root>".to_string(),
                message: format!("schema '{}' is not declared as config schema", schema_name),
            }],
        });
    }

    let Some(plan) = contract.validation_plans.get(&schema_symbol) else {
        return Err(ConfigValidationError {
            schema_name: schema_name.to_string(),
            diagnostics: vec![ConfigValidationDiagnostic {
                path: "<root>".to_string(),
                message: format!(
                    "missing canonical validation plan for config schema '{}'",
                    schema_name
                ),
            }],
        });
    };

    let mut diagnostics = Vec::new();
    match &plan.shape {
        ValidationShapePlan::Record(fields) => {
            validate_object_entries_against_plan_fields(
                &document.fields,
                fields,
                contract,
                "",
                &mut diagnostics,
            );
        }
        ValidationShapePlan::TaggedUnion(variants) => {
            validate_tagged_union_document(document, variants, contract, &mut diagnostics);
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(ConfigValidationError {
            schema_name: schema_name.to_string(),
            diagnostics,
        })
    }
}

impl ConfigContract {
    fn find_schema_decl(&self, schema_name: &str) -> Option<(sm_front::SymbolId, &SchemaDecl)> {
        self.schema_table.iter().find_map(|(name, decl)| {
            let resolved = resolve_symbol_name(&self.program.arena, *name).ok()?;
            if resolved == schema_name {
                Some((*name, decl))
            } else {
                None
            }
        })
    }
}

fn config_contract_build_error(error: FrontendError) -> ConfigContractBuildError {
    ConfigContractBuildError {
        message: error.message,
    }
}

fn validate_object_entries_against_plan_fields(
    entries: &[ConfigEntry],
    fields: &[ValidationFieldPlan],
    contract: &ConfigContract,
    parent_path: &str,
    diagnostics: &mut Vec<ConfigValidationDiagnostic>,
) {
    let entry_map = entries
        .iter()
        .map(|entry| (entry.key.as_str(), &entry.value))
        .collect::<BTreeMap<_, _>>();
    let mut expected = BTreeSet::new();

    for field in fields {
        let field_name = contract.program.arena.symbol_name(field.name).to_string();
        let field_path = extend_config_path(parent_path, &field_name);
        expected.insert(field_name.clone());
        match entry_map.get(field_name.as_str()) {
            Some(value) => {
                validate_value_against_type(value, &field.ty, contract, &field_path, diagnostics);
            }
            None => diagnostics.push(ConfigValidationDiagnostic {
                path: field_path,
                message: "missing required field".to_string(),
            }),
        }
    }

    for entry in entries {
        if !expected.contains(entry.key.as_str()) {
            diagnostics.push(ConfigValidationDiagnostic {
                path: extend_config_path(parent_path, &entry.key),
                message: "unexpected config field".to_string(),
            });
        }
    }
}

fn validate_object_entries_against_record_decl(
    entries: &[ConfigEntry],
    record_decl: &RecordDecl,
    contract: &ConfigContract,
    parent_path: &str,
    diagnostics: &mut Vec<ConfigValidationDiagnostic>,
) {
    let entry_map = entries
        .iter()
        .map(|entry| (entry.key.as_str(), &entry.value))
        .collect::<BTreeMap<_, _>>();
    let mut expected = BTreeSet::new();

    for field in &record_decl.fields {
        let field_name = contract.program.arena.symbol_name(field.name).to_string();
        let field_path = extend_config_path(parent_path, &field_name);
        expected.insert(field_name.clone());
        match entry_map.get(field_name.as_str()) {
            Some(value) => {
                validate_value_against_type(value, &field.ty, contract, &field_path, diagnostics);
            }
            None => diagnostics.push(ConfigValidationDiagnostic {
                path: field_path,
                message: "missing required field".to_string(),
            }),
        }
    }

    for entry in entries {
        if !expected.contains(entry.key.as_str()) {
            diagnostics.push(ConfigValidationDiagnostic {
                path: extend_config_path(parent_path, &entry.key),
                message: "unexpected config field".to_string(),
            });
        }
    }
}

fn validate_tagged_union_document(
    document: &ConfigDocument,
    variants: &[ValidationVariantPlan],
    contract: &ConfigContract,
    diagnostics: &mut Vec<ConfigValidationDiagnostic>,
) {
    if document.fields.len() != 1 {
        diagnostics.push(ConfigValidationDiagnostic {
            path: "<root>".to_string(),
            message:
                "tagged-union config document must contain exactly one variant object field"
                    .to_string(),
        });
        return;
    }

    let selected = &document.fields[0];
    let Some(variant_plan) = variants.iter().find(|variant| {
        contract.program.arena.symbol_name(variant.name) == selected.key.as_str()
    }) else {
        diagnostics.push(ConfigValidationDiagnostic {
            path: selected.key.clone(),
            message: format!("unknown tagged-union variant '{}'", selected.key),
        });
        return;
    };

    let ConfigValue::Object(entries) = &selected.value else {
        diagnostics.push(type_mismatch(
            &selected.key,
            &format!(
                "expected object payload for variant '{}'",
                contract.program.arena.symbol_name(variant_plan.name)
            ),
        ));
        return;
    };

    validate_object_entries_against_plan_fields(
        entries,
        &variant_plan.fields,
        contract,
        &selected.key,
        diagnostics,
    );
}

fn validate_value_against_type(
    value: &ConfigValue,
    ty: &Type,
    contract: &ConfigContract,
    path: &str,
    diagnostics: &mut Vec<ConfigValidationDiagnostic>,
) {
    match ty {
        Type::Bool => {
            if !matches!(value, ConfigValue::Bool(_)) {
                diagnostics.push(type_mismatch(path, "expected bool value"));
            }
        }
        Type::Text => {
            if !matches!(value, ConfigValue::String(_)) {
                diagnostics.push(type_mismatch(path, "expected text value"));
            }
        }
        Type::Quad => {
            if !matches!(value, ConfigValue::Quad(_)) {
                diagnostics.push(type_mismatch(path, "expected quad value"));
            }
        }
        Type::I32 => validate_integer_number(value, path, diagnostics, "i32", |raw| {
            raw.parse::<i32>().is_ok()
        }),
        Type::U32 => validate_integer_number(value, path, diagnostics, "u32", |raw| {
            raw.parse::<u32>().is_ok()
        }),
        Type::F64 => validate_decimal_number(value, path, diagnostics, "f64"),
        Type::Fx => validate_decimal_number(value, path, diagnostics, "fx"),
        Type::Measured(base, unit) => {
            let unit_name = contract.program.arena.symbol_name(*unit);
            let label = format!("{}[{}]", display_config_type(base, contract), unit_name);
            validate_measured_number(value, path, diagnostics, &label, base.as_ref(), contract);
        }
        Type::Record(record_name) => {
            let Some(record_decl) = contract.record_table.get(record_name) else {
                diagnostics.push(type_mismatch(
                    path,
                    &format!(
                        "missing canonical record declaration '{}'",
                        contract.program.arena.symbol_name(*record_name)
                    ),
                ));
                return;
            };
            let ConfigValue::Object(entries) = value else {
                diagnostics.push(type_mismatch(
                    path,
                    &format!(
                        "expected object value for record '{}'",
                        contract.program.arena.symbol_name(*record_name)
                    ),
                ));
                return;
            };
            validate_object_entries_against_record_decl(entries, record_decl, contract, path, diagnostics);
        }
        Type::Option(_)
        | Type::Result(_, _)
        | Type::Sequence(_)
        | Type::Map(_)
        | Type::Closure(_)
        | Type::Tuple(_)
        | Type::Adt(_)
        | Type::RangeI32
        | Type::Unit
        | Type::QVec(_)
        | Type::TypeVar(_) => diagnostics.push(type_mismatch(
            path,
            &format!(
                "config validation does not yet support field type '{}'",
                display_config_type(ty, contract)
            ),
        )),
    }
}

fn validate_integer_number(
    value: &ConfigValue,
    path: &str,
    diagnostics: &mut Vec<ConfigValidationDiagnostic>,
    label: &str,
    fits: impl Fn(&str) -> bool,
) {
    match value {
        ConfigValue::Number(number) if number.kind == ConfigNumberKind::Integer && fits(&number.raw) => {}
        _ => diagnostics.push(type_mismatch(
            path,
            &format!("expected {} integer value", label),
        )),
    }
}

fn validate_decimal_number(
    value: &ConfigValue,
    path: &str,
    diagnostics: &mut Vec<ConfigValidationDiagnostic>,
    label: &str,
) {
    match value {
        ConfigValue::Number(number) if number.raw.parse::<f64>().is_ok() => {}
        _ => diagnostics.push(type_mismatch(
            path,
            &format!("expected {} numeric value", label),
        )),
    }
}

fn validate_measured_number(
    value: &ConfigValue,
    path: &str,
    diagnostics: &mut Vec<ConfigValidationDiagnostic>,
    label: &str,
    base: &Type,
    contract: &ConfigContract,
) {
    match base {
        Type::I32 => validate_integer_number(value, path, diagnostics, label, |raw| {
            raw.parse::<i32>().is_ok()
        }),
        Type::U32 => validate_integer_number(value, path, diagnostics, label, |raw| {
            raw.parse::<u32>().is_ok()
        }),
        Type::F64 | Type::Fx => validate_decimal_number(value, path, diagnostics, label),
        _ => diagnostics.push(type_mismatch(
            path,
            &format!(
                "unsupported measured base type '{}'",
                display_config_type(base, contract)
            ),
        )),
    }
}

fn type_mismatch(path: &str, message: &str) -> ConfigValidationDiagnostic {
    ConfigValidationDiagnostic {
        path: path.to_string(),
        message: message.to_string(),
    }
}

fn extend_config_path(parent: &str, field: &str) -> String {
    if parent.is_empty() {
        field.to_string()
    } else {
        format!("{}.{}", parent, field)
    }
}

fn display_config_type(ty: &Type, contract: &ConfigContract) -> String {
    match ty {
        Type::Quad => "quad".to_string(),
        Type::QVec(width) => format!("qvec({})", width),
        Type::Bool => "bool".to_string(),
        Type::Text => "text".to_string(),
        Type::I32 => "i32".to_string(),
        Type::U32 => "u32".to_string(),
        Type::Fx => "fx".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Measured(base, unit) => format!(
            "{}[{}]",
            display_config_type(base, contract),
            contract.program.arena.symbol_name(*unit)
        ),
        Type::RangeI32 => "range<i32>".to_string(),
        Type::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(|item| display_config_type(item, contract))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Sequence(sequence) => {
            format!("ordered-sequence<{}>", display_config_type(sequence.item.as_ref(), contract))
        }
        Type::Map(map) => format!(
            "Map({}, {})",
            display_config_type(map.key.as_ref(), contract),
            display_config_type(map.val.as_ref(), contract)
        ),
        Type::Closure(_) => "closure".to_string(),
        Type::Option(item) => format!("Option({})", display_config_type(item, contract)),
        Type::Result(ok_ty, err_ty) => format!(
            "Result({}, {})",
            display_config_type(ok_ty, contract),
            display_config_type(err_ty, contract)
        ),
        Type::Record(name) => contract.program.arena.symbol_name(*name).to_string(),
        Type::Adt(name) => contract.program.arena.symbol_name(*name).to_string(),
        Type::Unit => "()".to_string(),
        // TypeVar is an owner-layer marker; display as a type variable placeholder.
        Type::TypeVar(name) => format!("<TypeVar:{}>", contract.program.arena.symbol_name(*name)),
    }
}

struct ConfigParser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ConfigParser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn error(&self, message: impl Into<String>) -> ConfigParseError {
        ConfigParseError {
            pos: self.pos,
            message: message.into(),
        }
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek() {
            if matches!(ch, b' ' | b'\n' | b'\r' | b'\t') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect_byte(&mut self, expected: u8, label: &str) -> Result<(), ConfigParseError> {
        match self.bump() {
            Some(ch) if ch == expected => Ok(()),
            Some(_) => Err(self.error(format!("expected {}", label))),
            None => Err(self.error(format!("expected {}", label))),
        }
    }

    fn parse_object_entries(&mut self) -> Result<Vec<ConfigEntry>, ConfigParseError> {
        self.expect_byte(b'{', "'{'")?;
        self.skip_ws();
        let mut entries = Vec::new();
        let mut seen = BTreeSet::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(entries);
        }

        loop {
            self.skip_ws();
            let key = self.parse_identifier()?;
            if !seen.insert(key.clone()) {
                return Err(self.error(format!("duplicate config key '{}'", key)));
            }
            self.skip_ws();
            self.expect_byte(b':', "':'")?;
            self.skip_ws();
            let value = self.parse_value()?;
            entries.push(ConfigEntry { key, value });
            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.pos += 1;
                    self.skip_ws();
                    if self.peek() == Some(b'}') {
                        self.pos += 1;
                        break;
                    }
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(self.error("expected ',' or '}' after config field")),
            }
        }

        Ok(entries)
    }

    fn parse_value(&mut self) -> Result<ConfigValue, ConfigParseError> {
        match self.peek() {
            Some(b'{') => Ok(ConfigValue::Object(self.parse_object_entries()?)),
            Some(b'"') => Ok(ConfigValue::String(self.parse_string()?)),
            Some(b't') | Some(b'f') => Ok(ConfigValue::Bool(self.parse_bool()?)),
            Some(b'N' | b'F' | b'T' | b'S') => Ok(ConfigValue::Quad(self.parse_quad()?)),
            Some(b'-' | b'0'..=b'9') => Ok(ConfigValue::Number(self.parse_number()?)),
            Some(_) => Err(self.error("unsupported config value")),
            None => Err(self.error("expected config value")),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ConfigParseError> {
        let start = self.pos;
        match self.peek() {
            Some(ch) if is_ident_start(ch) => {
                self.pos += 1;
            }
            _ => return Err(self.error("expected identifier key")),
        }
        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(self.src[start..self.pos].to_string())
    }

    fn parse_string(&mut self) -> Result<String, ConfigParseError> {
        self.expect_byte(b'"', "'\"'")?;
        let mut out = String::new();
        loop {
            let ch = self.bump().ok_or_else(|| self.error("unterminated string"))?;
            match ch {
                b'"' => break,
                b'\\' => {
                    let escaped = self
                        .bump()
                        .ok_or_else(|| self.error("unterminated escape sequence"))?;
                    let mapped = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        _ => return Err(self.error("unsupported escape sequence")),
                    };
                    out.push(mapped);
                }
                _ => out.push(ch as char),
            }
        }
        Ok(out)
    }

    fn parse_bool(&mut self) -> Result<bool, ConfigParseError> {
        if self.try_keyword("true")? {
            return Ok(true);
        }
        if self.try_keyword("false")? {
            return Ok(false);
        }
        Err(self.error("expected 'true' or 'false'"))
    }

    fn parse_quad(&mut self) -> Result<QuadVal, ConfigParseError> {
        let ch = self.bump().ok_or_else(|| self.error("expected quad literal"))?;
        let quad = match ch {
            b'N' => QuadVal::N,
            b'F' => QuadVal::F,
            b'T' => QuadVal::T,
            b'S' => QuadVal::S,
            _ => return Err(self.error("expected quad literal")),
        };
        if matches!(self.peek(), Some(next) if is_ident_continue(next)) {
            return Err(self.error("quad literal must be delimited"));
        }
        Ok(quad)
    }

    fn parse_number(&mut self) -> Result<ConfigNumber, ConfigParseError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        let digits_start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.pos == digits_start {
            return Err(self.error("expected decimal digits"));
        }

        let kind = if self.peek() == Some(b'.') {
            self.pos += 1;
            let frac_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            if self.pos == frac_start {
                return Err(self.error("expected decimal digits after '.'"));
            }
            ConfigNumberKind::Decimal
        } else {
            ConfigNumberKind::Integer
        };

        if matches!(self.peek(), Some(next) if is_ident_continue(next)) {
            return Err(self.error("numeric literal must be delimited"));
        }

        Ok(ConfigNumber {
            raw: self.src[start..self.pos].to_string(),
            kind,
        })
    }

    fn try_keyword(&mut self, kw: &str) -> Result<bool, ConfigParseError> {
        let bytes = kw.as_bytes();
        if self.bytes.get(self.pos..self.pos + bytes.len()) != Some(bytes) {
            return Ok(false);
        }
        let end = self.pos + bytes.len();
        if matches!(self.bytes.get(end).copied(), Some(next) if is_ident_continue(next)) {
            return Err(self.error(format!("keyword '{}' must be delimited", kw)));
        }
        self.pos = end;
        Ok(true)
    }
}

fn is_ident_start(ch: u8) -> bool {
    matches!(ch, b'a'..=b'z' | b'A'..=b'Z' | b'_')
}

fn is_ident_continue(ch: u8) -> bool {
    is_ident_start(ch) || matches!(ch, b'0'..=b'9')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config_contract_source() -> &'static str {
        r#"
record Point {
    x: i32,
    y: i32,
}

config schema AppConfig {
    enabled: bool,
    mode: quad,
    point: Point,
    interval_ms: u32[ms],
    gain: f64,
}
"#
    }

    fn sample_tagged_union_config_contract_source() -> &'static str {
        r#"
record Point {
    x: i32,
    y: i32,
}

config schema InputEnvelope {
    Empty {},
    Data {
        point: Point,
        interval_ms: u32[ms],
    },
}
"#
    }

    #[test]
    fn parse_config_document_accepts_nested_object_surface() {
        let doc = parse_config_document(
            r#"{
                enabled: true,
                mode: T,
                retries: 3,
                threshold: 0.25,
                nested: {
                    label: "alpha",
                },
            }"#,
        )
        .expect("config document should parse");

        assert_eq!(doc.fields.len(), 5);
        assert_eq!(doc.fields[0].key, "enabled");
        assert_eq!(doc.fields[0].value, ConfigValue::Bool(true));
        assert_eq!(doc.fields[1].value, ConfigValue::Quad(QuadVal::T));
        assert_eq!(
            doc.fields[2].value,
            ConfigValue::Number(ConfigNumber {
                raw: "3".to_string(),
                kind: ConfigNumberKind::Integer,
            })
        );
        assert_eq!(
            doc.fields[3].value,
            ConfigValue::Number(ConfigNumber {
                raw: "0.25".to_string(),
                kind: ConfigNumberKind::Decimal,
            })
        );
        let ConfigValue::Object(nested) = &doc.fields[4].value else {
            panic!("expected nested object");
        };
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].key, "label");
        assert_eq!(nested[0].value, ConfigValue::String("alpha".to_string()));
    }

    #[test]
    fn parse_config_document_rejects_duplicate_key_in_same_object() {
        let err = parse_config_document(
            r#"{
                enabled: true,
                enabled: false,
            }"#,
        )
        .expect_err("duplicate key must reject");

        assert!(err.message.contains("duplicate config key 'enabled'"));
    }

    #[test]
    fn parse_config_document_rejects_non_object_root() {
        let err = parse_config_document("true").expect_err("root must be object");
        assert!(err.message.contains("config document must start with '{'"));
    }

    #[test]
    fn validate_config_document_accepts_record_shaped_config_schema() {
        let contract = build_config_contract(sample_config_contract_source())
            .expect("config contract should build");
        let doc = parse_config_document(
            r#"{
                enabled: true,
                mode: T,
                point: {
                    x: 10,
                    y: 20,
                },
                interval_ms: 250,
                gain: 0.5,
            }"#,
        )
        .expect("config document should parse");

        validate_config_document(&contract, "AppConfig", &doc)
            .expect("config document should validate");
    }

    #[test]
    fn validate_config_document_reports_missing_and_unexpected_fields() {
        let contract = build_config_contract(sample_config_contract_source())
            .expect("config contract should build");
        let doc = parse_config_document(
            r#"{
                mode: F,
                point: {
                    x: 10,
                    y: 20,
                    extra: 99,
                },
                interval_ms: 250,
                gain: 0.5,
                extra: true,
            }"#,
        )
        .expect("config document should parse");

        let err = validate_config_document(&contract, "AppConfig", &doc)
            .expect_err("validation should fail");

        assert!(err
            .diagnostics
            .iter()
            .any(|diag| diag.path == "enabled" && diag.message == "missing required field"));
        assert!(err
            .diagnostics
            .iter()
            .any(|diag| diag.path == "extra" && diag.message == "unexpected config field"));
        assert!(err
            .diagnostics
            .iter()
            .any(|diag| diag.path == "point.extra" && diag.message == "unexpected config field"));
    }

    #[test]
    fn validate_config_document_rejects_unsupported_option_field_family() {
        let contract = build_config_contract(
            r#"
config schema AppConfig {
    label: Option(quad),
}
"#,
        )
        .expect("config contract should build");
        let doc = parse_config_document(
            r#"{
                label: T,
            }"#,
        )
        .expect("config document should parse");

        let err = validate_config_document(&contract, "AppConfig", &doc)
            .expect_err("validation should fail");

        assert!(err.diagnostics.iter().any(|diag| {
            diag.path == "label"
                && diag
                    .message
                    .contains("config validation does not yet support field type 'Option(quad)'")
        }));
    }

    #[test]
    fn validate_config_document_rejects_non_config_schema_role() {
        let contract = build_config_contract(
            r#"
api schema ApiPayload {
    enabled: bool,
}
"#,
        )
        .expect("config contract should build");
        let doc = parse_config_document(
            r#"{
                enabled: true,
            }"#,
        )
        .expect("config document should parse");

        let err = validate_config_document(&contract, "ApiPayload", &doc)
            .expect_err("non-config schema role must reject");

        assert_eq!(err.diagnostics.len(), 1);
        assert_eq!(err.diagnostics[0].path, "<root>");
        assert!(err.diagnostics[0]
            .message
            .contains("schema 'ApiPayload' is not declared as config schema"));
    }

    #[test]
    fn validate_config_document_accepts_tagged_union_config_schema() {
        let contract = build_config_contract(sample_tagged_union_config_contract_source())
            .expect("config contract should build");
        let doc = parse_config_document(
            r#"{
                Data: {
                    point: {
                        x: 10,
                        y: 20,
                    },
                    interval_ms: 250,
                },
            }"#,
        )
        .expect("config document should parse");

        validate_config_document(&contract, "InputEnvelope", &doc)
            .expect("tagged-union config document should validate");
    }

    #[test]
    fn validate_config_document_rejects_tagged_union_root_with_multiple_variants() {
        let contract = build_config_contract(sample_tagged_union_config_contract_source())
            .expect("config contract should build");
        let doc = parse_config_document(
            r#"{
                Empty: {},
                Data: {
                    point: {
                        x: 1,
                        y: 2,
                    },
                    interval_ms: 10,
                },
            }"#,
        )
        .expect("config document should parse");

        let err = validate_config_document(&contract, "InputEnvelope", &doc)
            .expect_err("validation should reject multi-variant root");

        assert!(err.diagnostics.iter().any(|diag| {
            diag.path == "<root>"
                && diag.message
                    == "tagged-union config document must contain exactly one variant object field"
        }));
    }

    #[test]
    fn validate_config_document_rejects_unknown_tagged_union_variant() {
        let contract = build_config_contract(sample_tagged_union_config_contract_source())
            .expect("config contract should build");
        let doc = parse_config_document(
            r#"{
                Missing: {},
            }"#,
        )
        .expect("config document should parse");

        let err = validate_config_document(&contract, "InputEnvelope", &doc)
            .expect_err("validation should reject unknown variant");

        assert!(err.diagnostics.iter().any(|diag| {
            diag.path == "Missing"
                && diag.message == "unknown tagged-union variant 'Missing'"
        }));
    }

    #[test]
    fn validate_config_document_rejects_non_object_tagged_union_payload() {
        let contract = build_config_contract(sample_tagged_union_config_contract_source())
            .expect("config contract should build");
        let doc = parse_config_document(
            r#"{
                Data: true,
            }"#,
        )
        .expect("config document should parse");

        let err = validate_config_document(&contract, "InputEnvelope", &doc)
            .expect_err("validation should reject non-object payload");

        assert!(err.diagnostics.iter().any(|diag| {
            diag.path == "Data"
                && diag.message == "expected object payload for variant 'Data'"
        }));
    }
}
