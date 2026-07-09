# CORE-QUAD-LATTICE-ALIASES

## Status
Completed

## Details
Added explicit lattice aliases (`lattice_meet`, `lattice_join`, `lattice_inverse`) on `QuadroReg32` to allow the VM to map legacy opcodes correctly. Verified through `cargo test -p semantic-core-quad` against mismatch cases.
