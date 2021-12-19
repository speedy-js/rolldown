use std::{collections::BTreeSet, hash::Hash};

use crate::graph::DepNode;

#[derive(PartialEq, Eq, Clone)]
pub struct ExternalModule {
  pub id: String,
  pub importers: BTreeSet<String>,
  pub dynamic_importers: BTreeSet<String>,
  pub exec_index: usize,
}
impl ExternalModule {
  pub fn new(id: String) -> Self {
    ExternalModule {
      id,
      importers: BTreeSet::default(),
      dynamic_importers: BTreeSet::default(),
      exec_index: usize::MAX,
    }
  }
}

impl std::fmt::Debug for ExternalModule {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("ExternalModule").field(&self.id).finish()
  }
}

impl Hash for ExternalModule {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    state.write(&self.id.as_bytes());
  }
}

impl Into<DepNode> for ExternalModule {
  fn into(self) -> DepNode {
    DepNode::Ext(self)
  }
}
