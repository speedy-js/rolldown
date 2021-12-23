use petgraph::dot::Dot;
use rolldown::graph::GraphContainer;
use swc_common::{Globals, Mark, SourceMap, GLOBALS};

fn main() {
  // let mut graph = GraphContainer::new("./tests/fixtures/preact/index.js".to_owned());
  // let mut graph = GraphContainer::new("./tests/fixtures/basic/main.js".to_owned());
  let mut graph = GraphContainer::new("./tests/fixtures/symbols.js".to_owned());
  // let mut graph = GraphContainer::new("../node_modules/lodash-es/lodash.js".to_owned());
  graph.build();
}
