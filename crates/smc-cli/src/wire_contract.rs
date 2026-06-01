use sm_front::{
    build_adt_table, build_record_table, canonicalize_declared_type, parse_program,
    resolve_symbol_name, AstArena, FrontendError, SchemaRole, SchemaShape, Type,
};
use std::error::Error;
use std::fmt;
use std::fmt::Write;

pub const GENERATED_WIRE_CONTRACT_FORMAT_VERSION: u32 = 1;
pub const GENERATED_WIRE_CONTRACT_GENERATOR: &str = env!("CARGO_PKG_NAME");
pub const GENERATED_WIRE_CONTRACT_GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedWireUnionField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedWireUnionVariant {
    pub name: String,
    pub fields: Vec<TaggedWireUnionField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedWireUnionContract {
    pub schema_name: String,
    pub variants: Vec<TaggedWireUnionVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WirePatchField {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WirePatchTypeContract {
    pub schema_name: String,
    pub fields: Vec<WirePatchField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedWireContractArtifact {
    pub format_version: u32,
    pub generator_name: String,
    pub generator_version: String,
    pub tagged_unions: Vec<TaggedWireUnionContract>,
    pub patch_types: Vec<WirePatchTypeContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedWireContractBuildError {
    pub message: String,
}

impl fmt::Display for GeneratedWireContractBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "generated wire contract build error: {}", self.message)
    }
}

impl Error for GeneratedWireContractBuildError {}

impl GeneratedWireContractArtifact {
    pub fn new(
        tagged_unions: Vec<TaggedWireUnionContract>,
        patch_types: Vec<WirePatchTypeContract>,
    ) -> Self {
        Self {
            format_version: GENERATED_WIRE_CONTRACT_FORMAT_VERSION,
            generator_name: GENERATED_WIRE_CONTRACT_GENERATOR.to_string(),
            generator_version: GENERATED_WIRE_CONTRACT_GENERATOR_VERSION.to_string(),
            tagged_unions,
            patch_types,
        }
    }
}

pub fn build_generated_wire_contract(
    src: &str,
) -> Result<GeneratedWireContractArtifact, GeneratedWireContractBuildError> {
    let program = parse_program(src).map_err(generated_wire_contract_build_error)?;
    let record_table = build_record_table(&program).map_err(generated_wire_contract_build_error)?;
    let adt_table = build_adt_table(&program).map_err(generated_wire_contract_build_error)?;
    let mut tagged_unions = Vec::new();
    let mut patch_types = Vec::new();

    for schema in &program.schemas {
        if schema.role != Some(SchemaRole::Wire) {
            continue;
        }
        let schema_name = resolve_symbol_name(&program.arena, schema.name)
            .map_err(generated_wire_contract_build_error)?
            .to_string();
        match &schema.shape {
            SchemaShape::TaggedUnion(variants) => {
                let variants = variants
                    .iter()
                    .map(|variant| {
                        Ok(TaggedWireUnionVariant {
                            name: resolve_symbol_name(&program.arena, variant.name)
                                .map_err(generated_wire_contract_build_error)?
                                .to_string(),
                            fields: variant
                                .fields
                                .iter()
                                .map(|field| {
                                    Ok(TaggedWireUnionField {
                                        name: resolve_symbol_name(&program.arena, field.name)
                                            .map_err(generated_wire_contract_build_error)?
                                            .to_string(),
                                        ty: display_generated_wire_type(
                                            &canonicalize_declared_type(
                                                &field.ty,
                                                &record_table,
                                                &adt_table,
                                                &program.arena,
                                            )
                                            .map_err(generated_wire_contract_build_error)?,
                                            &program.arena,
                                        )
                                        .map_err(generated_wire_contract_build_error)?,
                                    })
                                })
                                .collect::<Result<Vec<_>, GeneratedWireContractBuildError>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, GeneratedWireContractBuildError>>()?;
                tagged_unions.push(TaggedWireUnionContract {
                    schema_name,
                    variants,
                });
            }
            SchemaShape::Record(fields) => {
                let fields = fields
                    .iter()
                    .map(|field| {
                        Ok(WirePatchField {
                            name: resolve_symbol_name(&program.arena, field.name)
                                .map_err(generated_wire_contract_build_error)?
                                .to_string(),
                            ty: display_generated_wire_type(
                                &canonicalize_declared_type(
                                    &field.ty,
                                    &record_table,
                                    &adt_table,
                                    &program.arena,
                                )
                                .map_err(generated_wire_contract_build_error)?,
                                &program.arena,
                            )
                            .map_err(generated_wire_contract_build_error)?,
                        })
                    })
                    .collect::<Result<Vec<_>, GeneratedWireContractBuildError>>()?;
                patch_types.push(WirePatchTypeContract {
                    schema_name,
                    fields,
                });
            }
        }
    }

    Ok(GeneratedWireContractArtifact::new(
        tagged_unions,
        patch_types,
    ))
}

pub fn format_generated_wire_contract(artifact: &GeneratedWireContractArtifact) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "semantic_wire_contract v{}", artifact.format_version);
    let _ = writeln!(
        out,
        "generator {} {}",
        artifact.generator_name, artifact.generator_version
    );

    for tagged_union in &artifact.tagged_unions {
        out.push('\n');
        let _ = writeln!(out, "wire union {} {{", tagged_union.schema_name);
        for variant in &tagged_union.variants {
            let _ = writeln!(out, "    {} {{", variant.name);
            for field in &variant.fields {
                let _ = writeln!(out, "        {}: {}", field.name, field.ty);
            }
            out.push_str("    }\n");
        }
        out.push_str("}\n");
    }

    for patch_type in &artifact.patch_types {
        out.push('\n');
        let _ = writeln!(out, "wire patch {} {{", patch_type.schema_name);
        for field in &patch_type.fields {
            let _ = writeln!(out, "    {}?: {}", field.name, field.ty);
        }
        out.push_str("}\n");
    }

    out
}

fn generated_wire_contract_build_error(error: FrontendError) -> GeneratedWireContractBuildError {
    GeneratedWireContractBuildError {
        message: error.message,
    }
}

fn display_generated_wire_type(ty: &Type, arena: &AstArena) -> Result<String, FrontendError> {
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
            display_generated_wire_type(base, arena)?,
            resolve_symbol_name(arena, *unit)?
        ),
        Type::RangeI32 => "range<i32>".to_string(),
        Type::Tuple(items) => format!(
            "({})",
            items
                .iter()
                .map(|item| display_generated_wire_type(item, arena))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        ),
        Type::Sequence(_) => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "ordered sequence types are not part of the current M8.3 Wave 1 generated wire-contract surface"
                        .to_string(),
            })
        }
        Type::Closure(_) => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "first-class closure types are not part of the current M8.4 Wave 1 generated wire-contract surface"
                        .to_string(),
            })
        }
        Type::Map(_) => {
            return Err(FrontendError {
                pos: 0,
                message:
                    "Map types are not part of the current Wave 1 generated wire-contract surface"
                        .to_string(),
            })
        }
        Type::Option(item) => format!("Option({})", display_generated_wire_type(item, arena)?),
        Type::Result(ok_ty, err_ty) => format!(
            "Result({}, {})",
            display_generated_wire_type(ok_ty, arena)?,
            display_generated_wire_type(err_ty, arena)?
        ),
        Type::Record(name) | Type::Adt(name) => resolve_symbol_name(arena, *name)?.to_string(),
        Type::Unit => "()".to_string(),
        Type::TypeVar(_) => {
            return Err(FrontendError::policy_violation(
                0,
                "type variable is not part of the current generated wire contract surface; \
                 generic instantiation is deferred to M9.1 Wave 2",
            ))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_wire_contract_artifact_uses_canonical_metadata() {
        let artifact = GeneratedWireContractArtifact::new(Vec::new(), Vec::new());

        assert_eq!(
            artifact.format_version,
            GENERATED_WIRE_CONTRACT_FORMAT_VERSION
        );
        assert_eq!(artifact.generator_name, GENERATED_WIRE_CONTRACT_GENERATOR);
        assert_eq!(
            artifact.generator_version,
            GENERATED_WIRE_CONTRACT_GENERATOR_VERSION
        );
    }

    #[test]
    fn format_generated_wire_contract_preserves_contract_order() {
        let artifact = GeneratedWireContractArtifact::new(
            vec![TaggedWireUnionContract {
                schema_name: "Envelope".to_string(),
                variants: vec![
                    TaggedWireUnionVariant {
                        name: "Empty".to_string(),
                        fields: Vec::new(),
                    },
                    TaggedWireUnionVariant {
                        name: "Data".to_string(),
                        fields: vec![TaggedWireUnionField {
                            name: "count".to_string(),
                            ty: "i32".to_string(),
                        }],
                    },
                ],
            }],
            vec![WirePatchTypeContract {
                schema_name: "Telemetry".to_string(),
                fields: vec![
                    WirePatchField {
                        name: "enabled".to_string(),
                        ty: "bool".to_string(),
                    },
                    WirePatchField {
                        name: "interval_ms".to_string(),
                        ty: "u32[ms]".to_string(),
                    },
                ],
            }],
        );

        let formatted = format_generated_wire_contract(&artifact);
        let expected = "\
semantic_wire_contract v1
generator smc-cli 0.1.0

wire union Envelope {
    Empty {
    }
    Data {
        count: i32
    }
}

wire patch Telemetry {
    enabled?: bool
    interval_ms?: u32[ms]
}
";

        assert_eq!(formatted, expected);
    }

    #[test]
    fn build_generated_wire_contract_derives_only_tagged_wire_unions() {
        let artifact = build_generated_wire_contract(
            r#"
api schema ApiEnvelope {
    Empty {},
}

wire schema Envelope {
    Empty {},
    Data {
        count: i32,
        interval_ms: u32[ms],
    },
}

wire schema Telemetry {
    enabled: bool,
}
"#,
        )
        .expect("tagged wire-union derivation should build");

        assert_eq!(artifact.tagged_unions.len(), 1);
        assert_eq!(artifact.patch_types.len(), 1);
        assert_eq!(artifact.tagged_unions[0].schema_name, "Envelope");
        assert_eq!(artifact.tagged_unions[0].variants.len(), 2);
        assert_eq!(artifact.tagged_unions[0].variants[0].name, "Empty");
        assert!(artifact.tagged_unions[0].variants[0].fields.is_empty());
        assert_eq!(artifact.tagged_unions[0].variants[1].name, "Data");
        assert_eq!(artifact.tagged_unions[0].variants[1].fields.len(), 2);
        assert_eq!(
            artifact.tagged_unions[0].variants[1].fields[0].name,
            "count"
        );
        assert_eq!(artifact.tagged_unions[0].variants[1].fields[0].ty, "i32");
        assert_eq!(
            artifact.tagged_unions[0].variants[1].fields[1].name,
            "interval_ms"
        );
        assert_eq!(
            artifact.tagged_unions[0].variants[1].fields[1].ty,
            "u32[ms]"
        );
        assert_eq!(artifact.patch_types[0].schema_name, "Telemetry");
        assert_eq!(artifact.patch_types[0].fields.len(), 1);
        assert_eq!(artifact.patch_types[0].fields[0].name, "enabled");
        assert_eq!(artifact.patch_types[0].fields[0].ty, "bool");
    }

    #[test]
    fn build_generated_wire_contract_preserves_declaration_order_for_variants_and_fields() {
        let artifact = build_generated_wire_contract(
            r#"
wire schema Envelope {
    Ping {
        seq: u32,
        sent_at: u32[ms],
    },
    Pong {
        ack: u32,
    },
}
"#,
        )
        .expect("wire union derivation should preserve declaration order");

        let formatted = format_generated_wire_contract(&artifact);
        let expected = "\
semantic_wire_contract v1
generator smc-cli 0.1.0

wire union Envelope {
    Ping {
        seq: u32
        sent_at: u32[ms]
    }
    Pong {
        ack: u32
    }
}
";

        assert_eq!(formatted, expected);
    }

    #[test]
    fn build_generated_wire_contract_preserves_declaration_order_for_patch_fields() {
        let artifact = build_generated_wire_contract(
            r#"
wire schema TelemetryPatch {
    enabled: bool,
    interval_ms: u32[ms],
    retries: i32,
}
"#,
        )
        .expect("record-shaped wire schema should derive patch type");

        assert_eq!(artifact.tagged_unions.len(), 0);
        assert_eq!(artifact.patch_types.len(), 1);
        assert_eq!(artifact.patch_types[0].schema_name, "TelemetryPatch");
        assert_eq!(artifact.patch_types[0].fields.len(), 3);
        assert_eq!(artifact.patch_types[0].fields[0].name, "enabled");
        assert_eq!(artifact.patch_types[0].fields[0].ty, "bool");
        assert_eq!(artifact.patch_types[0].fields[1].name, "interval_ms");
        assert_eq!(artifact.patch_types[0].fields[1].ty, "u32[ms]");
        assert_eq!(artifact.patch_types[0].fields[2].name, "retries");
        assert_eq!(artifact.patch_types[0].fields[2].ty, "i32");
    }
}
