use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering::Relaxed};
use std::sync::Arc;
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]pub enum MapKey{I32(i32),U32(u32),Bool(bool),Text(String),Quad(u8)}
#[derive(Clone,Debug,PartialEq)]pub enum Value{I32(i32),F64(f64),Sequence(Arc<Vec<Value>>)}
#[path="map_value.rs"]mod map_value;
use map_value::MapValue;
struct Alloc;static LIVE:AtomicUsize=AtomicUsize::new(0);static PEAK:AtomicUsize=AtomicUsize::new(0);
unsafe impl GlobalAlloc for Alloc{
    unsafe fn alloc(&self,l:Layout)->*mut u8{let p=System.alloc(l);if !p.is_null(){PEAK.fetch_max(LIVE.fetch_add(l.size(),Relaxed)+l.size(),Relaxed);}p}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){LIVE.fetch_sub(l.size(),Relaxed);System.dealloc(p,l)}
}
#[global_allocator]static ALLOC:Alloc=Alloc;
fn main(){
    println!("model,n,versions,base_extra_live_bytes,retained_extra_live_bytes,peak_extra_live_bytes");
    for n in [1024,4096]{
        let before=LIVE.load(Relaxed);
        let base:Vec<_>=(0..n).map(|i|(MapKey::I32(i),Value::I32(i))).collect();
        let base_live=LIVE.load(Relaxed);PEAK.store(base_live,Relaxed);
        let mut versions=Vec::with_capacity(100);
        for j in 0..100{let mut next=base.clone();next.push((MapKey::I32(n+j),Value::I32(j)));versions.push(next);}
        println!("vec,{n},100,{},{},{}",base_live-before,LIVE.load(Relaxed)-base_live,PEAK.load(Relaxed)-base_live);
        drop(versions);drop(base);
        let before=LIVE.load(Relaxed);
        let base=MapValue::from((0..n).map(|i|(MapKey::I32(i),Value::I32(i))).collect::<Vec<_>>());
        let base_live=LIVE.load(Relaxed);PEAK.store(base_live,Relaxed);
        let mut versions=Vec::with_capacity(100);
        for j in 0..100{versions.push(base.updated(MapKey::I32(n+j),Value::I32(j)));}
        println!("persistent,{n},100,{},{},{}",base_live-before,LIVE.load(Relaxed)-base_live,PEAK.load(Relaxed)-base_live);
    }
}
