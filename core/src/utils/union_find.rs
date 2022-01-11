use dashmap::DashMap;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct UnionFind<T: UnifyValue> {
  pub union_map: DashMap<u32, u32>,
  pub key_to_value: DashMap<u32, T::Value>,
  pub value_to_key: DashMap<T::Value, u32>,
}

impl<T: UnifyValue> Default for UnionFind<T> {
  fn default() -> Self {
    Self {
      union_map: Default::default(),
      key_to_value: Default::default(),
      value_to_key: Default::default(),
    }
  }
}

pub trait UnifyValue {
  type Value: Clone + PartialEq + Eq + Hash;

  fn index(value: &Self::Value) -> u32;
  fn from_index(index: u32) -> Self::Value;
}

impl<T: UnifyValue> UnionFind<T> {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn union(&mut self, a: T::Value, b: T::Value) {
    let a_index = T::index(&a);
    let b_index = T::index(&b);

    self.value_to_key.entry(a.clone()).or_insert(a_index);
    self.value_to_key.entry(b.clone()).or_insert(b_index);
    self.key_to_value.entry(a_index).or_insert(a.clone());
    self.key_to_value.entry(b_index).or_insert(b.clone());

    self.union_map.entry(a_index).or_insert(b_index);
  }

  pub fn find(&self, item: T::Value) -> Option<u32> {
    if let Some(item) = self.value_to_key.get(&item) {
      let outer_index = *item;
      let node = T::from_index(outer_index);
      let internal_index = *self.value_to_key.get(&node).unwrap();
      let parent_node = self.key_to_value.get(&internal_index).unwrap();
      self.find(parent_node.clone())
    } else {
      Some(T::index(&item))
    }
  }
}

#[test]
fn should_work() {
  use dashmap::DashMap;
  use once_cell::sync::Lazy;

  struct Value(u32);

  static ctxt_to_idx: Lazy<DashMap<Value, u32>> = Lazy::new(|| Default::default());
  static idx_to_ctxt_map: Lazy<DashMap<u32, Value>> = Lazy::new(|| Default::default());

  impl Value {
    fn new() -> Self {
      let curr_len = ctxt_to_idx.len();
      ctxt_to_idx.entry(curr_len);
    }
  }

  impl UnifyValue for Value {
    type Value = Value;

    fn index(value: &Self::Value) -> u32 {}

    fn from_index(index: u32) -> Self::Value {
      todo!()
    }
  }
}
