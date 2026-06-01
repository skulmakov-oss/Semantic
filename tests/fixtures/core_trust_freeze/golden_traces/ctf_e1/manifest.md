# CTF-E1 Golden Trace Manifest
Status: checked-in trace selection
Owner: language maturity / execution contract
Scope: selected PCC fixture-backed golden traces only
Non-goal: exhaustive coverage, project-root traces, or release freeze

| trace_id | pcc | surface | source_fixture | artifact | expected_status |
| --- | --- | --- | --- | --- | --- |
| CTF-E1-PCC4-RECORD-POSITIVE-001 | PCC-4 | Records | examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm | CTF-E1-PCC4-RECORD-POSITIVE-001.trace.json | accept |
| CTF-E1-PCC5-ADT-MATCH-POSITIVE-001 | PCC-5 | ADT + basic match | tests/fixtures/pcc5_match/positive_match_unit_enum_label.sm | CTF-E1-PCC5-ADT-MATCH-POSITIVE-001.trace.json | accept |
| CTF-E1-PCC6-OPTION-POSITIVE-001 | PCC-6 | Option | tests/fixtures/pcc6_option/positive_option_some_match.sm | CTF-E1-PCC6-OPTION-POSITIVE-001.trace.json | accept |
| CTF-E1-PCC7-SEQUENCE-POSITIVE-001 | PCC-7 | Sequence | tests/fixtures/pcc7_sequence/positive_sequence_indexing.sm | CTF-E1-PCC7-SEQUENCE-POSITIVE-001.trace.json | accept |
| CTF-E1-PCC8-STDLIB-HELPER-001 | PCC-8 | Stdlib helper boundary | tests/fixtures/pcc8_stdlib/positive_assert_true.sm | CTF-E1-PCC8-STDLIB-HELPER-001.trace.json | accept |
