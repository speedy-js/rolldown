use std::{collections::HashSet, sync::Arc};

use ahash::RandomState;
use swc_ecma_ast::*;
use swc_ecma_visit::{swc_ecma_ast::FnExpr, Node, Visit, VisitWith};

use crate::ast::{self, scope::Scope};

pub struct StatementAnalyser {
  // variables doesn't belong to this statement
  pub depends_on: HashSet<String, RandomState>,
  pub scope: Arc<Scope>,
  pub should_create_block_scope: bool,
}

impl StatementAnalyser {
  pub fn new(root_scope: Arc<Scope>) -> Self {
    StatementAnalyser {
      depends_on: HashSet::default(),
      scope: root_scope,
      should_create_block_scope: true,
    }
  }

  pub fn create_new_scope(
    &mut self,
    parent: Option<Arc<Scope>>,
    params: Vec<String>,
    is_block: bool,
  ) -> Arc<Scope> {
    Arc::new(Scope::new(parent, params, is_block))
  }

  pub fn mark_should_not_create_block_scope(&mut self) {
    self.should_create_block_scope = false
  }

  #[inline]
  pub fn reset_to_parent_scope(&mut self, scope: &Arc<Scope>) {
    if let Some(parent) = scope.parent.as_ref() {
      self.scope = parent.clone()
    }
  }
}

impl Visit for StatementAnalyser {
  // --- collect deepened on
  fn visit_ident(&mut self, n: &Ident, _parent: &dyn Node) {
    let reserved_keywords: HashSet<String> = vec!["default", "undefined"]
      .into_iter()
      .map(|s| s.to_owned())
      .collect();
    // FIXME: the implement is not good enough
    let name = n.sym.to_string();
    if !self.scope.contains(&name) && !reserved_keywords.contains(&name) {
      self.depends_on.insert(name);
    }
  }

  fn visit_member_expr(&mut self, node: &MemberExpr, _parent: &dyn Node) {
    // for `foo.bar.biz` or  'foo.[bar]' we only care `foo`
    if let ExprOrSuper::Expr(exp) = &node.obj {
      if let Expr::Ident(id) = exp.as_ref() {
        self.depends_on.insert(id.sym.to_string());
      }
    }
  }
  // ---- collect defines
  fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl, _parent: &dyn Node) {
    let name = match &node.decl {
      DefaultDecl::Class(node) => node.ident.as_ref().map(|id| id.sym.to_string()),
      DefaultDecl::Fn(node) => node.ident.as_ref().map(|id| id.sym.to_string()),
      _ => None,
    };
    if let Some(name) = name {
      self.scope.add_declaration(&name, false)
    }
    node.visit_children_with(self);
  }

  fn visit_fn_expr(&mut self, node: &FnExpr, _parent: &dyn Node) {
    let mut params = node
      .function
      .params
      .iter()
      .flat_map(|p| ast::helper::collect_names_of_pat(&p.pat))
      .collect::<Vec<String>>();

    if let Some(ident) = &node.ident {
      // named function expressions - the name is considered part of the function's scope
      params.push(ident.sym.to_string());
    }

    let scope_of_this_node_generate =
      self.create_new_scope(Some(self.scope.clone()), params, false);
    self.scope = scope_of_this_node_generate.clone();
    self.mark_should_not_create_block_scope();
    node.visit_children_with(self);
    self.reset_to_parent_scope(&scope_of_this_node_generate)
  }

  fn visit_fn_decl(&mut self, node: &FnDecl, _parent: &dyn Node) {
    self
      .scope
      .add_declaration(&node.ident.sym.to_string(), false);

    let params = node
      .function
      .params
      .iter()
      .flat_map(|p| ast::helper::collect_names_of_pat(&p.pat))
      .collect();
    let scope_of_this_node_generate =
      self.create_new_scope(Some(self.scope.clone()), params, false);
    self.scope = scope_of_this_node_generate.clone();
    self.mark_should_not_create_block_scope();
    node.visit_children_with(self);
    self.reset_to_parent_scope(&scope_of_this_node_generate)
  }

  fn visit_arrow_expr(&mut self, node: &ArrowExpr, _parent: &dyn Node) {
    let params = node
      .params
      .iter()
      .map(|p| ast::helper::collect_names_of_pat(p))
      .flatten()
      .collect();

    let scope_of_this_node_generate =
      self.create_new_scope(Some(self.scope.clone()), params, false);
    self.scope = scope_of_this_node_generate.clone();
    self.mark_should_not_create_block_scope();
    node.visit_children_with(self);
    self.reset_to_parent_scope(&scope_of_this_node_generate)
  }

  fn visit_class_method(&mut self, node: &ClassMethod, _parent: &dyn Node) {
    let params = node
      .function
      .params
      .iter()
      .flat_map(|p| ast::helper::collect_names_of_pat(&p.pat))
      .collect();
    let scope_of_this_node_generate =
      self.create_new_scope(Some(self.scope.clone()), params, false);
    self.scope = scope_of_this_node_generate.clone();
    self.mark_should_not_create_block_scope();
    node.visit_children_with(self);
    self.reset_to_parent_scope(&scope_of_this_node_generate)
  }

  fn visit_method_prop(&mut self, node: &MethodProp, _parent: &dyn Node) {
    let params = node
      .function
      .params
      .iter()
      .flat_map(|p| ast::helper::collect_names_of_pat(&p.pat))
      .collect();

    let scope_of_this_node_generate =
      self.create_new_scope(Some(self.scope.clone()), params, false);
    self.scope = scope_of_this_node_generate.clone();
    self.mark_should_not_create_block_scope();
    node.visit_children_with(self);
    self.reset_to_parent_scope(&scope_of_this_node_generate)
  }

  fn visit_block_stmt(&mut self, node: &BlockStmt, _parent: &dyn Node) {
    // check whether this block is belong to function
    // if yes. we don't need generate another scope for block stmt
    if self.should_create_block_scope {
      let scope_of_this_node_generate =
        self.create_new_scope(Some(self.scope.clone()), vec![], true);
      self.scope = scope_of_this_node_generate.clone();
      node.visit_children_with(self);
      self.reset_to_parent_scope(&scope_of_this_node_generate)
    } else {
      self.should_create_block_scope = true;
      node.visit_children_with(self);
    }
  }

  fn visit_catch_clause(&mut self, node: &CatchClause, _parent: &dyn Node) {
    let scope_of_this_node_generate = self.create_new_scope(Some(self.scope.clone()), vec![], true);
    self.scope = scope_of_this_node_generate.clone();
    self.mark_should_not_create_block_scope();
    node.visit_children_with(self);
    self.reset_to_parent_scope(&scope_of_this_node_generate)
  }

  fn visit_var_decl(&mut self, node: &VarDecl, _parent: &dyn Node) {
    node.decls.iter().for_each(|declarator| {
      if let Pat::Ident(binding_ident) = &declarator.name {
        let name = binding_ident.id.sym.to_string();
        let is_block_declaration = matches!(node.kind, VarDeclKind::Let | VarDeclKind::Const);
        self.scope.add_declaration(&name, is_block_declaration);
      };
    });
    node.visit_children_with(self);
  }

  fn visit_class_decl(&mut self, node: &ClassDecl, _parent: &dyn Node) {
    self
      .scope
      .add_declaration(&node.ident.sym.to_string(), false);
    node.visit_children_with(self);
  }

  fn visit_for_stmt(&mut self, node: &ForStmt, _parent: &dyn Node) {
    let scope_of_this_node_generate = self.create_new_scope(Some(self.scope.clone()), vec![], true);
    self.scope = scope_of_this_node_generate.clone();
    self.mark_should_not_create_block_scope();
    node.visit_children_with(self);
    self.reset_to_parent_scope(&scope_of_this_node_generate)
  }
}
