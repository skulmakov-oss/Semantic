Title: bug(front/sema): local ADT binding is not recognized as enum match scrutinee

## Description

A local variable explicitly typed as an ADT is currently rejected by the typechecker when used as a `match` scrutinee.

## Minimal Reproducible Example

```rust
enum Maybe {
    None,
    Some(f64),
}

fn get_e() -> Maybe = Maybe::Some(42.0);

fn main() {
    let e: Maybe = get_e();
    let total: f64 = match e {
        Maybe::Some(ref v) => { 1.0 }
        _ => { 0.0 }
    };
    return;
}
```

## Expected Behavior
The `match e` expression should typecheck as an enum scrutinee since `e` is typed as `Maybe`.

## Actual Behavior
The compiler throws a typecheck error:
`match expression scrutinee must be quad, enum, Option(T), or Result(T, E)`

## Notes
- This issue was encountered during the PCC-ADT work (Wave 3).
- To bypass this issue in lowering tests, we currently pass `e: Maybe` as a function parameter instead of using a local `let` binding.
