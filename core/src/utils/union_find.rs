use dashmap::DashMap;

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct UnionFind<T: UnifyValue> {
  union_map: DashMap<u32, u32>,
  key_to_value: DashMap<u32, T::Value>,
  value_to_key: DashMap<T::Value, u32>,
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
    self
      .key_to_value
      .entry(a_index)
      .or_insert_with(|| a.clone());
    self
      .key_to_value
      .entry(b_index)
      .or_insert_with(|| b.clone());

    self
      .union_map
      .entry(self.find(a).unwrap_or(a_index))
      .or_insert_with(|| self.find(b).unwrap_or(b_index));
  }

  pub fn find(&self, item: T::Value) -> Option<u32> {
    if let Some(item) = self.value_to_key.get(&item) {
      match self.union_map.get(&item) {
        Some(internal_index) => {
          let parent_node = self.key_to_value.get(&internal_index).unwrap();
          self.find(parent_node.clone())
        }
        None => Some(T::index(item.key())),
      }
    } else {
      None
    }
  }
}

#[test]
fn should_work() {
  use dashmap::DashMap;
  use once_cell::sync::Lazy;

  static CTXT_TO_IDX_MAP: Lazy<DashMap<Value, u32>> = Lazy::new(|| Default::default());
  static IDX_TO_CTXT_MAP: Lazy<DashMap<u32, Value>> = Lazy::new(|| Default::default());

  #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
  struct Value(u32);

  impl Value {
    fn new() -> Self {
      let curr_len = CTXT_TO_IDX_MAP.len();
      let value = Self(curr_len as u32);

      CTXT_TO_IDX_MAP
        .entry(value.clone())
        .or_insert(curr_len as u32);

      value
    }
  }

  impl UnifyValue for Value {
    type Value = Value;

    fn index(value: &Self::Value) -> u32 {
      value.0
    }

    fn from_index(index: u32) -> Self::Value {
      IDX_TO_CTXT_MAP.get(&index).unwrap().clone()
    }
  }

  let value1 = Value::new();
  let value2 = Value::new();

  println!("{:?}{:?}", value1, value2);
  let mut union_rel: UnionFind<Value> = Default::default();

  union_rel.union(value1, value2);
  println!("{:?}", union_rel);

  let value2_res = union_rel.find(value2);
  assert_eq!(value2_res, Some(1));
  let value1_res = union_rel.find(value1);
  assert_eq!(value1_res, value2_res);

  let value3 = Value::new();
  union_rel.union(value1, value3);

  let value4 = Value::new();
  union_rel.union(value4, value1);

  assert_eq!(union_rel.find(value1), union_rel.find(value3));
  assert_eq!(union_rel.find(value2), union_rel.find(value4));
  assert_eq!(union_rel.find(value3), union_rel.find(value4));
  assert_eq!(union_rel.find(value1), union_rel.find(value4));
}
