//! Experimental persistent AVL indexes. Not approved production API.
use super::{MapKey, Value};
use std::sync::Arc;
#[cfg(rvm002_small_ordinals)]
const ORDER_LIMIT: u64 = 32;
#[cfg(not(rvm002_small_ordinals))]
const ORDER_LIMIT: u64 = u64::MAX;

type Tree<K, V> = Option<Arc<Node<K, V>>>;
#[derive(Debug)]
struct Node<K, V> { key: K, value: V, left: Tree<K,V>, right: Tree<K,V>, height: u16 }
fn height<K,V>(t: &Tree<K,V>) -> u16 { t.as_ref().map_or(0, |n| n.height) }
fn node<K,V>(key: K, value: V, left: Tree<K,V>, right: Tree<K,V>) -> Arc<Node<K,V>> {
    Arc::new(Node { key, value, height: 1 + height(&left).max(height(&right)), left, right })
}
fn balance<K: Clone,V: Clone>(key: K, value: V, left: Tree<K,V>, right: Tree<K,V>) -> Arc<Node<K,V>> {
    if height(&left) > height(&right) + 1 {
        let l=left.as_ref().unwrap();
        if height(&l.left) >= height(&l.right) {
            return node(l.key.clone(),l.value.clone(),l.left.clone(),Some(node(key,value,l.right.clone(),right)));
        }
        let m=l.right.as_ref().unwrap();
        return node(m.key.clone(),m.value.clone(),Some(node(l.key.clone(),l.value.clone(),l.left.clone(),m.left.clone())),Some(node(key,value,m.right.clone(),right)));
    }
    if height(&right) > height(&left) + 1 {
        let r=right.as_ref().unwrap();
        if height(&r.right) >= height(&r.left) {
            return node(r.key.clone(),r.value.clone(),Some(node(key,value,left,r.left.clone())),r.right.clone());
        }
        let m=r.left.as_ref().unwrap();
        return node(m.key.clone(),m.value.clone(),Some(node(key,value,left,m.left.clone())),Some(node(r.key.clone(),r.value.clone(),m.right.clone(),r.right.clone())));
    }
    node(key,value,left,right)
}
fn insert<K: Ord+Clone,V: Clone>(t: &Tree<K,V>, key: K, value: V) -> Tree<K,V> {
    Some(match t {
        None => node(key,value,None,None),
        Some(n) => match key.cmp(&n.key) {
            std::cmp::Ordering::Less => balance(n.key.clone(),n.value.clone(),insert(&n.left,key,value),n.right.clone()),
            std::cmp::Ordering::Greater => balance(n.key.clone(),n.value.clone(),n.left.clone(),insert(&n.right,key,value)),
            std::cmp::Ordering::Equal => node(key,value,n.left.clone(),n.right.clone()),
        }
    })
}
fn get<'a,K:Ord,V>(t: &'a Tree<K,V>,key:&K) -> Option<&'a V> {
    let mut cursor=t.as_deref();
    while let Some(n)=cursor { match key.cmp(&n.key) {
        std::cmp::Ordering::Less => cursor=n.left.as_deref(),
        std::cmp::Ordering::Greater => cursor=n.right.as_deref(),
        std::cmp::Ordering::Equal => return Some(&n.value),
    }} None
}
fn remove<K:Ord+Clone,V:Clone>(t:&Tree<K,V>,key:&K)->Tree<K,V> {
    let n=t.as_ref()?;
    match key.cmp(&n.key) {
        std::cmp::Ordering::Less => Some(balance(n.key.clone(),n.value.clone(),remove(&n.left,key),n.right.clone())),
        std::cmp::Ordering::Greater => Some(balance(n.key.clone(),n.value.clone(),n.left.clone(),remove(&n.right,key))),
        std::cmp::Ordering::Equal => {
            if n.left.is_none() { return n.right.clone(); }
            if n.right.is_none() { return n.left.clone(); }
            let mut successor=n.right.as_deref().unwrap();
            while let Some(l)=successor.left.as_deref() { successor=l; }
            Some(balance(successor.key.clone(),successor.value.clone(),n.left.clone(),remove(&n.right,&successor.key)))
        }
    }
}
fn entries<'a,K,V>(t:&'a Tree<K,V>,out:&mut Vec<(&'a K,&'a V)>) {
    if let Some(n)=t { entries(&n.left,out);out.push((&n.key,&n.value));entries(&n.right,out); }
}

#[derive(Debug,Clone,Default)]
pub struct MapValue {
    keys: Tree<Arc<MapKey>,Arc<Vec<u64>>>,
    order: Tree<u64,(Arc<MapKey>,Arc<Value>)>,
    next: u64,
}
impl MapValue {
    #[cfg(rvm002_qualification)]
    pub fn validate_indexes(&self) {
        fn valid<K:Ord,V>(t:&Tree<K,V>,lo:Option<&K>,hi:Option<&K>)->u16 {
            let Some(n)=t else {return 0};
            assert!(lo.is_none_or(|k| k<&n.key));assert!(hi.is_none_or(|k| &n.key<k));
            let l=valid(&n.left,lo,Some(&n.key));let r=valid(&n.right,Some(&n.key),hi);
            assert!(l.abs_diff(r)<=1);assert_eq!(n.height,1+l.max(r));n.height
        }
        valid(&self.keys,None,None);valid(&self.order,None,None);
        let mut keys=Vec::new();entries(&self.keys,&mut keys);
        let mut count=0;
        for (key,ordinals) in keys {
            assert!(!ordinals.is_empty());assert!(ordinals.windows(2).all(|w| w[0]<w[1]));
            for ordinal in ordinals.iter() {assert_eq!(get(&self.order,ordinal).unwrap().0,key.clone());count+=1;}
        }
        let mut order=Vec::new();entries(&self.order,&mut order);assert_eq!(count,order.len());
        for (ordinal,(key,_)) in order {assert!(get(&self.keys,key).unwrap().contains(ordinal));assert!(*ordinal<self.next);}
    }
    pub fn get(&self,key:&MapKey)->Option<&Value> {
        let mut cursor=self.keys.as_deref();
        let ord=loop {
            let n=cursor?;
            match key.cmp(n.key.as_ref()) {
                std::cmp::Ordering::Less => cursor=n.left.as_deref(),
                std::cmp::Ordering::Greater => cursor=n.right.as_deref(),
                std::cmp::Ordering::Equal => break n.value.first()?,
            }
        };
        Some(get(&self.order,ord)?.1.as_ref())
    }
    pub fn contains(&self,key:&MapKey)->bool { self.get(key).is_some() }
    pub fn updated(&self,key:MapKey,value:Value)->Self {
        // Exhaustion compacts order deterministically; no new runtime error.
        let mut result=if self.next>=ORDER_LIMIT { Self::from(self.to_pairs()) } else { self.clone() };
        let key=Arc::new(key);
        if let Some(ordinals)=get(&result.keys,&key) {
            for ordinal in ordinals.iter() { result.order=remove(&result.order,ordinal); }
        }
        result.keys=insert(&result.keys,key.clone(),Arc::new(vec![result.next]));
        result.order=insert(&result.order,result.next,(key,Arc::new(value)));
        result.next+=1;
        result
    }
    pub fn to_pairs(&self)->Vec<(MapKey,Value)> {
        let mut ordered=Vec::new(); entries(&self.order,&mut ordered);
        ordered.into_iter().map(|(_, (k,v))| (k.as_ref().clone(),v.as_ref().clone())).collect()
    }
}
impl From<Vec<(MapKey,Value)>> for MapValue {
    fn from(pairs:Vec<(MapKey,Value)>)->Self {
        let mut result=Self::default();
        for (key,value) in pairs {
            let key=Arc::new(key);
            let mut ordinals=get(&result.keys,&key).map_or_else(Vec::new,|v| v.as_ref().clone());
            ordinals.push(result.next);
            result.keys=insert(&result.keys,key.clone(),Arc::new(ordinals));
            result.order=insert(&result.order,result.next,(key,Arc::new(value)));
            result.next+=1;
        }
        result
    }
}
impl PartialEq for MapValue {
    fn eq(&self,other:&Self)->bool {
        let mut a=Vec::new();let mut b=Vec::new();entries(&self.order,&mut a);entries(&other.order,&mut b);
        a.len()==b.len() && a.into_iter().zip(b).all(|((_,x),(_,y))| x==y)
    }
}
