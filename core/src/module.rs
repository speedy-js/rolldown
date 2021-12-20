use std::{collections::HashMap, hash::Hash};
use rayon::prelude::*;
use crate::{graph::{DepNode, SOURCE_MAP}, statement::{self, Statement, analyse::{fold_export_decl_to_decl, relationship_analyzer::{parse_file, ExportDesc}}}};

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
    f.debug_struct("Module")
      .field("id", &self.id)
      .field("define", &self.definitions.keys())
      .field("final_names", &self.final_names)
      .finish()
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

  pub fn include_all(&mut self) {
    self.statements.par_iter_mut().for_each(|s| {
      s.is_included = true;
    });
    self.is_included = true;
  }

  pub fn set_source(&mut self, source: String) {
    let statements = parse_to_statements(source, self.id.clone());

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

  pub fn render(&self) -> Vec<Statement> {
    self
      .statements
      .iter()
      .filter_map(|s| if s.is_included { Some(s.clone()) } else { None })
      .map(|mut stmt| {
        fold_export_decl_to_decl(&mut stmt.node);
        stmt
      })
      .collect()
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
