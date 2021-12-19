use std::{collections::HashMap, io::Write};

use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::text_writer::JsWriter;

use crate::{chunk::Chunk, graph, module_loader, types::Shared, Module};

// #[derive(Debug, Error)]
// pub enum BundleError {
//   #[error("{0}")]
//   GraphError(crate::graph::GraphError),
//   #[error("{0}")]
//   IoError(io::Error),
//   #[error("No Module found")]
//   NoModule,
// }

// impl From<io::Error> for BundleError {
//   fn from(err: io::Error) -> Self {
//     Self::IoError(err)
//   }
// }

// impl From<graph::GraphError> for BundleError {
//   fn from(err: graph::GraphError) -> Self {
//     Self::GraphError(err)
//   }
// }

#[derive(Clone)]
#[non_exhaustive]
pub struct Bundle {
  pub graph: graph::GraphContainer,
}

impl Bundle {
  pub fn new(graph: graph::GraphContainer) -> Self {
    Self { graph }
  }

  pub fn generate(&self) -> String {
    self.generate_chunks().render()
  }

  pub fn generate_chunks(&self) -> Chunk {
    let chunk = Chunk {
      entry_modules: vec![],
      id: None,
      file_name: None,
      ordered_modules: self.graph.modules.clone(),
    };
    let _chunk_by_module: HashMap<Shared<Module>, Chunk> = HashMap::default();

    chunk
  }

  pub fn add_manual_chunks(&self) -> HashMap<Shared<Module>, String> {
    let manual_chunk_alias_by_entry = HashMap::default();

    manual_chunk_alias_by_entry
  }
}
