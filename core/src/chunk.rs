use std::collections::{HashMap, HashSet};

use crate::{
  graph::{DepGraph, DepNode},
  statement::Statement,
};
use dashmap::DashMap;
use ena::unify::{InPlace, UnificationStoreBase, UnificationTable, UnifyKey, InPlaceUnificationTable};
use log::debug;

use once_cell::sync::{Lazy, OnceCell};
use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_parser::Syntax;

pub struct Chunk {
  pub order_modules: Vec<NodeIndex>,
  pub symbol_uf_set: InPlaceUnificationTable<Ctxt>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Ctxt(pub SyntaxContext, u32);

impl From<SyntaxContext> for Ctxt {
  fn from(ctxt: SyntaxContext) -> Self {
      Ctxt::new(ctxt)
  }
}

impl AsRef<SyntaxContext> for Ctxt {
  fn as_ref(&self) -> &SyntaxContext {
      &self.0
  }
}

// static map: DashMap<u32, SyntaxContext> = DashMap::new();
static ctxt_to_idx: Lazy<DashMap<SyntaxContext, u32>> = Lazy::new(|| Default::default());
static idx_to_ctxt_map: Lazy<DashMap<u32, Ctxt>> = Lazy::new(|| Default::default());

impl Ctxt {
  pub fn new(ctxt: SyntaxContext) -> Self {
    println!("Ctxt");
    let default_id = ctxt_to_idx.len() as u32;
    let next_id = ctxt_to_idx.entry(ctxt.clone()).or_insert(default_id);
    println!("Ctxt2");
    idx_to_ctxt_map.entry(*next_id).or_insert_with(|| Self(ctxt, *next_id)).clone()
  }

  pub fn ctxt(&self) -> SyntaxContext {
    self.0
  }
}

impl UnifyKey for Ctxt {
  type Value = ();
  fn index(&self) -> u32 {
    self.1
  }
  fn from_index(u: u32) -> Self {
    idx_to_ctxt_map.get(&u).unwrap().clone()
  }
  fn tag() -> &'static str {
    "tag"
  }
}

impl Default for Chunk {
  fn default() -> Self {
    Self {
      order_modules: Default::default(),
      symbol_uf_set: Default::default(),
    }
  }
}

impl Chunk {
  pub fn new(order_modules: Vec<NodeIndex>) -> Self {
    Self { order_modules, ..Default::default() }
  }

  pub fn deconflict(&mut self, graph: &mut DepGraph) {
    debug!("deconflict");
    let mut definers = HashMap::new();
    let mut conflicted_names = HashSet::new();
    let mut ctxt_to_names: HashMap<SyntaxContext, JsWord> = HashMap::new();

    self.order_modules.iter().for_each(|idx| {
      if let DepNode::Mod(module) = &graph[*idx] {
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
            let ctxt = module.resolve_ctxt(name);
            if !module.is_entry {
              let mut safe_name: JsWord = format!("{}${}", name.to_string(), cnt).into();
              while conflicted_names.contains(&safe_name) {
                safe_name = format!("{}_", safe_name.to_string()).into();
              }

              conflicted_names.insert(safe_name.clone());
              ctxt_to_names.insert(ctxt, safe_name);
            }
          }
        })
      }
    });
    debug!("deconflict2");

    definers.into_iter().for_each(|(_name, idxs)| {
      idxs.into_iter().for_each(|idx| {
        if let DepNode::Mod(module) = &mut graph[idx] {
          module.rename(&mut self.symbol_uf_set, &ctxt_to_names);
        }
      });
    });

    debug!("conlicted_names {:#?}", conflicted_names);
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
