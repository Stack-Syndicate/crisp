mod error;
use inkwell::{builder::Builder, context::Context, module::Module, values::BasicValueEnum};
use std::collections::HashMap;

use crate::{compilation::error::CompilerError, parsing::ast::Expr};

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    variables: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self {
            context,
            builder,
            module,
            variables: HashMap::new(),
        }
    }
    pub fn compile_expr(
        &mut self,
        expr: Expr,
    ) -> Result<Option<BasicValueEnum<'ctx>>, CompilerError> {
        match expr {
            Expr::Literal(literal) => todo!(),
            Expr::Identifier(name) => {
                let val = self.variables.get(&name).cloned().unwrap();
                Ok(Some(val))
            }
            Expr::Block(exprs) => todo!(),
            Expr::Call { callee, args } => todo!(),
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => todo!(),
            Expr::Fn {
                params,
                return_type,
                body,
            } => todo!(),
            Expr::Def { name, value, .. } => todo!(),
            Expr::Loop { condition, body } => todo!(),
            Expr::Error => todo!(),
        }
    }
    pub fn declare_fns(&mut self, ast: &Vec<Expr>) {
        for expr in ast {
            let Expr::Def {
                name,
                type_annotation,
                value,
            } = expr
            else {
                continue;
            };
            let Expr::Fn {
                params,
                return_type,
                body,
            } = &**value
            else {
                continue;
            };
            if let Expr::Block(ast) = &**body {
                self.declare_fns(ast)
            }
        }
    }
}

pub fn compile(ast: Vec<Expr>, module_name: &str) -> Result<(), CompilerError> {
    let context = Context::create();
    let mut compiler = Compiler::new(&context, module_name);
    compiler.declare_fns(&ast);
    for expr in ast {
        compiler.compile_expr(expr)?;
    }
    Ok(())
}
