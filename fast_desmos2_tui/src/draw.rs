use std::fmt::Display;

use bitflags::bitflags;
use glam::UVec2;

use fast_desmos2_tree::tree::{
    CompletableSurrounds, EditorTree, EditorTreeAbs, EditorTreeBracket, EditorTreeCurly,
    EditorTreeFraction, EditorTreeKind, EditorTreeParen, EditorTreePower, EditorTreeSeq,
    EditorTreeSeqNormal, EditorTreeSqrt, EditorTreeSumProd, EditorTreeTerminal, FractionIndex,
    SumOrProd, SumProdIndex, SurroundIndex, SurroundsTreeSeq,
};
use termion::style;

#[derive(Debug, Clone, Copy)]
pub enum Surrounder {
    Rectangle,

    Sqrt,

    Abs,
    Parens,
    Brackets,
    Curly,
}

#[derive(Clone, Copy)]
struct SurroundInfo {
    top: (Option<char>, Option<char>, Option<char>),
    middle: (Option<char>, Option<char>),
    bottom: (Option<char>, Option<char>, Option<char>),

    compressed: (Option<char>, Option<char>),
    insert_middle: (Option<char>, Option<char>),
}

macro_rules! sn {
    (.) => {
        None
    };
    ($x: expr) => {
        Some($x)
    };
}

macro_rules! surround_info {
    ($($x: tt)*) => {
        SurroundInfo::new([$(sn!($x)),*])
    };
}

// ─━│┃┄┅┆┇┈┉┊┋┌┍┎┏┐┑┒┓└┕┖┗┘┙┚┛├┝┞┟┠┡┢┣┤┥┦┧┨┩┪┫┬┭┮┯┰┱┲┳┴┵┶┷┸┹┺┻┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋╌╍╎╏═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬╭╮╯╰╱╲╳╴╵╶╷╸╹╺╻╼╽╾╿
impl SurroundInfo {
    const RECTANGLE: Self = surround_info!(
        '┌''─''┐'
        '│'   '│'
        '└''─''┘'

        '['   ']'
         .     .
    );
    const SQRT: Self = surround_info!(
        '┌''─''─'
        '│'    .
        '│' .  .

        '√'    .
         .     .
    );
    const BRACKETS: Self = surround_info!(
        '┌' . '┐'
        '│'   '│'
        '└' . '┘'

        '['   ']'
         .     .
    );
    const ABS: Self = surround_info!(
        '│' . '│'
        '│'   '│'
        '│' . '│'

        '|'   '|'
         .     .
    );
    const PARENS: Self = surround_info!(
        '╭' . '╮'
        '│'   '│'
        '╰' . '╯'

        '('   ')'
         .     .
    );
    const CURLY: Self = surround_info!(
        '╭' . '╮'
        '│'   '│'
        '╰' . '╯'

        '{'   '}'
        '┤'   '├'
    );

    const fn new([a, b, c, d, e, f, g, h, i, j, k, l]: [Option<char>; 12]) -> Self {
        Self {
            top: (a, b, c),
            middle: (d, e),
            bottom: (f, g, h),
            compressed: (i, j),
            insert_middle: (k, l),
        }
    }
}

impl Surrounder {
    const fn offset(&self) -> UVec2 {
        match self {
            Surrounder::Rectangle | Surrounder::Sqrt => UVec2::ONE,
            Surrounder::Abs | Surrounder::Parens | Surrounder::Brackets | Surrounder::Curly => {
                UVec2::X
            }
        }
    }

    const fn size(&self) -> UVec2 {
        match self {
            Surrounder::Rectangle => UVec2::splat(2),
            Surrounder::Sqrt => UVec2::ONE,
            Surrounder::Abs | Surrounder::Parens | Surrounder::Brackets | Surrounder::Curly => {
                UVec2::new(2, 0)
            }
        }
    }

    const fn surround_info(&self) -> SurroundInfo {
        match self {
            Surrounder::Rectangle => SurroundInfo::RECTANGLE,
            Surrounder::Sqrt => SurroundInfo::SQRT,
            Surrounder::Abs => SurroundInfo::ABS,
            Surrounder::Parens => SurroundInfo::PARENS,
            Surrounder::Brackets => SurroundInfo::BRACKETS,
            Surrounder::Curly => SurroundInfo::CURLY,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct SurroundComponent: u8 {
        const TOP_LEFT     = 0b00000001;
        const TOP          = 0b00000010;
        const TOP_RIGHT    = 0b00000100;
        const LEFT         = 0b00001000;
        const RIGHT        = 0b00010000;
        const BOTTOM_LEFT  = 0b00100000;
        const BOTTOM       = 0b01000000;
        const BOTTOM_RIGHT = 0b10000000;

        const LEFT_EDGE    = 0b00101001;
        const RIGHT_EDGE   = 0b10010100;
        const TOP_EDGE     = 0b00000111;
        const BOTTOM_EDGE  = 0b11100000;

        const BOTH_EDGES   = 0b10111101;
    }
}

type Boldness = SurroundComponent;
type Inverted = SurroundComponent;

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    ch: char,
    bold: bool,
    invert: bool,
}

impl Display for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.bold {
            write!(f, "{}", style::Bold)?;
        }
        if self.invert {
            write!(f, "{}", style::Invert)?;
        }
        write!(f, "{}{}", self.ch, style::Reset)
    }
}

impl Pixel {
    pub const fn new(ch: char, bold: bool, invert: bool) -> Self {
        Self { ch, bold, invert }
    }

    const DEFAULT: Self = Pixel {
        ch: ' ',
        bold: false,
        invert: false,
    };

    const fn normal(ch: char) -> Self {
        Self {
            ch,
            bold: false,
            invert: false,
        }
    }
    //
    // const fn bold(ch: char) -> Self {
    //     Self { ch, bold: true }
    // }
}

trait Pixelable {
    fn pixel(self) -> Pixel;
    // fn bold(self) -> Pixel;
}

impl Pixelable for char {
    fn pixel(self) -> Pixel {
        Pixel::normal(self)
    }
    //
    // fn bold(self) -> Pixel {
    //     Pixel::bold(self)
    // }
}

#[derive(Debug)]
pub struct CharScreen {
    screen: Vec<Pixel>,
    width: usize,
    height: usize,
}

impl CharScreen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            screen: vec![Pixel::DEFAULT; width * height],
            width,
            height,
        }
    }

    fn calc_index(&self, pos: UVec2) -> usize {
        pos.x as usize + pos.y as usize * self.width
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn read(&self, pos: UVec2) -> Pixel {
        self.screen[self.calc_index(pos)]
    }

    fn write(&mut self, pos: UVec2, char: Pixel) {
        let index = self.calc_index(pos);
        self.screen[index] = char;
    }

    fn draw_rect(
        &mut self,
        offset: UVec2,
        size: UVec2,
        style: Surrounder,
        bold: Boldness,
        inverted: Inverted,
    ) {
        let info = style.surround_info();
        let outer = offset + size - UVec2::ONE;

        let mut write = |at: UVec2, ch: Option<char>, subset: Boldness| {
            if let Some(ch) = ch {
                self.write(
                    at,
                    Pixel::new(ch, bold.contains(subset), inverted.contains(subset)),
                );
            }
        };

        if size.y == 1 {
            write(offset, info.compressed.0, Boldness::LEFT_EDGE);
            write(outer, info.compressed.1, Boldness::RIGHT_EDGE);
        } else {
            for x in (offset.x + 1)..=(outer.x - 1) {
                write(offset.with_x(x), info.top.1, Boldness::TOP);
                write(outer.with_x(x), info.bottom.1, Boldness::BOTTOM);
            }
            for y in (offset.y + 1)..=(outer.y - 1) {
                write(offset.with_y(y), info.middle.0, Boldness::LEFT);
                write(outer.with_y(y), info.middle.1, Boldness::RIGHT);
            }
            let middle_y = offset.y + size.y / 2;
            write(
                offset.with_y(middle_y),
                info.insert_middle.0,
                Boldness::LEFT,
            );
            write(
                outer.with_y(middle_y),
                info.insert_middle.1,
                Boldness::RIGHT,
            );

            write(offset, info.top.0, Boldness::TOP_LEFT);
            write(offset.with_x(outer.x), info.top.2, Boldness::TOP_RIGHT);
            write(offset.with_y(outer.y), info.bottom.0, Boldness::BOTTOM_LEFT);
            write(outer, info.bottom.2, Boldness::BOTTOM_RIGHT);
        }
    }
}

impl Display for CharScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            let row_offset = y * self.width;
            for x in 0..self.width {
                let pixel = self.screen[x + row_offset];
                write!(f, "{}", pixel)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DebugTree {
    offset: UVec2,
    size: UVec2,
    baseline: u32,
    kind: DebugTreeKind,
}

#[derive(Debug)]
pub enum DebugTreeKind {
    Empty,
    Solid,
    Placeholder,
    HorizontalBar(Surrounder),
    Char(char),
    TwoChar([char; 2]),
    Surrounds(Surrounder, Boldness, Inverted, Box<DebugTree>),
    Children(Vec<DebugTree>),
    Power(Box<DebugTree>),
}

impl DebugTree {
    pub fn render(&self) -> CharScreen {
        let mut screen = CharScreen::new(self.size.x as usize, self.size.y as usize);
        self.render_to(&mut screen, UVec2::ZERO);
        screen
    }

    fn render_to(&self, screen: &mut CharScreen, offset: UVec2) {
        match self.kind {
            DebugTreeKind::Empty => {}
            DebugTreeKind::Power(ref child) => child.render_to(screen, offset + child.offset),
            DebugTreeKind::Solid => {
                for y in 0..self.size.y {
                    for x in 0..self.size.x {
                        screen.write(UVec2::new(x, y) + offset, '█'.pixel());
                    }
                }
            }
            DebugTreeKind::Placeholder => screen.write(offset, '𑑛'.pixel()),
            DebugTreeKind::HorizontalBar(surrounder) => {
                assert_eq!(self.size.y, 1);
                for x in 0..self.size.x {
                    screen.write(
                        UVec2::new(x, 0) + offset,
                        surrounder.surround_info().top.1.unwrap().pixel(),
                    );
                }
            }
            DebugTreeKind::Char(ch) => screen.write(offset, ch.pixel()),
            DebugTreeKind::TwoChar([ch1, ch2]) => {
                screen.write(offset, ch1.pixel());
                screen.write(offset + UVec2::X, ch2.pixel());
            }
            DebugTreeKind::Children(ref children) => children
                .iter()
                .for_each(|child| child.render_to(screen, offset + child.offset)),
            DebugTreeKind::Surrounds(surrounder, boldness, inverted, ref child) => {
                child.render_to(screen, offset + child.offset);
                screen.draw_rect(offset, self.size, surrounder, boldness, inverted);
            }
        }
    }
}

impl DebugTree {
    pub const fn new(size: UVec2, baseline: u32, kind: DebugTreeKind) -> Self {
        Self {
            offset: UVec2::ZERO,
            baseline,
            size,
            kind,
        }
    }

    pub fn surrounded(mut self, by: Surrounder, boldness: Boldness, inverted: Inverted) -> Self {
        self.offset += by.offset();
        Self::new(
            self.size + by.size(),
            self.baseline + by.size().y,
            DebugTreeKind::Surrounds(by, boldness, inverted, Box::new(self)),
        )
    }

    pub const fn horizontal_bar(width: u32, style: Surrounder) -> Self {
        Self::new(UVec2::new(width, 1), 0, DebugTreeKind::HorizontalBar(style))
    }

    pub fn power(self) -> Self {
        Self::new(
            self.size,
            self.baseline,
            DebugTreeKind::Power(Box::new(self)),
        )
    }

    pub const fn solid(size: UVec2, baseline: u32) -> Self {
        Self::new(size, baseline, DebugTreeKind::Solid)
    }

    pub const fn empty(size: UVec2, baseline: u32) -> Self {
        Self::new(size, baseline, DebugTreeKind::Empty)
    }

    pub const fn char(ch: char) -> Self {
        Self::new(UVec2::ONE, 0, DebugTreeKind::Char(ch))
    }

    pub const fn placeholder() -> Self {
        Self::new(UVec2::ONE, 0, DebugTreeKind::Placeholder)
    }

    pub const fn char2(chars: [char; 2]) -> Self {
        Self::new(UVec2::new(2, 1), 0, DebugTreeKind::TwoChar(chars))
    }

    pub fn horizontal(mut vec: Vec<DebugTree>) -> Self {
        let mut current_x = 0;
        let mut lowest_baseline = 0;
        let mut tallest_power = 0;
        for item in vec.iter_mut() {
            assert!(item.offset == UVec2::ZERO);
            item.offset.x += current_x;
            current_x += item.size.x;

            match item.kind {
                DebugTreeKind::Power(_) => tallest_power = tallest_power.max(item.size.y),
                _ => lowest_baseline = lowest_baseline.max(item.baseline),
            }
        }

        vec.iter_mut().for_each(|t| match t.kind {
            DebugTreeKind::Power(_) => t.offset.y = tallest_power - t.size.y,
            _ => t.offset.y = tallest_power + lowest_baseline - t.baseline,
        });

        let max_y = vec
            .iter()
            .map(|item| item.size.y + item.offset.y)
            .max()
            .unwrap_or(1);

        let mut last_lowest = 0;
        for item in vec.iter_mut() {
            if let DebugTreeKind::Power(child) = &mut item.kind {
                if let DebugTreeKind::Children(children) = &mut child.kind {
                    let first = &mut children[0];
                    if matches!(first.kind, DebugTreeKind::Solid) {
                        let size = last_lowest - child.offset.y;
                        first.size.y = size;
                    }
                }
            }

            last_lowest = item.offset.y + item.size.y;
        }

        Self::new(
            UVec2::new(current_x, max_y),
            lowest_baseline,
            DebugTreeKind::Children(vec),
        )
    }

    pub fn vertical(mut vec: Vec<DebugTree>) -> Self {
        let middle_item = vec.len() / 2;

        let mut current_y = 0;
        let mut max_x = 0;
        let mut baseline = 0;
        for (index, item) in vec.iter_mut().enumerate() {
            assert!(item.offset == UVec2::ZERO);
            item.offset.y += current_y;
            current_y += item.size.y;

            max_x = max_x.max(item.size.x);

            if index == middle_item {
                baseline = item.baseline + item.offset.y;
            }
        }
        vec.iter_mut()
            .for_each(|t| t.offset.x = (max_x - t.size.x) / 2);

        Self::new(
            UVec2::new(max_x, current_y),
            baseline,
            DebugTreeKind::Children(vec),
        )
    }
}

pub trait Debugable {
    fn debug(&self, with_cursor: bool) -> DebugTree;
}

impl Debugable for EditorTreeSeqNormal {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        let is_cursor_last = self.cursor() == self.children().len() && with_cursor;
        let mut nodes = Vec::with_capacity(self.children().len().max(1) + is_cursor_last as usize);

        for (index, child) in self.children().iter().enumerate() {
            nodes.push(child.debug(with_cursor && index == self.cursor()));
        }
        if self.children().is_empty() {
            nodes.push(DebugTree::placeholder());
        }
        let mut debug_tree = DebugTree::horizontal(nodes);

        if is_cursor_last {
            let DebugTreeKind::Children(children) = &mut debug_tree.kind else {
                unreachable!()
            };

            let last_item = children.last().unwrap();
            let last_baseline = last_item.baseline + last_item.offset.y;

            let x_pos = debug_tree.size.x;
            let mut cursor = DebugTree::solid(UVec2::new(1, debug_tree.size.y), last_baseline);
            cursor.offset.x = x_pos;
            children.push(cursor);
            debug_tree.size.x += 1;
        };

        debug_tree
    }
}

impl Debugable for EditorTreeTerminal {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        if with_cursor {
            DebugTree::char2(['█', self.ch()])
        } else {
            DebugTree::char(self.ch())
        }
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreeFraction<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        let top = self
            .top()
            .debug(with_cursor && self.cursor() == FractionIndex::Top);
        let bottom = self
            .bottom()
            .debug(with_cursor && self.cursor() == FractionIndex::Bottom);
        let bar = DebugTree::horizontal_bar(top.size.x.max(bottom.size.x), Surrounder::Rectangle);

        let tree = DebugTree::vertical(vec![top, bar, bottom]);

        if with_cursor && self.cursor() == FractionIndex::Left {
            let cursor = DebugTree::solid(UVec2::new(1, tree.size.y), tree.baseline);
            DebugTree::horizontal(vec![cursor, tree])
        } else {
            tree
        }
    }
}

fn debug_completable_surrounds<T, S>(
    tree: &T,
    with_cursor: bool,
    surrounder: Surrounder,
) -> DebugTree
where
    T: CompletableSurrounds + SurroundsTreeSeq<Seq = S>,
    S: EditorTreeSeq + Debugable,
{
    let debug_tree = tree
        .child()
        .debug(with_cursor && tree.cursor() == SurroundIndex::Inside)
        .surrounded(
            surrounder,
            match tree.is_complete() {
                true => Boldness::empty(),
                false => Boldness::empty(),
            },
            match tree.is_complete() {
                true => Inverted::empty(),
                false => Inverted::RIGHT_EDGE,
            },
        );

    if with_cursor && tree.cursor() == SurroundIndex::Left {
        DebugTree::horizontal(vec![
            DebugTree::solid(UVec2::new(1, debug_tree.size.y), debug_tree.baseline),
            debug_tree,
        ])
    } else {
        debug_tree
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreeParen<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        debug_completable_surrounds(self, with_cursor, Surrounder::Parens)
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreeAbs<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        debug_completable_surrounds(self, with_cursor, Surrounder::Abs)
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreeBracket<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        debug_completable_surrounds(self, with_cursor, Surrounder::Brackets)
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreeCurly<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        debug_completable_surrounds(self, with_cursor, Surrounder::Curly)
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreeSqrt<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        let tree = self
            .child()
            .debug(with_cursor && self.cursor() == SurroundIndex::Inside)
            .surrounded(Surrounder::Sqrt, Boldness::all(), Inverted::empty());
        if with_cursor && self.cursor() == SurroundIndex::Left {
            DebugTree::horizontal(vec![
                DebugTree::solid(UVec2::new(1, tree.size.y), tree.baseline),
                tree,
            ])
        } else {
            tree
        }
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreeSumProd<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        let bottom_row = DebugTree::horizontal(vec![
            self.ident()
                .debug(with_cursor && self.cursor() == SumProdIndex::BottomIdent),
            DebugTree::char('='),
            self.bottom()
                .debug(with_cursor && self.cursor() == SumProdIndex::BottomExpr),
        ]);
        let result = DebugTree::vertical(vec![
            self.top()
                .debug(with_cursor && self.cursor() == SumProdIndex::Top),
            DebugTree::char(match self.sum_or_prod() {
                SumOrProd::Sum => '∑',
                SumOrProd::Prod => '∏',
            }),
            bottom_row,
        ]);

        match (self.cursor(), with_cursor) {
            (SumProdIndex::Left, true) => DebugTree::horizontal(vec![
                DebugTree::solid(UVec2::new(1, result.size.y), result.baseline),
                result,
            ]),
            _ => result,
        }
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreePower<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        let power = self
            .child()
            .debug(with_cursor && self.cursor() == SurroundIndex::Inside);
        if with_cursor && self.cursor() == SurroundIndex::Left {
            DebugTree::horizontal(vec![
                DebugTree::solid(UVec2::new(1, power.size.y), power.baseline),
                power,
            ])
        } else {
            power
        }
        .power()
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTree<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        match self.kind() {
            EditorTreeKind::Terminal(term) => term.debug(with_cursor),
            EditorTreeKind::Power(power) => power.debug(with_cursor),
            EditorTreeKind::Fraction(fraction) => fraction.debug(with_cursor),
            EditorTreeKind::Sqrt(sqrt) => sqrt.debug(with_cursor),
            EditorTreeKind::Paren(paren) => paren.debug(with_cursor),
            EditorTreeKind::Abs(abs) => abs.debug(with_cursor),
            EditorTreeKind::Bracket(bracket) => bracket.debug(with_cursor),
            EditorTreeKind::Curly(curly) => curly.debug(with_cursor),
            EditorTreeKind::SumProd(sum_prod) => sum_prod.debug(with_cursor),
        }
    }
}
