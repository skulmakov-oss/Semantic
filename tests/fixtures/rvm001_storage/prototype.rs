//! Isolated storage probe. Not a VM implementation or production dependency.
use sm_vm::{MapKey, Value};
use std::{alloc::{GlobalAlloc, Layout, System}, collections::BTreeMap,
    hash::{BuildHasherDefault, Hash, Hasher}, sync::{Arc, atomic::{AtomicUsize, Ordering::Relaxed}}, time::Instant};
struct Allocator;
static COUNT: AtomicUsize = AtomicUsize::new(0);
static BYTES: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = System.alloc(l);
        if !p.is_null() { COUNT.fetch_add(1, Relaxed); BYTES.fetch_add(l.size(), Relaxed);
            PEAK.fetch_max(LIVE.fetch_add(l.size(), Relaxed)+l.size(), Relaxed); } p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { LIVE.fetch_sub(l.size(), Relaxed); System.dealloc(p,l); }
}
#[global_allocator] static ALLOCATOR: Allocator = Allocator;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key(Arc<MapKey>);
impl Hash for Key {
    fn hash<H: Hasher>(&self, h: &mut H) {
        // Fixed family tags and little-endian scalar encoding, independent of host endianness.
        match self.0.as_ref() {
            MapKey::I32(v) => { h.write(&[0]); h.write(&v.to_le_bytes()); }
            MapKey::U32(v) => { h.write(&[1]); h.write(&v.to_le_bytes()); }
            MapKey::Bool(v) => h.write(&[2, u8::from(*v)]),
            MapKey::Text(v) => { h.write(&[3]); h.write(&(v.len() as u64).to_le_bytes()); h.write(v.as_bytes()); }
            MapKey::Quad(v) => h.write(&[4,*v]),
        }
    }
}
struct FixedHash(u64);
impl Default for FixedHash { fn default() -> Self { Self(0xcbf29ce484222325) } }
impl Hasher for FixedHash {
    fn write(&mut self, bytes: &[u8]) { for b in bytes { self.0 = (self.0 ^ u64::from(*b)).wrapping_mul(0x100000001b3); } }
    fn finish(&self) -> u64 { self.0 }
}
type Entry = (u64, Arc<Value>);
#[derive(Default)] struct CollisionHash;
impl Hasher for CollisionHash { fn write(&mut self, _: &[u8]) {} fn finish(&self) -> u64 { 0 } }
#[derive(Clone)]
enum Index {
    Vec(Vec<(Key, Entry)>),
    Cow(Arc<BTreeMap<Key, Entry>>),
    Tree(im::OrdMap<Key, Entry>),
    Hamt(im::HashMap<Key, Entry, BuildHasherDefault<FixedHash>>),
}
#[derive(Clone)]
struct Model { index: Index, order: im::OrdMap<u64, (Key, Arc<Value>)>, next: u64 }
impl Model {
    fn new(kind: &str) -> Self { Self { index: match kind {
        "vec" => Index::Vec(vec![]), "cow_btree" => Index::Cow(Arc::new(BTreeMap::new())),
        "persistent_btree" => Index::Tree(im::OrdMap::new()),
        "hamt" => Index::Hamt(im::HashMap::with_hasher(BuildHasherDefault::<FixedHash>::default())),
        _ => panic!("unknown model"), }, order: im::OrdMap::new(), next: 0 } }
    fn get(&self, key: &Key) -> Option<&Entry> { match &self.index {
        Index::Vec(v) => v.iter().find(|(k,_)|k==key).map(|(_,v)|v),
        Index::Cow(v) => v.get(key), Index::Tree(v) => v.get(key), Index::Hamt(v) => v.get(key), } }
    fn set(&self, key: Key, val: Arc<Value>) -> Self {
        let mut out = self.clone();
        let ordinal = out.next; out.next = out.next.checked_add(1).expect("ordinal capacity");
        if !matches!(out.index, Index::Vec(_)) {
            if let Some((old,_))=self.get(&key) { out.order.remove(old); }
            out.order.insert(ordinal,(key.clone(),val.clone()));
        }
        let entry=(ordinal,val);
        match &mut out.index {
            Index::Vec(v) => { v.retain(|(k,_)|k!=&key); v.push((key,entry)); }
            Index::Cow(v) => { Arc::make_mut(v).insert(key,entry); }
            Index::Tree(v) => { v.insert(key,entry); }
            Index::Hamt(v) => { v.insert(key,entry); }
        } out
    }
    fn ordered(&self) -> Vec<(Key,Arc<Value>)> { match &self.index {
        Index::Vec(v) => v.iter().map(|(k,(_,v))|(k.clone(),v.clone())).collect(),
        _ => self.order.values().cloned().collect(), } }
}
fn measure<F: FnOnce()>(kind:&str, case:&str,n:usize,sample:usize,f:F) {
    let live=LIVE.load(Relaxed); PEAK.store(live,Relaxed);
    let a=COUNT.load(Relaxed); let b=BYTES.load(Relaxed); let t=Instant::now(); f();
    let ns=t.elapsed().as_nanos();
    println!("{kind},{case},{n},{sample},{ns},{},{},{},{}",COUNT.load(Relaxed)-a,BYTES.load(Relaxed)-b,
        PEAK.load(Relaxed).saturating_sub(live),LIVE.load(Relaxed).saturating_sub(live));
}
fn main() {
    let kinds=["vec","cow_btree","persistent_btree","hamt"];
    // Differential checks across all five key families, replacement order, values and snapshots.
    let keys:Vec<_>=[MapKey::I32(-1),MapKey::U32(1),MapKey::Bool(true),MapKey::Text("é".into()),MapKey::Quad(3)]
        .into_iter().map(|k|Key(Arc::new(k))).collect();
    let mut models:Vec<_>=kinds.iter().map(|k|Model::new(k)).collect();
    for i in 0..100 { for m in &mut models {
        let before=m.ordered(); let old=m.clone();
        *m=m.set(keys[i%keys.len()].clone(),Arc::new(Value::I32(i as i32)));
        assert_eq!(old.ordered(),before); }
        for m in &models[1..] { assert_eq!(m.ordered(),models[0].ordered()); }
    }
    // Fixed hashing must handle genuine collisions rather than identify keys by hash alone.
    let mut collision = im::HashMap::<Key, Entry, BuildHasherDefault<CollisionHash>>::with_hasher(BuildHasherDefault::default());
    for i in 0..128 { collision.insert(Key(Arc::new(MapKey::I32(i))), (i as u64,Arc::new(Value::I32(i)))); }
    let previous = collision.clone();
    collision.insert(Key(Arc::new(MapKey::I32(0))), (128,Arc::new(Value::I32(-1))));
    for i in 0..128 { assert_eq!(*previous.get(&Key(Arc::new(MapKey::I32(i)))).unwrap().1,Value::I32(i)); }
    assert_eq!(*collision.get(&Key(Arc::new(MapKey::I32(0)))).unwrap().1,Value::I32(-1));
    // Equality must compare ordered values, never short-circuit on shared-root identity.
    for kind in kinds { let nan=Model::new(kind).set(keys[0].clone(),Arc::new(Value::F64(f64::NAN)));
        assert_ne!(nan.ordered(),nan.ordered()); }
    println!("model,case,n,sample,ns,allocations,allocated_bytes,peak_extra_live_bytes,retained_extra_bytes");
    for n in [16,128,1024] {
        let keys:Vec<_>=(0..=n).map(|i|Key(Arc::new(MapKey::Text(format!("symbol_{i}"))))).collect();
        let value=Arc::new(Value::Text("x".repeat(256)));
        // Models interleaved by sample. All use identical preallocated keys and values.
        for sample in 0..9 { for kind in kinds {
            let mut map=Model::new(kind);
            measure(kind,"build",n,sample,|| { for key in &keys[..n] { map=map.set(key.clone(),value.clone()); } });
            measure(kind,"get",n,sample,|| { for _ in 0..1000 { std::hint::black_box(map.get(&keys[n-1])); } });
            measure(kind,"missing",n,sample,|| { for _ in 0..1000 { assert!(map.get(&keys[n]).is_none()); } });
            measure(kind,"clone",n,sample,|| { for _ in 0..1000 { std::hint::black_box(map.clone()); } });
            measure(kind,"insert",n,sample,|| { for _ in 0..100 { std::hint::black_box(map.set(keys[n].clone(),value.clone())); } });
            measure(kind,"replace",n,sample,|| { for _ in 0..100 { std::hint::black_box(map.set(keys[0].clone(),value.clone())); } });
            let mut snapshots=Vec::with_capacity(100);
            measure(kind,"retain_100_versions",n,sample,|| { for _ in 0..100 {
                map=map.set(keys[0].clone(),value.clone()); snapshots.push(map.clone()); } });
            assert_eq!(snapshots.len(),100);
        } }
    }
}
