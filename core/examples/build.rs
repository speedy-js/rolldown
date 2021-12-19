use petgraph::dot::Dot;
use rolldown::graph::GraphContainer;

fn main() {
  // let mut graph = GraphContainer::new("./tests/fixtures/preact/index.js".to_owned());
  // let mut graph = GraphContainer::new("./tests/fixtures/basic/main.js".to_owned());
  let mut graph = GraphContainer::new("../node_modules/lodash-es/lodash.js".to_owned());
  graph.build();
  // toposort(graph.graph.into_inner().unwrap(), Default::default());
  println!(
    "entry_modules {:?}",
    Dot::new(&graph.graph)
  )
}
