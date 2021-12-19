use petgraph::algo::toposort;
use petgraph::dot::Dot;
use petgraph::prelude::Graph;
use rolldown::graph::GraphContainer;

fn main() {
    // let mut graph = GraphContainer::new("./tests/fixtures/preact/index.js".to_owned());
    let mut graph = GraphContainer::new("./tests/fixtures/dynamic-import/main.js".to_owned());
    graph.build();
    // toposort(graph.graph.into_inner().unwrap(), Default::default());
    println!(
        "entry_modules {:?}",
        Dot::new(&*graph.graph.read().unwrap())
    )
}
