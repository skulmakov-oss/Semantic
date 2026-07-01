use sm_front::lexer::lex_tokens;
use sm_front::types::TokenKind;

// Helper to get non-layout tokens for easy matching
fn lex_kinds(input: &str) -> Vec<TokenKind> {
    let tokens = lex_tokens(input).unwrap();
    tokens
        .into_iter()
        .map(|t| t.kind)
        .filter(|k| {
            !matches!(
                k,
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
            )
        })
        .collect()
}

#[test]
fn test_operator_tokenization() {
    let input = "{ } ( ) [ ] ; , : :: := . .. ..= ! != = == => < <= > >= && &&= || ||= | |> + += - -= -> * *= / /= % %=";
    let expected = vec![
        TokenKind::LBrace,
        TokenKind::RBrace,
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBracket,
        TokenKind::RBracket,
        TokenKind::Semi,
        TokenKind::Comma,
        TokenKind::Colon,
        TokenKind::PathSep,
        TokenKind::Assign,
        TokenKind::Dot,
        TokenKind::DotDot,
        TokenKind::DotDotEq,
        TokenKind::Bang,
        TokenKind::Ne,
        TokenKind::Assign,
        TokenKind::EqEq,
        TokenKind::FatArrow,
        TokenKind::LAngle,
        TokenKind::Le,
        TokenKind::RAngle,
        TokenKind::Ge,
        TokenKind::AndAnd,
        TokenKind::AndAndAssign,
        TokenKind::OrOr,
        TokenKind::OrOrAssign,
        TokenKind::Pipe,
        TokenKind::PipeForward,
        TokenKind::Plus,
        TokenKind::PlusAssign,
        TokenKind::Minus,
        TokenKind::MinusAssign,
        TokenKind::Implies,
        TokenKind::Star,
        TokenKind::StarAssign,
        TokenKind::Slash,
        TokenKind::SlashAssign,
        TokenKind::Percent,
        TokenKind::PercentAssign,
    ];
    assert_eq!(lex_kinds(input), expected);
}

#[test]
fn test_keyword_tokenization() {
    let input = "fn requires ensures invariant record schema enum const trait impl let for in guard if else is when while loop break continue where with return match true false System Entity Law When Pulse Profile Import ref quad bool i32 u32 fx f64 N F T S";
    let expected = vec![
        TokenKind::KwFn,
        TokenKind::KwRequires,
        TokenKind::KwEnsures,
        TokenKind::KwInvariant,
        TokenKind::KwRecord,
        TokenKind::KwSchema,
        TokenKind::KwEnum,
        TokenKind::KwConst,
        TokenKind::KwTrait,
        TokenKind::KwImpl,
        TokenKind::KwLet,
        TokenKind::KwFor,
        TokenKind::KwIn,
        TokenKind::KwGuard,
        TokenKind::KwIf,
        TokenKind::KwElse,
        TokenKind::KwIs,
        TokenKind::KwWhen,
        TokenKind::KwWhile,
        TokenKind::KwLoop,
        TokenKind::KwBreak,
        TokenKind::KwContinue,
        TokenKind::KwWhere,
        TokenKind::KwWith,
        TokenKind::KwReturn,
        TokenKind::KwMatch,
        TokenKind::KwTrue,
        TokenKind::KwFalse,
        TokenKind::KwSystem,
        TokenKind::KwEntity,
        TokenKind::KwLaw,
        TokenKind::KwWhen,
        TokenKind::KwPulse,
        TokenKind::KwProfile,
        TokenKind::KwImport,
        TokenKind::KwRef,
        TokenKind::TyQuad,
        TokenKind::TyBool,
        TokenKind::TyI32,
        TokenKind::TyU32,
        TokenKind::TyFx,
        TokenKind::TyF64,
        TokenKind::QuadN,
        TokenKind::QuadF,
        TokenKind::QuadT,
        TokenKind::QuadS,
    ];
    assert_eq!(lex_kinds(input), expected);
}

#[test]
fn test_comments() {
    let input = "
# comment ignored
// comment ignored
fn // comment
let # comment
";
    assert_eq!(lex_kinds(input), vec![TokenKind::KwFn, TokenKind::KwLet]);

    // Blank/comment-only lines do not produce unexpected tokens except layout/newline
    let tokens = lex_tokens("   // blank").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Newline);
    assert_eq!(tokens.len(), 1);
}

#[test]
fn test_numeric_literals() {
    let input = "123 1_2_3 0x1A 0x1_A 1i32 1u32 1f64 1fx 1.23 1.2_3";
    let kinds = lex_kinds(input);
    for k in kinds {
        assert_eq!(k, TokenKind::Num);
    }

    let tokens = lex_tokens(input).unwrap();
    let texts: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Num))
        .map(|t| t.text.as_str())
        .collect();
    assert_eq!(
        texts,
        vec!["123", "1_2_3", "0x1A", "0x1_A", "1i32", "1u32", "1f64", "1fx", "1.23", "1.2_3"]
    );
}

#[test]
fn test_string_literals() {
    let input = "\"hello\" \"world\"";
    let tokens = lex_tokens(input).unwrap();
    let texts: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::String))
        .map(|t| t.text.as_str())
        .collect();
    assert_eq!(texts, vec!["\"hello\"", "\"world\""]);
}

#[test]
fn test_negative_diagnostics() {
    // unexpected character returns E0001
    let err = lex_tokens("fn main() { ^ }").unwrap_err();
    assert!(
        err.message
            .contains("error[E0001]: unexpected character '^'"),
        "got {}",
        err.message
    );

    // single & returns E0002
    let err = lex_tokens("let x = a & b").unwrap_err();
    assert!(
        err.message.contains("error[E0002]: expected '&&'"),
        "got {}",
        err.message
    );

    // unterminated string returns E0004
    let err = lex_tokens("let x = \"unterminated").unwrap_err();
    assert!(
        err.message
            .contains("error[E0004]: unterminated string literal"),
        "got {}",
        err.message
    );

    // bad indentation returns E0101
    let input = "fn main()\n  x\n x";
    let err = lex_tokens(input).unwrap_err();
    assert!(
        err.message.contains("error[E0101]: Bad Indent"),
        "got {}",
        err.message
    );
}

#[test]
fn test_source_marks() {
    let input = "fn\n  let\n\n  # \n    foo";
    let tokens = lex_tokens(input).unwrap();

    // fn is first token on first line
    assert_eq!(tokens[0].kind, TokenKind::KwFn);
    assert_eq!(tokens[0].mark.line, 1);
    assert_eq!(tokens[0].mark.col, 1);

    // let is token after indentation (line 2, col 3)
    let let_tok = tokens.iter().find(|t| t.kind == TokenKind::KwLet).unwrap();
    assert_eq!(let_tok.mark.line, 2);
    assert_eq!(let_tok.mark.col, 3);

    // foo is line 5, col 5
    let foo_tok = tokens.iter().find(|t| t.kind == TokenKind::Ident).unwrap();
    assert_eq!(foo_tok.mark.line, 5);
    assert_eq!(foo_tok.mark.col, 5);

    // error caret location for one negative case
    let err = lex_tokens("  &").unwrap_err();
    assert!(
        err.message.contains(" --> <input>:1:3"),
        "got {}",
        err.message
    );
    assert!(err.message.contains("  |   ^"), "got {}", err.message);
}

#[test]
fn test_layout() {
    let input = "
a
  b
    c
  d
e
";
    let tokens = lex_tokens(input).unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    let expected = vec![
        TokenKind::Newline,
        TokenKind::Ident,
        TokenKind::Newline, // a
        TokenKind::Indent,
        TokenKind::Ident,
        TokenKind::Newline, // b
        TokenKind::Indent,
        TokenKind::Ident,
        TokenKind::Newline, // c
        TokenKind::Dedent,
        TokenKind::Ident,
        TokenKind::Newline, // d
        TokenKind::Dedent,
        TokenKind::Ident,
        TokenKind::Newline, // e
    ];
    assert_eq!(kinds, expected);
}
