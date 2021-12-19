use std::{cell::RefCell, collections::{HashMap, HashSet}, fmt::Debug, hash::Hash, rc::Rc};

use swc_ecma_ast::*;
use swc_ecma_visit::{VisitAllWith, VisitWith};

use crate::{
    graph::DepNode,
    statement::analyse::scope::{Scope, ScopeKind},
};

use self::analyse::relationship_analyzer::ExportDesc;

pub mod analyse;

#[derive(Clone, PartialEq, Eq)]
pub struct Statement {
    pub node: ModuleItem,
    pub is_import_declaration: bool,
    pub is_export_declaration: bool,
    pub defines: HashSet<String>,
    pub depends_on: HashSet<String>,
    pub exports: HashMap<String, ExportDesc>,
}

impl Statement {
    pub fn new(node: ModuleItem) -> Self {
        let is_import_declaration = matches!(&node, ModuleItem::ModuleDecl(ModuleDecl::Import(_)));
        let is_export_declaration = if let ModuleItem::ModuleDecl(module_decl) = &node {
            matches!(
                module_decl,
                ModuleDecl::ExportAll(_)
                    | ModuleDecl::ExportDecl(_)
                    | ModuleDecl::ExportDefaultDecl(_)
                    | ModuleDecl::ExportDefaultExpr(_)
                    | ModuleDecl::ExportNamed(_)
            )
        } else {
            false
        };
        let scope = Rc::new(RefCell::new(Scope {
            kind: ScopeKind::Mod,
            ..Default::default()
        }));
        let mut scope_analyser = analyse::scope_analyzer::ScopeAnalyser::new(scope.clone());
        node.visit_children_with(&mut scope_analyser);

        let defines = scope.as_ref().borrow().defines.clone();
        Statement {
            node,
            // module,
            defines,
            depends_on: scope_analyser.depends_on,
            is_import_declaration,
            is_export_declaration,
            exports: Default::default(),
            // modifies: HashSet::default(),
            // scope,
        }
    }

    // fn analyse(&mut self) {
    //   let mut statement_analyser = analyser::StatementAnalyser::new(self.scope.clone());
    //   self.node.visit_children_with(&mut statement_analyser);
    //   self.defines = statement_analyser.scope.borrow().defines.clone();
    //   self.depends_on = statement_analyser.depends_on.clone();
    //   // consider all depends as modifies for now, even they are only read-only.
    //   // self.modifies = statement_analyser.depends_on;
    //   // debug!("defines: {:?}, scope defines: {:?}", self.defines, self.scope.defines);
    // }

    // pub fn expand(self: &Arc<Self>, module: &Module, graph: &graph::Graph) -> Vec<Arc<Self>> {
    //   if self.is_included.swap(true, Ordering::SeqCst) {
    //     return vec![];
    //   }

    //   let mut result = vec![];

    //   log::debug!(
    //     "expand statement depends on {:?} in module {}",
    //     self.depends_on,
    //     module.id
    //   );

    //   // We have a statement, and it hasn't been included yet. First, include
    //   // the statements it depends on
    //   self.depends_on.iter().for_each(|name| {
    //     if !self.defines.contains(name) {
    //       // The name doesn't belong to this statement, we need to search it in module.
    //       result.append(&mut module.define(name, graph));
    //     }
    //   });

    //   // include the statement itself
    //   result.push(self.clone());

    //   // then include any statements that could modify the
    //   self.defines.iter().for_each(|name| {
    //     if let Some(modifications) = module.modifications.get(name) {
    //       modifications.iter().for_each(|statement| {
    //         result.append(&mut statement.expand(module, graph));
    //       });
    //     }
    //   });

    //   result
    // }
}

impl Into<DepNode> for Statement {
    fn into(self) -> DepNode {
        DepNode::Stmt(self)
    }
}

impl Hash for Statement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state)
    }
}

impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Statement")
            .field("defines", &self.defines)
            .field("depends_on", &self.depends_on)
            .field("exports", &self.exports)
            .finish()
    }
}

// pub fn parse_to_statements(
//   ast: swc_ecma_ast::Module,
//   module: &Shared<Module>,
// ) -> (RelationshipAnalyzer, Vec<Statement>) {
//   let mut m = RelationshipAnalyzer::new(module.clone());
//   ast.visit_all_children_with(&mut m);
//   let statements = ast.body.into_iter().map(|node| Statement::new(node, module.clone())).collect();
//   (m, statements)
// }
