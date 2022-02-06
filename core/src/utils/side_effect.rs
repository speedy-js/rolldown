use swc_ecma_ast::{Expr, ModuleItem, PatOrExpr, Stmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffect {
  Todo,
  FnCall,
  VisitProp,
  VisitThis,
  NonTopLevel,
  VisitGlobalVar,
  ModuleDecl,
}

fn detect_side_effect_of_expr(expr: &Expr) -> Option<SideEffect> {
  match expr {
    Expr::This(_ThisExpr) => Some(SideEffect::VisitThis),
    Expr::Array(ArrayLit) => ArrayLit.elems.iter().find_map(|expr_or_spread| {
      expr_or_spread
        .as_ref()
        .and_then(|exp| detect_side_effect_of_expr(exp.expr.as_ref()))
    }),
    Expr::Object(_ObjectLit) => Some(SideEffect::Todo),

    Expr::Fn(_FnExpr) => None,

    Expr::Unary(_UnaryExpr) => Some(SideEffect::Todo),

    /// `++v`, `--v`, `v++`, `v--`
    Expr::Update(UpdateExpr) => detect_side_effect_of_expr(UpdateExpr.arg.as_ref()),

    Expr::Bin(BinExpr) => [BinExpr.left.as_ref(), BinExpr.right.as_ref()]
      .into_iter()
      .find_map(|expr| detect_side_effect_of_expr(expr)),

    Expr::Assign(AssignExpr) => match &AssignExpr.left {
      PatOrExpr::Expr(expr) => detect_side_effect_of_expr(expr.as_ref()),
      PatOrExpr::Pat(_) => Some(SideEffect::Todo),
    },
    Expr::Member(_MemberExpr) => Some(SideEffect::VisitProp),
    Expr::SuperProp(_SuperPropExpr) => Some(SideEffect::VisitProp),

    // true ? 'a' : 'b'
    Expr::Cond(CondExpr) => [
      CondExpr.test.as_ref(),
      CondExpr.cons.as_ref(),
      CondExpr.alt.as_ref(),
    ]
    .into_iter()
    .find_map(|expr| detect_side_effect_of_expr(expr)),

    Expr::Call(_CallExpr) => Some(SideEffect::FnCall),
    // `new Cat()`
    Expr::New(_NewExpr) => Some(SideEffect::FnCall),

    Expr::Seq(SeqExpr) => SeqExpr
      .exprs
      .iter()
      .find_map(|expr| detect_side_effect_of_expr(expr)),

    Expr::Ident(_Ident) => None,

    Expr::Lit(_Lit) => None,

    Expr::Tpl(Tpl) => Tpl
      .exprs
      .iter()
      .find_map(|expr| detect_side_effect_of_expr(expr)),

    Expr::TaggedTpl(_TaggedTpl) => Some(SideEffect::FnCall),

    Expr::Arrow(_ArrowExpr) => None,

    Expr::Class(_ClassExpr) => None,

    Expr::Yield(_YieldExpr) => Some(SideEffect::Todo),

    Expr::MetaProp(_MetaPropExpr) => Some(SideEffect::Todo),

    Expr::Await(_AwaitExpr) => Some(SideEffect::Todo),

    Expr::Paren(ParenExpr) => detect_side_effect_of_expr(ParenExpr.expr.as_ref()),

    Expr::JSXMember(_JSXMemberExpr) => Some(SideEffect::Todo),

    Expr::JSXNamespacedName(_JSXNamespacedName) => Some(SideEffect::Todo),

    Expr::JSXEmpty(_JSXEmptyExpr) => Some(SideEffect::Todo),

    Expr::JSXElement(_s) => Some(SideEffect::Todo),

    Expr::JSXFragment(_JSXFragment) => Some(SideEffect::Todo),

    Expr::TsTypeAssertion(_TsTypeAssertion) => None,

    Expr::TsConstAssertion(_TsConstAssertion) => None,

    Expr::TsNonNull(_TsNonNullExpr) => None,

    Expr::TsAs(_TsAsExpr) => None,

    Expr::PrivateName(_PrivateName) => Some(SideEffect::Todo),

    Expr::OptChain(OptChainExpr) => detect_side_effect_of_expr(OptChainExpr.expr.as_ref()),

    Expr::Invalid(_Invalid) => Some(SideEffect::Todo),
  }
}

// ESM environment
pub fn detect_side_effect(item: &ModuleItem) -> Option<SideEffect> {
  match item {
    ModuleItem::ModuleDecl(_) => Some(SideEffect::ModuleDecl),
    ModuleItem::Stmt(stmt) => match stmt {
      // `{ }`
      Stmt::Block(_BlockStmt) => Some(SideEffect::NonTopLevel),
      // `;`
      Stmt::Empty(_EmptyStmt) => None,
      // `debugger`
      Stmt::Debugger(_DebuggerStmt) => Some(SideEffect::Todo),
      // `with(foo) {}`
      Stmt::With(_WithStmt) => Some(SideEffect::Todo),
      // `return`
      Stmt::Return(_ReturnStmt) => Some(SideEffect::Todo),
      // s
      Stmt::Labeled(_LabeledStmt) => Some(SideEffect::Todo),

      Stmt::Break(_BreakStmt) => Some(SideEffect::Todo),

      Stmt::Continue(_ContinueStmt) => Some(SideEffect::Todo),

      Stmt::If(_IfStmt) => Some(SideEffect::Todo),

      Stmt::Switch(_SwitchStmt) => Some(SideEffect::Todo),

      Stmt::Throw(_ThrowStmt) => Some(SideEffect::Todo),
      Stmt::Try(_TryStmt) => Some(SideEffect::Todo),

      Stmt::While(_WhileStmt) => Some(SideEffect::Todo),

      Stmt::DoWhile(_DoWhileStmt) => Some(SideEffect::Todo),

      Stmt::For(_ForStmt) => Some(SideEffect::Todo),

      Stmt::ForIn(_ForInStmt) => Some(SideEffect::Todo),

      Stmt::ForOf(_ForOfStmt) => Some(SideEffect::Todo),
      Stmt::Decl(_Decl) => None,
      Stmt::Expr(ExprStmt) => detect_side_effect_of_expr(ExprStmt.expr.as_ref()),
    },
  }
}
