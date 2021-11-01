use std::{collections::HashMap, path::Path, sync::Arc};

use swc_common::DUMMY_SP;
use swc_ecma_ast::{
  BindingIdent, ClassDecl, Decl, DefaultDecl, EmptyStmt, EsVersion, ExportSpecifier, Expr, FnDecl,
  Ident, ModuleDecl, ModuleItem, Pat, Stmt, VarDecl, VarDeclarator,
};

use crate::{ast, Statement};

use super::Module;
use swc_common::sync::Lrc;
use swc_common::{
  errors::{ColorConfig, Handler},
  FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_parser::{EsConfig, TsConfig};
impl Module {
  pub fn analyse(
    statements: &[Arc<Statement>],
  ) -> (HashMap<String, ImportDesc>, HashMap<String, ExportDesc>) {
    // analyse imports and exports
    // @TODO
    // Handle duplicated
    let mut imports = HashMap::new();
    let mut exports = HashMap::new();
    statements
      .iter()
      .filter(|s| s.is_import_declaration)
      .for_each(|s| {
        let module_item: &ModuleItem = &s.node.read();
        if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = module_item {
          import_decl.specifiers.iter().for_each(|specifier| {
            let local_name;
            let name;
            match specifier {
              // import foo from './foo'
              swc_ecma_ast::ImportSpecifier::Default(n) => {
                local_name = n.local.sym.to_string();
                name = "default".to_owned();
              }
              // import { foo } from './foo'
              // import { foo as foo2 } from './foo'
              swc_ecma_ast::ImportSpecifier::Named(n) => {
                local_name = n.local.sym.to_string();
                name = n.imported.as_ref().map_or(
                  local_name.clone(), // `import { foo } from './foo'` doesn't has local name
                  |ident| ident.sym.to_string(), // `import { foo as _foo } from './foo'` has local name '_foo'
                );
              }
              // import * as foo from './foo'
              swc_ecma_ast::ImportSpecifier::Namespace(n) => {
                local_name = n.local.sym.to_string();
                name = "*".to_owned()
              }
            }
            assert_ne!(local_name, "default");
            imports.insert(
              local_name.clone(),
              ImportDesc {
                source: import_decl.src.value.to_string(),
                name,
                local_name,
              },
            );
          })
        }
      });

    statements
      .iter()
      .filter(|s| s.is_export_declaration)
      .for_each(|s| {
        let module_item: &ModuleItem = &s.node.read();
        if let ModuleItem::ModuleDecl(module_decl) = module_item {
          match module_decl {
            ModuleDecl::ExportDefaultDecl(node) => {
              // let isAnonymous = matches!(node, ExportDefaultDecl::)
              let declared_name = match &node.decl {
                DefaultDecl::Class(node) => node.ident.as_ref().map(|id| id.sym.to_string()),
                DefaultDecl::Fn(node) => node.ident.as_ref().map(|id| id.sym.to_string()),
                _ => None,
              };

              exports.insert(
                "default".to_owned(),
                ExportDesc::Default(DefaultExportDesc {
                  statement: s.clone(),
                  name: "default".to_owned(),
                  local_name: declared_name
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                  declared_name,
                  identifier: None,
                  is_declaration: true,
                  is_anonymous: false,
                  is_modified: false,
                }),
              );
            }
            ModuleDecl::ExportDefaultExpr(node) => {
              let is_anonymous = matches!(node.expr.as_ref(), Expr::Class(_) | Expr::Fn(_));
              let identifier = match node.expr.as_ref() {
                Expr::Ident(id) => Some(id.sym.to_string()),
                _ => None,
              };
              exports.insert(
                "default".to_owned(),
                ExportDesc::Default(DefaultExportDesc {
                  statement: s.clone(),
                  name: "default".to_owned(),
                  local_name: "default".to_owned(),
                  declared_name: None,
                  identifier,
                  is_declaration: true,
                  is_anonymous,
                  is_modified: false,
                }),
              );
            }
            ModuleDecl::ExportNamed(node) => {
              // export { foo, bar, baz }
              node.specifiers.iter().for_each(|specifier| {
                match specifier {
                  ExportSpecifier::Named(s) => {
                    let local_name = s.orig.sym.to_string();
                    let exported_name = s
                      .exported
                      .as_ref()
                      .map_or(local_name.clone(), |id| id.sym.to_string());
                    exports.insert(
                      exported_name.clone(),
                      ExportDesc::Named(NamedExportDesc {
                        local_name: local_name.clone(),
                        exported_name: exported_name.clone(),
                      }),
                    );
                    if let Some(src) = &node.src {
                      imports.insert(
                        exported_name.clone(),
                        ImportDesc {
                          source: src.value.to_string(),
                          local_name: exported_name,
                          name: local_name,
                        },
                      );
                    };
                  }
                  ExportSpecifier::Namespace(_) => {
                    // TODO:
                  }
                  ExportSpecifier::Default(_) => {
                    // TODO:
                  }
                };
              })
            }
            ModuleDecl::ExportDecl(node) => {
              // export var foo = 42;
              // export function foo () {}
              // TODO: export const { name1, name2: bar } = o;
              match &node.decl {
                Decl::Class(node) => {
                  let name = node.ident.sym.to_string();
                  exports.insert(
                    node.ident.sym.to_string(),
                    ExportDesc::Decl(DeclExportDesc {
                      statement: s.clone(),
                      local_name: name,
                    }),
                  );
                }
                Decl::Fn(node) => {
                  let name = node.ident.sym.to_string();
                  exports.insert(
                    node.ident.sym.to_string(),
                    ExportDesc::Decl(DeclExportDesc {
                      statement: s.clone(),
                      local_name: name,
                    }),
                  );
                }
                Decl::Var(node) => {
                  node.decls.iter().for_each(|decl| {
                    ast::helper::collect_names_of_pat(&decl.name)
                      .into_iter()
                      .for_each(|name| {
                        exports.insert(
                          name.clone(),
                          ExportDesc::Decl(DeclExportDesc {
                            statement: s.clone(),
                            local_name: name,
                          }),
                        );
                      });
                  });
                }
                _ => {}
              }
            }
            _ => {}
          }
        }
      });

    (imports, exports)
  }
}

#[derive(PartialEq, Clone)]
pub struct ImportDesc {
  pub source: String,
  pub name: String,
  pub local_name: String,
}

#[derive(Clone)]
pub enum ExportDesc {
  Default(DefaultExportDesc),
  Named(NamedExportDesc),
  Decl(DeclExportDesc),
}

impl ExportDesc {
  pub fn has_identifier(&self) -> bool {
    match self {
      ExportDesc::Default(n) => n.identifier.is_some(),
      _ => false,
    }
  }

  pub fn default(self) -> Option<DefaultExportDesc> {
    if let ExportDesc::Default(v) = self {
      Some(v)
    } else {
      None
    }
  }
  pub fn named(self) -> Option<NamedExportDesc> {
    if let ExportDesc::Named(v) = self {
      Some(v)
    } else {
      None
    }
  }
  pub fn decl(self) -> Option<DeclExportDesc> {
    if let ExportDesc::Decl(v) = self {
      Some(v)
    } else {
      None
    }
  }
}

#[derive(Clone)]
pub struct NamedExportDesc {
  pub local_name: String,
  pub exported_name: String,
}

#[derive(Clone)]
pub struct DeclExportDesc {
  pub statement: Arc<Statement>,
  pub local_name: String,
}

#[derive(Clone)]
pub struct DefaultExportDesc {
  pub statement: Arc<Statement>,
  pub name: String,
  pub local_name: String,
  pub declared_name: Option<String>,
  // export default foo;
  pub identifier: Option<String>,
  pub is_declaration: bool,
  // is anonymous function
  pub is_anonymous: bool,
  pub is_modified: bool, // in case of `export default foo; foo = somethingElse`
}

pub fn fold_export_decl_to_decl(module_item: &mut ModuleItem, module: Arc<super::Module>) {
  if let ModuleItem::ModuleDecl(module_decl) = &module_item {
    *module_item = match module_decl {
      // remove export { foo, baz }
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
              .unwrap_or_else(|| Ident::new(module.get_canonical_name("default").into(), DUMMY_SP)),
            declare: false,
            class: node.class.clone(),
          })))
        } else if let DefaultDecl::Fn(node) = &export_decl.decl {
          ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
            ident: node
              .ident
              .clone()
              .unwrap_or_else(|| Ident::new(module.get_canonical_name("default").into(), DUMMY_SP)),
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
                id: Ident::new(module.get_canonical_name("default").into(), DUMMY_SP),
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
      _ => ModuleItem::ModuleDecl(module_decl.clone()),
    };
  }
}

pub fn parse_file(
  source_code: String,
  filename: String,
  src_map: &Lrc<SourceMap>,
) -> Result<swc_ecma_ast::Module, swc_ecma_parser::error::Error> {
  let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(src_map.clone()));
  let p = Path::new(filename.as_str());
  let fm = src_map.new_source_file(FileName::Custom(filename.clone()), source_code);
  let ext = p.extension().and_then(|ext| ext.to_str()).unwrap_or("js");
  let syntax = if ext == "ts" || ext == "tsx" {
    Syntax::Typescript(TsConfig {
      dynamic_import: true,
      decorators: false,
      import_assertions: true,
      tsx: ext == "tsx",
      ..Default::default()
    })
  } else {
    Syntax::Es(EsConfig {
      dynamic_import: true,
      num_sep: true,
      static_blocks: true,
      private_in_object: true,
      import_assertions: true,
      top_level_await: true,
      import_meta: true,
      jsx: ext == "jsx",
      optional_chaining: true,
      nullish_coalescing: true,
      export_namespace_from: true,
      export_default_from: true,
      decorators_before_export: true,
      decorators: true,
      fn_bind: true,
      class_props: true,
      class_private_props: true,
      class_private_methods: true,
    })
  };

  let lexer = Lexer::new(
    syntax,
    EsVersion::latest(),
    StringInput::from(fm.as_ref()),
    None,
  );

  let mut parser = Parser::new_from(lexer);

  parser.take_errors().into_iter().for_each(|e| {
    e.into_diagnostic(&handler).emit();
  });
  parser.parse_module()
}

#[cfg(test)]
pub(crate) fn parse_code(code: &str) -> Result<swc_ecma_ast::Module, ()> {
  use swc_common::BytePos;
  let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, None);
  let lexer = Lexer::new(
    // We want to parse ecmascript
    Syntax::Es(EsConfig::default()),
    // JscTarget defaults to es5
    EsVersion::latest(),
    StringInput::new(code, BytePos(0), BytePos(0)),
    None,
  );

  let mut parser = Parser::new_from(lexer);

  parser.take_errors().into_iter().for_each(|e| {
    e.into_diagnostic(&handler).emit();
  });
  parser.parse_module().map_err(|e| {
    // Unrecoverable fatal error occurred
    e.into_diagnostic(&handler).emit()
  })
}
