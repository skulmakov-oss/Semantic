# LOGOS Grammar v0.1

## Indentation Rules

- Logical blocks are defined by indentation (`INDENT`/`DEDENT`).
- Empty lines and comment-only lines (`// ...` or `# ...`) do not affect indentation.
- `tab` in indentation is interpreted as 4 spaces (explicit v0.1 policy).
- Bad indentation levels produce `E0101`.

## Line Continuation

Indent checks are suspended while continuation is active:

- inside open parentheses `(` ... `)`;
- after trailing `->` until expression/effect continues.

This enables multi-line gate/effect calls.

## Tokens (Logos profile)

Keywords:
- `System`, `Entity`, `Law`, `When`, `Pulse`, `Profile`, `Import`

Structural:
- `Newline`, `INDENT`, `DEDENT`, `:`, `,`, `.`, `(`, `)`, `[`, `]`

Operators:
- `:=`, `=`, `->`, `==`, `!=`

Literals:
- numeric (`Int`/`Fx` lexical form), string (`"..."`), identifiers.

## Minimal Grammar

```ebnf
LogosProgram = { LogosDecl } ;
LogosDecl    = SystemDecl | EntityDecl | LawDecl | ImportDecl | PulseDecl | ProfileDecl ;

ImportDecl   = "Import" [ "pub" ] ImportSpec [ "as" Ident ] Newline? ;
ImportSpec   = String | Ident ;

PulseDecl    = "Pulse" OpaqueLine ;
ProfileDecl  = "Profile" OpaqueLine ;
OpaqueLine   = { token_except(Newline) } Newline? ;

SystemDecl   = "System" Ident [ "(" ParamList ")" ] [":" ] Newline? ;
ParamList    = Param { "," Param } ;
Param        = Ident ("=" | ":=") (Ident | Num | String) ;

EntityDecl   = "Entity" Ident ":" Newline INDENT { EntityField Newline? } DEDENT ;
EntityField  = ("state" | "prop") Ident ":" Type ;

LawDecl      = "Law" String [ "[" "priority" Num "]" ] ":"
               Newline INDENT { WhenClause Newline? } DEDENT ;
WhenClause   = "When" Condition "->" Effect ;
Condition    = Expr ;
Effect       = Expr | Newline ExprLine ;
ExprLine     = { token_except(Newline, DEDENT) } ;

Type         = "quad" | "bool" | "i32" | "u32" | "fx" | "f64" | "qvec" "[" Num "]" ;
```

Admission rules (PB-02):

- `qvec` requires an explicit dimension: `qvec[N]` with `N >= 1` that fits
  `usize`, closed by `]`. There is no implicit or default dimension (#1641,
  #1643); bare `qvec`, `qvec[]`, `qvec[0]`, overflowing or non-numeric
  dimensions, and `qvec(N)` are rejected.
- A program declares at most one `SystemDecl`; a second `System` is rejected
  with `E0201`, never a replacement of the first (#1644).
- Every accepted `ImportDecl` is preserved verbatim in
  `LogosProgram::imports` (directive text, byte span and source mark). The
  parser applies no further import grammar; `sm-sema` is the sole owner of
  import semantics and consumes exactly those preserved directives (#1645).
- `PulseDecl` and `ProfileDecl` are legacy-compatibility-gated,
  Logos-exclusive directives. Every accepted one is preserved verbatim, in
  source order, in `LogosProgram::legacy_directives` as an opaque
  `LogosLegacyDirective` (typed kind, exact text from the keyword to the end
  of the line excluding the line ending, byte span, source mark). Their
  payload has no current semantic interpretation, and they take no part in
  SemCode generation or VM execution (#1987). With legacy compatibility
  disabled they remain a policy violation.

## Import v0.1 Policy

- Path resolve: relative to current module directory; if extension is missing, `.sm` is appended.
- Cycle detection: recursive import loops are rejected with `E0238`.
- Namespace isolation: each module is analyzed in its own scope; same `Entity`/`Law` names in different modules are allowed.
- Alias collisions: duplicate import aliases inside one module are rejected with `E0241`.
  - Default alias is derived from import file stem.
- Re-export: `Import pub ...` is parsed but rejected in v0.1 with `E0240`.

### Import Examples

```exo
Import "core/math.sm"
Import "drivers/sensor.sm" as SensorDrv
```

```exo
# E0241 (alias collision):
Import "a.sm" as Core
Import "b.sm" as Core
```

```exo
# E0240 (re-export disabled in v0.1):
Import pub "shared.sm"
```

## Example

```exo
Entity Sensor:
    state val: quad
    prop active: bool

Law "CheckSignal" [priority 10]:
    When Sensor.val == T ->
        Log.emit("Signal OK")
    When Sensor.val == S ->
        System.recovery()
```

## Recovery Anchors

On parse errors the Logos parser synchronizes on anchors:
- `Newline`
- `DEDENT`
- `System`
- `Entity`
- `Law`

This allows collecting multiple diagnostics in one pass.
