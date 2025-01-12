use std::fmt::Display;

use bitflags::bitflags;
use glam::UVec2;

use fast_desmos2_tree::tree::{
    CompletableSurrounds, EditorTree, EditorTreeAbs, EditorTreeBracket, EditorTreeFraction,
    EditorTreeKind, EditorTreeParen, EditorTreeSeq, EditorTreeSeqNormal, EditorTreeSqrt,
    EditorTreeSumProd, EditorTreeTerminal, FractionIndex, SumOrProd, SumProdIndex, SurroundIndex,
    SurroundsTreeSeq,
};
use termion::style;

#[derive(Debug, Clone, Copy)]
pub enum Surrounder {
    Rectangle,

    Sqrt,

    Abs,
    Parens,
    Brackets,
}

#[derive(Clone, Copy)]
struct SurroundInfo {
    top: (Option<char>, Option<char>, Option<char>),
    middle: (Option<char>, Option<char>),
    bottom: (Option<char>, Option<char>, Option<char>),

    compressed: (Option<char>, Option<char>),
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
    );
    const SQRT: Self = surround_info!(
        '┌''─''─'
        '│'    .
        '│' .  .

        '√'    .
    );
    const BRACKETS: Self = surround_info!(
        '┌' . '┐'
        '│'   '│'
        '└' . '┘'

        '['   ']'
    );
    const ABS: Self = surround_info!(
        '│' . '│'
        '│'   '│'
        '│' . '│'

        '|'   '|'
    );
    const PARENS: Self = surround_info!(
        '╭' . '╮'
        '│'   '│'
        '╰' . '╯'

        '('   ')'
    );

    const fn new([a, b, c, d, e, f, g, h, i, j]: [Option<char>; 10]) -> Self {
        Self {
            top: (a, b, c),
            middle: (d, e),
            bottom: (f, g, h),
            compressed: (i, j),
        }
    }
}

impl Surrounder {
    const fn offset(&self) -> UVec2 {
        match self {
            Surrounder::Rectangle | Surrounder::Sqrt => UVec2::ONE,
            Surrounder::Abs | Surrounder::Parens | Surrounder::Brackets => UVec2::X,
        }
    }

    const fn size(&self) -> UVec2 {
        match self {
            Surrounder::Rectangle => UVec2::splat(2),
            Surrounder::Sqrt => UVec2::ONE,
            Surrounder::Abs | Surrounder::Parens | Surrounder::Brackets => UVec2::new(2, 0),
        }
    }

    const fn surround_info(&self) -> SurroundInfo {
        match self {
            Surrounder::Rectangle => SurroundInfo::RECTANGLE,
            Surrounder::Sqrt => SurroundInfo::SQRT,
            Surrounder::Abs => SurroundInfo::ABS,
            Surrounder::Parens => SurroundInfo::PARENS,
            Surrounder::Brackets => SurroundInfo::BRACKETS,
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
    kind: DebugTreeKind,
}

#[derive(Debug)]
pub enum DebugTreeKind {
    Solid,
    Placeholder,
    HorizontalBar(Surrounder),
    Char(char),
    TwoChar([char; 2]),
    Surrounds(Surrounder, Boldness, Inverted, Box<DebugTree>),
    Children(Vec<DebugTree>),
}

impl DebugTree {
    pub fn render(&self) -> CharScreen {
        let mut screen = CharScreen::new(self.size.x as usize, self.size.y as usize);
        self.render_to(&mut screen, UVec2::ZERO);
        screen
    }

    fn render_to(&self, screen: &mut CharScreen, offset: UVec2) {
        match self.kind {
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
    pub const fn new(size: UVec2, kind: DebugTreeKind) -> Self {
        Self {
            offset: UVec2::ZERO,
            size,
            kind,
        }
    }

    pub fn surrounded(mut self, by: Surrounder, boldness: Boldness, inverted: Inverted) -> Self {
        self.offset += by.offset();
        Self {
            size: self.size + by.size(),
            offset: UVec2::ZERO,
            kind: DebugTreeKind::Surrounds(by, boldness, inverted, Box::new(self)),
        }
    }

    pub const fn horizontal_bar(width: u32, style: Surrounder) -> Self {
        Self::new(UVec2::new(width, 1), DebugTreeKind::HorizontalBar(style))
    }

    pub const fn solid(size: UVec2) -> Self {
        Self::new(size, DebugTreeKind::Solid)
    }

    pub const fn char(ch: char) -> Self {
        Self::new(UVec2::ONE, DebugTreeKind::Char(ch))
    }

    pub const fn placeholder() -> Self {
        Self::new(UVec2::ONE, DebugTreeKind::Placeholder)
    }

    pub const fn char2(chars: [char; 2]) -> Self {
        Self::new(UVec2::new(2, 1), DebugTreeKind::TwoChar(chars))
    }

    pub fn horizontal(mut vec: Vec<DebugTree>) -> Self {
        let mut current_x = 0;
        let mut max_y = 0;
        for item in vec.iter_mut() {
            assert!(item.offset == UVec2::ZERO);
            item.offset.x += current_x;
            current_x += item.size.x;

            max_y = max_y.max(item.size.y);
        }
        vec.iter_mut()
            .for_each(|t| t.offset.y = (max_y - t.size.y) / 2);

        Self::new(UVec2::new(current_x, max_y), DebugTreeKind::Children(vec))
    }

    pub fn vertical(mut vec: Vec<DebugTree>) -> Self {
        let mut current_y = 0;
        let mut max_x = 0;
        for item in vec.iter_mut() {
            assert!(item.offset == UVec2::ZERO);
            item.offset.y += current_y;
            current_y += item.size.y;

            max_x = max_x.max(item.size.x);
        }
        vec.iter_mut()
            .for_each(|t| t.offset.x = (max_x - t.size.x) / 2);

        Self::new(UVec2::new(max_x, current_y), DebugTreeKind::Children(vec))
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

        if is_cursor_last {
            let max_y = nodes.iter().map(|node| node.size.y).max().unwrap_or(1);
            nodes.push(DebugTree::solid(UVec2::new(1, max_y)))
        }

        DebugTree::horizontal(nodes)
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
            let cursor = DebugTree::solid(UVec2::new(1, tree.size.y));
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
            DebugTree::solid(UVec2::new(1, debug_tree.size.y)),
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

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTreeSqrt<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        let tree = self
            .child()
            .debug(with_cursor && self.cursor() == SurroundIndex::Inside)
            .surrounded(Surrounder::Sqrt, Boldness::all(), Inverted::empty());
        if with_cursor && self.cursor() == SurroundIndex::Left {
            DebugTree::horizontal(vec![DebugTree::solid(UVec2::new(1, tree.size.y)), tree])
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
            (SumProdIndex::Left, true) => {
                DebugTree::horizontal(vec![DebugTree::solid(UVec2::new(1, result.size.y)), result])
            }
            _ => result,
        }
    }
}

impl<S: EditorTreeSeq + Debugable> Debugable for EditorTree<S> {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        match self.kind() {
            EditorTreeKind::Terminal(term) => term.debug(with_cursor),
            EditorTreeKind::Power(_) => todo!(),
            EditorTreeKind::Fraction(fraction) => fraction.debug(with_cursor),
            EditorTreeKind::Sqrt(sqrt) => sqrt.debug(with_cursor),
            EditorTreeKind::Paren(paren) => paren.debug(with_cursor),
            EditorTreeKind::Abs(abs) => abs.debug(with_cursor),
            EditorTreeKind::Bracket(bracket) => bracket.debug(with_cursor),
            EditorTreeKind::Curly(_) => todo!(),
            EditorTreeKind::SumProd(sum_prod) => sum_prod.debug(with_cursor),
        }
    }
}
