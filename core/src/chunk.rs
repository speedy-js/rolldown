use std::collections::{HashMap, HashSet};

use log::debug;
use petgraph::graph::NodeIndex;
use rayon::prelude::*;
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::text_writer::JsWriter;

use crate::{
  graph::{DepGraph, DepNode},
  statement::{analyse::fold_export_decl_to_decl, Statement},
};

pub struct Chunk {
  pub order_modules: Vec<NodeIndex>,
}

impl Default for Chunk {
  fn default() -> Self {
    Self {
      order_modules: Default::default(),
    }
  }
}

impl Chunk {
  pub fn new(order_modules: Vec<NodeIndex>) -> Self {
    Self { order_modules }
  }

  // pub fn deconflict(&self, graph: &mut DepGraph) {
  //   // TODO: replace name
  //   let mut definers: HashMap<String, Vec<NodeIndex>> = HashMap::new();
  //   let mut conflicted_names: HashSet<String> = Default::default();

  //   self.order_modules.iter().for_each(|idx| {
  //     if let DepNode::Mod(module) = &graph[*idx] {
  //       module.definitions.keys().for_each(|name| {
  //         if definers.contains_key(name) {
  //           conflicted_names.insert(name.clone());
  //         } else {
  //           definers.insert(name.clone(), vec![]);
  //         }

  //         definers.get_mut(name).unwrap().push(*idx);
  //       });
  //     }
  //   });

  //   conflicted_names.clone().iter().for_each(|name| {
  //     let module_idxs = definers.get(name).unwrap();

  //     module_idxs.iter()
  //       .enumerate()
  //       .for_each(|(cnt, idx)| {
  //         if cnt == 0 { return };

  //         if let DepNode::Mod(module) = &mut graph[*idx] {
  //           let mut safe_name = format!("{}${}", name, cnt);
  //           while conflicted_names.contains(&safe_name) {
  //             safe_name.push_str("_");
  //           }

  //           conflicted_names.insert(safe_name.clone());

  //           module.final_names.insert(name.clone(), safe_name);
  //         }
  //       })
  //   });

  //   debug!("conlicted_names {:#?}", conflicted_names);
  // }

  pub fn render(&self, graph: &mut DepGraph) -> String {
    // self.deconflict(graph);

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
