use crate::{
  graph::{DepNode, SOURCE_MAP},
  module::scope::ScopeKind,
  statement::{
    analyse::{
      fold_export_decl_to_decl,
      relationship_analyzer::{parse_file, ExportDesc},
    },
    Statement,
  },
};
use log::debug;
use rayon::prelude::*;
use std::{
  collections::{HashMap, HashSet},
  hash::Hash,
};
use swc_atoms::{js_word, JsWord};
use swc_common::{Mark, SyntaxContext};
use swc_ecma_ast::{
  ArrowExpr, BlockStmt, CatchClause, ClassDecl, ClassMethod, DefaultDecl, ExportDefaultDecl, Expr,
  ExprOrSuper, FnDecl, FnExpr, ForInStmt, ForOfStmt, ForStmt, Ident, MemberExpr, MethodProp,
  ModuleItem, Param, Pat, Stmt, VarDecl, VarDeclKind,
};
use swc_ecma_visit::{
  as_folder, noop_visit_mut_type, FoldWith, Visit, VisitAllWith, VisitMut, VisitMutWith,
};

use super::scope::Scope;

pub struct SymbolResolver {
  pub stacks: Vec<Scope>,
  pub reuse_scope: bool,
}

impl SymbolResolver {
  pub fn new(scope: Scope) -> Self {
    Self {
      stacks: vec![scope],
      reuse_scope: false,
    }
  }

  pub fn get_cur_scope(&self) -> &Scope {
    self.stacks.get(self.stacks.len() - 1).unwrap()
  }

  pub fn into_cur_scope(self) -> Scope {
    self.stacks.into_iter().next().unwrap()
  }

  pub fn get_cur_scope_mut(&mut self) -> &mut Scope {
    let len = self.stacks.len();
    self.stacks.get_mut(len - 1).unwrap()
  }

  pub fn declare(&mut self, name: &JsWord, kind: VarDeclKind) {
    debug!("declare \"{}\" kind: {:?}", name.to_string(), kind);
    let is_var_decl = match kind {
      VarDeclKind::Let => false,
      VarDeclKind::Const => false,
      VarDeclKind::Var => true,
    };

    let cur_mark = self.get_cur_scope().mark.clone();

    for scope in &mut self.stacks.iter_mut() {
      if !is_var_decl && scope.declared_symbols.contains_key(name) {
        if scope.kind == ScopeKind::Block {
          panic!("duplicate declare {}", name)
        }
      }

      let ctxt = SyntaxContext::empty().apply_mark(cur_mark);
      if is_var_decl {
        if scope.kind == ScopeKind::Fn {
          scope.declared_symbols.insert(name.clone(), ctxt);
        }
      } else {
        scope.declared_symbols.insert(name.clone(), ctxt);
      }
    }
  }

  pub fn declare_pat(&mut self, pat: &mut Pat, kind: VarDeclKind) {
    debug!("declare pat {:?} kind: {:?}", pat, kind);
    collect_mut_ident_of_pat(pat)
      .into_iter()
      .for_each(|ident| self.declare(&mut ident.sym, kind));
  }

  pub fn resolve_ctxt_for_ident(&mut self, ident: &mut Ident) {
    for scope in &mut self.stacks {
      if let Some(ctxt) = scope.declared_symbols.get(&ident.sym) {
        ident.span.ctxt = ctxt.clone();
        break;
      };
    }
  }

  pub fn resolve_ctxt_for_pat(&mut self, pat: &mut Pat) {
    collect_mut_ident_of_pat(pat).into_iter().for_each(|id| {
      self.resolve_ctxt_for_ident(id);
    });
  }
}

impl VisitMut for SymbolResolver {
  fn visit_mut_ident(&mut self, node: &mut Ident) {
    self.resolve_ctxt_for_ident(node);
  }

  fn visit_mut_member_expr(&mut self, node: &mut MemberExpr) {
    // for `foo.bar.biz` or  'foo.[bar]' we only care `foo`
    if let ExprOrSuper::Expr(exp) = &mut node.obj {
      if let Expr::Ident(id) = exp.as_mut() {
        self.resolve_ctxt_for_ident(id);
      }
    }
  }
  // ---- collect defines
  fn visit_mut_export_default_decl(&mut self, node: &mut ExportDefaultDecl) {
    let id = match &mut node.decl {
      DefaultDecl::Class(node) => node.ident.as_mut().map(|s| (s, VarDeclKind::Let)),
      DefaultDecl::Fn(node) => node.ident.as_mut().map(|s| (s, VarDeclKind::Var)),
      _ => None,
    };
    if let Some((id, kind)) = id {
      self.declare(&id.sym, kind);
    }
    node.visit_mut_children_with(self);
  }

  fn visit_mut_fn_expr(&mut self, node: &mut FnExpr) {
    let scope = Scope::new(
      ScopeKind::Fn,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);

    node.function.params.iter_mut().for_each(|p| {
      self.declare_pat(&mut p.pat, VarDeclKind::Var);
    });

    if let Some(ident) = &mut node.ident {
      self.declare(&mut ident.sym, VarDeclKind::Var);
    }

    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }

  fn visit_mut_fn_decl(&mut self, node: &mut FnDecl) {
    self.declare(&mut node.ident.sym, VarDeclKind::Var);

    let scope = Scope::new(
      ScopeKind::Fn,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);

    node.function.params.iter_mut().for_each(|p| {
      self.declare_pat(&mut p.pat, VarDeclKind::Var);
    });

    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }

  fn visit_mut_arrow_expr(&mut self, node: &mut ArrowExpr) {
    let scope = Scope::new(
      ScopeKind::Fn,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);

    node.params.iter_mut().for_each(|p| {
      self.declare_pat(p, VarDeclKind::Var);
    });

    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }

  fn visit_mut_class_method(&mut self, node: &mut ClassMethod) {
    let scope = Scope::new(
      ScopeKind::Fn,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);

    node.function.params.iter_mut().for_each(|p| {
      self.declare_pat(&mut p.pat, VarDeclKind::Var);
    });

    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }

  fn visit_mut_method_prop(&mut self, node: &mut MethodProp) {
    let scope = Scope::new(
      ScopeKind::Fn,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);

    node.function.params.iter_mut().for_each(|p| {
      self.declare_pat(&mut p.pat, VarDeclKind::Var);
    });

    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }

  fn visit_mut_block_stmt(&mut self, node: &mut BlockStmt) {
    // check whether this block is belong to function or ...
    // if yes. we don't need generate another scope for block stmt
    if self.reuse_scope {
      self.reuse_scope = false;
      node.visit_mut_children_with(self);
    } else {
      let scope = Scope::new(
        ScopeKind::Block,
        Mark::fresh(self.get_cur_scope().mark.clone()),
      );
      self.stacks.push(scope);
      node.visit_mut_children_with(self);
      self.stacks.pop();
    }
  }

  fn visit_mut_catch_clause(&mut self, node: &mut CatchClause) {
    let scope = Scope::new(
      ScopeKind::Block,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);
    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }

  fn visit_mut_var_decl(&mut self, node: &mut VarDecl) {
    node.decls.iter_mut().for_each(|declarator| {
      self.declare_pat(&mut declarator.name, node.kind.clone());
    });
    node.visit_mut_children_with(self);
  }

  fn visit_mut_class_decl(&mut self, node: &mut ClassDecl) {
    self.declare(&mut node.ident.sym, VarDeclKind::Const);
    node.visit_mut_children_with(self);
  }

  // --- for

  fn visit_mut_for_stmt(&mut self, node: &mut ForStmt) {
    let scope = Scope::new(
      ScopeKind::Block,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);
    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }

  fn visit_mut_for_in_stmt(&mut self, node: &mut ForInStmt) {
    let scope = Scope::new(
      ScopeKind::Block,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);
    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }

  fn visit_mut_for_of_stmt(&mut self, node: &mut ForOfStmt) {
    let scope = Scope::new(
      ScopeKind::Block,
      Mark::fresh(self.get_cur_scope().mark.clone()),
    );
    self.stacks.push(scope);
    self.reuse_scope = true;
    node.visit_mut_children_with(self);
    self.stacks.pop();
  }
}

fn collect_mut_ident_of_pat(pat: &mut Pat) -> Vec<&mut Ident> {
  match pat {
    // export const a = 1;
    Pat::Ident(pat) => vec![&mut pat.id],
    // export const [a] = [1]
    Pat::Array(pat) => pat
      .elems
      .iter_mut()
      .flat_map(|pat| pat.as_mut().map_or(vec![], collect_mut_ident_of_pat))
      .collect(),
    // TODO: export const { a } = { a: 1 }
    // Pat::Object()
    _ => vec![],
  }
}

// fn collect_ident_of_pat(pat: &Pat) -> Vec<&Ident> {
//   match pat {
//     // export const a = 1;
//     Pat::Ident(pat) => vec![&pat.id],
//     // export const [a] = [1]
//     Pat::Array(pat) => pat
//       .elems
//       .iter()
//       .flat_map(|pat| pat.as_ref().map_or(vec![], collect_ident_of_pat))
//       .collect(),
//     // TODO: export const { a } = { a: 1 }
//     // Pat::Object()
//     _ => vec![],
//   }
// }
