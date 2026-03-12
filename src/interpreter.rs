#![allow(dead_code, unused)]

use crate::ast::{BinaryOpKind, Expr, Program, Stmt, Type, UnaryOpKind};
use std::collections::HashMap;
use std::io::{self, BufRead};

/// 런타임 값
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Void,
    Function {
        params: Vec<(String, Type)>,
        body: Vec<Stmt>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "참" } else { "거짓" }),
            Value::Void => write!(f, "없음"),
            Value::Function { .. } => write!(f, "<함수>"),
        }
    }
}

/// 런타임 에러
#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

/// 변수 환경 (스코프 체인)
pub struct Environment {
    store: HashMap<String, Value>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    /// 새 최상위 환경
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            outer: None,
        }
    }

    /// 새 중첩 환경 (함수 호출 시)
    pub fn new_enclosed(outer: Environment) -> Self {
        Self {
            store: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    /// 변수 조회 (현재 → 외부 순)
    pub fn get(&self, name: &str) -> Option<Value> {
        match self.store.get(name) {
            Some(v) => Some(v.clone()),
            None => self.outer.as_ref()?.get(name),
        }
    }

    /// 변수 설정 (현재 스코프)
    pub fn set(&mut self, name: String, val: Value) {
        self.store.insert(name, val);
    }

    /// 변수 업데이트 (존재하는 스코프에서)
    pub fn update(&mut self, name: &str, val: Value) -> bool {
        if self.store.contains_key(name) {
            self.store.insert(name.to_string(), val);
            true
        } else if let Some(outer) = &mut self.outer {
            outer.update(name, val)
        } else {
            false
        }
    }

    pub fn collect_functions(&self) -> Vec<(String, Value)> {
        let mut funcs: Vec<(String, Value)> = self
            .store
            .iter()
            .filter(|(_, v)| matches!(v, Value::Function { .. }))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        if let Some(outer) = &self.outer {
            for (k, v) in outer.collect_functions() {
                if !funcs.iter().any(|(name, _)| name == &k) {
                    funcs.push((k, v));
                }
            }
        }
        funcs
    }
}

/// 조기 종료 신호 (반환, 멈춰, 계속)
pub enum Signal {
    Return(Value),
    Break,
    Continue,
}

/// 표현식 평가 → Value
pub fn eval_expr(expr: &Expr, env: &mut Environment) -> Result<Value, RuntimeError> {
    match expr {
        Expr::IntLiteral(n) => Ok(Value::Int(*n)),
        Expr::FloatLiteral(f) => Ok(Value::Float(*f)),
        Expr::StringLiteral(s) => Ok(Value::Str(s.clone())),
        Expr::BoolLiteral(b) => Ok(Value::Bool(*b)),

        Expr::Identifier(name) => env
            .get(name)
            .ok_or_else(|| RuntimeError::new(format!("정의되지 않은 변수: {}", name))),

        Expr::Assign { name, value } => {
            let val = eval_expr(value, env)?;
            if !env.update(name, val.clone()) {
                // 현재 스코프에 없으면 새로 설정
                env.set(name.clone(), val.clone());
            }
            Ok(val)
        }

        Expr::BinaryOp { op, left, right } => {
            let lv = eval_expr(left, env)?;
            let rv = eval_expr(right, env)?;
            eval_binary_op(op, lv, rv)
        }

        Expr::UnaryOp { op, expr } => {
            let val = eval_expr(expr, env)?;
            match op {
                UnaryOpKind::Neg => match val {
                    Value::Int(n) => Ok(Value::Int(-n)),
                    Value::Float(f) => Ok(Value::Float(-f)),
                    _ => Err(RuntimeError::new("단항 음수는 정수/실수에만 적용 가능")),
                },
                UnaryOpKind::Not => match val {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err(RuntimeError::new("논리 부정은 불 값에만 적용 가능")),
                },
            }
        }

        Expr::Call { name, args } => {
            // 내장 함수: 출력
            if name == "출력" {
                let mut parts = Vec::new();
                for arg in args {
                    let v = eval_expr(arg, env)?;
                    parts.push(v.to_string());
                }
                println!("{}", parts.join(" "));
                return Ok(Value::Void);
            }

            // 내장 함수: 입력
            if name == "입력" {
                let stdin = io::stdin();
                let mut line = String::new();
                stdin
                    .lock()
                    .read_line(&mut line)
                    .map_err(|e| RuntimeError::new(format!("입력 오류: {}", e)))?;
                return Ok(Value::Str(line.trim_end_matches('\n').to_string()));
            }

            // 사용자 정의 함수 호출
            let func_val = env
                .get(name)
                .ok_or_else(|| RuntimeError::new(format!("정의되지 않은 함수: {}", name)))?;

            match func_val {
                Value::Function { params, body } => {
                    if args.len() != params.len() {
                        return Err(RuntimeError::new(format!(
                            "함수 '{}': 인자 수 불일치 (기대 {}, 실제 {})",
                            name,
                            params.len(),
                            args.len()
                        )));
                    }

                    let mut arg_vals = Vec::new();
                    for arg in args {
                        arg_vals.push(eval_expr(arg, env)?);
                    }

                    let mut func_env = Environment::new();
                    for (fname, fval) in env.collect_functions() {
                        func_env.set(fname, fval);
                    }
                    for ((param_name, _ty), val) in params.iter().zip(arg_vals) {
                        func_env.set(param_name.clone(), val);
                    }

                    match eval_block(&body, &mut func_env)? {
                        Some(Signal::Return(v)) => Ok(v),
                        _ => Ok(Value::Void),
                    }
                }
                _ => Err(RuntimeError::new(format!("'{}' 는 함수가 아닙니다", name))),
            }
        }
    }
}

fn eval_binary_op(op: &BinaryOpKind, lv: Value, rv: Value) -> Result<Value, RuntimeError> {
    match op {
        // 산술
        BinaryOpKind::Add => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Str(a), Value::Str(b)) => Ok(Value::Str(a + &b)),
            _ => Err(RuntimeError::new("+ 연산: 타입 불일치")),
        },
        BinaryOpKind::Sub => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            _ => Err(RuntimeError::new("- 연산: 타입 불일치")),
        },
        BinaryOpKind::Mul => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            _ => Err(RuntimeError::new("* 연산: 타입 불일치")),
        },
        BinaryOpKind::Div => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::new("0으로 나눌 수 없습니다"))
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            _ => Err(RuntimeError::new("/ 연산: 타입 불일치")),
        },
        BinaryOpKind::Mod => match (lv, rv) {
            (Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    Err(RuntimeError::new("0으로 나머지 연산 불가"))
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            _ => Err(RuntimeError::new("% 연산: 정수에만 적용 가능")),
        },
        // 비교
        BinaryOpKind::Eq => Ok(Value::Bool(values_equal(&lv, &rv))),
        BinaryOpKind::NotEq => Ok(Value::Bool(!values_equal(&lv, &rv))),
        BinaryOpKind::Lt => compare_values(lv, rv, |a, b| a < b),
        BinaryOpKind::Gt => compare_values(lv, rv, |a, b| a > b),
        BinaryOpKind::LtEq => compare_values(lv, rv, |a, b| a <= b),
        BinaryOpKind::GtEq => compare_values(lv, rv, |a, b| a >= b),
        // 논리
        BinaryOpKind::And => match (lv, rv) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            _ => Err(RuntimeError::new("&& 연산: 불 값에만 적용 가능")),
        },
        BinaryOpKind::Or => match (lv, rv) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            _ => Err(RuntimeError::new("|| 연산: 불 값에만 적용 가능")),
        },
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Void, Value::Void) => true,
        _ => false,
    }
}

fn compare_values<F>(lv: Value, rv: Value, cmp: F) -> Result<Value, RuntimeError>
where
    F: Fn(f64, f64) -> bool,
{
    match (lv, rv) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(cmp(a as f64, b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(cmp(a, b))),
        _ => Err(RuntimeError::new("비교 연산: 숫자 타입에만 적용 가능")),
    }
}

/// 문장 실행 → Option<Signal>
pub fn eval_stmt(stmt: &Stmt, env: &mut Environment) -> Result<Option<Signal>, RuntimeError> {
    match stmt {
        Stmt::VarDecl { name, value, .. } => {
            let val = eval_expr(value, env)?;
            env.set(name.clone(), val);
            Ok(None)
        }

        Stmt::FuncDef {
            name, params, body, ..
        } => {
            let func = Value::Function {
                params: params.clone(),
                body: body.clone(),
            };
            env.set(name.clone(), func);
            Ok(None)
        }

        Stmt::Return(expr_opt) => {
            let val = match expr_opt {
                Some(expr) => eval_expr(expr, env)?,
                None => Value::Void,
            };
            Ok(Some(Signal::Return(val)))
        }

        Stmt::If {
            cond,
            then_block,
            else_block,
        } => {
            let cond_val = eval_expr(cond, env)?;
            match cond_val {
                Value::Bool(true) => eval_block(then_block, env),
                Value::Bool(false) => {
                    if let Some(else_stmts) = else_block {
                        eval_block(else_stmts, env)
                    } else {
                        Ok(None)
                    }
                }
                _ => Err(RuntimeError::new("조건문: 불 값이 필요합니다")),
            }
        }

        Stmt::WhileLoop { cond, body } => {
            loop {
                let cond_val = eval_expr(cond, env)?;
                match cond_val {
                    Value::Bool(true) => {}
                    Value::Bool(false) => break,
                    _ => return Err(RuntimeError::new("동안 조건: 불 값이 필요합니다")),
                }
                match eval_block(body, env)? {
                    Some(Signal::Break) => break,
                    Some(Signal::Continue) => continue,
                    Some(sig @ Signal::Return(_)) => return Ok(Some(sig)),
                    None => {}
                }
            }
            Ok(None)
        }

        Stmt::ForLoop {
            init,
            cond,
            step,
            body,
        } => {
            eval_stmt(init, env)?;
            loop {
                let cond_val = eval_expr(cond, env)?;
                match cond_val {
                    Value::Bool(true) => {}
                    Value::Bool(false) => break,
                    _ => return Err(RuntimeError::new("반복 조건: 불 값이 필요합니다")),
                }
                match eval_block(body, env)? {
                    Some(Signal::Break) => break,
                    Some(Signal::Continue) => {
                        eval_stmt(step, env)?;
                        continue;
                    }
                    Some(sig @ Signal::Return(_)) => return Ok(Some(sig)),
                    None => {}
                }
                eval_stmt(step, env)?;
            }
            Ok(None)
        }

        Stmt::Break => Ok(Some(Signal::Break)),
        Stmt::Continue => Ok(Some(Signal::Continue)),

        Stmt::ExprStmt(expr) => {
            eval_expr(expr, env)?;
            Ok(None)
        }
    }
}

/// 블록(문장 목록) 실행
pub fn eval_block(stmts: &[Stmt], env: &mut Environment) -> Result<Option<Signal>, RuntimeError> {
    for stmt in stmts {
        if let Some(sig) = eval_stmt(stmt, env)? {
            return Ok(Some(sig));
        }
    }
    Ok(None)
}

/// 공개 진입점
pub fn interpret(program: Program) -> Result<(), RuntimeError> {
    let mut env = Environment::new();
    eval_block(&program.stmts, &mut env)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_set_get() {
        let mut env = Environment::new();
        env.set("나이".to_string(), Value::Int(20));
        assert!(matches!(env.get("나이"), Some(Value::Int(20))));
    }

    #[test]
    fn test_env_scope_chain() {
        let mut outer = Environment::new();
        outer.set("x".to_string(), Value::Int(10));
        let inner = Environment::new_enclosed(outer);
        assert!(matches!(inner.get("x"), Some(Value::Int(10))));
        assert!(inner.get("y").is_none());
    }

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Bool(true).to_string(), "참");
        assert_eq!(Value::Bool(false).to_string(), "거짓");
        assert_eq!(Value::Void.to_string(), "없음");
    }

    #[test]
    fn test_env_update() {
        let mut env = Environment::new();
        env.set("x".to_string(), Value::Int(1));
        env.update("x", Value::Int(2));
        assert!(matches!(env.get("x"), Some(Value::Int(2))));
    }

    #[test]
    fn test_eval_arithmetic() {
        use crate::ast::{BinaryOpKind, Expr};
        let mut env = Environment::new();
        let expr = Expr::BinaryOp {
            op: BinaryOpKind::Add,
            left: Box::new(Expr::IntLiteral(3)),
            right: Box::new(Expr::BinaryOp {
                op: BinaryOpKind::Mul,
                left: Box::new(Expr::IntLiteral(5)),
                right: Box::new(Expr::IntLiteral(2)),
            }),
        };
        let result = eval_expr(&expr, &mut env).unwrap();
        assert!(matches!(result, Value::Int(13)));
    }

    #[test]
    fn test_eval_var_decl() {
        use crate::ast::{Expr, Stmt};
        let mut env = Environment::new();
        let stmt = Stmt::VarDecl {
            name: "나이".to_string(),
            ty: None,
            value: Expr::IntLiteral(20),
            mutable: true,
        };
        eval_stmt(&stmt, &mut env).unwrap();
        assert!(matches!(env.get("나이"), Some(Value::Int(20))));
    }

    #[test]
    fn test_eval_if_stmt() {
        use crate::ast::{Expr, Stmt};
        let mut env = Environment::new();
        let stmt = Stmt::If {
            cond: Expr::BoolLiteral(true),
            then_block: vec![Stmt::VarDecl {
                name: "x".to_string(),
                ty: None,
                value: Expr::IntLiteral(1),
                mutable: false,
            }],
            else_block: None,
        };
        eval_stmt(&stmt, &mut env).unwrap();
        assert!(matches!(env.get("x"), Some(Value::Int(1))));
    }

    #[test]
    fn test_eval_fibonacci() {
        use crate::ast::{BinaryOpKind, Expr, Program, Stmt, Type};
        let fib_body = vec![
            Stmt::If {
                cond: Expr::BinaryOp {
                    op: BinaryOpKind::LtEq,
                    left: Box::new(Expr::Identifier("n".to_string())),
                    right: Box::new(Expr::IntLiteral(1)),
                },
                then_block: vec![Stmt::Return(Some(Expr::Identifier("n".to_string())))],
                else_block: None,
            },
            Stmt::Return(Some(Expr::BinaryOp {
                op: BinaryOpKind::Add,
                left: Box::new(Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::BinaryOp {
                        op: BinaryOpKind::Sub,
                        left: Box::new(Expr::Identifier("n".to_string())),
                        right: Box::new(Expr::IntLiteral(1)),
                    }],
                }),
                right: Box::new(Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::BinaryOp {
                        op: BinaryOpKind::Sub,
                        left: Box::new(Expr::Identifier("n".to_string())),
                        right: Box::new(Expr::IntLiteral(2)),
                    }],
                }),
            })),
        ];

        let program = Program::new(vec![
            Stmt::FuncDef {
                name: "피보나치".to_string(),
                params: vec![("n".to_string(), Type::정수)],
                return_type: Some(Type::정수),
                body: fib_body,
            },
            Stmt::VarDecl {
                name: "결과".to_string(),
                ty: None,
                value: Expr::Call {
                    name: "피보나치".to_string(),
                    args: vec![Expr::IntLiteral(10)],
                },
                mutable: false,
            },
        ]);

        let mut env = Environment::new();
        eval_block(&program.stmts, &mut env).unwrap();
        assert!(matches!(env.get("결과"), Some(Value::Int(55))));
    }

    #[test]
    fn test_출력() {
        use crate::ast::{Expr, Program, Stmt};
        let program = Program::new(vec![Stmt::ExprStmt(Expr::Call {
            name: "출력".to_string(),
            args: vec![Expr::StringLiteral("안녕".to_string())],
        })]);
        let result = interpret(program);
        assert!(result.is_ok());
    }
}
