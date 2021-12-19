use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    graph::{DepNode, SOURCE_MAP},
};

#[derive(Clone, PartialEq, Eq)]
pub struct Module {
    // pub original_code: Option<String>,
    // pub statements: Vec<Statement>,
    // pub is_entry: bool,
    pub id: String,
    // pub imports: HashMap<String, ImportDesc>,
    // pub exports: HashMap<String, ExportDesc>,
    // pub dynamic_imports: HashSet<DynImportDesc>,
    // Named re_export. sush as `export { foo } from ...` or `export * as foo from '...'`
    // pub re_exports: HashMap<String, ReExportDesc>,
    // Just re-export. sush as `export * from ...`
    // pub export_all_sources: HashSet<String>,
    // pub exports_all: HashMap<String, String>,
    // id of imported modules
    // pub sources: HashSet<String>,
    // pub resolved_ids: HashMap<String, ResolvedId>,
    // id of importers
    // pub importers: HashSet<String>,
    // pub export_all_modules: Vec<ModOrExt>,
    // pub dependencies: HashSet<ModOrExt>,
    // pub dynamic_dependencies: HashSet<ModOrExt>,
    // pub dynamic_importers: HashSet<String>,
    // pub cycles: HashSet<String>,
    // pub exec_index: usize,
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Module").field(&self.id).finish()
    }
}

impl Into<DepNode> for Module {
    fn into(self) -> DepNode {
        DepNode::Mod(self)
    }
}

impl Module {
    pub fn new(id: String) -> Self {
        Module {
            // original_code: None,
            id,
            // is_entry,
            // imports: HashMap::default(),
            // exports: HashMap::default(),
            // re_exports: HashMap::default(),
            // dynamic_imports: Default::default(),
            // export_all_sources: HashSet::default(),
            // exports_all: HashMap::default(),
            // sources: HashSet::default(),
            // resolved_ids: HashMap::default(),
            // dependencies: HashSet::default(),
            // dynamic_dependencies: HashSet::default(),
            // importers: HashSet::default(),
            // dynamic_importers: HashSet::default(),
            // export_all_modules: Vec::default(),
            // cycles: HashSet::default(),
            // exec_index: usize::MAX,
            // statements: vec![],
            // definitions,
            // modifications,
            // defined: RwLock::new(HashSet::default()),
            // suggested_names: RwLock::new(HashMap::default()),
        }
    }
}

impl Hash for Module {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(&self.id.as_bytes());
    }
}