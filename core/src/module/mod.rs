use crate::module::symbol_resolver::SymbolResolver;
use crate::{
  graph::{DepNode, SOURCE_MAP},
  statement::{
    analyse::{
      fold_export_decl_to_decl,
      relationship_analyzer::{parse_file, ExportDesc},
    },
    Statement,
  },
};
use rayon::prelude::*;
use swc_atoms::JsWord;
use swc_ecma_ast::Ident;
use swc_ecma_visit::{FoldWith, VisitMut, VisitMutWith, as_folder};
use std::{
  collections::{HashMap},
  hash::Hash,
};
use swc_common::{Mark, SyntaxContext};


use self::renamer::Renamer;
use self::scope::{Scope, ScopeKind};

pub mod scope;
pub mod symbol_resolver;
pub mod renamer;

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
  pub need_renamed: HashMap<JsWord, JsWord>,
  pub scope: Scope,
}

impl std::fmt::Debug for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Module")
      .field("id", &self.id)
      .field("declared", &self.scope.declared_symbols.keys().map(|s| s.to_string()).collect::<Vec<String>>())
      .field("need_renamed", &self.need_renamed)
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
      need_renamed: Default::default(),
      is_included: false,
      scope: Scope::new(ScopeKind::Fn, Mark::fresh(Mark::root())),
    }
  }

  pub fn include_all(&mut self) {
    self.statements.par_iter_mut().for_each(|s| {
      s.is_included = true;
    });
    self.is_included = true;
  }

  pub fn set_source(&mut self, source: String) {
    let mut ast = parse_file(source, &self.id, &SOURCE_MAP).unwrap();

    ast.visit_mut_children_with(&mut ClearMark);

    let mut symbol_declator = SymbolResolver {
      stacks: vec![self.scope.clone()],
      reuse_scope: false,
    };
    ast.visit_mut_children_with(&mut symbol_declator);

    self.scope = symbol_declator.into_cur_scope();

    let statements = ast
      .body
      .into_iter()
      .map(|node| Statement::new(node))
      .collect::<Vec<Statement>>();

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

  pub fn rename(&mut self) {
    self.statements.par_iter_mut().for_each(|stmt| {
      let mut renamer = Renamer {
        ctxt_mapping: &self.scope.declared_symbols,
        mapping: &self.need_renamed,
      };
      stmt.node.visit_mut_with(&mut renamer);
    });
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

#[derive(Clone, Copy)]
struct ClearMark;
impl VisitMut for ClearMark {

  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    ident.span.ctxt = SyntaxContext::empty();
  }
}
