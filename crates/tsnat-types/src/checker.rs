use crate::ty::{TypeArena, TypeId, TYPE_NUMBER, TYPE_STRING, TYPE_BOOLEAN, TYPE_NULL, TYPE_UNDEFINED, TYPE_ANY};
use tsnat_parse::ast::{Expr, BinaryOp, UnaryOp};
use tsnat_common::interner::Interner;

pub struct Checker<'a> {
    pub arena: &'a mut TypeArena,
    pub interner: &'a Interner,
}

impl<'a> Checker<'a> {
    pub fn new(arena: &'a mut TypeArena, interner: &'a Interner) -> Self {
        Self { arena, interner }
    }

    pub fn check_expr(&mut self, expr: &Expr, _expected: TypeId) -> TypeId {
        let actual = self.infer_expr(expr);
        // We would call assignability.is_assignable(actual, expected) here
        // and issue a diagnostic if false. For Phase 3A, we just return actual.
        actual
    }

    pub fn infer_expr(&mut self, expr: &Expr) -> TypeId {
        match expr {
            Expr::Number(n, _) => {
                // If we have literal tracking, we could return LiteralNumber.
                // For simplicity, we just infer Number.
                let _ = n;
                TYPE_NUMBER
            }
            Expr::String(s, _) => {
                let _ = s;
                TYPE_STRING
            }
            Expr::Bool(b, _) => {
                let _ = b;
                TYPE_BOOLEAN
            }
            Expr::Null(_) => TYPE_NULL,
            Expr::Undefined(_) => TYPE_UNDEFINED,
            Expr::Ident(_, _) => {
                // To fetch from environment scope
                TYPE_ANY
            }
            Expr::Binary(binary) => {
                let _left = self.infer_expr(binary.left);
                let _right = self.infer_expr(binary.right);

                match binary.op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        // Normally string concatenation check vs math check
                        TYPE_NUMBER
                    }
                    BinaryOp::EqEq | BinaryOp::EqEqEq | BinaryOp::BangEq | BinaryOp::BangEqEq | BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => {
                        TYPE_BOOLEAN
                    }
                    _ => TYPE_ANY,
                }
            }
            Expr::Unary(unary) => {
                let _operand = self.infer_expr(unary.operand);
                match unary.op {
                    UnaryOp::Not => TYPE_BOOLEAN,
                    UnaryOp::Neg | UnaryOp::Plus => TYPE_NUMBER,
                    _ => TYPE_ANY,
                }
            }
            Expr::Paren(inner, _) => self.infer_expr(inner),
            // TODO: other expression forms
            _ => TYPE_ANY,
        }
    }
}
