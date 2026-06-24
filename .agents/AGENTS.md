# Rules

## CI & Testing
Always use `cargo check --workspace --all-targets` and `cargo clippy --workspace --all-targets` rather than just `cargo check` or `cargo test --all-targets`. This ensures that all crates in the workspace are validated, preventing errors that only surface during CI pipeline checks.
