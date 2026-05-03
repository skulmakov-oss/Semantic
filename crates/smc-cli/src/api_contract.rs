use sm_front::{
    build_adt_table, build_record_table, canonicalize_declared_type, parse_program,
    resolve_symbol_name, AstArena, FrontendError, SchemaRole, SchemaShape, Type,
};
use std::error::Error;
use std::fmt;
use std::fmt::Write;

pub const GENERATED_API_CONTRACT_FORMAT_VERSION: u32 = 1;
pub const GENERATED_API_CONTRACT_GENERATOR: &str = env!("CARGO_PKG_NAME");
pub const GENERATED_API_CONTRACT_GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratedApiSchemaRole {
    Api,
    Wire,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiVariant {
    pub name: String,
    pub fields: Vec<GeneratedApiField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratedApiSchemaShape {
    Record(Vec<GeneratedApiField>),
    TaggedUnion(Vec<GeneratedApiVariant>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiSchema {
    pub name: String,
    pub role: GeneratedApiSchemaRole,
    pub shape: GeneratedApiSchemaShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiContractArtifact {
    pub format_version: u32,
    pub generator_name: String,
    pub generator_version: String,
    pub schemas: Vec<GeneratedApiSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedApiContractBuildError {
    pub message: String,
}

impl fmt::Display for GeneratedApiContractBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "generated API contract build error: {}", self.message)
    }
}

impl Error for GeneratedApiContractBuildError {}

impl GeneratedApiContractArtifact {
    pub fn new(schemas: Vec<GeneratedApiSchema>) -> Self {
        Self {
            format_version: GENERATED_API_CONTRACT_FORMAT_VERSION,
            generator_name: GENERATED_API_CONTRACT_GENERATOR.to_string(),
            generator_version: GENERATED_API_CONTRACT_GENERATOR_VERSION.to_string(),
            schemas,
        }
    }
}

pub fn build_generated_api_contract(
    src: &str,
) -> Result<GeneratedApiContractArtifact, GeneratedApiContractBuildError> {
    let program = parse_program(src).map_err(generated_api_contract_build_error)?;
    let record_table = build_record_table(&program).map_err(generated_api_contract_build_error)?;
    let adt_table = build_adt_table(&program).map_err(generated_api_contract_build_error)?;
    let mut schemas = Vec::new();

    for schema in &program.schemas {
        let Some(role) = schema.role else {
            continue;
        };
        let Some(generated_role) = generated_role(role) else {
            continue;
        };
        let schema_name = resolve_symbol_name(&program.arena, schema.name)
            .map_err(generated_api_contract_build_error)?
            .to_string();
        let shape = match &schema.shape {
            SchemaShape::Record(fields) => GeneratedApiSchemaShape::Record(
                fields
                    .iter()
                    .map(|field| {
                        Ok(GeneratedApiField {
                            name: resolve_symbol_name(&program.arena, field.name)
                                .map_err(generated_api_contract_build_error)?
                                .to_string(),
                            ty: display_generated_api_type(
                                &canonicalize_declared_type(
                                    &field.ty,
                                    &record_table,
                                    &adt_table,
                                    &program.arena,
                                )
                                .map_err(generated_api_contract_build_error)?,
                                &program.arena,
                            )
                            .map_err(generated_api_contract_build_error)?,
                        })
                    })
                    .collect::<Result<Vec<_>, GeneratedApiContractBuildError>>()?,
            ),
            SchemaShape::TaggedUnion(variants) => GeneratedApiSchemaShape::TaggedUnion(
                variants
                    .iter()
                    .map(|variant| {
                        Ok(GeneratedApiVariant {
                            name: resolve_symbol_name(&program.arena, variant.name)
                                .map_err(generated_api_contract_build_error)?
                                .to_string(),
                            fields: variant
                                .fields
                                .iter()
                                .map(|field| {
                                    Ok(GeneratedApiField {
                                        name: resolve_symbol_name(&program.arena, field.name)
                                            .map_err(generated_api_contract_build_error)?
                                            .to_string(),
                                        ty: display_generated_api_type(
                                            &canonicalize_declared_type(
                                                &field.ty,
                                                &record_table,
                                                &adt_table,
                                                &program.arena,
                                            )
                                            .map_err(generated_api_contract_build_error)?,
                                            &program.arena,
                                        )
                                        .map_err(generated_api_contract_build_error)?,
                                    })
                                })
                                .collect::<Result<Vec<_>, GeneratedApiContractBuildError>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, GeneratedApiContractBuildError>>()?,
            ),
        };
        schemas.push(GeneratedApiSchema {
            name: schema_name,
            role: generated_role,
            shape,
        });
    }

    Ok(GeneratedApiContractArtifact::new(schemas))
}

pub fn format_generated_api_contract(artifact: &GeneratedApiContractArtifact) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "semantic_api_contract v{}",
        artifact.format_version
    );
    let _ = writeln!(
        out,
        "generator {} {}",
        artifact.generator_name, artifact.generator_version
    );

    for schema in &artifact.schemas {
        out.push('\n');
        let _ = write!(
            out,
            "{} schema {} ",
            display_generated_api_role(schema.role),
            schema.name
        );
        match &schema.shape {
            GeneratedApiSchemaShape::Record(fields) => {
                out.push_str("{\n");
                for field in fields {
                    let _ = writeln!(out, "    {}: {}", field.name, field.ty);
                }
                out.push_str("}\n");
            }
            GeneratedApiSchemaShape::TaggedUnion(variants) => {
                out.push_str("{\n");
                for variant in variants {
                    let _ = writeln!(out, "    {} {{", variant.name);
                    for field in &variant.fields {
                        let _ = writeln!(out, "        {}: {}", field.name, field.ty);
                    }
                    out.push_str("    }\n");
                }
                out.push_str("}\n");
            }
        }
    }

    out
}

fn generated_api_contract_build_error(error: FrontendError) -> GeneratedApiContractBuildError {
    GeneratedApiContractBuildError {
        message: error.message,
    }
}

fn generated_role(role: SchemaRole) -> Option<GeneratedApiSchemaRole> {
    match role {
        SchemaRole::Api => Some(GeneratedApiSchemaRole::Api),
        SchemaRole::Wire => Some(GeneratedApiSchemaRole::Wire),
        SchemaRole::Config => None,
    }
}

fn display_generated_api_role(role: GeneratedApiSchemaRole) -> &'static str {
    match role {
        GeneratedApiSchemaRole::Api => "api",
        GeneratedApiSchemaRole::Wire => "wire",
    }
}

fn display_generated_api_type(
    ty: &Type,
    arena: &AstArena,
) -> Result<String, FrontendError> {
    Ok(match ty {
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
            display_generated_api_type(base, arena)?,
            resolve_symbol_name(arena, *unit)?
        ),
        Type::RangeI32 => "range<i32>".to_string(),
        Type::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(|item| display_generated_api_type(item, arena))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        ),
        Type::Sequence(_) => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "ordered sequence types are not part of the current M8.3 Wave 1 generated API contract surface"
                        .to_string(),
            })
        }
        Type::Closure(_) => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "first-class closure types are not part of the current M8.4 Wave 1 generated API contract surface"
                        .to_string(),
            })
        }
        Type::Map(_) => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "Map types are not part of the current Wave 1 generated API contract surface"
                        .to_string(),
            })
        }
        Type::Option(item) => format!("Option({})", display_generated_api_type(item, arena)?),
        Type::Result(ok_ty, err_ty) => format!(
            "Result({}, {})",
            display_generated_api_type(ok_ty, arena)?,
            display_generated_api_type(err_ty, arena)?
        ),
        Type::Record(name) | Type::Adt(name) => resolve_symbol_name(arena, *name)?.to_string(),
        Type::Unit => "()".to_string(),
        Type::TypeVar(_) => {
            return Err(FrontendError::policy_violation(
                0,
                "type variable is not part of the current generated API contract surface; \
                 generic instantiation is deferred to M9.1 Wave 2",
            ))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_api_contract_artifact_uses_canonical_metadata() {
        let artifact = GeneratedApiContractArtifact::new(Vec::new());

        assert_eq!(artifact.format_version, GENERATED_API_CONTRACT_FORMAT_VERSION);
        assert_eq!(artifact.generator_name, GENERATED_API_CONTRACT_GENERATOR);
        assert_eq!(
            artifact.generator_version,
            GENERATED_API_CONTRACT_GENERATOR_VERSION
        );
    }

    #[test]
    fn format_generated_api_contract_preserves_schema_and_field_order() {
        let artifact = GeneratedApiContractArtifact::new(vec![
            GeneratedApiSchema {
                name: "Telemetry".to_string(),
                role: GeneratedApiSchemaRole::Api,
                shape: GeneratedApiSchemaShape::Record(vec![
                    GeneratedApiField {
                        name: "enabled".to_string(),
                        ty: "bool".to_string(),
                    },
                    GeneratedApiField {
                        name: "interval_ms".to_string(),
                        ty: "u32[ms]".to_string(),
                    },
                ]),
            },
            GeneratedApiSchema {
                name: "Envelope".to_string(),
                role: GeneratedApiSchemaRole::Wire,
                shape: GeneratedApiSchemaShape::TaggedUnion(vec![
                    GeneratedApiVariant {
                        name: "Empty".to_string(),
                        fields: Vec::new(),
                    },
                    GeneratedApiVariant {
                        name: "Data".to_string(),
                        fields: vec![GeneratedApiField {
                            name: "sample_count".to_string(),
                            ty: "i32".to_string(),
                        }],
                    },
                ]),
            },
        ]);

        let formatted = format_generated_api_contract(&artifact);
        let expected = "\
semantic_api_contract v1
generator smc-cli 0.1.0

api schema Telemetry {
    enabled: bool
    interval_ms: u32[ms]
}

wire schema Envelope {
    Empty {
    }
    Data {
        sample_count: i32
    }
}
";
        assert_eq!(formatted, expected);
    }

    #[test]
    fn build_generated_api_contract_derives_record_shaped_api_and_wire_schemas() {
        let artifact = build_generated_api_contract(
            r#"
record Point {
    x: i32,
    y: i32,
}

config schema AppConfig {
    enabled: bool,
}

api schema Telemetry {
    enabled: bool,
    point: Point,
    interval_ms: u32[ms],
}

wire schema Envelope {
    sample_count: Result(i32, quad),
}
"#,
        )
        .expect("API contract artifact should build");

        assert_eq!(artifact.schemas.len(), 2);
        assert_eq!(artifact.schemas[0].name, "Telemetry");
        assert_eq!(artifact.schemas[0].role, GeneratedApiSchemaRole::Api);
        let GeneratedApiSchemaShape::Record(fields) = &artifact.schemas[0].shape else {
            panic!("expected record-shaped API schema");
        };
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].name, "enabled");
        assert_eq!(fields[0].ty, "bool");
        assert_eq!(fields[1].name, "point");
        assert_eq!(fields[1].ty, "Point");
        assert_eq!(fields[2].name, "interval_ms");
        assert_eq!(fields[2].ty, "u32[ms]");

        assert_eq!(artifact.schemas[1].name, "Envelope");
        assert_eq!(artifact.schemas[1].role, GeneratedApiSchemaRole::Wire);
        let GeneratedApiSchemaShape::Record(fields) = &artifact.schemas[1].shape else {
            panic!("expected record-shaped wire schema");
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "sample_count");
        assert_eq!(fields[0].ty, "Result(i32, quad)");
    }

    #[test]
    fn build_generated_api_contract_derives_tagged_union_api_schema_variants() {
        let artifact = build_generated_api_contract(
            r#"
wire schema Envelope {
    Empty {},
    Data {
        sample_count: i32,
        status: quad,
    },
}
"#,
        )
        .expect("tagged-union generation should now be available");

        assert_eq!(artifact.schemas.len(), 1);
        assert_eq!(artifact.schemas[0].name, "Envelope");
        assert_eq!(artifact.schemas[0].role, GeneratedApiSchemaRole::Wire);
        let GeneratedApiSchemaShape::TaggedUnion(variants) = &artifact.schemas[0].shape else {
            panic!("expected tagged-union generated schema");
        };
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].name, "Empty");
        assert!(variants[0].fields.is_empty());
        assert_eq!(variants[1].name, "Data");
        assert_eq!(variants[1].fields.len(), 2);
        assert_eq!(variants[1].fields[0].name, "sample_count");
        assert_eq!(variants[1].fields[0].ty, "i32");
        assert_eq!(variants[1].fields[1].name, "status");
        assert_eq!(variants[1].fields[1].ty, "quad");
    }
}
