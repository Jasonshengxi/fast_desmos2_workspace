use std::collections::HashMap;

use cranelift::prelude::*;
use fast_desmos2_eval::{AddOrSub, EvalKind, EvalNode, IdentId};

pub trait Translate {
    fn translate(&self, builder: &mut FunctionBuilder) -> Value;
}

impl Translate for EvalNode {
    fn translate(&self, builder: &mut FunctionBuilder) -> Value {
        match self.kind() {
            &EvalKind::Identifier(ident_id) => builder.use_var(Variable::new(ident_id.get())),
            EvalKind::BuiltinsCall { .. } => todo!(),
            EvalKind::FunctionCall { .. } => todo!(),
            EvalKind::With { .. } => todo!(),
            EvalKind::For { .. } => todo!(),
            EvalKind::ListComp { .. } => todo!(),
            EvalKind::SumProd { .. } => todo!(),

            &EvalKind::Number(number) => builder.ins().f64const(number),
            EvalKind::Abs(value) => {
                let value = value.translate(builder);
                builder.ins().fabs(value)
            }
            EvalKind::Point(_, _) => todo!(),
            EvalKind::List(_) => todo!(),
            EvalKind::Multiply(values) => {
                assert!(values.len() > 0);

                let mut iter = values.iter();
                let first = iter.next().unwrap();
                let mut result = first.translate(builder);

                for item in iter {
                    let this_value = item.translate(builder);
                    let next_result = builder.ins().fmul(result, this_value);
                    result = next_result;
                }

                result
            }
            EvalKind::AddSub(values) => {
                assert!(values.len() > 0);

                let mut iter = values.iter();
                let (first_sign, first) = iter.next().unwrap();
                let first = first.translate(builder);
                let mut result = match first_sign {
                    AddOrSub::Add => first,
                    AddOrSub::Sub => builder.ins().fneg(first),
                };

                for (sign, item) in iter {
                    let this_value = item.translate(builder);
                    let next_result = match sign {
                        AddOrSub::Add => builder.ins().fadd(result, this_value),
                        AddOrSub::Sub => builder.ins().fsub(result, this_value),
                    };
                    result = next_result;
                }

                result
            }
            EvalKind::Frac { top, bottom } => {
                let top = top.translate(builder);
                let bottom = bottom.translate(builder);
                builder.ins().fdiv(top, bottom)
            }
            EvalKind::Sqrt(value) => {
                let value = value.translate(builder);

                todo!()
            }
            EvalKind::Power { base, power } => todo!(),
            EvalKind::ListRange { from, next, to } => todo!(),
            EvalKind::IfElse { conds, yes, no } => todo!(),
            EvalKind::ElemAccess { expr, element } => todo!(),
            EvalKind::ListIndexing { expr, index } => todo!(),
        }
    }
}
