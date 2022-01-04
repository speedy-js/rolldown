use crate::chunk::Ctxt;
use crate::scanner::rel::ExportDesc;
use crate::scanner::scope::{Scope, ScopeKind};
use crate::scanner::Scanner;
use crate::utils::parse_file;
use crate::{
  graph::{DepNode, SOURCE_MAP},
  statement::Statement,
};
use ena::unify::InPlaceUnificationTable;
use rayon::prelude::*;
use std::{collections::HashMap, hash::Hash};
use swc_atoms::JsWord;
use swc_common::{Mark, SyntaxContext};
use swc_ecma_ast::{Ident, ModuleItem};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use self::renamer::Renamer;

pub mod renamer;

#[derive(Clone, PartialEq, Eq)]
struct Symbol {
  pub name: JsWord,
  pub ctxt: SyntaxContext,
}
impl Symbol {

}

#[derive(Clone, PartialEq, Eq)]
pub struct Namespace {
  name: Symbol,
  all: bool,
  // TODO: we use this to handle case like `export * from 'react'`
  merged_namespaces: Vec<Symbol>
}

#[derive(Clone, PartialEq, Eq)]
pub struct Module {
  pub id: String,
  pub module_side_effects: bool,
  pub statements: Vec<Statement>,
  pub exports: HashMap<String, ExportDesc>,

  pub is_included: bool,
  pub need_renamed: HashMap<JsWord, JsWord>,
  pub scope: Scope,
  pub is_entry: bool,

  pub namespace: Option<Namespace>,
}

impl std::fmt::Debug for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Module")
      .field("id", &self.id)
      .field(
        "declared",
        &self
          .scope
          .declared_symbols
          .keys()
          .map(|s| s.to_string())
          .collect::<Vec<String>>(),
      )
      .field("need_renamed", &self.need_renamed)
      .field("scope", &self.scope)
      .finish()
  }
}

impl Into<DepNode> for Module {
  fn into(self) -> DepNode {
    DepNode::Mod(self)
  }
}

impl Module {
  pub fn new(id: String, is_entry: bool) -> Self {
    Module {
      id,
      module_side_effects: true,
      statements: Default::default(),
      // definitions: Default::default(),
      // modifications: Default::default(),
      exports: Default::default(),
      need_renamed: Default::default(),
      is_included: false,
      scope: Scope::new(ScopeKind::Fn, Mark::fresh(Mark::root())),
      is_entry,
      namespace: None,
    }
  }

  pub fn include_all(&mut self) {
    self.statements.par_iter_mut().for_each(|s| {
      s.is_included = true;
    });
    self.is_included = true;
  }

  pub fn set_source(&mut self, source: String) -> Scanner {
    let mut ast = parse_file(source, &self.id, &SOURCE_MAP).unwrap();

    ast.body.sort_by(|a, b| {
      use std::cmp::Ordering;
      let is_a_module_decl = matches!(a, ModuleItem::ModuleDecl(_));
      let is_b_module_decl = matches!(b, ModuleItem::ModuleDecl(_));
      if is_a_module_decl && !is_b_module_decl {
        Ordering::Less
      } else if is_b_module_decl && !is_a_module_decl {
        Ordering::Greater
      } else {
        Ordering::Equal
      }
    });

    ast.visit_mut_children_with(&mut ClearMark);

    let mut scanner = Scanner::new(self.scope.clone());

    ast.visit_mut_children_with(&mut scanner);

    println!("ast {:#?}", ast);

    self.scope = scanner.get_cur_scope().clone();

    let statements = ast
      .body
      .into_par_iter()
      .map(|node| Statement::new(node))
      .collect::<Vec<Statement>>();

    self.statements = statements;

    scanner
  }

  pub fn rename(&mut self, symbol_uf: &mut InPlaceUnificationTable<Ctxt>, ctxt_to_name: &HashMap<SyntaxContext, JsWord>) {
    let s = symbol_uf.new_key(());
    self.statements.iter_mut().for_each(|stmt| {
      let mut renamer = Renamer {
        ctxt_mapping: &self.scope.declared_symbols,
        mapping: &self.need_renamed,
        symbol_uf,
        ctxt_to_name,
      };
      stmt.node.visit_mut_with(&mut renamer);
    });
  }


  pub fn resolve_ctxt(&self, name: &JsWord) -> SyntaxContext {
    self.scope.declared_symbols.get(name).unwrap().clone()
  }

  pub fn render(&self) -> Vec<Statement> {
    self
      .statements
      .iter()
      .filter_map(|s| if s.is_included { Some(s.clone()) } else { None })
      .map(|stmt| {
        // fold_export_decl_to_decl(&mut stmt.node);
        stmt
      })
      .collect()
  }

  pub fn render_namespace() -> Vec<Statement> {
    vec![]
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

mod ast_gen {
  use swc_atoms::{js_word, JsWord};
  use swc_common::{util::take::Take, DUMMY_SP};
  use swc_ecma_ast::{
    BindingIdent, CallExpr, Decl, Expr, ExprOrSpread, ExprOrSuper, Ident, KeyValueProp, Lit,
    MemberExpr, Null, ObjectLit, Pat, Prop, PropName, PropOrSpread, Stmt, Str, VarDecl,
    VarDeclKind, VarDeclarator,
  };

  #[inline]
  fn jsword(s: &str) -> JsWord {
    s.to_owned().into()
  }

  #[inline]
  fn str(s: &str) -> Str {
    Str {
      value: jsword(s),
      ..Str::dummy()
    }
  }

  fn ident(s: &str) -> Ident {
    Ident {
      sym: jsword(s),
      ..Ident::dummy()
    }
  }

  #[inline]
  fn expr_ident(s: &str) -> Box<Expr> {
    Box::new(Expr::Ident(Ident {
      sym: jsword(s),
      ..Ident::dummy()
    }))
  }

  pub fn namespace(name: JsWord, key_values: &[(JsWord, JsWord)]) -> Stmt {
    let mut props = vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
      key: PropName::Str(str("__proto__")),
      value: Box::new(Expr::Lit(Lit::Null(Null::dummy()))),
    })))];
    props.append(
      &mut key_values
        .iter()
        .map(|(key, value)| {
          PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Str(str(key)),
            value: Box::new(Expr::Ident(ident(value))),
          })))
        })
        .collect(),
    );
    Stmt::Decl(Decl::Var(VarDecl {
      span: DUMMY_SP,
      kind: VarDeclKind::Const,
      declare: false,
      decls: vec![VarDeclarator {
        span: DUMMY_SP,
        definite: false,
        name: Pat::Ident(BindingIdent {
          type_ann: None,
          id: Ident {
            span: DUMMY_SP,
            sym: name,
            optional: false,
          },
        }),
        init: Some(Box::new(Expr::Call(CallExpr {
          callee: ExprOrSuper::Expr(Box::new(Expr::Member(MemberExpr {
            obj: ExprOrSuper::Expr(Box::new(Expr::Ident(Ident {
              sym: jsword("Object"),
              ..Ident::dummy()
            }))),
            prop: Box::new(Expr::Ident(Ident {
              sym: jsword("freeze"),
              ..Ident::dummy()
            })),
            ..MemberExpr::dummy()
          }))),
          args: vec![ExprOrSpread {
            expr: Box::new(Expr::Object(ObjectLit {
              span: DUMMY_SP,
              props,
            })),
            spread: None,
          }],
          ..CallExpr::dummy()
        }))),
      }],
    }))
  }
}
