use swc_ecma_ast::{Expr, ModuleItem, PatOrExpr, Stmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideEffect {
  Todo,
  FnCall,
  VisitProp,
  VisitThis,
  NonTopLevel,
  VisitGlobalVar,
}

fn detect_side_effect_of_expr(expr: &Expr) -> Option<SideEffect> {
  match expr {
    Expr::This(ThisExpr) => Some(SideEffect::VisitThis),
    Expr::Array(ArrayLit) => ArrayLit.elems.iter().find_map(|expr_or_spread| {
      expr_or_spread
        .as_ref()
        .and_then(|exp| detect_side_effect_of_expr(exp.expr.as_ref()))
    }),
    Expr::Object(ObjectLit) => Some(SideEffect::Todo),

    Expr::Fn(FnExpr) => None,

    Expr::Unary(UnaryExpr) => Some(SideEffect::Todo),

    /// `++v`, `--v`, `v++`, `v--`
    Expr::Update(UpdateExpr) => detect_side_effect_of_expr(UpdateExpr.arg.as_ref()),

    Expr::Bin(BinExpr) => [BinExpr.left.as_ref(), BinExpr.right.as_ref()]
      .into_iter()
      .find_map(|expr| detect_side_effect_of_expr(expr)),

    Expr::Assign(AssignExpr) => match &AssignExpr.left {
      PatOrExpr::Expr(expr) => detect_side_effect_of_expr(expr.as_ref()),
      PatOrExpr::Pat(_) => Some(SideEffect::Todo),
    },
    Expr::Member(MemberExpr) => Some(SideEffect::VisitProp),
    Expr::SuperProp(SuperPropExpr) => Some(SideEffect::VisitProp),

    // true ? 'a' : 'b'
    Expr::Cond(CondExpr) => [
      CondExpr.test.as_ref(),
      CondExpr.cons.as_ref(),
      CondExpr.alt.as_ref(),
    ]
    .into_iter()
    .find_map(|expr| detect_side_effect_of_expr(expr)),

    Expr::Call(CallExpr) => Some(SideEffect::FnCall),
    // `new Cat()`
    Expr::New(NewExpr) => Some(SideEffect::FnCall),

    Expr::Seq(SeqExpr) => SeqExpr
      .exprs
      .iter()
      .find_map(|expr| detect_side_effect_of_expr(expr)),

    Expr::Ident(Ident) => None,

    Expr::Lit(Lit) => None,

    Expr::Tpl(Tpl) => Tpl
      .exprs
      .iter()
      .find_map(|expr| detect_side_effect_of_expr(expr)),

    Expr::TaggedTpl(TaggedTpl) => Some(SideEffect::FnCall),

    Expr::Arrow(ArrowExpr) => None,

    Expr::Class(ClassExpr) => None,

    Expr::Yield(YieldExpr) => Some(SideEffect::Todo),

    Expr::MetaProp(MetaPropExpr) => Some(SideEffect::Todo),

    Expr::Await(AwaitExpr) => Some(SideEffect::Todo),

    Expr::Paren(ParenExpr) => detect_side_effect_of_expr(ParenExpr.expr.as_ref()),

    Expr::JSXMember(JSXMemberExpr) => Some(SideEffect::Todo),

    Expr::JSXNamespacedName(JSXNamespacedName) => Some(SideEffect::Todo),

    Expr::JSXEmpty(JSXEmptyExpr) => Some(SideEffect::Todo),

    Expr::JSXElement(s) => Some(SideEffect::Todo),

    Expr::JSXFragment(JSXFragment) => Some(SideEffect::Todo),

    Expr::TsTypeAssertion(TsTypeAssertion) => None,

    Expr::TsConstAssertion(TsConstAssertion) => None,

    Expr::TsNonNull(TsNonNullExpr) => None,

    Expr::TsAs(TsAsExpr) => None,

    Expr::PrivateName(PrivateName) => Some(SideEffect::Todo),

    Expr::OptChain(OptChainExpr) => detect_side_effect_of_expr(OptChainExpr.expr.as_ref()),

    Expr::Invalid(Invalid) => Some(SideEffect::Todo),
  }
}

// ESM environment
pub fn detect_side_effect(item: &ModuleItem) -> Option<SideEffect> {
  match item {
    ModuleItem::ModuleDecl(_) => None,
    ModuleItem::Stmt(stmt) => match stmt {
      // `{ }`
      Stmt::Block(BlockStmt) => Some(SideEffect::NonTopLevel),
      // `;`
      Stmt::Empty(EmptyStmt) => None,
      // `debugger`
      Stmt::Debugger(DebuggerStmt) => Some(SideEffect::Todo),
      // `with(foo) {}`
      Stmt::With(WithStmt) => Some(SideEffect::Todo),
      // `return`
      Stmt::Return(ReturnStmt) => Some(SideEffect::Todo),
      // s
      Stmt::Labeled(LabeledStmt) => Some(SideEffect::Todo),

      Stmt::Break(BreakStmt) => Some(SideEffect::Todo),

      Stmt::Continue(ContinueStmt) => Some(SideEffect::Todo),

      Stmt::If(IfStmt) => Some(SideEffect::Todo),

      Stmt::Switch(SwitchStmt) => Some(SideEffect::Todo),

      Stmt::Throw(ThrowStmt) => Some(SideEffect::Todo),
      Stmt::Try(TryStmt) => Some(SideEffect::Todo),

      Stmt::While(WhileStmt) => Some(SideEffect::Todo),

      Stmt::DoWhile(DoWhileStmt) => Some(SideEffect::Todo),

      Stmt::For(ForStmt) => Some(SideEffect::Todo),

      Stmt::ForIn(ForInStmt) => Some(SideEffect::Todo),

      Stmt::ForOf(ForOfStmt) => Some(SideEffect::Todo),
      Stmt::Decl(Decl) => None,
      Stmt::Expr(ExprStmt) => detect_side_effect_of_expr(ExprStmt.expr.as_ref()),
    },
  }
}
