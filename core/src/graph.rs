use ena::unify::InPlaceUnificationTable;
use log::debug;
use petgraph::dot::Dot;
use petgraph::visit::{DfsPostOrder, EdgeRef, IntoEdgesDirected};
use std::collections::HashMap;

use once_cell::sync::Lazy;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use swc_common::sync::Lrc;
use swc_common::{Globals, SourceMap, GLOBALS};

use crate::chunk::Ctxt;
use crate::external_module::ExternalModule;
use crate::module::Module;
use crate::scanner::rel::{DynImportDesc, ImportDesc, ReExportDesc};
use crate::scanner::Scanner;
use crate::types::ResolvedId;
use crate::utils::resolve_id::resolve_id;

pub(crate) static SOURCE_MAP: Lazy<Lrc<SourceMap>> = Lazy::new(Default::default);

#[derive(Debug, Hash, PartialEq, Eq, Clone)]

pub enum DepNode {
  Mod(Module),
  Ext(ExternalModule),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Rel {
  Import(ImportDesc),
  DynImport(DynImportDesc),
  ReExport(ReExportDesc),
  ReExportAll,
}

pub type DepGraph = Graph<DepNode, Rel>;

#[non_exhaustive]
pub struct GraphContainer {
  pub entry_path: String,
  pub graph: DepGraph,
  pub entries: Vec<NodeIndex>,
  pub ordered_modules: Vec<NodeIndex>,
  pub symbol_uf: InPlaceUnificationTable<Ctxt>,
  // pub globals: Globals,
}

impl GraphContainer {
  pub fn new(entry: String) -> Self {
    env_logger::init();

    let graph = Graph::default();

    let s = Self {
      entry_path: entry,
      graph,
      entries: Default::default(),
      ordered_modules: Default::default(),
      symbol_uf: Default::default(),
    };
    s
  }

  // build dependency graph via entry modules.
  fn generate_module_graph(&mut self) {
    let entry_module = Module::new(self.entry_path.clone(), true);
    let mut module_id_to_node_idx_map = Default::default();
    let mut ctx = AnalyseContext {
      graph: &mut self.graph,
      module_id_to_node_idx_map: &mut module_id_to_node_idx_map,
    };
    let entry = analyse_module(&mut ctx, entry_module, None, Rel::ReExportAll);
    self.entries.push(entry)
  }

  pub fn build(&mut self) {
    let globals = Globals::new();
    GLOBALS.set(&globals, || {
      self.generate_module_graph();

      self.sort_modules();

      self.include_statements();
    });

    println!("entry_modules {:?}", Dot::new(&self.graph))
  }

  fn include_statements(&mut self) {
    // TODO: tree-shaking
    self.graph.node_indices().into_iter().for_each(|idx| {
      if let DepNode::Mod(m) = &mut self.graph[idx] {
        m.include_all();
      }
    });
  }

  fn sort_modules(&mut self) {
    debug!("sort_modules");
    let mut dfs = DfsPostOrder::new(&self.graph, self.entries[0]);
    let mut ordered_modules = vec![];
    // FIXME: is this correct?
    while let Some(node) = dfs.next(&self.graph) {
      debug!("sort_modules dfs");
      
      ordered_modules.push(node);
    }
    debug!("sort_modules dfs end");
    self.ordered_modules = ordered_modules;

    self.ordered_modules.iter().for_each(|idx| {
      debug!("sort_modules 0");
      let dep = &self.graph[*idx];
      if let DepNode::Mod(module) = dep {
        debug!("sort_modules 1.1");
        module.scope.declared_symbols.values().for_each(|c| {
          debug!("sort_modules 1");
          let ctxt: Ctxt = c.clone().into();
          self.symbol_uf.unify_var_value(ctxt, ()).unwrap();
          debug!("sort_modules 2");
        });
      }
    });
    self.ordered_modules.iter().for_each(|idx| {
      let dep = &self.graph[*idx];
      if let DepNode::Mod(module) = dep {
        let rels = self
          .graph
          .edges_directed(*idx, petgraph::Direction::Outgoing);
        rels.for_each(|rel| match rel.weight() {
          Rel::Import(desc) => {
            let imported = &self.graph[rel.target()];
            if let DepNode::Mod(imported_module) = imported {
              let ctxt: Ctxt = module
                .scope
                .declared_symbols
                .get(&desc.name)
                .unwrap()
                .clone()
                .into();
              let local_ctxt: Ctxt = imported_module
                .scope
                .declared_symbols
                .get(&desc.local_name)
                .unwrap()
                .clone()
                .into();
              self.symbol_uf.union(ctxt, local_ctxt);
            }
          }
          Rel::ReExport(desc) => {
            let re_exported = &self.graph[rel.target()];
            if let DepNode::Mod(imported_module) = re_exported {
              let ctxt: Ctxt = module
                .scope
                .declared_symbols
                .get(&desc.name)
                .unwrap()
                .clone()
                .into();
              let local_ctxt: Ctxt = imported_module
                .scope
                .declared_symbols
                .get(&desc.local_name)
                .unwrap()
                .clone()
                .into();
              self.symbol_uf.union(ctxt, local_ctxt);
            }
          }
          _ => {}
        });
      }
    });
    debug!("sort_modules end");
  }
}

fn analyse_module(
  ctx: &mut AnalyseContext,
  mut module: Module,
  parent: Option<NodeIndex>,
  rel: Rel,
) -> NodeIndex {
  let source = std::fs::read_to_string(&module.id).unwrap();
  let scanner = module.set_source(source.clone());
  let module_id = module.id.clone();

  let node_idx;
  let has_seen;
  if let Some(idx) = ctx.module_id_to_node_idx_map.get(&module_id) {
    has_seen = true;
    node_idx = idx.clone();
  } else {
    has_seen = false;
    node_idx = ctx.graph.add_node(module.into());
    ctx
      .module_id_to_node_idx_map
      .insert(module_id.clone(), node_idx.clone());
  }

  if let Some(parent) = parent {
    ctx.graph.add_edge(parent, node_idx.clone(), rel);
  }

  if !has_seen {
    analyse_dep(ctx, scanner, &module_id, node_idx);
  }

  node_idx
}

struct AnalyseContext<'me> {
  pub graph: &'me mut DepGraph,
  pub module_id_to_node_idx_map: &'me mut HashMap<String, NodeIndex>,
}

fn analyse_external_module(
  ctx: &mut AnalyseContext,
  module: ExternalModule,
  parent: NodeIndex,
  rel: Rel,
) {
  let node_idx = ctx.graph.add_node(module.into());
  ctx.graph.add_edge(parent, node_idx, rel);
}

fn analyse_dep(ctx: &mut AnalyseContext, scanner: Scanner, module_id: &str, parent: NodeIndex) {
  scanner.imports.into_values().into_iter().for_each(|imp| {
    let unresolved_id = &imp.source;
    let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
    let mod_or_ext = resove_module_by_resolved_id(resolved_id);
    analyse_mod_or_ext(ctx, mod_or_ext, parent, Rel::Import(imp));
  });

  scanner.dynamic_imports.into_iter().for_each(|dyn_imp| {
    let unresolved_id = &dyn_imp.argument;
    let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
    let mod_or_ext = resove_module_by_resolved_id(resolved_id);
    analyse_mod_or_ext(ctx, mod_or_ext, parent, Rel::DynImport(dyn_imp));
  });

  scanner
    .re_exports
    .into_values()
    .into_iter()
    .for_each(|re_expr| {
      let unresolved_id = &re_expr.source;
      let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
      let mod_or_ext = resove_module_by_resolved_id(resolved_id);
      analyse_mod_or_ext(ctx, mod_or_ext, parent, Rel::ReExport(re_expr));
    });

  scanner.export_all_sources.into_iter().for_each(|source| {
    let unresolved_id = &source;
    let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
    let mod_or_ext = resove_module_by_resolved_id(resolved_id);
    analyse_mod_or_ext(ctx, mod_or_ext, parent, Rel::ReExportAll);
  });
}

fn analyse_mod_or_ext(ctx: &mut AnalyseContext, mod_or_ext: ModOrExt, parent: NodeIndex, rel: Rel) {
  match mod_or_ext {
    ModOrExt::Ext(ext) => analyse_external_module(ctx, ext, parent, rel),
    ModOrExt::Mod(m) => {
      analyse_module(ctx, m, Some(parent), rel);
    }
  }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ModOrExt {
  Mod(Module),
  Ext(ExternalModule),
}

fn resove_module_by_resolved_id(resolved: ResolvedId) -> ModOrExt {
  if resolved.external {
    ModOrExt::Ext(ExternalModule::new(resolved.id))
  } else {
    ModOrExt::Mod(Module::new(resolved.id, false))
  }
}
