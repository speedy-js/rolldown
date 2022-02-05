use std::collections::HashMap;

use dashmap::DashSet;

use crate::{chunk::Chunk, graph, structs::OutputChunk, types::NormalizedOutputOptions};

#[non_exhaustive]
pub struct Bundle {
  pub graph: graph::Graph,
  pub output_options: NormalizedOutputOptions,
}

impl Bundle {
  pub fn new(graph: graph::Graph, output_options: NormalizedOutputOptions) -> Self {
    Self {
      graph,
      output_options,
    }
  }

  pub fn generate(&mut self) -> String {
    let entries = DashSet::new();
    self.graph.entry_indexs.iter().for_each(|entry| {
      let entry = self.graph.graph[*entry].to_owned();
      entries.insert(entry);
    });

    let mut chunk = Chunk {
      id: Default::default(),
      order_modules: self
        .graph
        .ordered_modules
        .clone()
        .into_iter()
        .map(|idx| self.graph.graph[idx].clone())
        .collect(),
      symbol_box: self.graph.symbol_box.clone(),
      entries,
    };

    chunk.render(&mut self.graph.module_by_id)
  }

  pub fn generate_new(&mut self) -> HashMap<String, OutputChunk> {
    let entries = DashSet::new();
    self.graph.entry_indexs.iter().for_each(|entry| {
      let entry = self.graph.graph[*entry].to_owned();
      entries.insert(entry);
    });

    let mut chunks = vec![Chunk {
      id: Default::default(),
      order_modules: self
        .graph
        .ordered_modules
        .clone()
        .into_iter()
        .map(|idx| self.graph.graph[idx].clone())
        .collect(),
      symbol_box: self.graph.symbol_box.clone(),
      entries,
    }];

    chunks.iter_mut().for_each(|chunk| {
      if let Some(file) = &self.output_options.file {
        chunk.id = nodejs_path::basename!(file).into();
      } else {
        chunk.id = chunk.generate_id(&self.output_options);
      }
    });

    let rendered_chunks = chunks
      .iter_mut()
      .map(|chunk| chunk.render_new(&self.output_options, &mut self.graph.module_by_id))
      .collect::<Vec<_>>();

    rendered_chunks
      .into_iter()
      .map(|chunk| OutputChunk {
        file_name: chunk.file_name,
        code: chunk.code,
      })
      .map(|output_chunk| (output_chunk.file_name.clone(), output_chunk))
      .collect()
  }
}
