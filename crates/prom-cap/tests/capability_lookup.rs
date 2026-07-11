use prom_cap::{
    CapabilityKind, CapabilityLookupBuildError, CapabilityLookupEntry, CapabilityLookupError,
    CapabilityLookupView,
};
use prom_refs::{CapabilityRef, ReferenceToken};

fn reference(issuer: u64, namespace: u32, generation: u32, value: u64) -> CapabilityRef {
    CapabilityRef::new(ReferenceToken::new(issuer, namespace, generation, value))
}

fn entry(
    issuer: u64,
    namespace: u32,
    generation: u32,
    value: u64,
    kind: CapabilityKind,
) -> CapabilityLookupEntry {
    CapabilityLookupEntry::new(reference(issuer, namespace, generation, value), kind)
}

#[test]
fn lookup_view_try_new_accepts_empty_slice() {
    let entries: [CapabilityLookupEntry; 0] = [];
    let view = CapabilityLookupView::try_new(&entries).expect("empty slice must be valid");
    assert_eq!(
        view.lookup(reference(1, 0, 0, 0)),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_view_try_new_accepts_single_entry() {
    let entries = [entry(1, 0, 0, 1, CapabilityKind::GateRead)];
    let view = CapabilityLookupView::try_new(&entries).expect("single entry must be valid");
    assert_eq!(
        view.lookup(reference(1, 0, 0, 1))
            .expect("exact hit must succeed")
            .kind(),
        CapabilityKind::GateRead
    );
}

#[test]
fn lookup_view_try_new_accepts_strictly_sorted_entries() {
    let entries = [
        entry(1, 0, 0, 1, CapabilityKind::GateRead),
        entry(2, 0, 0, 0, CapabilityKind::GateWrite),
        entry(2, 1, 0, 0, CapabilityKind::PulseEmit),
        entry(2, 1, 1, 0, CapabilityKind::StateQuery),
        entry(2, 1, 1, 1, CapabilityKind::StateUpdate),
    ];

    CapabilityLookupView::try_new(&entries).expect("strict full-key ordering must be valid");
}

#[test]
fn lookup_view_try_new_rejects_duplicate_full_key() {
    let entries = [
        entry(1, 0, 0, 1, CapabilityKind::GateRead),
        entry(1, 0, 0, 1, CapabilityKind::GateWrite),
    ];

    assert!(matches!(
        CapabilityLookupView::try_new(&entries),
        Err(CapabilityLookupBuildError::DuplicateReference)
    ));
}

#[test]
fn lookup_view_try_new_rejects_descending_full_key() {
    let entries = [
        entry(2, 0, 0, 0, CapabilityKind::GateWrite),
        entry(1, 9, 9, 9, CapabilityKind::GateRead),
    ];

    assert!(matches!(
        CapabilityLookupView::try_new(&entries),
        Err(CapabilityLookupBuildError::UnsortedEntries)
    ));
}

#[test]
fn lookup_view_try_new_reports_first_invalid_adjacent_pair() {
    let entries = [
        entry(2, 0, 0, 0, CapabilityKind::GateWrite),
        entry(1, 0, 0, 0, CapabilityKind::GateRead),
        entry(1, 0, 0, 0, CapabilityKind::PulseEmit),
    ];

    assert!(matches!(
        CapabilityLookupView::try_new(&entries),
        Err(CapabilityLookupBuildError::UnsortedEntries)
    ));
}

#[test]
fn lookup_view_try_new_classifies_equal_pair_as_duplicate() {
    let entries = [
        entry(1, 0, 0, 0, CapabilityKind::GateRead),
        entry(1, 0, 0, 0, CapabilityKind::PulseEmit),
        entry(2, 0, 0, 0, CapabilityKind::GateWrite),
    ];

    assert!(matches!(
        CapabilityLookupView::try_new(&entries),
        Err(CapabilityLookupBuildError::DuplicateReference)
    ));
}

#[test]
fn lookup_view_try_new_ignores_later_invalid_pairs_after_first() {
    let entries = [
        entry(1, 0, 0, 0, CapabilityKind::GateRead),
        entry(1, 0, 0, 0, CapabilityKind::GateWrite),
        entry(0, 9, 9, 9, CapabilityKind::PulseEmit),
    ];

    assert!(matches!(
        CapabilityLookupView::try_new(&entries),
        Err(CapabilityLookupBuildError::DuplicateReference)
    ));
}

#[test]
fn lookup_returns_exact_full_key_hit() {
    let entries = [
        entry(1, 0, 0, 1, CapabilityKind::GateRead),
        entry(1, 0, 0, 2, CapabilityKind::GateWrite),
    ];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");

    assert_eq!(
        view.lookup(reference(1, 0, 0, 2))
            .expect("exact hit must succeed")
            .kind(),
        CapabilityKind::GateWrite
    );
}

#[test]
fn lookup_rejects_unknown_issuer() {
    let entries = [entry(1, 7, 8, 9, CapabilityKind::GateRead)];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");

    assert_eq!(
        view.lookup(reference(2, 7, 8, 9)),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_rejects_unknown_namespace() {
    let entries = [entry(1, 7, 8, 9, CapabilityKind::GateRead)];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");

    assert_eq!(
        view.lookup(reference(1, 8, 8, 9)),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_rejects_unknown_generation() {
    let entries = [entry(1, 7, 8, 9, CapabilityKind::GateRead)];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");

    assert_eq!(
        view.lookup(reference(1, 7, 9, 9)),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_rejects_unknown_value() {
    let entries = [entry(1, 7, 8, 9, CapabilityKind::GateRead)];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");

    assert_eq!(
        view.lookup(reference(1, 7, 8, 10)),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_same_value_different_issuer_does_not_alias() {
    let entries = [
        entry(1, 7, 8, 9, CapabilityKind::GateRead),
        entry(2, 7, 8, 9, CapabilityKind::GateWrite),
    ];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");

    assert_eq!(
        view.lookup(reference(2, 7, 8, 9))
            .expect("exact issuer match must succeed")
            .kind(),
        CapabilityKind::GateWrite
    );
    assert_eq!(
        view.lookup(reference(3, 7, 8, 9)),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_same_value_different_namespace_does_not_alias() {
    let entries = [
        entry(2, 7, 8, 9, CapabilityKind::GateRead),
        entry(2, 8, 8, 9, CapabilityKind::GateWrite),
    ];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");

    assert_eq!(
        view.lookup(reference(2, 8, 8, 9))
            .expect("exact namespace match must succeed")
            .kind(),
        CapabilityKind::GateWrite
    );
    assert_eq!(
        view.lookup(reference(2, 9, 8, 9)),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_same_value_different_generation_does_not_fallback() {
    let entries = [
        entry(2, 8, 7, 9, CapabilityKind::GateRead),
        entry(2, 8, 8, 9, CapabilityKind::GateWrite),
    ];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");

    assert_eq!(
        view.lookup(reference(2, 8, 8, 9))
            .expect("exact generation match must succeed")
            .kind(),
        CapabilityKind::GateWrite
    );
    assert_eq!(
        view.lookup(reference(2, 8, 9, 9)),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_returns_entry_with_exact_reference_and_kind() {
    let expected = reference(5, 6, 7, 8);
    let entries = [
        entry(1, 0, 0, 1, CapabilityKind::GateRead),
        CapabilityLookupEntry::new(expected, CapabilityKind::StateUpdate),
    ];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");
    let hit = view.lookup(expected).expect("exact hit must succeed");

    assert_eq!(hit.reference(), expected);
    assert_eq!(hit.kind(), CapabilityKind::StateUpdate);
}

#[test]
fn lookup_repeated_hit_is_deterministic() {
    let expected = reference(5, 6, 7, 8);
    let entries = [
        entry(1, 0, 0, 1, CapabilityKind::GateRead),
        CapabilityLookupEntry::new(expected, CapabilityKind::StateUpdate),
    ];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");
    let first = view.lookup(expected).expect("first hit must succeed");
    let second = view.lookup(expected).expect("second hit must succeed");

    assert_eq!(first.reference(), second.reference());
    assert_eq!(first.kind(), second.kind());
    assert!(core::ptr::eq(first, second));
}

#[test]
fn lookup_repeated_miss_is_deterministic() {
    let entries = [entry(1, 0, 0, 1, CapabilityKind::GateRead)];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");
    let missing = reference(9, 9, 9, 9);

    assert_eq!(
        view.lookup(missing),
        Err(CapabilityLookupError::UnknownReference)
    );
    assert_eq!(
        view.lookup(missing),
        Err(CapabilityLookupError::UnknownReference)
    );
}

#[test]
fn lookup_result_is_non_authoritative_entry_only() {
    let expected = reference(5, 6, 7, 8);
    let entries = [CapabilityLookupEntry::new(
        expected,
        CapabilityKind::ControlledObservationSink,
    )];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");
    let hit = view.lookup(expected).expect("exact hit must succeed");

    let borrowed: &CapabilityLookupEntry = hit;
    assert_eq!(borrowed.reference(), expected);
    assert_eq!(borrowed.kind(), CapabilityKind::ControlledObservationSink);
}

#[test]
fn capability_lookup_public_api_contract_compiles() {
    let entries = [
        CapabilityLookupEntry::new(reference(1, 0, 0, 1), CapabilityKind::GateRead),
        CapabilityLookupEntry::new(reference(1, 0, 0, 2), CapabilityKind::GateWrite),
    ];
    let view = CapabilityLookupView::try_new(&entries).expect("view must build");
    let hit = view
        .lookup(reference(1, 0, 0, 1))
        .expect("approved public lookup API must compile");

    assert_eq!(hit.reference(), reference(1, 0, 0, 1));
    assert_eq!(hit.kind(), CapabilityKind::GateRead);
    assert_eq!(
        view.lookup(reference(9, 9, 9, 9)),
        Err(CapabilityLookupError::UnknownReference)
    );
    assert_eq!(
        CapabilityLookupBuildError::DuplicateReference,
        CapabilityLookupBuildError::DuplicateReference
    );
    assert_eq!(
        CapabilityLookupBuildError::UnsortedEntries,
        CapabilityLookupBuildError::UnsortedEntries
    );
}

#[test]
fn capability_lookup_entry_const_construction_compiles() {
    const TOKEN: ReferenceToken = ReferenceToken::new(7, 8, 9, 10);
    const REFERENCE: CapabilityRef = CapabilityRef::new(TOKEN);
    const ENTRY: CapabilityLookupEntry =
        CapabilityLookupEntry::new(REFERENCE, CapabilityKind::GateRead);
    const ENTRY_REFERENCE: CapabilityRef = ENTRY.reference();
    const ENTRY_KIND: CapabilityKind = ENTRY.kind();

    assert_eq!(ENTRY_REFERENCE, REFERENCE);
    assert_eq!(ENTRY_KIND, CapabilityKind::GateRead);
}
