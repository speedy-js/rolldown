use crate::{analyser::StatementAnalyser, ast::scope::Scope, module::analyse::parse_code};
use std::{collections::HashSet, sync::Arc};

use swc_ecma_visit::VisitWith;

#[cfg(test)]
mod declaration {
  use ahash::RandomState;

  use super::*;
  #[test]
  fn defines() {
    let cases = vec![
      (
        "basic.js",
        vec!["let_foo", "const_foo", "var_foo", "fn_foo", "class_foo"],
      ),
      ("export_default_class.js", vec!["foo"]),
      ("export_default_fn.js", vec!["foo"]),
    ];
    for (name, raw_right) in cases {
      let root_scope = Arc::new(Scope::new(None, vec![], false));
      let mut analyser = StatementAnalyser::new(root_scope);
      let source =
        std::fs::read_to_string("./src/statement/tests/fixtures/declaration/".to_owned() + name)
          .unwrap();
      let ast = parse_code(&source).unwrap();
      ast.visit_children_with(&mut analyser);
      let left = analyser.scope.defines.read().clone();
      let right = raw_right
        .into_iter()
        .map(|s| s.to_owned())
        .collect::<HashSet<String, RandomState>>();
      assert_eq!(left, right);
    }
  }
}
