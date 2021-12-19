use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

use rayon::{prelude::*, string};

use once_cell::sync::Lazy;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_visit::{VisitAllWith, VisitWith};

use crate::external_module::ExternalModule;
use crate::module::Module;
use crate::statement::analyse::relationship_analyzer::{
    parse_file, DynImportDesc, ImportDesc, ReExportDesc, RelationshipAnalyzer,
};
use crate::statement::{self, Statement};
use crate::types::ResolvedId;
use crate::utils::resolve_id::resolve_id;

pub(crate) static SOURCE_MAP: Lazy<Lrc<SourceMap>> = Lazy::new(Default::default);

#[derive(Debug, Hash, PartialEq, Eq, Clone)]

pub enum DepNode {
    Mod(Module),
    Stmt(Statement),
    Ext(ExternalModule),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum Rel {
    Import(ImportDesc),
    DynImport(DynImportDesc),
    ReExport(ReExportDesc),
    Contain,
}

type JustGraph = Graph<DepNode, Rel>;

#[non_exhaustive]
pub struct GraphContainer {
    pub entry: String,
    pub graph: RwLock<JustGraph>,
}

impl GraphContainer {
    pub fn new(entry: String) -> Self {
        env_logger::init();

        let graph = Graph::default();

        let graph = Self {
            entry,
            // entry_modules: vec![],
            // module_loader: module_container,
            // plugin_driver,
            graph: RwLock::new(graph),
            // modules: vec![],
            // external_modules: vec![],
        };

        graph
    }

    // build dependency graph via entry modules.
    pub fn generate_module_graph(&mut self) {
        let mut entry_module = Module::new(self.entry.clone());
        let mut module_id_to_node_idx_map = Default::default();
        let mut ctx = AnalyseContext {
            graph: &mut self.graph.write().unwrap(),
            module_id_to_node_idx_map: &mut module_id_to_node_idx_map,
        };
        analyse_module(&mut ctx, entry_module, None, Rel::Contain)
    }

    pub fn build(&mut self) {
        self.generate_module_graph();

        // self.sort_modules();

        // self.include_statements();
    }
}

// pub fn analyse_dep(
//     graph: &RwLock<JustGraph>,
//     module_id: String,
//     module_node_idx: NodeIndex,
//     static_dep: HashSet<String>,
//     dyn_dep: HashSet<DynImportDesc>,
//     seen: &RwLock<HashMap<String, NodeIndex>>,
// ) -> NodeIndex {
//     static_dep
//         .par_iter()
//         .map(|source| resolve_id(source, Some(&module_id), false))
//         .map(resove_module_by_resolved_id)
//         .for_each(|imported| match imported {
//             ModOrExt::Ext(ext) => {
//                 let mut graph = graph.write().unwrap();
//                 let imported = graph.add_node(ext.into());
//                 graph.add_edge(module_node_idx, imported, Rel::Import);
//             }
//             ModOrExt::Mod(mut imported) => {
//                 let imported_id = imported.id.clone();
//                 if let Some(dep_idx) = seen.read().unwrap().get(&imported_id) {
//                     graph
//                         .write()
//                         .unwrap()
//                         .add_edge(module_node_idx, dep_idx.clone(), Rel::Import);
//                     return;
//                 }

//                 let source = std::fs::read_to_string(&imported.id).unwrap();
//                 imported.set_source(source);
//                 let static_dep = imported.sources.clone();
//                 let module_id = imported.id.clone();
//                 let dyn_dep = imported.dynamic_imports.clone();
//                 let dep_idx = graph.write().unwrap().add_node(imported.into());
//                 graph
//                     .write()
//                     .unwrap()
//                     .add_edge(module_node_idx, dep_idx.clone(), Rel::Import);
//                 graph
//                     .write()
//                     .unwrap()
//                     .add_edge(module_node_idx, dep_idx.clone(), Rel::Import);
//                 seen.write().unwrap().insert(imported_id, dep_idx.clone());
//                 analyse_dep(graph, module_id, dep_idx, static_dep, dyn_dep, seen);
//             }
//         });
//     dyn_dep
//         .par_iter()
//         .map(|dyn_imp| resolve_id(&dyn_imp.argument, Some(&module_id), false))
//         .map(resove_module_by_resolved_id)
//         .for_each(|imported| match imported {
//             ModOrExt::Ext(ext) => {
//                 let mut graph = graph.write().unwrap();
//                 let imported = graph.add_node(ext.into());
//                 graph.add_edge(module_node_idx, imported, Rel::DynImport);
//             }
//             ModOrExt::Mod(mut imported) => {
//                 let imported_id = imported.id.clone();
//                 if let Some(dep_idx) = seen.read().unwrap().get(&imported_id) {
//                     graph
//                         .write()
//                         .unwrap()
//                         .add_edge(module_node_idx, dep_idx.clone(), Rel::DynImport);
//                     return;
//                 }

//                 let source = std::fs::read_to_string(&imported.id).unwrap();
//                 imported.set_source(source);
//                 let static_dep = imported.sources.clone();
//                 let module_id = imported.id.clone();
//                 let dyn_dep = imported.dynamic_imports.clone();
//                 let dep_idx = graph.write().unwrap().add_node(imported.into());
//                 seen.write().unwrap().insert(imported_id, dep_idx.clone());
//                 analyse_dep(graph, module_id, dep_idx, static_dep, dyn_dep, seen);
//             }
//         });
//     module_node_idx
// }

fn analyse_module(ctx: &mut AnalyseContext, module: Module, parent: Option<NodeIndex>, rel: Rel) {
    let source = std::fs::read_to_string(&module.id).unwrap();
    let module_id = module.id.clone();
    let node_idx = ctx
        .module_id_to_node_idx_map
        .entry(module_id.clone())
        .or_insert_with(|| ctx.graph.add_node(module.into())).clone();

    if let Some(parent) = parent {
        ctx.graph.add_edge(parent, node_idx.clone(), rel);
    }
    
    parse_to_statements(source, module_id.clone())
        .into_iter()
        .for_each(|stmt| analyse_statement(ctx, stmt, &module_id, node_idx.clone()));
}

struct AnalyseContext<'me> {
    pub graph: &'me mut JustGraph,
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

fn analyse_statement(
    ctx: &mut AnalyseContext,
    mut stmt: Statement,
    module_id: &str,
    parent: NodeIndex,
) {
    let mut relationship_analyzer = RelationshipAnalyzer::new();
    stmt.node
        .visit_all_children_with(&mut relationship_analyzer);
      stmt.exports = relationship_analyzer.exports;
    let idx = ctx.graph.add_node(stmt.into());
    ctx.graph.add_edge(parent, idx, Rel::Contain);

    relationship_analyzer
        .imports
        .into_values()
        .into_iter()
        .for_each(|imp| {
            let unresolved_id = &imp.source;
            let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
            let mod_or_ext = resove_module_by_resolved_id(resolved_id);
            analyse_mod_or_ext(ctx, mod_or_ext, idx, Rel::Import(imp));
        });

    relationship_analyzer
        .dynamic_imports
        .into_iter()
        .for_each(|dyn_imp| {
            let unresolved_id = &dyn_imp.argument;
            let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
            let mod_or_ext = resove_module_by_resolved_id(resolved_id);
            analyse_mod_or_ext(ctx, mod_or_ext, idx, Rel::DynImport(dyn_imp));
        });

    relationship_analyzer
        .re_exports
        .into_values()
        .into_iter()
        .for_each(|re_expr| {
            let unresolved_id = &re_expr.source;
            let resolved_id = resolve_id(unresolved_id, Some(module_id), false);
            let mod_or_ext = resove_module_by_resolved_id(resolved_id);
            analyse_mod_or_ext(ctx, mod_or_ext, idx, Rel::ReExport(re_expr));
        });
}

fn analyse_mod_or_ext(ctx: &mut AnalyseContext, mod_or_ext: ModOrExt, parent: NodeIndex, rel: Rel) {
    match mod_or_ext {
        ModOrExt::Ext(ext) => analyse_external_module(ctx, ext, parent, rel),
        ModOrExt::Mod(m) => analyse_module(ctx, m, Some(parent), rel),
    }
}

fn parse_to_statements(source: String, id: String) -> Vec<Statement> {
    let ast = parse_file(source, id, &SOURCE_MAP).unwrap();
    ast.body
        .into_iter()
        .map(|node| Statement::new(node))
        .collect()
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
        ModOrExt::Mod(Module::new(resolved.id))
    }
}
