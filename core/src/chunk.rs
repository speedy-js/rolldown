use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::{
    module::Module, renamer::Renamer, symbol_box::SymbolBox, utils::fold_export_decl_to_decl,
};

use rayon::prelude::*;
use swc_atoms::{js_word, JsWord};
use swc_common::{
    comments::{Comment, CommentKind, Comments, SingleThreadedComments},
    BytePos, SyntaxContext, DUMMY_SP,
};
use swc_ecma_ast::{EmptyStmt, EsVersion, ModuleItem, Stmt};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_visit::VisitMutWith;

pub struct Chunk {
    pub order_modules: Vec<String>,
    pub symbol_box: Arc<Mutex<SymbolBox>>,
    // SyntaxContext to Safe name mapping
    pub canonical_names: HashMap<SyntaxContext, JsWord>,
}

impl Chunk {
    pub fn new(
        order_modules: Vec<String>,
        symbol_box: Arc<Mutex<SymbolBox>>,
        canonical_names: HashMap<SyntaxContext, JsWord>,
    ) -> Self {
        Self {
            order_modules,
            symbol_box,
            canonical_names,
        }
    }

    pub fn deconflict(&mut self, modules: &mut HashMap<String, Module>) {
        let mut used_names = HashSet::new();
        let mut mark_to_name = HashMap::new();
        modules.values().for_each(|module| {
            module.delcared.iter().for_each(|(name, mark)| {
                let root_mark = self.symbol_box.lock().unwrap().find_root(*mark);
                if mark_to_name.contains_key(&root_mark) {
                } else {
                    let mut name = name.to_string();
                    let mut count = 0;
                    while used_names.contains(&name) {
                        name = format!("{}${}", name, count);
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
            module.ast.visit_mut_with(&mut renamer);
        });
        println!("mark_to_name {:#?}", mark_to_name);
    }

    pub fn render(&mut self, modules: &mut HashMap<String, Module>) -> String {
        modules.par_iter_mut().for_each(|(_key, module)| {
            let mut default_export_name = module
                .suggested_names
                .get(&js_word!("default"))
                .map(|s| s.to_string())
                .unwrap_or("default_".to_string());
            while module
                .delcared
                .contains_key(&default_export_name.clone().into())
            {
                default_export_name.push('_');
            }
            if module.exports.contains_key(&"default".into()) {
                module.delcared.insert(
                    default_export_name.clone().into(),
                    module.exports.get(&"default".into()).unwrap().clone(),
                );
            }
            println!("default_export_name {}", default_export_name);
            module.ast.body.iter_mut().for_each(|module_item| {
                fold_export_decl_to_decl(module_item, &default_export_name.clone().into());
            });
        });

        self.deconflict(modules);

        let mut output = Vec::new();
        let comments = SingleThreadedComments::default();

        // TODO: There's an problem in SWC, so we had to do following. See https://github.com/swc-project/swc/issues/3354.
        self.order_modules.iter().for_each(|idx| {
            let module = modules.get_mut(idx).unwrap();
            module.ast.body.insert(
                0,
                ModuleItem::Stmt(Stmt::Empty(EmptyStmt {
                    span: swc_common::Span {
                        lo: module.ast.span.lo,
                        hi: module.ast.span.hi,
                        ..Default::default()
                    },
                })),
            );
            comments.add_leading(
                module.ast.span.lo,
                Comment {
                    kind: CommentKind::Line,
                    span: DUMMY_SP,
                    text: format!(" {}", module.id),
                },
            );
            println!("module.ast.span.lo {:?}", module.ast.span.lo);
        });

        println!("comments {:#?}", comments);

        let mut emitter = swc_ecma_codegen::Emitter {
            cfg: swc_ecma_codegen::Config { minify: false },
            cm: crate::graph::SOURCE_MAP.clone(),
            comments: Some(&comments),
            wr: Box::new(JsWriter::with_target(
                crate::graph::SOURCE_MAP.clone(),
                "\n",
                &mut output,
                None,
                EsVersion::latest(),
            )),
        };

        self.order_modules.iter().for_each(|idx| {
            let module = modules.get(idx).unwrap();
            emitter.emit_module(&module.ast).unwrap();
        });

        String::from_utf8(output).unwrap()
    }
}
