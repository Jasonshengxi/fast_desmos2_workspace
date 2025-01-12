use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use fast_desmos2_tree::tree::{EditorTree, EditorTreeSeq};
use fast_desmos2_utils::{self as utils, leak};

use super::{
    error::{AltBehavior, ParseErrorKind, ParseLayer},
    ParseError, ParseExtra, ParseInput, ParseResult, ResExt,
};

fn start_bound_inclusive<R: RangeBounds<usize>>(x: &R) -> usize {
    match x.start_bound() {
        Bound::Included(&x) => x,
        Bound::Excluded(&x) => x + 1,
        Bound::Unbounded => 0,
    }
}

fn end_bound_inclusive<R: RangeBounds<usize>>(x: &R) -> usize {
    match x.end_bound() {
        Bound::Included(&x) => x,
        Bound::Excluded(&x) => x - 1,
        Bound::Unbounded => usize::MAX,
    }
}

pub trait MyParser<S: EditorTreeSeq>: Sized {
    type Output;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output>;

    #[track_caller]
    fn parse_whole(
        &mut self,
        eof_msg: &'static str,
        mut input: ParseInput<S>,
    ) -> ParseResult<Self::Output> {
        let outcome = self.parse_next(&mut input)?;
        Eof(eof_msg).parse_next(&mut input)?;
        Ok(outcome)
    }

    fn then_eof(self, eof_msg: &'static str) -> impl MyParser<S, Output = Self::Output> {
        (self, Eof(eof_msg)).map(|x| x.0)
    }

    fn map<O>(self, func: impl Fn(Self::Output) -> O) -> impl MyParser<S, Output = O> {
        Map::new(self, func)
    }

    fn map_with_extra<O>(
        self,
        func: impl Fn(Self::Output, ParseExtra) -> O,
    ) -> impl MyParser<S, Output = O> {
        MapWithExtra::new(self, func)
    }

    fn surround_whitespace(self) -> impl MyParser<S, Output = Self::Output> {
        (Whitespace::new(), self, Whitespace::new()).map(|x| x.1)
    }

    fn preceding<P: MyParser<S>>(self, after: P) -> impl MyParser<S, Output = P::Output> {
        (self, after).map(|x| x.1)
    }

    fn whitespace_after(self) -> impl MyParser<S, Output = Self::Output> {
        self.ignore_after(Whitespace::new())
    }

    fn ignore_after(self, after: impl MyParser<S>) -> impl MyParser<S, Output = Self::Output> {
        (self, after).map(|x| x.0)
    }

    fn fatal(self) -> impl MyParser<S, Output = Self::Output> {
        Fatal::new(self)
    }

    fn opt(self) -> impl MyParser<S, Output = Option<Self::Output>> {
        Optional::new(self)
    }

    fn repeated<A>(
        self,
        range: impl RangeBounds<usize>,
        acc: impl Fn() -> A,
        func: impl Fn(A, Self::Output) -> A,
    ) -> impl MyParser<S, Output = A> {
        Repeat::new(self, range, acc, func)
    }

    fn repeat_collected<A: Aggregate<Self::Output>>(
        self,
        range: impl RangeBounds<usize>,
    ) -> impl MyParser<S, Output = A> {
        Repeat::new(self, range, A::default, A::fold)
    }

    fn then_fold_repeated<O>(
        self,
        range: impl RangeBounds<usize>,
        parser: impl MyParser<S, Output = O>,
        func: impl Fn(Self::Output, O) -> Self::Output,
    ) -> impl MyParser<S, Output = Self::Output> {
        Fold::new(self, range, parser, func)
    }

    fn separated<A>(
        self,
        separator: impl MyParser<S>,
        range: impl RangeBounds<usize>,
        acc: impl Fn() -> A,
        func: impl Fn(A, Self::Output) -> A,
    ) -> impl MyParser<S, Output = A> {
        Separated::new(range, self, separator, acc, func)
    }
}

pub trait Aggregate<I>: Default {
    fn aggregate(&mut self, other: I);
    fn fold(mut self, other: I) -> Self {
        self.aggregate(other);
        self
    }
}

impl Aggregate<char> for String {
    fn aggregate(&mut self, other: char) {
        self.push(other)
    }
}

impl<T> Aggregate<T> for Vec<T> {
    fn aggregate(&mut self, other: T) {
        self.push(other)
    }
}

#[derive(Clone)]
pub struct Chained<F, P, S>
where
    F: Fn(&EditorTree<S>) -> Option<&S>,
    P: MyParser<S>,
    S: EditorTreeSeq,
{
    func: F,
    parser: P,
    _phantom: PhantomData<S>,
}

impl<F, P, S> Chained<F, P, S>
where
    F: Fn(&EditorTree<S>) -> Option<&S>,
    P: MyParser<S>,
    S: EditorTreeSeq,
{
    pub fn new(func: F, parser: P) -> Self {
        Self {
            func,
            parser,
            _phantom: PhantomData,
        }
    }
}

impl<F, P, S> MyParser<S> for Chained<F, P, S>
where
    F: Fn(&EditorTree<S>) -> Option<&S>,
    P: MyParser<S>,
    S: EditorTreeSeq,
{
    type Output = P::Output;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        let next = input.peek().ok_or(input.err_eof())?;
        let as_tree = (self.func)(next).ok_or(input.err_expected("chained success"))?;
        input.advance();
        let mut derived = input.derived(as_tree.children());
        let inner = self.parser.parse_next(&mut derived);
        inner.map_err(|err| input.err(ParseErrorKind::Chained(err)))
    }
}

#[derive(Debug)]
pub struct Fatal<P, S>
where
    P: MyParser<S>,
    S: EditorTreeSeq,
{
    parser: P,
    _phantom: PhantomData<S>,
}

impl<P, S> Fatal<P, S>
where
    P: MyParser<S>,
    S: EditorTreeSeq,
{
    pub const fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom: PhantomData,
        }
    }
}

impl<P, S> MyParser<S> for Fatal<P, S>
where
    P: MyParser<S>,
    S: EditorTreeSeq,
{
    type Output = P::Output;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        self.parser.parse_next(input).fatal()
    }
}

#[derive(Debug)]
pub struct Eof(&'static str);

impl<S: EditorTreeSeq> MyParser<S> for Eof {
    type Output = ();

    #[track_caller]
    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        input
            .peek()
            .is_none()
            .then_some(())
            .ok_or(input.err_not_eof(self.0))
    }
}

#[derive(Debug)]
pub struct Todo<O>(PhantomData<O>);

impl<O> Todo<O> {
    #[track_caller]
    pub const fn new() -> Self {
        panic!("TODO")
    }
}

impl<S: EditorTreeSeq, O> MyParser<S> for Todo<O> {
    type Output = O;

    fn parse_next(&mut self, _: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Separated<PA, PS, FA, FF, A, S, R>
where
    PA: MyParser<S>,
    PS: MyParser<S>,
    FA: Fn() -> A,
    FF: Fn(A, PA::Output) -> A,
    S: EditorTreeSeq,
    R: RangeBounds<usize>,
{
    range: R,
    parser: PA,
    separator: PS,
    acc: FA,
    func: FF,
    _phantom: PhantomData<S>,
}

impl<PA, PS, FA, FF, A, S, R> Separated<PA, PS, FA, FF, A, S, R>
where
    PA: MyParser<S>,
    PS: MyParser<S>,
    FA: Fn() -> A,
    FF: Fn(A, PA::Output) -> A,
    S: EditorTreeSeq,
    R: RangeBounds<usize>,
{
    pub const fn new(range: R, parser: PA, separator: PS, acc: FA, func: FF) -> Self {
        Self {
            range,
            parser,
            separator,
            acc,
            func,
            _phantom: PhantomData,
        }
    }
}

fn wrap_not_enough_repeat<S: EditorTreeSeq>(
    input: &ParseInput<S>,
    err: Option<ParseError>,
) -> ParseError {
    err.map_or_else(
        || input.err(ParseErrorKind::NotEnoughRepeat),
        |err| err.layered(ParseErrorKind::NotEnoughRepeat),
    )
}

impl<PA, PS, FA, FF, A, S, R> MyParser<S> for Separated<PA, PS, FA, FF, A, S, R>
where
    PA: MyParser<S>,
    PS: MyParser<S>,
    FA: Fn() -> A,
    FF: Fn(A, PA::Output) -> A,
    S: EditorTreeSeq,
    R: RangeBounds<usize>,
{
    type Output = A;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        let before_all = input.checkpoint();
        let start_bound = start_bound_inclusive(&self.range);
        let end_bound = end_bound_inclusive(&self.range);
        let mut acc = (self.acc)();
        let mut total_count = 0;
        let mut last_error = None;

        // this loop is for a goto. alternatives are functions, but this seems easier.
        for _ in 0..1 {
            if end_bound > 0 {
                let first = match self.parser.parse_next(input) {
                    Ok(ok) => ok,
                    Err(err) => {
                        last_error = Some(err);
                        break;
                    }
                };
                acc = (self.func)(acc, first);
                total_count += 1;

                while total_count < end_bound {
                    let before_separator = input.checkpoint();
                    match self.separator.parse_next(input) {
                        Ok(_) => {}
                        Err(err) => {
                            last_error = Some(err.layered(ParseErrorKind::NoSeparator));
                            break;
                        }
                    }

                    let this = match self.parser.parse_next(input) {
                        Ok(ok) => ok,
                        Err(err) => {
                            last_error = Some(err);
                            input.reset_to(before_separator);
                            break;
                        }
                    };

                    acc = (self.func)(acc, this);
                    total_count += 1;
                }
            }
        }

        if total_count < start_bound {
            input.reset_to(before_all);
            return Err(wrap_not_enough_repeat(input, last_error));
        }

        Ok(acc)
    }
}

#[derive(Debug)]
pub struct Fold<P1, PA, F, S, R>
where
    P1: MyParser<S>,
    PA: MyParser<S>,
    F: Fn(P1::Output, PA::Output) -> P1::Output,
    R: RangeBounds<usize>,
    S: EditorTreeSeq,
{
    init_parser: P1,
    range: R,
    parser: PA,
    func: F,
    _phantom: PhantomData<S>,
}

impl<P1, PA, F, S, R> MyParser<S> for Fold<P1, PA, F, S, R>
where
    P1: MyParser<S>,
    PA: MyParser<S>,
    F: Fn(P1::Output, PA::Output) -> P1::Output,
    R: RangeBounds<usize>,
    S: EditorTreeSeq,
{
    type Output = P1::Output;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        let before_all = input.checkpoint();

        let end_bound = end_bound_inclusive(&self.range);
        let start_bound = start_bound_inclusive(&self.range);

        let mut acc = self
            .init_parser
            .parse_next(input)
            .layered(ParseErrorKind::NoInitial)?;
        let mut total_count = 0;
        let mut last_error = None;

        while total_count < end_bound {
            let this_parsed = match self.parser.parse_next(input) {
                Ok(ok) => ok,
                Err(err) => {
                    last_error = Some(err);
                    break;
                }
            };
            acc = (self.func)(acc, this_parsed);
            total_count += 1;
        }

        if total_count < start_bound {
            input.reset_to(before_all);
            return Err(wrap_not_enough_repeat(input, last_error));
        }

        Ok(acc)
    }
}

impl<P1, PA, F, S, R> Fold<P1, PA, F, S, R>
where
    P1: MyParser<S>,
    PA: MyParser<S>,
    F: Fn(P1::Output, PA::Output) -> P1::Output,
    R: RangeBounds<usize>,
    S: EditorTreeSeq,
{
    pub fn new(init_parser: P1, range: R, parser: PA, func: F) -> Self {
        Self {
            init_parser,
            range,
            parser,
            func,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Repeat<P, FA, A, F, S, R>
where
    P: MyParser<S>,
    FA: Fn() -> A,
    F: Fn(A, P::Output) -> A,
    R: RangeBounds<usize>,
    S: EditorTreeSeq,
{
    parser: P,
    range: R,
    acc: FA,
    func: F,
    _phantom: PhantomData<S>,
}

impl<P, FA, A, F, S, R> Repeat<P, FA, A, F, S, R>
where
    P: MyParser<S>,
    FA: Fn() -> A,
    F: Fn(A, P::Output) -> A,
    R: RangeBounds<usize>,
    S: EditorTreeSeq,
{
    pub const fn new(parser: P, range: R, acc: FA, func: F) -> Self {
        Self {
            parser,
            range,
            acc,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<P, FA, A, F, S, R> MyParser<S> for Repeat<P, FA, A, F, S, R>
where
    P: MyParser<S>,
    FA: Fn() -> A,
    F: Fn(A, P::Output) -> A,
    R: RangeBounds<usize>,
    S: EditorTreeSeq,
{
    type Output = A;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        let before_all = input.checkpoint();

        let start_bound = start_bound_inclusive(&self.range);
        let end_bound = end_bound_inclusive(&self.range);

        let mut acc = (self.acc)();
        let mut total_count = 0usize;

        let mut last_error = None;

        while total_count < end_bound {
            let parsed = match self.parser.parse_next(input) {
                Ok(x) => x,
                Err(err) => {
                    last_error = Some(err);
                    break;
                }
            };
            acc = (self.func)(acc, parsed);
            total_count += 1;
        }

        if total_count < start_bound {
            // terminated too early.
            input.reset_to(before_all);
            return Err(wrap_not_enough_repeat(input, last_error));
        }

        Ok(acc)
    }
}

#[derive(Debug)]
pub struct Optional<P, O, S>
where
    P: MyParser<S, Output = O>,
    S: EditorTreeSeq,
{
    parser: P,
    _phantom: PhantomData<S>,
}

impl<P, O, S> Optional<P, O, S>
where
    P: MyParser<S, Output = O>,
    S: EditorTreeSeq,
{
    pub const fn new(parser: P) -> Self {
        Self {
            parser,
            _phantom: PhantomData,
        }
    }
}

impl<P, O, S> MyParser<S> for Optional<P, O, S>
where
    P: MyParser<S, Output = O>,
    S: EditorTreeSeq,
{
    type Output = Option<O>;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        Ok(self.parser.parse_next(input).map_or(None, Some))
    }
}

#[derive(Debug)]
pub struct Filter<F, S>
where
    F: for<'a> Fn(&'a EditorTree<S>) -> bool,
    S: EditorTreeSeq,
{
    func: F,
    _phantom: PhantomData<S>,
}

impl<F, S> Filter<F, S>
where
    F: for<'a> Fn(&'a EditorTree<S>) -> bool,
    S: EditorTreeSeq,
{
    pub const fn new(func: F) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }
}

impl<F, S> MyParser<S> for Filter<F, S>
where
    F: for<'a> Fn(&'a EditorTree<S>) -> bool,
    S: EditorTreeSeq,
{
    type Output = ();

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        let next = input.peek().ok_or(input.err_eof())?;
        (self.func)(next)
            .then_some(())
            .ok_or(input.err_expected("filter success"))?;
        input.advance();
        Ok(())
    }
}

#[derive(Debug)]
pub struct FilterMap<F, O, S>
where
    F: for<'a> Fn(&'a EditorTree<S>) -> Option<O>,
    S: EditorTreeSeq,
{
    func: F,
    _phantom: PhantomData<S>,
}

impl<F, O, S> FilterMap<F, O, S>
where
    F: for<'a> Fn(&'a EditorTree<S>) -> Option<O>,
    S: EditorTreeSeq,
{
    pub const fn new(func: F) -> Self {
        Self {
            func,
            _phantom: PhantomData,
        }
    }
}

impl<F, O, S> MyParser<S> for FilterMap<F, O, S>
where
    F: for<'a> Fn(&'a EditorTree<S>) -> Option<O>,
    S: EditorTreeSeq,
{
    type Output = O;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<O> {
        let next = input.peek().ok_or(input.err_eof())?;
        let output = (self.func)(next).ok_or(input.err_expected("filter map success"))?;
        input.advance();
        Ok(output)
    }
}

#[derive(Debug)]
pub struct MapWithExtra<F, P, I, O, S>
where
    F: Fn(I, ParseExtra) -> O,
    P: MyParser<S, Output = I>,
    S: EditorTreeSeq,
{
    parser: P,
    func: F,
    _phantom: PhantomData<S>,
}

impl<F, P, I, O, S> MyParser<S> for MapWithExtra<F, P, I, O, S>
where
    F: Fn(I, ParseExtra) -> O,
    P: MyParser<S, Output = I>,
    S: EditorTreeSeq,
{
    type Output = O;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        self.parser
            .parse_next(input)
            .map(|out| (self.func)(out, input.extra()))
    }
}

impl<F, P, I, O, S> MapWithExtra<F, P, I, O, S>
where
    F: Fn(I, ParseExtra) -> O,
    P: MyParser<S, Output = I>,
    S: EditorTreeSeq,
{
    pub fn new(parser: P, func: F) -> Self {
        Self {
            parser,
            func,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Map<F, P, I, O, S>
where
    F: Fn(I) -> O,
    P: MyParser<S, Output = I>,
    S: EditorTreeSeq,
{
    parser: P,
    func: F,
    _phantom: PhantomData<S>,
}

impl<F, P, I, O, S> Map<F, P, I, O, S>
where
    F: Fn(I) -> O,
    P: MyParser<S, Output = I>,
    S: EditorTreeSeq,
{
    pub const fn new(parser: P, func: F) -> Self {
        Self {
            parser,
            func,
            _phantom: PhantomData,
        }
    }
}

impl<F, P, I, O, S> MyParser<S> for Map<F, P, I, O, S>
where
    F: Fn(I) -> O,
    P: MyParser<S, Output = I>,
    S: EditorTreeSeq,
{
    type Output = O;

    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        self.parser.parse_next(input).map(&self.func)
    }
}

#[derive(Debug)]
pub struct Alt<S, A>(A, PhantomData<S>)
where
    A: Alternable<S>,
    S: EditorTreeSeq;

impl<S, A> Alt<S, A>
where
    A: Alternable<S>,
    S: EditorTreeSeq,
{
    pub const fn new(alt: A) -> Self {
        Self(alt, PhantomData)
    }
}

impl<S, A> MyParser<S> for Alt<S, A>
where
    A: Alternable<S>,
    S: EditorTreeSeq,
{
    type Output = A::Output;

    #[track_caller]
    fn parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output> {
        self.0.alt_parse_next(input)
    }
}

pub trait Alternable<S: EditorTreeSeq> {
    type Output;

    fn alt_parse_next(&mut self, input: &mut ParseInput<S>) -> ParseResult<Self::Output>;
}

macro_rules! implement_tuples {
    ($($ty:ident $pos:tt)*) => {
        impl<S, O, $($ty),*> Alternable<S> for ($($ty,)*)
        where
            S: EditorTreeSeq,
            $($ty: MyParser<S, Output = O>,)*
        {
            type Output = O;

            #[track_caller]
            fn alt_parse_next(&mut self, input: &mut ParseInput<S> ) -> ParseResult<Self::Output> {
                $(
                    match self.$pos.parse_next(input) {
                        Ok(ok) => return Ok(ok),
                        Err(err) => match err.alt_behavior() {
                            AltBehavior::Backtrack => {}
                            AltBehavior::Fatal => return Err(err),
                        },
                    }
                )*

                Err(input.err(ParseErrorKind::Alt))
            }
        }

        impl<S, $($ty),*> MyParser<S> for ($($ty,)*)
        where
            S: EditorTreeSeq,
            $($ty: MyParser<S>),*
        {
            type Output = ($($ty::Output,)*);

            fn parse_next(&mut self, input: &mut ParseInput<S> ) -> ParseResult<Self::Output> {
                let checkpoint = input.checkpoint();
                Ok((
                    $(
                        match self.$pos.parse_next(input) {
                            Ok(ok) => ok,
                            Err(err) => {
                                input.reset_to(checkpoint);
                                return Err(err);
                            }
                        },
                    )*
                ))
            }
        }
    };
}
implement_tuples!(T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9 T10 10 T11 11);
implement_tuples!(T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9 T10 10);
implement_tuples!(T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8 T9 9);
implement_tuples!(T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7 T8 8);
implement_tuples!(T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6 T7 7);
implement_tuples!(T0 0 T1 1 T2 2 T3 3 T4 4 T5 5 T6 6);
implement_tuples!(T0 0 T1 1 T2 2 T3 3 T4 4 T5 5);
implement_tuples!(T0 0 T1 1 T2 2 T3 3 T4 4);
implement_tuples!(T0 0 T1 1 T2 2 T3 3);
implement_tuples!(T0 0 T1 1 T2 2);
implement_tuples!(T0 0 T1 1);
implement_tuples!(T0 0);

#[macro_export]
macro_rules! function_as_parser {
    (
        $($vis:vis fn $name:ident<$S:ident>($input:ident) -> <$r:ty> $body:tt)*
    ) => {
        $( #[derive(Default, Debug, Clone, Copy)]
        $vis struct $name<$S: EditorTreeSeq>(::core::marker::PhantomData<$S>);

        impl<$S: EditorTreeSeq> $name<$S> {
            #[allow(unused)]
            pub const fn new() -> Self { Self(::core::marker::PhantomData) }
        }

        impl<$S: EditorTreeSeq> MyParser<$S> for $name<$S> {
            type Output = $r;
            fn parse_next(&mut self, $input: &mut ParseInput<$S>) -> ParseResult<Self::Output> $body
        })*
    };
}

function_as_parser! {
    pub fn Whitespace<S>(input) -> <()> { input.skip_whitespace(); Ok(()) }
}
