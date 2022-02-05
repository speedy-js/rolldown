use dashmap::DashSet;
use smol_str::SmolStr;
use std::{
  cmp::Ordering,
  collections::{HashMap, HashSet},
  path::Path,
  sync::{Arc, Mutex},
};

use crate::{
  compiler::SOURCE_MAP,
  module::Module,
  renamer::Renamer,
  structs::{OutputChunk, RenderedChunk},
  symbol_box::SymbolBox,
  types::NormalizedOutputOptions,
};

use rayon::prelude::*;
use swc_atoms::JsWord;
use swc_common::{
  comments::{Comment, CommentKind, Comments, SingleThreadedComments},
  Mark, SyntaxContext, DUMMY_SP,
};
use swc_ecma_ast::{EmptyStmt, EsVersion, ModuleItem, Stmt};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_visit::VisitMutWith;

pub struct Chunk {
  pub id: SmolStr,
  pub order_modules: Vec<SmolStr>,
  pub symbol_box: Arc<Mutex<SymbolBox>>,
  pub entries: DashSet<SmolStr>,
}

impl Chunk {
  pub fn new(
    order_modules: Vec<SmolStr>,
    symbol_box: Arc<Mutex<SymbolBox>>,
    entries: DashSet<SmolStr>,
  ) -> Self {
    Self {
      id: Default::default(),
      order_modules,
      symbol_box,
      entries,
    }
  }

  pub fn deconflict(&mut self, modules: &mut HashMap<SmolStr, Module>) {
    let mut used_names = HashSet::new();
    let mut mark_to_name = HashMap::new();
    let mut entry_first_modules = self
      .order_modules
      .iter()
      .map(|id| modules.get(id).unwrap())
      .collect::<Vec<_>>();
    entry_first_modules.sort_by(|a, b| {
      if a.is_user_defined_entry_point && !b.is_user_defined_entry_point {
        Ordering::Less
      } else if b.is_user_defined_entry_point && !a.is_user_defined_entry_point {
        Ordering::Greater
      } else {
        Ordering::Equal
      }
    });
    println!(
      "entry_first_modules {:#?}",
      entry_first_modules
        .iter()
        .map(|m| &m.id)
        .collect::<Vec<_>>()
    );
    entry_first_modules.into_iter().for_each(|module| {
      let mut declared_symbols = module.declared_symbols.iter().collect::<Vec<_>>();
      // declared_symbols.sort_by(|a, b| {
      //   a.0.cmp(b.0)
      // });
      // println!("declared_symbols {:#?}", declared_symbols.iter().map(|s| s.0.as_ref()).collect::<Vec<_>>());
      declared_symbols.into_iter().for_each(|(name, mark)| {
        let root_mark = self.symbol_box.lock().unwrap().find_root(*mark);
        if mark_to_name.contains_key(&root_mark) {
        } else {
          let original_name = name.to_string();
          let mut name = name.to_string();
          let mut count = 0;
          while used_names.contains(&name) {
            name = format!("{}${}", original_name, count);
            count += 1;
          }
          mark_to_name.insert(root_mark, name.clone());
          used_names.insert(name);
        }
      });
    });

    modules.par_iter_mut().for_each(|(_, module)| {
      let mut renamer = Renamer {
        mark_to_names: &mark_to_name,
        symbox_box: self.symbol_box.clone(),
      };
      module.statements.iter_mut().for_each(|stmt| {
        stmt.node.visit_mut_with(&mut renamer);
      });
      module.appended_statments.iter_mut().for_each(|stmt| {
        stmt.node.visit_mut_with(&mut renamer);
      });
    });

    log::debug!("mark_to_name {:#?}", mark_to_name);
  }

  pub fn render(&mut self, modules: &mut HashMap<SmolStr, Module>) -> String {
    // let modules = modules.par_iter_mut().map(|(key, module)| (key.clone(), module))
    modules.par_iter_mut().for_each(|(_key, module)| {
      module.trim_exports();
      if module.is_user_defined_entry_point {
        module.generate_exports();
      }
    });

    self.deconflict(modules);

    let mut output = Vec::new();
    let comments = SingleThreadedComments::default();

    let mut emitter = swc_ecma_codegen::Emitter {
      cfg: swc_ecma_codegen::Config { minify: false },
      cm: SOURCE_MAP.clone(),
      comments: Some(&comments),
      wr: Box::new(JsWriter::with_target(
        SOURCE_MAP.clone(),
        "\n",
        &mut output,
        None,
        EsVersion::latest(),
      )),
    };

    self.order_modules.iter().for_each(|idx| {
      if let Some(module) = modules.get(idx) {
        module.render(&mut emitter);
      }
    });

    String::from_utf8(output).unwrap()
  }

  pub fn render_new(
    &mut self,
    options: &NormalizedOutputOptions,
    modules: &mut HashMap<SmolStr, Module>,
  ) -> RenderedChunk {
    assert!(!self.id.is_empty());

    // let modules = modules.par_iter_mut().map(|(key, module)| (key.clone(), module))
    modules.par_iter_mut().for_each(|(_key, module)| {
      module.trim_exports();
      if module.is_user_defined_entry_point {
        module.generate_exports();
      }
    });

    self.deconflict(modules);

    let mut output = Vec::new();
    let comments = SingleThreadedComments::default();

    let mut emitter = swc_ecma_codegen::Emitter {
      cfg: swc_ecma_codegen::Config {
        minify: options.minify,
      },
      cm: SOURCE_MAP.clone(),
      comments: Some(&comments),
      wr: Box::new(JsWriter::with_target(
        SOURCE_MAP.clone(),
        "\n",
        &mut output,
        None,
        EsVersion::latest(),
      )),
    };

    self.order_modules.iter().for_each(|idx| {
      if let Some(module) = modules.get(idx) {
        module.render(&mut emitter);
      }
    });

    RenderedChunk {
      code: String::from_utf8(output).unwrap(),
      file_name: self.id.clone().into(),
    }
  }

  pub fn get_chunk_info_with_file_names(&self) -> OutputChunk {
    OutputChunk {
      code: "".to_string(),
      file_name: self.id.clone().into(),
    }
  }

  #[inline]
  pub fn get_fallback_chunk_name(&self) -> &str {
    // if (this.manualChunkAlias) {
    // 	return this.manualChunkAlias;
    // }
    // if (this.dynamicName) {
    // 	return this.dynamicName;
    // }
    // if (this.fileName) {
    // 	return getAliasName(this.fileName);
    // }
    get_alias_name(self.order_modules.last().unwrap())
    // return getAliasName(this.orderedModules[this.orderedModules.length - 1].id);
  }

  #[inline]
  pub fn get_chunk_name(&self) -> &str {
    return self.get_fallback_chunk_name();
  }

  pub fn generate_id(&self, options: &NormalizedOutputOptions) -> SmolStr {
    let pattern = &options.entry_file_names;
    pattern.replace("[name]", self.get_chunk_name()).into()
  }
}

#[inline]
fn get_alias_name(id: &str) -> &str {
  let p = Path::new(id);
  // +1 to include `.`
  let ext_len = p.extension().map_or(0, |s| s.to_string_lossy().len() + 1);
  let file_name = p.file_name().unwrap().to_str().unwrap();
  &file_name[0..file_name.len() - ext_len]
}
