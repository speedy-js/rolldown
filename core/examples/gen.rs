
use petgraph::dot::Dot;
use rolldown::{bundle::Bundle, graph::GraphContainer};

fn main() {
  let mut graph = GraphContainer::new("./tests/fixtures/preact/index.js".to_owned());
  // let mut graph = GraphContainer::new("./tests/fixtures/basic/main.js".to_owned());
  // let mut graph = GraphContainer::new("../node_modules/lodash-es/lodash.js".to_owned());
  graph.build();

  let mut bundle = Bundle::new(graph);
  
  std::fs::write("output.js", bundle.generate()).unwrap();

  println!(
    "entry_modules {:?}",
    Dot::new(&bundle.graph_container.graph)
  )
}
