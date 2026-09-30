//! SSF-09 D2: the canonical ADT descriptor table of a compiled program.

use crate::error::{CompilePipelineError, IrError};
use crate::frontend::{build_adt_table, resolve_symbol_name, Program};
use sm_format::semcode_format::{AdtDescriptor, AdtDescriptorTable, AdtVariantDescriptor};

/// Builds the canonical ADT descriptor table of `program`: one descriptor
/// per declared enum, taken from the frontend's canonical enum declaration
/// authority (`build_adt_table`), plus the built-in `Option` and `Result`
/// descriptors. Descriptors are never inferred from lowered instructions.
///
/// A rejected enum declaration (for example a duplicate or reserved name) is
/// a `Frontend` error; a declaration exceeding the `ADT0` u16 limits is an
/// `InternalIr` error.
pub fn adt_descriptor_table(program: &Program) -> Result<AdtDescriptorTable, CompilePipelineError> {
    let internal = |message: String| CompilePipelineError::from(IrError { message });
    let adts = build_adt_table(program)?;
    let mut user = Vec::with_capacity(adts.len());
    for adt in adts.values() {
        let adt_name = resolve_symbol_name(&program.arena, adt.name)?;
        let mut variants = Vec::with_capacity(adt.variants.len());
        for variant in &adt.variants {
            let name = resolve_symbol_name(&program.arena, variant.name)?;
            let arity = u16::try_from(variant.payload.len()).map_err(|_| {
                internal(format!(
                    "enum variant '{adt_name}::{name}' payload arity exceeds the u16 limit"
                ))
            })?;
            variants
                .push(AdtVariantDescriptor::new(name, arity).map_err(|e| internal(e.to_string()))?);
        }
        user.push(AdtDescriptor::new(adt_name, variants).map_err(|e| internal(e.to_string()))?);
    }
    AdtDescriptorTable::with_builtins(user).map_err(|e| internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semcode_format::{header_spec_from_magic, HEADER_V22, MAGIC22};
    use crate::{compile_program_to_ir, compile_program_to_semcode, IrInstr};
    use sm_front::{parse_program, FrontendError};

    fn table(src: &str) -> AdtDescriptorTable {
        adt_descriptor_table(&parse_program(src).expect("parse")).expect("descriptor table")
    }

    fn names(table: &AdtDescriptorTable) -> Vec<&str> {
        table.descriptors().iter().map(|d| d.name()).collect()
    }

    fn shape<'a>(table: &'a AdtDescriptorTable, adt: &str) -> Vec<(&'a str, u16)> {
        table
            .descriptors()
            .iter()
            .find(|d| d.name() == adt)
            .expect("descriptor present")
            .variants()
            .iter()
            .map(|v| (v.name(), v.payload_arity()))
            .collect()
    }

    const MAIN: &str = "fn main() {\n    return;\n}\n";

    #[test]
    fn user_enum_descriptor_comes_from_its_declaration() {
        let t = table(&format!(
            "enum E {{\n    A,\n    B(i32),\n    C(i32, bool),\n}}\n{MAIN}"
        ));
        assert_eq!(shape(&t, "E"), [("A", 0), ("B", 1), ("C", 2)]);
    }

    #[test]
    fn program_without_enums_still_gets_option_and_result() {
        let t = table(MAIN);
        assert_eq!(names(&t), ["Option", "Result"]);
        assert_eq!(shape(&t, "Option"), [("None", 0), ("Some", 1)]);
        assert_eq!(shape(&t, "Result"), [("Ok", 1), ("Err", 1)]);
    }

    #[test]
    fn source_declaration_order_does_not_affect_descriptor_order() {
        let z = "enum Z {\n    Z0,\n}\n";
        let a = "enum A {\n    A0(i32),\n}\n";
        let za = table(&format!("{z}{a}{MAIN}"));
        let az = table(&format!("{a}{z}{MAIN}"));
        assert_eq!(za, az);
        assert_eq!(names(&za), ["A", "Option", "Result", "Z"]);
    }

    #[test]
    fn variant_order_is_declaration_order() {
        let t = table(&format!("enum E {{\n    Zebra,\n    Alpha,\n}}\n{MAIN}"));
        assert_eq!(shape(&t, "E"), [("Zebra", 0), ("Alpha", 0)]);
    }

    #[test]
    fn reserved_builtin_adt_names_are_frontend_errors() {
        for name in ["Option", "Result"] {
            let program =
                parse_program(&format!("enum {name} {{\n    A,\n}}\n{MAIN}")).expect("parse");
            assert_eq!(
                adt_descriptor_table(&program),
                Err(CompilePipelineError::Frontend(FrontendError {
                    detail: None,
                    pos: 0,
                    message: format!("enum name '{name}' is reserved for the built-in ADT"),
                }))
            );
        }
    }

    #[test]
    fn lowered_constructors_agree_with_the_descriptor_table() {
        // The tags lowering already emits must select the descriptor
        // variant of the same name and payload arity.
        let src = "enum E {\n    A,\n    B(i32),\n}\n\
             fn main() {\n    let a: E = E::A;\n    let b: E = E::B(1);\n    \
             let o: Option(i32) = Option::Some(1);\n    let n: Option(i32) = Option::None;\n    \
             let r: Result(i32, i32) = Result::Ok(1);\n    let e: Result(i32, i32) = Result::Err(2);\n    \
             return;\n}\n";
        let t = table(src);
        let mut seen = 0;
        for f in compile_program_to_ir(src).expect("lower") {
            for instr in &f.instrs {
                if let IrInstr::MakeAdt {
                    adt_name,
                    variant_name,
                    tag,
                    items,
                    ..
                } = instr
                {
                    let variant = &t
                        .descriptors()
                        .iter()
                        .find(|d| d.name() == adt_name)
                        .expect("descriptor for constructed ADT")
                        .variants()[usize::from(*tag)];
                    assert_eq!(variant.name(), variant_name);
                    assert_eq!(usize::from(variant.payload_arity()), items.len());
                    seen += 1;
                }
            }
        }
        assert_eq!(seen, 6);
    }

    #[test]
    fn compiled_artifacts_use_the_d2_header_and_the_declared_adt0_table() {
        // SSF-09 D2-2 (P10): SEMCOD22, then exactly the ADT0 section of the
        // table built from the program's declarations - with or without
        // user enums, and whether or not any ADT is ever constructed.
        for src in [
            format!("enum E {{\n    A,\n    B(i32),\n}}\n{MAIN}"),
            MAIN.to_string(),
        ] {
            let bytes = compile_program_to_semcode(&src).expect("compile");
            assert_eq!(bytes[..8], MAGIC22);
            assert_eq!(
                header_spec_from_magic(&bytes[..8].try_into().expect("magic")),
                Some(HEADER_V22)
            );
            let section = table(&src).encode_section();
            assert_eq!(bytes[8..8 + section.len()], section[..]);
        }
    }

    #[test]
    fn declaration_order_does_not_change_the_emitted_artifact() {
        let z = "enum Z {\n    Z0,\n}\n";
        let a = "enum A {\n    A0(i32),\n}\n";
        let za = compile_program_to_semcode(&format!("{z}{a}{MAIN}")).expect("compile");
        let az = compile_program_to_semcode(&format!("{a}{z}{MAIN}")).expect("compile");
        assert_eq!(za, az);
        // Repeated compilation is byte-identical.
        assert_eq!(
            za,
            compile_program_to_semcode(&format!("{z}{a}{MAIN}")).expect("compile")
        );
    }
}
