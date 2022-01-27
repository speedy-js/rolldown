use std::collections::HashMap;
use swc_atoms::{JsWord};
use swc_common::{util::take::Take, Mark, Span, DUMMY_SP};
use swc_ecma_ast::{
  BindingIdent, CallExpr, Decl, ExportNamedSpecifier, ExportSpecifier, Expr, ExprOrSpread,
  ExprOrSuper, Ident, KeyValueProp, Lit, MemberExpr, ModuleDecl, NamedExport, Null, ObjectLit, Pat,
  Prop, PropName, PropOrSpread, Stmt, Str, VarDecl, VarDeclKind, VarDeclarator,
};

use crate::ext::MarkExt;

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

fn ident(s: &str, mark: &Mark) -> Ident {
  Ident {
    sym: jsword(s),
    span: Span {
      ctxt: mark.as_ctxt(),
      ..DUMMY_SP
    },
    ..Ident::dummy()
  }
}

#[inline]
fn mark_ident(mark: &Mark) -> Ident {
  let mut i = Ident::dummy();
  i.span.ctxt = mark.as_ctxt();
  i
}

#[inline]
fn expr_ident(s: &str) -> Box<Expr> {
  Box::new(Expr::Ident(Ident {
    sym: jsword(s),
    ..Ident::dummy()
  }))
}

pub fn export(exports: &HashMap<JsWord, Mark>) -> ModuleDecl {
  ModuleDecl::ExportNamed(NamedExport {
    span: Default::default(),
    specifiers: exports
      .iter()
      .map(|(name, mark)| {
        ExportSpecifier::Named(ExportNamedSpecifier {
          span: Default::default(),
          orig: mark_ident(mark),
          exported: Some(Ident {
            sym: name.clone(),
            ..Ident::dummy()
          }),
          is_type_only: false,
        })
      })
      .collect::<Vec<_>>(),
    src: None,
    type_only: false,
    asserts: None,
  })
}

pub fn namespace(var_name: (JsWord, Mark), key_values: &[(JsWord, Mark)]) -> Stmt {
  let mut props = vec![PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
    key: PropName::Str(str("__proto__")),
    value: Box::new(Expr::Lit(Lit::Null(Null::dummy()))),
  })))];
  props.append(
    &mut key_values
      .into_iter()
      .map(|(key, value)| {
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
          key: PropName::Str(str(key)),
          value: Box::new(Expr::Ident(mark_ident(value))),
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
        id: ident(&var_name.0, &var_name.1),
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
