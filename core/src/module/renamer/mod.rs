use std::collections::HashMap;

use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::{Expr, Ident, ImportDecl, KeyValueProp, ObjectLit, Prop, PropName, PropOrSpread};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::graph::Ctxt;
use crate::utils::union_find::{UnifyValue, UnionFind};

pub struct Renamer<'me> {
  pub ctxt_mapping: &'me HashMap<JsWord, SyntaxContext>,
  pub ctxt_jsword_mapping: HashMap<SyntaxContext, JsWord>,
  pub mapping: &'me HashMap<JsWord, JsWord>,
  pub symbol_rel: &'me UnionFind<Ctxt>,
  pub canonical_names: &'me HashMap<SyntaxContext, JsWord>,
}

impl std::fmt::Debug for Renamer<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Renamer")
      .field(
        "ctxt_mapping",
        &self
          .ctxt_mapping
          .iter()
          .map(|k| format!("{}: {:?}", k.0.as_ref(), k.1))
          .collect::<Vec<_>>(),
      )
      .field(
        "ctxt_jsword_mapping",
        &self
          .ctxt_jsword_mapping
          .iter()
          .map(|k| format!("{:?}: {}", k.0, k.1.as_ref()))
          .collect::<Vec<_>>(),
      )
      .field("mapping", &self.mapping)
      .field("symbol_rel", &self.symbol_rel)
      .finish()
  }
}

fn find_jsword_in_current_module(
  ctxt_jsword_mapping: &HashMap<SyntaxContext, JsWord>,
  symbol_rel: &UnionFind<Ctxt>,
  canonical_ctxt: SyntaxContext,
) -> Option<JsWord> {
  for (&ctxt, word) in ctxt_jsword_mapping.iter() {
    if symbol_rel.equiv(ctxt.into(), canonical_ctxt.into()) {
      return Some(word.clone());
    }
  }

  None
}

impl<'me> VisitMut for Renamer<'me> {
  fn visit_mut_import_decl(&mut self, node: &mut ImportDecl) {
    // We won't remove import statement which import external module. So we need to consider following situation
    // ```a.js
    // import { useState } from 'react'
    // console.log(useState)
    // ```
    // ```b.js
    // const useState = () => {}
    // useState()
    // ```
    // ```a+b.js
    // import { useState as useState$1 } from 'react'
    // console.log(useState$1)
    // const useState = () => {}
    // useState()
    // ```
    // TODO:
  }


  fn visit_mut_ident(&mut self, node: &mut Ident) {
    if let Some(&original_ctxt) = self.ctxt_mapping.get(&node.sym) {
      println!(
        "original ctxt {:?} original name: {}",
        original_ctxt,
        node.sym.as_ref()
      );
      if let Some(wrapped_ctxt) = self.symbol_rel.find(original_ctxt.into()) {
        let canonical_ctxt = wrapped_ctxt.0;
        println!("canonical ctxt {:?}", canonical_ctxt);

        if let Some(replacement) = self.canonical_names.get(&canonical_ctxt).map(|s| s.clone()) {
          node.sym = replacement
        } else if let Some(sym) =
          find_jsword_in_current_module(&self.ctxt_jsword_mapping, self.symbol_rel, canonical_ctxt)
        {
          println!("name to replace: {}", sym.as_ref());
          if let Some(replacement) = self.mapping.get(&sym).map(|s| s.clone()) {
            println!("replacement {}", replacement.as_ref());
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
