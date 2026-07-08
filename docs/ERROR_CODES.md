# Semantic Error Codes

Semantic diagnostic codes reference.
CLI source: `smc explain <code>` and `smc explain --list`.

## How to use

- View a specific code:
  - `smc explain E0201`
- List all codes:
  - `smc explain --list`

## Catalog

- `E0000`: Generic frontend parse/type error. See caret span for exact location.
- `E0001`: Unexpected character in source input.
- `E0002`: Expected logical operator `&&`.
- `E0003`: Expected logical operator `||`.
- `E0004`: Unterminated string literal.
- `E0101`: Bad indentation level (INDENT/DEDENT mismatch).
- `E0200`: Expected Logos declaration (System/Entity/Law).
- `E0201`: Type mismatch. Example: expected QVec/Bool, found other type.
- `E0210`: Malformed Entity declaration header.
- `E0211`: Expected `:` after Entity name.
- `E0212`: Expected newline after Entity header.
- `E0213`: Expected INDENT for Entity body.
- `E0214`: Expected Entity field declaration.
- `E0215`: Entity field must start with `state` or `prop`.
- `E0216`: Expected `:` in Entity field declaration.
- `E0220`: Duplicate Entity declaration.
- `E0221`: Duplicate Law inside the same Entity scope.
- `E0222`: Law body is empty.
- `E0223`: Shadowing is forbidden inside a Law scope.
- `E0224`: Empty When condition.
- `E0225`: Empty When body/effect.
- `E0230`: Expected `When` clause in Law body.
- `E0234`: Expected type annotation.
- `E0238`: Cyclic import detected.
- `E0239`: Import resolution/read/parse failure.
- `E0240`: Import re-export is not supported in v0.1.
- `E0241`: Duplicate import alias within one module.
- `W0240`: Dead law branch detected: When condition is always false.
- `W0241`: Constant folding candidate detected for `fx.*` call with literal args.
- `W0250`: Law name style warning (expected `UpperCamelCase`).
- `W0251`: Large Law block warning (too many `When` clauses).
- `W0252`: Unused Entity field warning (`state/prop` not referenced).
- `W0253`: Magic number warning (consider named constant).

## Maintenance

When adding new codes:

1. Update the catalog in `src/bin/smc.rs` (`diagnostic_catalog`).
2. Update this document.
