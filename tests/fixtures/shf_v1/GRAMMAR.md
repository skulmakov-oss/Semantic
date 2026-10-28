# MiniLang frontend v1

This fixture language is distinct from Semantic. Input is UTF-8 text; token and diagnostic spans are zero-based half-open byte offsets. ASCII spaces, tabs, CR, LF and `//` through LF or EOF are trivia. Identifiers are `[A-Za-z_][A-Za-z_0-9]*`; integers are nonempty decimal digit sequences (stored as source spans, with no numeric conversion). Any other byte is invalid. `let` is reserved in declaration-name position.

```text
program  = { "let" identifier "=" expression ";" } EOF ;
expression = sum ;
sum      = product { ("+" | "-") product } ;
product  = primary { ("*" | "/") primary } ;
primary  = integer | identifier | "(" expression ")" ;
```

Operators associate left to right. A name must be declared by a preceding completed statement; duplicates, self-reference and forward references are rejected. There are no assignments, scopes, unary operators or evaluation. AST nodes are append-only and use zero-based indices, with `-1` for no child: identifiers 1, integers 2, operators 6вЂ“9, declaration 12. Declaration left child is the expression root. Parentheses do not add nodes. Token kinds 0 EOF, 1 identifier, 2 integer, 3 `=`, 4 `;`, 5 invalid, 6вЂ“11 `+ - * / ( )`.

Before scanning or parsing, input exceeding 4096 bytes is rejected with diagnostic 6 and an empty span at the input byte length, even if an invalid byte or other error appears earlier. For inputs within that length, parsing stops at its first encountered error: 1 invalid byte, 2 syntax, 3 duplicate/reserved declaration, 4 unknown name, 6 limit. The diagnostic range is the offending token (EOF has an empty span). The parser limits input to 4096 bytes, 511 non-EOF tokens and 64 recursive expression levels. The implementation contains a guard after 511 non-EOF tokens. Under the tested default 100000-step VM budget, the token-boundary and 508-node workloads exhaust Steps before the guard or 508-node result can be observed; those boundaries are not qualified. These are structural bounds, not a promise that the default VM quota admits every bounded input: long scans and persistent collection updates require a larger explicit step budget. Errors preserve any earlier arena nodes; callers must check the error before using them. Comment contents are skipped as UTF-8 bytes.
