// Regression test for issue #1738: sm-emit's root facade must re-export the
// complete sm-format ownership-path component vocabulary, not a hand-curated
// subset that can silently drift behind sm-format's canonical exports.
//
// This is an external-consumer integration test (crate boundary, not
// `use super::*`) so it actually proves the public facade, not just an
// internal alias.

#[test]
fn ownership_path_component_vocabulary_is_complete_and_value_identical() {
    assert_eq!(
        sm_emit::OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX,
        sm_format::semcode_format::OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX
    );
    assert_eq!(
        sm_emit::OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL,
        sm_format::semcode_format::OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL
    );
    assert_eq!(
        sm_emit::OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD,
        sm_format::semcode_format::OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD
    );
    assert_eq!(
        sm_emit::OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX,
        sm_format::semcode_format::OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX
    );
}
