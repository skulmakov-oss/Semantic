# CTF-E2 Collection Replay Manifest
Status: checked-in replay selection
Owner: language maturity / execution contract
Scope: admitted PCC-7 collection replay evidence only
Non-goal: collection policy changes, project-root determinism, or release freeze

| replay_id | pcc | surface | source_fixture | artifact | expected_status |
| --- | --- | --- | --- | --- | --- |
| CTF-E2-PCC7-SEQUENCE-INDEXING-REPLAY-001 | PCC-7 | Sequence indexing | tests/fixtures/pcc7_sequence/positive_sequence_indexing.sm | CTF-E2-PCC7-SEQUENCE-INDEXING-REPLAY-001.replay.json | accept |
| CTF-E2-PCC7-SEQUENCE-ITERATION-REPLAY-001 | PCC-7 | Sequence iteration | tests/fixtures/pcc7_sequence/positive_sequence_iteration.sm | CTF-E2-PCC7-SEQUENCE-ITERATION-REPLAY-001.replay.json | accept |
| CTF-E2-PCC7-SEQUENCE-MUTATION-REPLAY-001 | PCC-7 | Sequence mutation | tests/fixtures/pcc7_sequence/positive_sequence_push_prepend.sm | CTF-E2-PCC7-SEQUENCE-MUTATION-REPLAY-001.replay.json | accept |
| CTF-E2-PCC7-MAP-INSERT-LOOKUP-REPLAY-001 | PCC-7 | Map insert/lookup | tests/fixtures/pcc7_map/positive_map_basic_insert_lookup.sm | CTF-E2-PCC7-MAP-INSERT-LOOKUP-REPLAY-001.replay.json | accept |
| CTF-E2-PCC7-MAP-PERSISTENT-UPDATE-REPLAY-001 | PCC-7 | Map persistent update | tests/fixtures/pcc7_map/positive_map_persistent_update.sm | CTF-E2-PCC7-MAP-PERSISTENT-UPDATE-REPLAY-001.replay.json | accept |
