use super::{EvalError, EvalErrorKind, IdentId, TypeMismatch, Value};
use fast_desmos2_parser::{AstKind, AstNode, Span};
use std::cell::Cell;

pub(super) fn maybe_type_mismatch(
    result: Result<Value, TypeMismatch>,
    left_span: Span,
    right_span: Span,
    op_name: &'static str,
) -> Result<(Span, Value), EvalError> {
    let whole_span = left_span.union(right_span);
    Ok((
        right_span,
        result.map_err(|mismatch| {
            assert!(!left_span.is_empty());
            assert!(!right_span.is_empty());
            EvalErrorKind::TypeMismatch {
                op_name,
                left: (left_span, mismatch.expect),
                right: (right_span, mismatch.got),
            }
            .with_span(whole_span)
        })?,
    ))
}

#[track_caller]
pub(super) fn unwrap_identifier(node: &AstNode) -> IdentId {
    match node.kind() {
        &AstKind::Identifier(ident) => ident,
        _ => unreachable!(),
    }
}

#[track_caller]
pub(super) fn unwrap_var_def<'a, 'b>(
    source: &'a str,
    node: &'b AstNode,
) -> (&'a str, IdentId, &'b AstNode) {
    match node.kind() {
        AstKind::VarDef { ident, expr } => {
            (ident.span_as_str(source), unwrap_identifier(ident), expr)
        }
        _ => unreachable!(),
    }
}

pub(super) fn wrong_type(mismatch: TypeMismatch, span: Span) -> EvalError {
    let TypeMismatch { expect, got } = mismatch;
    EvalErrorKind::WrongType { expect, got }.with_span(span)
}

pub(super) fn try_single_number(
    value: Value,
    span: Span,
    (for_what, why): (&'static str, &'static str),
) -> Result<f64, EvalError> {
    value
        .try_number()
        .map_err(|m| wrong_type(m, span))?
        .try_term()
        .ok_or_else(|| {
            EvalErrorKind::InvalidValue { for_what }
                .with_span(span)
                .with_note(why)
        })
}
