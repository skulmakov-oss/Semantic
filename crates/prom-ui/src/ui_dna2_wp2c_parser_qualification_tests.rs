use crate::contract_primitives::SourceId;
use crate::projection_source_parser::{
    parse_projection_source, preflight_source_len, ProjectionSourceInputError,
    ProjectionSourceParseError, ProjectionSourceParserDiagnostic,
    ProjectionSourceParserDiagnosticKind as Kind, MAX_PROJECTION_SOURCE_BYTES,
};

const SOURCE_RAW: u64 = 41;

fn source_id() -> SourceId {
    SourceId::new(SOURCE_RAW).expect("non-zero test source id")
}

fn diagnostic(source: &str) -> ProjectionSourceParserDiagnostic {
    match parse_projection_source(source_id(), source) {
        Err(ProjectionSourceParseError::Diagnostic(diagnostic)) => diagnostic,
        other => panic!("expected parser diagnostic, got {other:?}"),
    }
}

fn assert_diagnostic(source: &str, kind: Kind, start: usize, end: usize) {
    let diagnostic = diagnostic(source);
    assert_eq!(diagnostic.kind(), kind);
    assert_eq!(diagnostic.code(), kind.code());
    assert_eq!(diagnostic.source_id(), source_id());
    assert_eq!(
        diagnostic.span().start(),
        u32::try_from(start).expect("test span start is representable")
    );
    assert_eq!(
        diagnostic.span().end(),
        u32::try_from(end).expect("test span end is representable")
    );
    let coordinate = diagnostic.coordinate();
    assert_eq!(coordinate.code(), kind.code());
    assert_eq!(
        coordinate.source().expect("source coordinate").source(),
        source_id()
    );
    assert_eq!(
        coordinate.domain(),
        u64::try_from(start).expect("test coordinate start is representable")
    );
    assert_eq!(
        coordinate.secondary(),
        u64::try_from(end).expect("test coordinate end is representable")
    );
}

fn assert_at(source: &str, kind: Kind, needle: &str) {
    let start = source.find(needle).expect("diagnostic needle");
    assert_diagnostic(source, kind, start, checked_test_add(start, needle.len()));
}

fn assert_zero_width_at(source: &str, needle: &str) {
    let start = source.find(needle).expect("diagnostic needle");
    assert_diagnostic(source, Kind::MissingSemicolon, start, start);
}

fn checked_test_add(position: usize, width: usize) -> usize {
    position.checked_add(width).expect("test offset arithmetic")
}

#[test]
fn ui_dna2_wp2c_parser_accepts_complete_grammar_and_preserves_order() {
    let source = concat!(
        "projection v0 {\n",
        "revision 7; epoch 9;\n",
        "surface 2 root 20 key 200;\n",
        "node 20 role surface key 201 { child 22 order 4; child 21 order 1; }\n",
        "surface 1 root 21 key 100;\n",
        "node 21 role button key 101 {}\n",
        "}"
    );
    let document = parse_projection_source(source_id(), source).expect("valid source");

    assert_eq!(document.source_id(), source_id());
    assert_eq!(document.revision().raw(), 7);
    assert_eq!(document.epoch().raw(), 9);
    assert_eq!(document.surfaces().len(), 2);
    assert_eq!(document.surfaces()[0].id().raw(), 2);
    assert_eq!(document.surfaces()[1].id().raw(), 1);
    assert_eq!(document.nodes().len(), 2);
    assert_eq!(document.nodes()[0].role().as_str(), "surface");
    assert_eq!(document.nodes()[1].role().as_str(), "button");
    assert_eq!(document.nodes()[0].children()[0].child().raw(), 22);
    assert_eq!(document.nodes()[0].children()[0].order().raw(), 4);
    assert_eq!(document.nodes()[0].children()[1].child().raw(), 21);
    assert_eq!(document.nodes()[0].children()[1].order().raw(), 1);
    assert_eq!(parse_projection_source(source_id(), source), Ok(document));
}

#[test]
fn ui_dna2_wp2c_parser_accepts_trivia_comments_and_unknown_roles() {
    let cases = [
        "projection v0 { revision 0; epoch 0; }",
        "projection v0 { // LF comment\n revision 0; epoch 0; }",
        "projection v0 { // CRLF comment\r\n revision 0; epoch 0; }",
        "projection v0 { // non-ASCII: Привет 根\n revision 0; epoch 0; }",
        "projection v0 { revision 0; epoch 0; node 1 role node key 1 {} }",
        "projection v0 { revision 0; epoch 0; node 1 role button key 1 {} }",
        "projection v0 { revision 0; epoch 0; node 1 role root// role comment\n key 1 {} }",
    ];
    for source in cases {
        parse_projection_source(source_id(), source).expect(source);
    }

    let unknown = cases[5];
    let document = parse_projection_source(source_id(), unknown).expect("unknown role parses");
    assert_eq!(document.nodes()[0].role().as_str(), "button");
}

#[test]
fn ui_dna2_wp2c_parser_constructs_exact_declaration_spans() {
    let source = concat!(
        "projection v0 { revision 0; epoch 0;\n",
        "  surface 1 root 10 key 11;\n",
        "  node 10 role root key 12 { child 11 order 0; }\n",
        "}"
    );
    let document = parse_projection_source(source_id(), source).expect("valid source");
    let surface_text = "surface 1 root 10 key 11;";
    let surface_start = source.find(surface_text).unwrap();
    let surface_span = document.surfaces()[0].source().span();
    assert_eq!(
        surface_span.start(),
        u32::try_from(surface_start).expect("surface start is representable")
    );
    let surface_end = surface_start
        .checked_add(surface_text.len())
        .expect("surface end arithmetic");
    assert_eq!(
        surface_span.end(),
        u32::try_from(surface_end).expect("surface end is representable")
    );

    let node_text = "node 10 role root key 12 { child 11 order 0; }";
    let node_start = source.find(node_text).unwrap();
    let node_span = document.nodes()[0].source().span();
    assert_eq!(
        node_span.start(),
        u32::try_from(node_start).expect("node start is representable")
    );
    let node_end = node_start
        .checked_add(node_text.len())
        .expect("node end arithmetic");
    assert_eq!(
        node_span.end(),
        u32::try_from(node_end).expect("node end is representable")
    );
}

#[test]
fn ui_dna2_wp2c_parser_covers_every_diagnostic_kind() {
    let cases = [
        ("\u{feff}projection v0 {}", Kind::UnexpectedChar, "\u{feff}"),
        ("Projection v0 {}", Kind::UnexpectedToken, "Projection"),
        (
            "projection v0 { revision 0; epoch 0;",
            Kind::UnexpectedEof,
            "",
        ),
        (
            "projection v0 { revision 0 epoch 0; }",
            Kind::MissingSemicolon,
            "epoch",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role Root key 1 {} }",
            Kind::InvalidIdentifier,
            "Root",
        ),
        (
            "projection v0 { revision +1; epoch 0; }",
            Kind::InvalidNumber,
            "+1",
        ),
        (
            "projection v0 { revision 18446744073709551616; epoch 0; }",
            Kind::NumberOverflow,
            "18446744073709551616",
        ),
        (
            "projection v0 { revision 0; epoch 0; surface 0 root 1 key 1; }",
            Kind::ZeroId,
            "0 root",
        ),
        (
            "projection v0 { revision 0; revision 1; epoch 0; }",
            Kind::DuplicateRevision,
            "revision 1",
        ),
        (
            "projection v0 { revision 0; epoch 0; epoch 1; }",
            Kind::DuplicateEpoch,
            "epoch 1",
        ),
        (
            "projection v1 { revision 0; epoch 0; }",
            Kind::UnsupportedVersion,
            "v1",
        ),
        (
            "projection v0 { revision 0; epoch 0; background blue; }",
            Kind::UnknownClause,
            "background",
        ),
    ];

    for (source, kind, needle) in cases {
        if kind == Kind::UnexpectedEof {
            assert_diagnostic(source, kind, source.len(), source.len());
        } else if kind == Kind::MissingSemicolon {
            assert_zero_width_at(source, needle);
        } else if kind == Kind::ZeroId {
            let start = source.find(needle).unwrap();
            assert_diagnostic(source, kind, start, checked_test_add(start, 1));
        } else if matches!(kind, Kind::DuplicateRevision | Kind::DuplicateEpoch) {
            let start = source.find(needle).unwrap();
            let token_len = needle.split_once(' ').unwrap().0.len();
            assert_diagnostic(source, kind, start, checked_test_add(start, token_len));
        } else {
            assert_at(source, kind, needle);
        }
    }
}

#[test]
fn ui_dna2_wp2c_parser_enforces_character_and_identifier_boundaries() {
    let cases = [
        (
            "projection v0 {\r revision 0; epoch 0; }",
            Kind::UnexpectedChar,
            "\r",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role røot key 1 {} }",
            Kind::UnexpectedChar,
            "ø",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root/name key 1 {} }",
            Kind::UnexpectedChar,
            "/",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root/*x*/ key 1 {} }",
            Kind::UnexpectedChar,
            "/",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role -root key 1 {} }",
            Kind::UnexpectedChar,
            "-",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role _root key 1 {} }",
            Kind::InvalidIdentifier,
            "_root",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role 9root key 1 {} }",
            Kind::InvalidIdentifier,
            "9root",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root-name key 1 {} }",
            Kind::InvalidIdentifier,
            "root-name",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root.name key 1 {} }",
            Kind::InvalidIdentifier,
            "root.name",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root::name key 1 {} }",
            Kind::InvalidIdentifier,
            "root::name",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role \"root\" key 1 {} }",
            Kind::InvalidIdentifier,
            "\"root\"",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role \"\" key 1 {} }",
            Kind::InvalidIdentifier,
            "\"\"",
        ),
    ];
    for (source, kind, needle) in cases {
        assert_at(source, kind, needle);
    }

    let unterminated = "projection v0 { revision 0; epoch 0; node 1 role \"root\nkey 1 {} }";
    let start = unterminated.find('"').unwrap();
    let end = unterminated.find('\n').unwrap();
    assert_diagnostic(unterminated, Kind::InvalidIdentifier, start, end);
}

#[test]
fn ui_dna2_wp2c_parser_distinguishes_p2_clause_contexts() {
    let cases = [
        ("projection v0 { revision 0; epoch 0; child 2 order 0; }", Kind::UnexpectedToken, "child"),
        ("projection v0 { revision 0; epoch 0; node 1 role root key 1 { surface 1 root 1 key 1; } }", Kind::UnexpectedToken, "surface"),
        ("projection v0 { revision 0; epoch 0; background blue; }", Kind::UnknownClause, "background"),
        ("projection v0 { revision 0; epoch 0; node 1 role root key 1 { width 100; } }", Kind::UnknownClause, "width"),
        ("projection v0 { revision 0; epoch 0; Background blue; }", Kind::InvalidIdentifier, "Background"),
        ("projection v0 { revision 0; epoch 0; background-color blue; }", Kind::InvalidIdentifier, "background-color"),
    ];
    for (source, kind, needle) in cases {
        assert_at(source, kind, needle);
    }
}

#[test]
fn ui_dna2_wp2c_parser_enforces_numeric_diagnostics() {
    let cases = [
        ("projection v0 { revision 01; epoch 0; }", Kind::InvalidNumber, "01"),
        ("projection v0 { revision 1_000; epoch 0; }", Kind::InvalidNumber, "1_000"),
        ("projection v0 { revision -0; epoch 0; }", Kind::InvalidNumber, "-0"),
        ("projection v0 { revision +18446744073709551616; epoch 0; }", Kind::InvalidNumber, "+18446744073709551616"),
        ("projection v0 { revision root; epoch 0; }", Kind::UnexpectedToken, "root"),
        ("projection v0 { revision 0; epoch 0; node 1 role root key 1 { child 2 order 4294967296; } }", Kind::NumberOverflow, "4294967296"),
    ];
    for (source, kind, needle) in cases {
        assert_at(source, kind, needle);
    }
}

#[test]
fn ui_dna2_wp2c_parser_enforces_compound_sign_boundaries() {
    let cases = [
        ("++1", Kind::UnexpectedToken, "+"),
        ("+-1", Kind::UnexpectedToken, "+"),
        ("-+1", Kind::UnexpectedChar, "-"),
        ("--1", Kind::UnexpectedChar, "-"),
    ];
    for (value, kind, needle) in cases {
        let source = format!("projection v0 {{ revision {value}; epoch 0; }}");
        assert_at(&source, kind, needle);
    }
}

#[test]
fn ui_dna2_wp2c_parser_preserves_terminal_semicolon_precedence() {
    let cases = [
        ("projection v0 { revision 1+1; epoch 0; }", "+"),
        ("projection v0 { revision 1-1; epoch 0; }", "-"),
        ("projection v0 { revision 0; epoch 1+1; }", "+"),
        ("projection v0 { revision 0; epoch 1-1; }", "-"),
        (
            "projection v0 { revision 0; epoch 0; surface 1 root 10 key 1+1; }",
            "+",
        ),
        (
            "projection v0 { revision 0; epoch 0; surface 1 root 10 key 1-1; }",
            "-",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root key 1 { child 2 order 1+1; } }",
            "+",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root key 1 { child 2 order 1-1; } }",
            "-",
        ),
    ];
    for (source, needle) in cases {
        assert_zero_width_at(source, needle);
    }

    let separated = "projection v0 { revision 0; epoch 0; surface 1 root 10 key 1 1+1; }";
    let first = checked_test_add(separated.find("1 1+1").unwrap(), 2);
    assert_diagnostic(separated, Kind::MissingSemicolon, first, first);
}

#[test]
fn ui_dna2_wp2c_parser_preserves_nonterminal_plus_minus_boundaries() {
    let cases = [
        (
            "projection v0 { revision 0; epoch 0; surface 1+1 root 10 key 1; }",
            Kind::UnexpectedToken,
            "+",
        ),
        (
            "projection v0 { revision 0; epoch 0; surface 1-1 root 10 key 1; }",
            Kind::UnexpectedChar,
            "-",
        ),
        (
            "projection v0 { revision 0; epoch 0; surface 1 root 10+1 key 1; }",
            Kind::UnexpectedToken,
            "+",
        ),
        (
            "projection v0 { revision 0; epoch 0; surface 1 root 10-1 key 1; }",
            Kind::UnexpectedChar,
            "-",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1+1 role root key 1 {} }",
            Kind::UnexpectedToken,
            "+",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1-1 role root key 1 {} }",
            Kind::UnexpectedChar,
            "-",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root key 1+1 {} }",
            Kind::UnexpectedToken,
            "+",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root key 1-1 {} }",
            Kind::UnexpectedChar,
            "-",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root key 1 { child 2+1 order 0; } }",
            Kind::UnexpectedToken,
            "+",
        ),
        (
            "projection v0 { revision 0; epoch 0; node 1 role root key 1 { child 2-1 order 0; } }",
            Kind::UnexpectedChar,
            "-",
        ),
    ];
    for (source, kind, needle) in cases {
        assert_at(source, kind, needle);
    }
}

#[test]
fn ui_dna2_wp2c_parser_handles_post_document_and_fail_fast_boundaries() {
    let complete = "projection v0 { revision 0; epoch 0; } ";
    for (tail, kind, width) in [
        ("Root", Kind::UnexpectedToken, 4),
        ("root-name", Kind::UnexpectedToken, 9),
        ("9root", Kind::UnexpectedToken, 5),
        ("+1", Kind::UnexpectedToken, 1),
        ("-1", Kind::UnexpectedChar, 1),
    ] {
        let source = format!("{complete}{tail}");
        assert_diagnostic(
            &source,
            kind,
            complete.len(),
            checked_test_add(complete.len(), width),
        );
    }
}

#[test]
fn ui_dna2_wp2c_parser_checks_cursor_advancement_boundaries() {
    let ascii = "projection v0 { revision 0; epoch 0; }+";
    assert_at(ascii, Kind::UnexpectedToken, "+");

    let crlf = "projection v0 {\r\nrevision 0; epoch 0; background blue; }";
    assert_at(crlf, Kind::UnknownClause, "background");

    let multibyte_comment =
        "projection v0 { // Привет 根\r\nrevision 0; epoch 0; background blue; }";
    assert_at(multibyte_comment, Kind::UnknownClause, "background");

    let token_at_eof = "Projection";
    assert_diagnostic(token_at_eof, Kind::UnexpectedToken, 0, token_at_eof.len());

    let eof = "projection v0 { revision 0; epoch 0;";
    assert_diagnostic(eof, Kind::UnexpectedEof, eof.len(), eof.len());
}

#[test]
fn ui_dna2_wp2c_parser_preflights_source_size_without_allocating_input() {
    assert_eq!(
        preflight_source_len(
            source_id(),
            usize::try_from(MAX_PROJECTION_SOURCE_BYTES).expect("u32 fits supported usize"),
        ),
        Ok(())
    );
    if usize::BITS > 32 {
        let actual_len = usize::try_from(MAX_PROJECTION_SOURCE_BYTES)
            .expect("u32 fits supported usize")
            .checked_add(1)
            .expect("64-bit usize represents u32 max plus one");
        assert_eq!(
            preflight_source_len(source_id(), actual_len),
            Err(ProjectionSourceInputError::SourceTooLarge {
                source_id: source_id(),
                actual_len,
                max_len: MAX_PROJECTION_SOURCE_BYTES,
            })
        );
    }
}
