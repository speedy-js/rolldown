use std::collections::{HashMap, HashSet};

use crate::{
  graph::{DepGraph, DepNode},
  statement::Statement,
};
use log::debug;

use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::text_writer::JsWriter;

use crate::graph::Ctxt;
use crate::utils::union_find::UnionFind;

pub struct Chunk<'a> {
  pub order_modules: Vec<NodeIndex>,
  pub symbol_rel: &'a UnionFind<Ctxt>,
  // SyntaxContext to Safe name mapping
  pub canonical_names: HashMap<SyntaxContext, JsWord>,
}

impl<'a> Chunk<'a> {
  pub fn new(
    order_modules: Vec<NodeIndex>,
    symbol_rel: &'a UnionFind<Ctxt>,
    canonical_names: HashMap<SyntaxContext, JsWord>,
  ) -> Self {
    Self {
      order_modules,
      symbol_rel,
      canonical_names,
    }
  }

  pub fn deconflict(&mut self, graph: &mut DepGraph) {
    let mut definers = HashMap::new();
    let mut conflicted_names = HashSet::new();

    self.order_modules.iter().for_each(|idx| {
      if let DepNode::Mod(module) = &graph[*idx] {
        println!("module name: {} scope {:?}", module.id, module.scope);
        module.scope.declared_symbols.keys().for_each(|name| {
          if definers.contains_key(name) {
            conflicted_names.insert(name.clone());
          } else {
            definers.insert(name.clone(), vec![]);
          }

          definers.get_mut(name).unwrap().push(*idx);
        });
      }
    });

    conflicted_names.clone().iter().for_each(|name| {
      let module_idxs = definers.get(name).unwrap();
      if module_idxs.len() > 1 {
        module_idxs.iter().enumerate().for_each(|(cnt, idx)| {
          if let DepNode::Mod(module) = &mut graph[*idx] {
            if !module.is_entry {
              let mut safe_name: JsWord = format!("{}${}", name.to_string(), cnt).into();
              while conflicted_names.contains(&safe_name) {
                safe_name = format!("{}_", safe_name.to_string()).into();
              }

              conflicted_names.insert(safe_name.clone());

              module.need_renamed.insert(name.clone(), safe_name.clone());

              if let Some(ctxt) = module.scope.declared_symbols.get(&name) {
                if let Some(repr_ctxt) = self.symbol_rel.find(ctxt.clone().into()) {
                  self.canonical_names.insert(repr_ctxt.into(), safe_name);
                }
              }
            }
          }
        })
      }
    });

    definers.into_iter().for_each(|(_name, idxs)| {
      idxs.into_iter().for_each(|idx| {
        if let DepNode::Mod(module) = &mut graph[idx] {
          module.rename(self.symbol_rel, &self.canonical_names);
        }
      });
    });

    println!("conlicted_names {:#?}", conflicted_names);
  }

  pub fn render(&mut self, graph: &mut DepGraph) -> String {
    self.deconflict(graph);

    let mut output = Vec::new();

    let mut emitter = swc_ecma_codegen::Emitter {
      cfg: swc_ecma_codegen::Config { minify: false },
      cm: crate::graph::SOURCE_MAP.clone(),
      comments: None,
      wr: Box::new(JsWriter::with_target(
        crate::graph::SOURCE_MAP.clone(),
        "\n",
        &mut output,
        None,
        EsVersion::latest(),
      )),
    };

    self
      .order_modules
      .par_iter()
      .flat_map(|idx| {
        if let DepNode::Mod(module) = &graph[*idx] {
          module.render()
        } else {
          vec![]
        }
      })
      .collect::<Vec<Statement>>()
      .iter()
      .for_each(|stmt| {
        if !stmt.is_import_declaration {
          emitter.emit_module_item(&stmt.node).unwrap();
        }
      });

    String::from_utf8(output).unwrap()
  }
}
