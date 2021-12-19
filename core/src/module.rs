use std::{collections::HashMap, hash::Hash};

use crate::{graph::{DepNode, SOURCE_MAP}, statement::{self, Statement, analyse::relationship_analyzer::{ExportDesc, parse_file}}};

type StatementIndex = usize;
#[derive(Clone, PartialEq, Eq)]
pub struct Module {
  pub id: String,
  pub module_side_effects: bool,
  pub statements: Vec<Statement>,
  pub definitions: HashMap<String, StatementIndex>,
  pub modifications: HashMap<String, Vec<StatementIndex>>,
  pub exports: HashMap<String, ExportDesc>,
  pub is_included: bool,
  pub final_names: HashMap<String, String>,
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
      id,
      module_side_effects: true,
      statements: Default::default(),
      definitions: Default::default(),
      modifications: Default::default(),
      exports: Default::default(),
      final_names: Default::default(),
      is_included: false,
    }
  }

  pub fn set_source(&mut self, source: String) {
    let statements =  parse_to_statements(source, self.id.clone());

    statements.iter().enumerate().for_each(|(idx, s)| {
      s.defines.iter().for_each(|s| {
        self.definitions.insert(s.clone(), idx);
      });

      s.modifies.iter().for_each(|name| {
        if !self.modifications.contains_key(name) {
          self.modifications.insert(name.clone(), vec![]);
        }
        if let Some(vec) = self.modifications.get_mut(name) {
          vec.push(idx);
        }
      });

    });
    self.statements = statements;
  }
}

impl Hash for Module {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    state.write(&self.id.as_bytes());
  }
}


fn parse_to_statements(source: String, id: String) -> Vec<Statement> {
  let ast = parse_file(source, id, &SOURCE_MAP).unwrap();
  ast
    .body
    .into_iter()
    .map(|node| Statement::new(node))
    .collect()
}