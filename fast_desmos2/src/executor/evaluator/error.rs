use std::ops::Range;

use super::IdentId;
use ariadne::{ColorGenerator, Config, Label, Report, ReportBuilder, ReportKind, Source};
use fast_desmos2_comms::value::ValueKind;
use fast_desmos2_comms::TypeMismatch;
use fast_desmos2_parser::{Builtins, Span};
use fast_desmos2_utils::OptExt;

#[derive(Debug, Clone)]
pub enum EvalErrorKind {
    UnknownIdent(IdentId),
    TypeMismatch {
        op_name: &'static str,
        left: (Span, ValueKind),
        right: (Span, ValueKind),
    },
    WrongType {
        expect: ValueKind,
        got: ValueKind,
    },
    InvalidValue {
        for_what: &'static str,
    },
    BadParamCount {
        expect: usize,
        got: usize,
    },
    CannotInvert(Builtins),
}

impl EvalErrorKind {
    pub fn wrong_type(TypeMismatch { expect, got }: TypeMismatch) -> Self {
        Self::WrongType { expect, got }
    }

    pub fn summary_message(&self) -> &'static str {
        match self {
            Self::TypeMismatch { .. } => "type mismatch",
            Self::UnknownIdent(_) => "unknown ident",
            Self::WrongType { .. } => "wrong type",
            Self::InvalidValue { .. } => "invalid value",
            Self::BadParamCount { .. } => "bad param count",
            Self::CannotInvert(_) => "cannot invert",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalError {
    note: Option<&'static str>,
    span: Span,
    kind: EvalErrorKind,
}

impl EvalErrorKind {
    pub fn with_span(self, span: Span) -> EvalError {
        EvalError {
            span,
            kind: self,
            note: None,
        }
    }
}

impl EvalError {
    pub fn with_note(self, note: &'static str) -> Self {
        Self {
            note: Some(note),
            ..self
        }
    }

    pub fn display(&self, source: &str) {
        let span = Range::from(self.span);
        let config = Config::default().with_compact(false);
        let report = Report::build(ReportKind::Error, (), 0).with_config(config);

        let mut colors = ColorGenerator::new();
        let mut label = |span| Label::new(span).with_color(colors.next());

        let report: ReportBuilder<Range<usize>> = match self.kind {
            EvalErrorKind::UnknownIdent(id) => {
                let raw_name = self.span.select(source);
                let message = format!("unknown ident: `{raw_name}` (id {})", id.get());
                report
                    .with_message(&message)
                    .with_label(label(span).with_message("here"))
            }
            EvalErrorKind::InvalidValue { for_what } => report
                .with_message(format!("invalid value for {for_what}"))
                .with_label(label(span).with_message("here")),
            EvalErrorKind::TypeMismatch {
                left: (left_span, left_type),
                right: (right_span, right_type),
                op_name,
            } => {
                let message =
                    format!("mismatched types for {op_name}: `{left_type}` and `{right_type}`");
                let left_message = format!("this expression has type `{left_type}`");
                let right_message = format!("this expression has type `{right_type}`");
                println!("{left_span:?}, {right_span:?}");
                report
                    .with_message(message)
                    .with_label(label(left_span.into()).with_message(left_message))
                    .with_label(label(right_span.into()).with_message(right_message))
            }
            EvalErrorKind::WrongType { expect, got } => report
                .with_message(format!("wrong type: expected `{expect}`, got `{got}`"))
                .with_label(label(span).with_message(format!(
                    "this expression has type `{got}`, should be `{expect}`"
                ))),
            EvalErrorKind::BadParamCount { expect, got } => report
                .with_message(format!(
                    "bad param count: expected {expect} parameters, got {got}"
                ))
                .with_label(label(span).with_message("here")),
            EvalErrorKind::CannotInvert(func) => report
                .with_message(format!("cannot invert {}", func.as_str()))
                .with_label(label(span).with_message("this one")),
        };

        let report = match self.note {
            Some(note) => report.with_note(note),
            None => report,
        };

        report
            .finish()
            .eprint(Source::from(source))
            .unwrap_unreach();
    }
}
