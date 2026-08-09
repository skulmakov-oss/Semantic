# SSF-04 Controlled Application Boundary Evidence

Status: candidate exit evidence; not Published Stable

Contract: `semantic.foundation.application/0.1`

Base: `e5743d3a49d59d6554f3aa044995a01696ad7096`

## Implementation authority

| Concern | Owner |
|---|---|
| host-call identities and application ABI | `crates/prom-abi` |
| exact capability IDs, profiles, and structured denial | `crates/prom-cap` |
| SemCode v17 capability declaration | `crates/sm-format` and `crates/sm-ir` |
| builtin source signatures | `crates/sm-front` |
| artifact admission | `crates/sm-verify` |
| runtime grant check before host dispatch | `crates/sm-vm` |
| root-contained host adapter and redacted audit records | `crates/smc-cli` |

No application effect bypasses the verifier/runtime chain. The separate
`PrometheusHostAbi` Stable v1 trait was not widened.

## Executable evidence

`tests/ssf04_application_boundary.rs` proves:

- exact SemCode v17 selection for application builtins;
- deterministic repeated args -> read -> transform -> write results;
- `CliReadOnly` denial before host write dispatch;
- runtime denial of a write before any captured observation, before host dispatch;
- a real CLI file transform inside the declared root;
- parent-traversal rejection without creation of an escaped file.

`crates/smc-cli/src/application_host.rs` unit tests additionally guard
lexical traversal denial and observation-before-write. Existing ABI,
capability, frontend, IR, verifier, and VM suites remain regression gates.

## Security and replay evidence

- Missing capabilities return `CapabilityDenied` with the exact call and
  manifest metadata.
- Host failures return `AbiError` with the exact operation and failure class.
- Absolute/parent/symlink/reparse/root-escape paths fail closed.
- Paths are capped at 4096 bytes and host text is read through a bounded
  16 MiB reader before UTF-8 admission.
- Allow and deny audit records expose only hashes and lengths, never raw host
  payloads; denied operations carry no path or payload hash.
- Duration is explicit input; no wall-clock or ambient environment is read.
- Identical captured inputs produce identical host call order and writes.

## Phase boundary

Network, subprocesses, ambient environment, unrestricted paths, UI expansion,
package-based grants, and Stable promotion remain excluded. SSF-05 may consume
this capability/profile contract but may not redefine it silently.
