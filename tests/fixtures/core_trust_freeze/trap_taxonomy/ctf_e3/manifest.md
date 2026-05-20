# CTF-E3 Trap Taxonomy Manifest
Status: checked-in taxonomy selection
Owner: language maturity / execution contract
Scope: selected PCC failure-surface taxonomy evidence only
Non-goal: trap-class promotion, Map policy widening, or release freeze

| taxonomy_id | pcc | surface | source_fixture | artifact | expected_status |
| --- | --- | --- | --- | --- | --- |
| CTF-E3-PCC7-SEQUENCE-OOB-VM-TRAP-001 | PCC-7 | Sequence out-of-bounds | tests/fixtures/pcc7_collections_diagnostics/negative_sequence_index_out_of_bounds.sm | CTF-E3-PCC7-SEQUENCE-OOB-VM-TRAP-001.taxonomy.json | trap |
| CTF-E3-PCC7-SEQUENCE-EMPTY-POP-VM-TRAP-001 | PCC-7 | Sequence empty pop | tests/fixtures/pcc7_collections_diagnostics/negative_sequence_pop_empty.sm | CTF-E3-PCC7-SEQUENCE-EMPTY-POP-VM-TRAP-001.taxonomy.json | trap |
| CTF-E3-PCC8-ASSERT-FALSE-VM-TRAP-001 | PCC-8 | Stdlib assert(false) | tests/fixtures/pcc8_stdlib_diagnostics/negative_assert_false_trap.sm | CTF-E3-PCC8-ASSERT-FALSE-VM-TRAP-001.taxonomy.json | trap |
| CTF-E3-PCC8-TO-TEXT-UNSUPPORTED-DIAGNOSTIC-001 | PCC-8 | Unsupported to_text(record) | tests/fixtures/pcc8_stdlib_diagnostics/negative_to_text_record.sm | CTF-E3-PCC8-TO-TEXT-UNSUPPORTED-DIAGNOSTIC-001.taxonomy.json | reject |
| CTF-E3-PCC9-MANIFEST-MISSING-FIELD-DIAGNOSTIC-001 | PCC-9 | Project manifest missing package directive | tests/fixtures/pcc9_project_model_diagnostics/negative_manifest_missing_package/Semantic.package | CTF-E3-PCC9-MANIFEST-MISSING-FIELD-DIAGNOSTIC-001.taxonomy.json | reject |
