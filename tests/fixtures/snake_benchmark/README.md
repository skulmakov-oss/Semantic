Snake benchmark gap matrix fixtures for the application-completeness program.

Purpose:

- freeze the current pass baseline for already-landed benchmark-critical source
  surfaces
- freeze the current fail baseline for still-missing snake blockers that
  already have a meaningful current source spelling

Current landed positive baseline includes:

- same-family text equality
- enum/control-flow basics
- same-family plain `i32` relational operators
- same-family plain `i32` unary `-` and binary `+`, `-`, `*`, `/`, `%`
- `let mut`, plain reassignment, and compound assignment over mutable locals
- `while condition { ... }` statement loops with `bool` conditions
- statement `loop`, bare `break;`, and `continue;` for admitted control-flow
- ordered `Sequence(T)` indexing and iteration
- `len(sequence) -> i32`
- `is_empty(sequence) -> bool`
- `contains(sequence, value) -> bool` for admitted comparable scalar element types
- persistent `push(sequence, value) -> Sequence(T)`
- persistent `prepend(sequence, value) -> Sequence(T)`
- persistent `pop(sequence) -> Sequence(T)`
- first-class closure capture
- persistent `Map(K, V)` lookup tables with scalar key families
- deterministic seeded PRNG through `random_seed` and `random_next_i32`

The sequence update helpers are functional/persistent. They do not mutate a
sequence in place; benchmark code should assign the returned sequence when
evolving state.

The map update helper is also functional/persistent. `map_set` returns a new
`Map(K, V)` value; benchmark code should assign the returned map when evolving
Q-tables or visit counters.

Current runtime-negative baseline includes:

- invalid PRNG range (`lo >= hi`)
- `i32` division by zero
- `i32` modulo by zero

Current static-negative baseline includes:

- `map_empty()` without contextual `Map(K, V)` type
- discarded statement-form `map_empty();`
- text concatenation before PR-E1
- bare `break;` outside `while` / statement `loop`
- `continue;` outside `while` / statement `loop`

Remaining benchmark blocker families:

- text concatenation / minimal formatting for traces
- narrow stdout experiment surface

Those remaining gaps should be frozen in tests only after their scope PRs choose
the public source forms.
