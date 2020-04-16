use rustc_middle::ty::BorrowKind;
use rustc_typeck::expr_use_visitor::ConsumeMode;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExpressionUseKind {
    Copy,
    Move,
    ImmBorrow,
    UniqueImmBorrow,
    MutBorrow,
    Mut
}
impl ExpressionUseKind {
    pub fn from_consume_mode(cm: ConsumeMode) -> Self {
        match cm {
            ConsumeMode::Copy => ExpressionUseKind::Copy,
            ConsumeMode::Move => ExpressionUseKind::Move
        }
    }
    pub fn from_borrow_kind(bk: BorrowKind) -> Self {
        match bk {
            BorrowKind::ImmBorrow => ExpressionUseKind::ImmBorrow,
            BorrowKind::UniqueImmBorrow => ExpressionUseKind::UniqueImmBorrow,
            BorrowKind::MutBorrow => ExpressionUseKind::MutBorrow,
        }
    }
    pub fn is_borrow(&self) -> bool {
        match self {
            ExpressionUseKind::MutBorrow => true,
            ExpressionUseKind::ImmBorrow => true,
            ExpressionUseKind::UniqueImmBorrow => true,
            _ => false
        }
    }
    pub fn is_mutated(&self) -> bool {
        match self {
            ExpressionUseKind::MutBorrow => true,
            ExpressionUseKind::Mut => true,
            _ => false
        }
    }
}
