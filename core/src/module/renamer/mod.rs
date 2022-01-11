use std::collections::HashMap;

use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::{Expr, Ident, KeyValueProp, ObjectLit, Prop, PropName, PropOrSpread};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::graph::Ctxt;
use crate::utils::union_find::{UnifyValue, UnionFind};

#[derive(Debug)]
pub struct Renamer<'me> {
  pub ctxt_mapping: &'me HashMap<JsWord, SyntaxContext>,
  pub ctxt_jsword_mapping: HashMap<SyntaxContext, JsWord>,
  pub mapping: &'me HashMap<JsWord, JsWord>,
  pub symbol_rel: &'me UnionFind<Ctxt>,
}

// impl<'me> std::fmt::Debug for Renamer<'me> {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     f.debug_struct("Renamer")
//       .field("ctxt_mapping", &self.ctxt_mapping)
//       .field("ctxt_jsword_mapping", &self.ctxt_jsword_mapping)
//       .field("mapping", &self.mapping)
//       .finish()
//   }
// }

impl<'me> VisitMut for Renamer<'me> {
  fn visit_mut_ident(&mut self, node: &mut Ident) {
    if let Some(&original_ctxt) = self.ctxt_mapping.get(&node.sym) {
      println!(
        "original ctxt {:?} original name: {}",
        original_ctxt,
        node.sym.as_ref()
      );
      if let Some(id) = self.symbol_rel.find(original_ctxt.into()) {
        let canonical_ctxt = Ctxt::from_index(id).0;
        println!("canonical ctxt {:?}", canonical_ctxt);

        if let Some(sym) = self.ctxt_jsword_mapping.get(&canonical_ctxt) {
          println!("name to replace: {}", sym.as_ref());
          if let Some(replacement) = self.mapping.get(&sym).map(|s| s.clone()) {
            node.sym = replacement
          }
        }
      } else {
        if &node.span.ctxt == &original_ctxt {
          if let Some(replacement) = self.mapping.get(&node.sym).map(|s| s.clone()) {
            node.sym = replacement
          }
        }
      }
    }
  }

  fn visit_mut_object_lit(&mut self, node: &mut ObjectLit) {
    node
      .props
      .iter_mut()
      .for_each(|prop_or_spread| match prop_or_spread {
        PropOrSpread::Prop(prop) => {
          if prop.is_shorthand() {
            if let Prop::Shorthand(ident) = prop.as_mut() {
              let mut key = ident.clone();
              key.span.ctxt = SyntaxContext::empty();
              let replacement = Box::new(Prop::KeyValue(KeyValueProp {
                key: PropName::Ident(key),
                value: Box::new(Expr::Ident(ident.clone())),
              }));
              *prop = replacement;
            }
          }
        }
        _ => {}
      });
    node.visit_mut_children_with(self);
  }
}
