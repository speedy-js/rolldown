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
  /// Create a new Union Find instance
  pub fn new() -> Self {
    Self::default()
  }

  /// Union two `T::Value`, `T` must impl trait `UnifyValue`
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
      .entry(self.find_index(a).unwrap_or(a_index))
      .or_insert_with(|| self.find_index(b).unwrap_or(b_index));
  }

  fn _find(&self, element: T::Value) -> Option<u32> {
    if let Some(element) = self.value_to_key.get(&element) {
      match self.union_map.get(&element) {
        Some(internal_index) => {
          let parent_node = self.key_to_value.get(&internal_index).unwrap();
          self._find(parent_node.clone())
        }
        None => Some(T::index(element.key())),
      }
    } else {
      None
    }
  }

  /// Find the representative element for the given element's set
  pub fn find(&self, element: T::Value) -> Option<T::Value> {
    match self.find_index(element) {
      Some(index) => Some(T::from_index(index)),
      None => None,
    }
  }

  /// Find the representative element's index which is defined by user for the given element's set
  pub fn find_index(&self, element: T::Value) -> Option<u32> {
    self._find(element)
  }

  /// Test if two elements are in the same set
  pub fn equiv(&self, a: T::Value, b: T::Value) -> bool {
    let a_result = self.find_index(a);
    let b_result = self.find_index(b);

    if a_result.is_none() || b_result.is_none() {
      return false;
    }

    a_result == b_result
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
