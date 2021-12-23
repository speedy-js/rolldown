use std::collections::HashMap;

use swc_atoms::JsWord;
use swc_common::{Mark, SyntaxContext};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeKind {
  Block,
  Fn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
  // pub depth: usize,
  pub kind: ScopeKind,
  pub mark: Mark,
  pub declared_symbols: HashMap<JsWord, SyntaxContext>,
}

impl Scope {
  pub fn new(kind: ScopeKind, mark: Mark) -> Self {
    
    Self {
      // depth,
      kind,
      mark,
      declared_symbols: Default::default(),
    }
  }
}