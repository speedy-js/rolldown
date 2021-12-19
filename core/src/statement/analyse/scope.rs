use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeKind {
    Block,
    Fn,
    Mod,
    For,
    Catch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub depth: usize,
    pub defines: HashSet<String>,
    pub kind: ScopeKind,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            parent: None,
            depth: 0,
            defines: HashSet::default(),
            kind: ScopeKind::Block,
        }
    }
}

impl Scope {
    pub fn new(parent: Option<Rc<RefCell<Scope>>>, params: Vec<String>, kind: ScopeKind) -> Scope {
        let mut defines = HashSet::default();
        params.into_iter().for_each(|p| {
            defines.insert(p);
        });
        let depth = parent.as_ref().map_or(0, |p| p.borrow().depth + 1);
        Scope {
            depth,
            parent,
            defines,
            kind,
        }
    }

    pub fn get_is_block_scope(&self) -> bool {
        matches!(
            self.kind,
            ScopeKind::Block | ScopeKind::For | ScopeKind::Catch
        )
    }

    pub fn add_declaration(&mut self, name: &str, is_block_declaration: bool) {
        if !is_block_declaration && self.get_is_block_scope() {
            self.parent
                .as_ref()
                .unwrap_or_else(|| panic!("parent not found for name {:?}", name))
                .as_ref()
                .borrow_mut()
                .add_declaration(name, is_block_declaration)
        } else {
            self.defines.insert(name.to_owned());
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        if self.defines.contains(name) {
            true
        } else if let Some(parent) = self.parent.as_ref() {
            parent.as_ref().borrow().contains(name)
        } else {
            false
        }
    }

    // pub fn find_defining_scope(self: &Rc<Self>, name: &str) -> Option<Rc<Self>> {
    //   if self.defines.contains(name) {
    //     Some(self.clone())
    //   } else if let Some(parent) = self.parent.as_ref() {
    //     parent.as_ref().borrow().find_defining_scope(name)
    //   } else {
    //     None
    //   }
    // }
}
