use std::sync::Arc;
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub enum MapKey { I32(i32), U32(u32), Bool(bool), Text(String), Quad(u8) }
#[derive(Clone,Debug,PartialEq)]
pub enum Value { I32(i32), F64(f64), Sequence(Arc<Vec<Value>>) }
#[path="map_value.rs"] mod map_value;
use map_value::MapValue;
fn main(){
    if cfg!(rvm002_small_ordinals) {
        let mut map=MapValue::from(vec![(MapKey::I32(1),Value::I32(1)),(MapKey::I32(2),Value::I32(2)),(MapKey::I32(1),Value::I32(3))]);
        let old=map.clone();
        for i in 0..300 {map=map.updated(MapKey::I32(2),Value::I32(i));assert_eq!(map.get(&MapKey::I32(1)),Some(&Value::I32(1)));assert_eq!(map.to_pairs(),vec![(MapKey::I32(1),Value::I32(1)),(MapKey::I32(1),Value::I32(3)),(MapKey::I32(2),Value::I32(i))]);#[cfg(rvm002_qualification)] map.validate_indexes();}
        assert_eq!(old.to_pairs(),vec![(MapKey::I32(1),Value::I32(1)),(MapKey::I32(2),Value::I32(2)),(MapKey::I32(1),Value::I32(3))]);
        assert_eq!(map.updated(MapKey::I32(1),Value::I32(9)).to_pairs(),vec![(MapKey::I32(2),Value::I32(299)),(MapKey::I32(1),Value::I32(9))]);
        println!("PASS synthetic limit32: 300 updates, duplicate order, first GET, snapshots, post-compaction replacement");return;
    }
    let mut tree=MapValue::default();let mut reference=Vec::new();let mut versions=Vec::new();let mut state=17u64;
    for i in 0..10000 {
        state=state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let x=((state>>32)%257) as i32;
        let key=match i%5 {0=>MapKey::I32(x),1=>MapKey::U32(x as u32),2=>MapKey::Bool(x%2==0),3=>MapKey::Text(format!("{x:08}")),_=>MapKey::Quad((x%4) as u8)};
        if i%500==0 {versions.push((tree.clone(),reference.clone()));}
        reference.retain(|(k,_)| k!=&key);reference.push((key.clone(),Value::I32(i)));
        tree=tree.updated(key.clone(),Value::I32(i));
        #[cfg(rvm002_qualification)]
        if i%100==0 {tree.validate_indexes();}
        assert_eq!(tree.get(&key),Some(&Value::I32(i)));assert!(tree.contains(&key));
        assert_eq!(tree.to_pairs(),reference);
    }
    for (t,v) in versions {assert_eq!(t.to_pairs(),v);}
    let duplicates=MapValue::from(vec![(MapKey::I32(1),Value::I32(2)),(MapKey::I32(0),Value::I32(0)),(MapKey::I32(1),Value::I32(3))]);
    assert_eq!(duplicates.get(&MapKey::I32(1)),Some(&Value::I32(2)));
    assert_eq!(duplicates.updated(MapKey::I32(1),Value::I32(4)).to_pairs(),vec![(MapKey::I32(0),Value::I32(0)),(MapKey::I32(1),Value::I32(4))]);
    let nan=MapValue::from(vec![(MapKey::Bool(true),Value::Sequence(Arc::new(vec![Value::F64(f64::NAN)])))]);
    assert_ne!(nan,nan.clone());
    let key=MapKey::Text("α".repeat(32768));let large=tree.updated(key.clone(),Value::I32(42));assert_eq!(large.get(&key),Some(&Value::I32(42)));assert!(!tree.contains(&key));
    println!("PASS 10000 differential updates; 20 retained snapshots; 5 key families; duplicates; nested NaN; 65536-byte UTF8 key");
}
