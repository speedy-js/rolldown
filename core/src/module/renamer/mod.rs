use std::collections::HashMap;

use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::Ident;
use swc_ecma_visit::VisitMut;

pub struct Renamer<'me> {
  pub ctxt_mapping: &'me HashMap<JsWord, SyntaxContext>,
  pub mapping: &'me HashMap<JsWord, JsWord>,
}

impl<'me> VisitMut for Renamer<'me> {
  fn visit_mut_ident(&mut self, node: &mut Ident) {
    if let Some(ctxt) = self.ctxt_mapping.get(&node.sym) {
      if &node.span.ctxt == ctxt {
        if let Some(replacement) = self.mapping.get(&node.sym).map(|s| s.clone()) {
          node.sym = replacement
        }
      }
    }
  }
}
