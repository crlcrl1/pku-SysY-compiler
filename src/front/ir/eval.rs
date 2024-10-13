use crate::front::ast::*;
use crate::front::ident_table::Identifier;
use crate::front::ir::scope::Scoop;
use crate::util::logger::show_error;

pub enum EvalError {
    DivisionByZero,
    Overflow,
    NotSupportedVariable,
}

type EvalResult = Result<i32, EvalError>;

pub trait Eval {
    fn eval(&self, scope: &mut Scoop) -> EvalResult;
}

impl Eval for ConstExpr {
    fn eval(&self, scope: &mut Scoop) -> EvalResult {
        self.0.eval(scope)
    }
}

impl Eval for AddExpr {
    fn eval(&self, scope: &mut Scoop) -> EvalResult {
        match self {
            AddExpr::MulExpr(mul_expr) => mul_expr.eval(scope),
            AddExpr::Add(left, op, right) => match op {
                AddOp::Add => left
                    .eval(scope)?
                    .checked_add(right.eval(scope)?)
                    .ok_or(EvalError::Overflow),
                AddOp::Sub => left
                    .eval(scope)?
                    .checked_sub(right.eval(scope)?)
                    .ok_or(EvalError::Overflow),
            },
        }
    }
}

impl Eval for MulExpr {
    fn eval(&self, scope: &mut Scoop) -> EvalResult {
        match self {
            MulExpr::UnaryExpr(unary_expr) => unary_expr.eval(scope),
            MulExpr::Mul(left, op, right) => match op {
                MulOp::Div => left
                    .eval(scope)?
                    .checked_div(right.eval(scope)?)
                    .ok_or(EvalError::DivisionByZero),
                MulOp::Mod => left
                    .eval(scope)?
                    .checked_rem(right.eval(scope)?)
                    .ok_or(EvalError::DivisionByZero),
                MulOp::Mul => left
                    .eval(scope)?
                    .checked_mul(right.eval(scope)?)
                    .ok_or(EvalError::Overflow),
            },
        }
    }
}

impl Eval for UnaryExpr {
    fn eval(&self, scope: &mut Scoop) -> EvalResult {
        match self {
            UnaryExpr::PrimaryExpr(primary_expr) => primary_expr.eval(scope),
            UnaryExpr::FuncCall(_) => {
                show_error("Function call in constant expression is not supported.", 1);
            }
            UnaryExpr::Unary(op, unary_expr) => match op {
                UnaryOp::Neg => unary_expr.eval(scope).map(|x| -x),
                UnaryOp::Not => unary_expr.eval(scope).map(|x| if x == 0 { 1 } else { 0 }),
                UnaryOp::Pos => unary_expr.eval(scope),
            },
        }
    }
}

impl Eval for PrimaryExpr {
    fn eval(&self, scope: &mut Scoop) -> EvalResult {
        match self {
            PrimaryExpr::Expr(expr) => expr.eval(scope),
            PrimaryExpr::LVal(lval) => lval.eval(scope),
            PrimaryExpr::Number(num) => Ok(*num),
        }
    }
}

impl Eval for Expr {
    fn eval(&self, scope: &mut Scoop) -> EvalResult {
        self.0.eval(scope)
    }
}

impl Eval for LVal {
    fn eval(&self, scope: &mut Scoop) -> EvalResult {
        match self {
            LVal::Var(var) => {
                if let Some(id) = scope.get_identifier(var) {
                    let id = id.clone();
                    match id {
                        Identifier::Constant(constant) => match *constant.def {
                            ConstDef::NormalConstDef(ref const_def) => const_def.value.eval(scope),
                            ConstDef::ArrayConstDef(_) => Err(EvalError::NotSupportedVariable),
                        },
                        _ => Err(EvalError::NotSupportedVariable),
                    }
                } else {
                    Err(EvalError::NotSupportedVariable)
                }
            }
            LVal::ArrayElem(_) => Err(EvalError::NotSupportedVariable),
        }
    }
}
