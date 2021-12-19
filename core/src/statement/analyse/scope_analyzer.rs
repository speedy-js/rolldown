use std::{cell::RefCell, collections::HashSet, rc::Rc};

use swc_ecma_ast::*;
use swc_ecma_visit::{swc_ecma_ast::FnExpr, Node, Visit, VisitWith};

use crate::utils::ast::collect_names_of_pat;

use super::scope::{Scope, ScopeKind};

pub struct ScopeAnalyser {
  // variables doesn't belong to this statement
  pub depends_on: HashSet<String>,
  pub scope: Rc<RefCell<Scope>>,
}

impl ScopeAnalyser {
  pub fn new(root_scope: Rc<RefCell<Scope>>) -> Self {
    ScopeAnalyser {
      depends_on: HashSet::default(),
      scope: root_scope,
    }
  }

  pub fn create_new_scope(&mut self, params: Vec<String>, kind: ScopeKind) -> Rc<RefCell<Scope>> {
    // create an scope based on current scope
    Rc::new(RefCell::new(Scope::new(
      Some(self.scope.clone()),
      params,
      kind,
    )))
  }
}

impl Visit for ScopeAnalyser {
  // --- collect deepened on
  fn visit_ident(&mut self, n: &Ident, _parent: &dyn Node) {
    let reserved_keywords: HashSet<String> = vec!["default", "undefined"]
      .into_iter()
      .map(|s| s.to_owned())
      .collect();
    // FIXME: the implement is not good enough
    let name = n.sym.to_string();
    if !self.scope.borrow().contains(&name) && !reserved_keywords.contains(&name) {
      self.depends_on.insert(name);
    }
  }

  fn visit_member_expr(&mut self, node: &MemberExpr, _parent: &dyn Node) {
    // for `foo.bar.biz` or  'foo.[bar]' we only care `foo`
    if let ExprOrSuper::Expr(exp) = &node.obj {
      if let Expr::Ident(id) = exp.as_ref() {
        let name = id.sym.to_string();
        if !self.scope.borrow().contains(&name) {
          self.depends_on.insert(name);
        }
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
      self.scope.borrow_mut().add_declaration(&name, false)
    }
    node.visit_children_with(self);
  }

  fn visit_fn_expr(&mut self, node: &FnExpr, _parent: &dyn Node) {
    let prev_scope = self.scope.clone();
    let mut params = node
      .function
      .params
      .iter()
      .flat_map(|p| collect_names_of_pat(&p.pat))
      .collect::<Vec<String>>();
    if let Some(ident) = &node.ident {
      // named function expressions - the name is considered part of the function's scope
      params.push(ident.sym.to_string());
    }

    self.scope = self.create_new_scope(params, ScopeKind::Fn);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }

  fn visit_fn_decl(&mut self, node: &FnDecl, _parent: &dyn Node) {
    self
      .scope
      .borrow_mut()
      .add_declaration(&node.ident.sym.to_string(), false);

    let prev_scope = self.scope.clone();
    let params = node
      .function
      .params
      .iter()
      .flat_map(|p| collect_names_of_pat(&p.pat))
      .collect();

    self.scope = self.create_new_scope(params, ScopeKind::Fn);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }

  fn visit_arrow_expr(&mut self, node: &ArrowExpr, _parent: &dyn Node) {
    let prev_scope = self.scope.clone();
    let params = node.params.iter().flat_map(collect_names_of_pat).collect();

    self.scope = self.create_new_scope(params, ScopeKind::Fn);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }

  fn visit_class_method(&mut self, node: &ClassMethod, _parent: &dyn Node) {
    let prev_scope = self.scope.clone();
    let params = node
      .function
      .params
      .iter()
      .flat_map(|p| collect_names_of_pat(&p.pat))
      .collect();

    self.scope = self.create_new_scope(params, ScopeKind::Fn);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }

  fn visit_method_prop(&mut self, node: &MethodProp, _parent: &dyn Node) {
    let prev_scope = self.scope.clone();
    let params = node
      .function
      .params
      .iter()
      .flat_map(|p| collect_names_of_pat(&p.pat))
      .collect();

    self.scope = self.create_new_scope(params, ScopeKind::Fn);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }

  fn visit_block_stmt(&mut self, node: &BlockStmt, _parent: &dyn Node) {
    let should_reuse_scope = matches!(
      self.scope.borrow().kind,
      ScopeKind::Fn | ScopeKind::For | ScopeKind::Catch
    );
    // check whether this block is belong to function or ...
    // if yes. we don't need generate another scope for block stmt
    if should_reuse_scope {
      node.visit_children_with(self);
    } else {
      let prev_scope = self.scope.clone();
      self.scope = self.create_new_scope(vec![], ScopeKind::Block);
      node.visit_children_with(self);
      self.scope = prev_scope;
    }
  }

  fn visit_catch_clause(&mut self, node: &CatchClause, _parent: &dyn Node) {
    let prev_scope = self.scope.clone();
    self.scope = self.create_new_scope(vec![], ScopeKind::Catch);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }

  fn visit_var_decl(&mut self, node: &VarDecl, _parent: &dyn Node) {
    node.decls.iter().for_each(|declarator| {
      if let Pat::Ident(binding_ident) = &declarator.name {
        let name = binding_ident.id.sym.to_string();
        let is_block_declaration = matches!(node.kind, VarDeclKind::Let | VarDeclKind::Const);
        self
          .scope
          .borrow_mut()
          .add_declaration(&name, is_block_declaration);
      };
    });
    node.visit_children_with(self);
  }

  fn visit_class_decl(&mut self, node: &ClassDecl, _parent: &dyn Node) {
    self
      .scope
      .borrow_mut()
      .add_declaration(&node.ident.sym.to_string(), false);
    node.visit_children_with(self);
  }

  // --- for

  fn visit_for_stmt(&mut self, node: &ForStmt, _parent: &dyn Node) {
    let prev_scope = self.scope.clone();
    self.scope = self.create_new_scope(vec![], ScopeKind::For);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }

  fn visit_for_in_stmt(&mut self, node: &ForInStmt, _parent: &dyn Node) {
    let prev_scope = self.scope.clone();
    self.scope = self.create_new_scope(vec![], ScopeKind::For);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }

  fn visit_for_of_stmt(&mut self, node: &ForOfStmt, _parent: &dyn Node) {
    let prev_scope = self.scope.clone();
    self.scope = self.create_new_scope(vec![], ScopeKind::For);
    node.visit_children_with(self);
    self.scope = prev_scope;
  }
  // --- end for
}
