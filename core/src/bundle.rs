use crate::{chunk::Chunk, graph};

#[non_exhaustive]
pub struct Bundle {
  pub graph_container: graph::GraphContainer,
}

impl Bundle {
  pub fn new(graph: graph::GraphContainer) -> Self {
    Self {
      graph_container: graph,
    }
  }

  pub fn generate(&mut self) -> String {
    let chunk = Chunk {
      order_modules: self.graph_container.ordered_modules.clone(),
      symbol_rel: &self.graph_container.symbol_rel,
    };

    chunk.render(&mut self.graph_container.graph)
  }
}
