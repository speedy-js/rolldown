use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  fmt::Debug,
  hash::Hash,
  rc::Rc,
};

use swc_ecma_ast::*;
use swc_ecma_visit::{VisitAllWith, VisitWith};

use crate::{
  graph::DepNode,
  statement::analyse::scope::{Scope, ScopeKind},
};

use self::analyse::relationship_analyzer::{ExportDesc, RelationshipAnalyzer};

pub mod analyse;

#[derive(Clone, PartialEq, Eq)]
pub struct Statement {
  pub node: ModuleItem,
  pub is_import_declaration: bool,
  pub is_export_declaration: bool,
  pub defines: HashSet<String>,
  pub modifies: HashSet<String>,
  pub depends_on: HashSet<String>,
  pub exports: HashMap<String, ExportDesc>,
  pub is_included: bool,
}

impl Statement {
  pub fn new(node: ModuleItem) -> Self {
    let is_import_declaration = matches!(&node, ModuleItem::ModuleDecl(ModuleDecl::Import(_)));
    let is_export_declaration = if let ModuleItem::ModuleDecl(module_decl) = &node {
      matches!(
        module_decl,
        ModuleDecl::ExportAll(_)
          | ModuleDecl::ExportDecl(_)
          | ModuleDecl::ExportDefaultDecl(_)
          | ModuleDecl::ExportDefaultExpr(_)
          | ModuleDecl::ExportNamed(_)
      )
    } else {
      false
    };
    let scope = Rc::new(RefCell::new(Scope {
      kind: ScopeKind::Mod,
      ..Default::default()
    }));
    let mut scope_analyser = analyse::scope_analyzer::ScopeAnalyser::new(scope.clone());
    node.visit_children_with(&mut scope_analyser);

    let defines = scope.as_ref().borrow().defines.clone();
    Statement {
      node,
      defines,
      modifies: scope_analyser.depends_on.clone(),
      depends_on: scope_analyser.depends_on,
      is_import_declaration,
      is_export_declaration,
      exports: Default::default(),
      is_included: false,
    }
  }

  pub fn analyse(&self) -> RelationshipAnalyzer {
    let mut relationship_analyzer = RelationshipAnalyzer::new();
    self
      .node
      .visit_all_children_with(&mut relationship_analyzer);
    relationship_analyzer
  }
}

impl Hash for Statement {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.node.hash(state)
  }
}

impl Debug for Statement {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Statement")
      .field("defines", &self.defines)
      .field("depends_on", &self.depends_on)
      .field("exports", &self.exports)
      .finish()
  }
}
