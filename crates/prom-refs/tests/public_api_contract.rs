use prom_refs::*;

fn assert_value_traits<T>()
where
    T: core::fmt::Debug + Clone + Copy + PartialEq + Eq + PartialOrd + Ord + core::hash::Hash,
{
}

#[test]
fn prom_refs_public_value_contract_compiles() {
    assert_value_traits::<ReferenceToken>();
    assert_value_traits::<CapabilityRef>();
    assert_value_traits::<ActorRef>();
    assert_value_traits::<SessionRef>();
    assert_value_traits::<ClientRef>();
    assert_value_traits::<RevisionRef>();
    assert_value_traits::<EpochRef>();
}

const TOKEN: ReferenceToken = ReferenceToken::new(1, 2, 3, 4);

const ISSUER: u64 = TOKEN.issuer();
const NAMESPACE: u32 = TOKEN.namespace();
const GENERATION: u32 = TOKEN.generation();
const VALUE: u64 = TOKEN.value();

const CAPABILITY: CapabilityRef = CapabilityRef::new(TOKEN);
const CAPABILITY_TOKEN: ReferenceToken = CAPABILITY.token();

const ACTOR: ActorRef = ActorRef::new(TOKEN);
const ACTOR_TOKEN: ReferenceToken = ACTOR.token();

const SESSION: SessionRef = SessionRef::new(TOKEN);
const SESSION_TOKEN: ReferenceToken = SESSION.token();

const CLIENT: ClientRef = ClientRef::new(TOKEN);
const CLIENT_TOKEN: ReferenceToken = CLIENT.token();

const REVISION: RevisionRef = RevisionRef::new(TOKEN);
const REVISION_TOKEN: ReferenceToken = REVISION.token();

const EPOCH: EpochRef = EpochRef::new(TOKEN);
const EPOCH_TOKEN: ReferenceToken = EPOCH.token();

#[test]
fn runtime_assertions_for_prom_refs_constants() {
    assert_eq!(ISSUER, 1);
    assert_eq!(NAMESPACE, 2);
    assert_eq!(GENERATION, 3);
    assert_eq!(VALUE, 4);

    assert_eq!(CAPABILITY_TOKEN, TOKEN);
    assert_eq!(ACTOR_TOKEN, TOKEN);
    assert_eq!(SESSION_TOKEN, TOKEN);
    assert_eq!(CLIENT_TOKEN, TOKEN);
    assert_eq!(REVISION_TOKEN, TOKEN);
    assert_eq!(EPOCH_TOKEN, TOKEN);
}
