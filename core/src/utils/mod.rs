pub mod ast_sugar;
pub mod name_helpers;
use std::error::Error;
use std::path::Path;

use swc::config::IsModule;
use swc_ecma_ast::EsVersion;

use swc_common::sync::Lrc;
use swc_common::{
  errors::{ColorConfig, Handler},
  FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_parser::{EsConfig, TsConfig};

mod hook;
mod statement;
pub use hook::*;
pub use statement::*;

use crate::compiler;

pub mod path {
  pub fn relative_id(id: String) -> String {
    if nodejs_path::is_absolute(&id) {
      nodejs_path::relative(&nodejs_path::resolve!("."), &id)
    } else {
      id
    }
  }
}

pub fn is_external_module(source: &str) -> bool {
  source.starts_with("node:") || (!nodejs_path::is_absolute(source) && !source.starts_with("."))
}

pub fn parse_file(
  source_code: String,
  filename: &str,
) -> swc_ecma_ast::Module {
  let compiler = compiler::COMPILER.clone();
  let fm = compiler
    .cm
    .new_source_file(FileName::Custom(filename.to_owned()), source_code);
  let handler =
    Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(compiler.cm.clone()));
  let p = Path::new(filename);
  let ext = p.extension().and_then(|ext| ext.to_str()).unwrap_or("js");
  let syntax = if ext == "ts" || ext == "tsx" {
    Syntax::Typescript(TsConfig {
      decorators: false,
      tsx: ext == "tsx",
      ..Default::default()
    })
  } else {
    Syntax::Es(EsConfig {
      static_blocks: true,
      private_in_object: true,
      import_assertions: true,
      jsx: ext == "jsx",
      export_default_from: true,
      decorators_before_export: true,
      decorators: true,
      fn_bind: true,
    })
  };
  compiler
    .parse_js(
      fm,
      &handler,
      EsVersion::Es2022,
      syntax,
      IsModule::Bool(true),
      false,
    )
    .unwrap().module().unwrap()

  //   let lexer = Lexer::new(
  //     syntax,
  //     EsVersion::latest(),
  //     StringInput::from(fm.as_ref()),
  //     None,
  //   );

  //   let mut parser = Parser::new_from(lexer);

  //   parser.take_errors().into_iter().for_each(|e| {
  //     e.into_diagnostic(&handler).emit();
  //   });
  //   parser.parse_module()
  // }

  // pub fn parse_code(code: &str) -> Result<swc_ecma_ast::Module, ()> {
  //   use swc_common::BytePos;
  //   let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, None);
  //   let lexer = Lexer::new(
  //     // We want to parse ecmascript
  //     Syntax::Es(EsConfig::default()),
  //     // JscTarget defaults to es5
  //     EsVersion::latest(),
  //     StringInput::new(code, BytePos(0), BytePos(0)),
  //     None,
  //   );

  //   let mut parser = Parser::new_from(lexer);

  //   parser.take_errors().into_iter().for_each(|e| {
  //     e.into_diagnostic(&handler).emit();
  //   });
  //   parser.parse_module().map_err(|e| {
  //     // Unrecoverable fatal error occurred
  //     e.into_diagnostic(&handler).emit()
  //   })
}
