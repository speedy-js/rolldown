use std::{
  hash::Hash,
};

use crate::graph::{DepNode};

#[derive(Clone, PartialEq, Eq)]
pub struct Module {
  pub id: String,
}

impl std::fmt::Debug for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("Module").field(&self.id).finish()
  }
}

impl Into<DepNode> for Module {
  fn into(self) -> DepNode {
    DepNode::Mod(self)
  }
}

impl Module {
  pub fn new(id: String) -> Self {
    Module {
      // original_code: None,
      id,
    }
  }
}

impl Hash for Module {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    state.write(&self.id.as_bytes());
  }
}
