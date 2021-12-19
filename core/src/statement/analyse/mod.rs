use swc_common::DUMMY_SP;
use swc_ecma_ast::{ClassDecl, Decl, DefaultDecl, EmptyStmt, Expr, FnDecl, Ident, ModuleDecl, ModuleItem, Stmt, VarDecl, VarDeclarator, Pat, BindingIdent};

pub mod relationship_analyzer;
pub mod scope;
pub mod scope_analyzer;




pub fn fold_export_decl_to_decl(module_item: &mut ModuleItem) {
  if let ModuleItem::ModuleDecl(module_decl) = &module_item {
    *module_item = match module_decl {
      // remove export { foo, baz }
      // FIXME: this will also remove `export * as foo from './foo'`. How we handle this? 
      ModuleDecl::ExportNamed(_) => ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP })),
      // remove `export` from `export class Foo {...}`
      ModuleDecl::ExportDecl(export_decl) => ModuleItem::Stmt(Stmt::Decl(export_decl.decl.clone())),
      // remove `export default` from `export default class Foo {...}`
      ModuleDecl::ExportDefaultDecl(export_decl) => {
        if let DefaultDecl::Class(node) = &export_decl.decl {
          ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
            ident: node
              .ident
              .clone()
              .unwrap_or_else(|| Ident::new("_default".into(), DUMMY_SP)),
            declare: false,
            class: node.class.clone(),
          })))
        } else if let DefaultDecl::Fn(node) = &export_decl.decl {
          ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
            ident: node
              .ident
              .clone()
              .unwrap_or_else(|| Ident::new("_default".into(), DUMMY_SP)),
            declare: false,
            function: node.clone().function,
          })))
        } else {
          ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export_decl.clone()))
        }
      }
      ModuleDecl::ExportDefaultExpr(export_decl) => {
        if let Expr::Arrow(node) = export_decl.expr.as_ref() {
          ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
            span: DUMMY_SP,
            kind: swc_ecma_ast::VarDeclKind::Var,
            declare: false,
            decls: vec![VarDeclarator {
              span: DUMMY_SP,
              name: Pat::Ident(BindingIdent {
                id: Ident::new("_default".into(), DUMMY_SP),
                type_ann: None,
              }),
              definite: false,
              init: Some(Box::new(Expr::Arrow(node.clone()))),
            }],
          })))
        } else {
          ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
        }
      }
      // FIXME: How we handle case like `export * from './foo'`
      _ => ModuleItem::ModuleDecl(module_decl.clone()),
    };
  }
}