use std::{fmt::Display, io::Write};

use glam::UVec2;

use super::{
    EditorTree, EditorTreeFraction, EditorTreeKind as TK, EditorTreePower, EditorTreeSeq,
    EditorTreeTerminal, FractionIndex, PowerIndex,
};

trait RectStyle {
    const LINE_Y: char;
    const LINE_X: char;
    const CORNER_UL: char;
    const CORNER_UR: char;
    const CORNER_DL: char;
    const CORNER_DR: char;
}

// box drawers: ─━│┃┄┅┆┇┈┉┊┋┌┍┎┏┐┑┒┓└┕┖┗┘┙┚┛├┝┞┟┠┡┢┣┤┥┦┧┨┩┪┫┬┭┮┯┰┱┲┳┴┵┶┷┸┹┺┻┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋╌╍╎╏═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬╭╮╯╰╱╲╳╴╵╶╷╸╹╺╻╼╽╾╿
struct NormalRect;
impl RectStyle for NormalRect {
    const LINE_Y: char = '│';
    const LINE_X: char = '─';
    const CORNER_UL: char = '┌';
    const CORNER_UR: char = '┐';
    const CORNER_DL: char = '└';
    const CORNER_DR: char = '┘';
}

struct WeakRect;
impl RectStyle for WeakRect {
    const LINE_Y: char = '┆';
    const LINE_X: char = '┄';
    const CORNER_UL: char = '╭';
    const CORNER_UR: char = '╮';
    const CORNER_DL: char = '╰';
    const CORNER_DR: char = '╯';
}

// box drawers: ─━│┃┄┅┆┇┈┉┊┋┌┍┎┏┐┑┒┓└┕┖┗┘┙┚┛├┝┞┟┠┡┢┣┤┥┦┧┨┩┪┫┬┭┮┯┰┱┲┳┴┵┶┷┸┹┺┻┼┽┾┿╀╁╂╃╄╅╆╇╈╉╊╋╌╍╎╏═║╒╓╔╕╖╗╘╙╚╛╜╝╞╟╠╡╢╣╤╥╦╧╨╩╪╫╬╭╮╯╰╱╲╳╴╵╶╷╸╹╺╻╼╽╾╿
struct BoldRect;
impl RectStyle for BoldRect {
    const LINE_Y: char = '┃';
    const LINE_X: char = '━';
    const CORNER_UL: char = '┏';
    const CORNER_UR: char = '┓';
    const CORNER_DL: char = '┗';
    const CORNER_DR: char = '┛';
}

#[derive(Debug, Clone, Copy)]
pub enum RectStyles {
    Normal,
    Bold,
    Weak,
}

#[derive(Debug)]
pub struct CharScreen {
    screen: Vec<char>,
    width: usize,
    height: usize,
}

impl CharScreen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            screen: vec![' '; width * height],
            width,
            height,
        }
    }

    fn calc_index(&self, pos: UVec2) -> usize {
        pos.x as usize + pos.y as usize * self.width
    }

    fn write(&mut self, pos: UVec2, char: char) {
        let index = self.calc_index(pos);
        self.screen[index] = char;
    }

    fn draw_rect<S: RectStyle>(&mut self, offset: UVec2, size: UVec2) {
        let outer = offset + size - 1;
        self.write(offset, S::CORNER_UL);
        self.write(offset.with_x(outer.x), S::CORNER_UR);
        self.write(offset.with_y(outer.y), S::CORNER_DL);
        self.write(outer, S::CORNER_DR);

        for x in (offset.x + 1)..outer.x {
            self.write(offset.with_x(x), S::LINE_X);
            self.write(outer.with_x(x), S::LINE_X);
        }

        for y in (offset.y + 1)..outer.y {
            self.write(offset.with_y(y), S::LINE_Y);
            self.write(outer.with_y(y), S::LINE_Y);
        }
    }

    fn draw_rect_styled(&mut self, offset: UVec2, size: UVec2, style: RectStyles) {
        match style {
            RectStyles::Bold => self.draw_rect::<BoldRect>(offset, size),
            RectStyles::Normal => self.draw_rect::<NormalRect>(offset, size),
            RectStyles::Weak => self.draw_rect::<WeakRect>(offset, size),
        }
    }

    #[cfg(feature = "binary")]
    pub fn display_raw(
        &self,
        to: &mut termion::raw::RawTerminal<std::io::Stdout>,
        offset: UVec2,
    ) -> std::io::Result<()> {
        use termion::cursor;
        for y in 0..self.height {
            let row_start = offset.with_y(offset.y + y as u32);
            write!(
                to,
                "{}",
                cursor::Goto(row_start.x as u16 + 1, row_start.y as u16 + 1)
            )?;
            for x in 0..self.width {
                write!(
                    to,
                    "{}",
                    self.screen[self.calc_index(UVec2::new(x as u32, y as u32))]
                )?;
            }
        }
        Ok(())
    }
}

impl Display for CharScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            let row_offset = y * self.width;
            for x in 0..self.width {
                let char = self.screen[x + row_offset];
                write!(f, "{}", char)?;
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
    Empty,
    Solid,
    Text(String),
    BoxAround(RectStyles, Box<DebugTree>),
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
                        screen.write(UVec2::new(x, y) + offset, '█');
                    }
                }
            }
            DebugTreeKind::Text(ref string) => {
                screen.write(offset, '[');
                let mut len = 0;
                for (index, char) in string.chars().enumerate() {
                    screen.write(UVec2::new(index as u32, 0) + UVec2::X + offset, char);
                    len = index;
                }
                screen.write(UVec2::new(len as u32 + 2, 0) + offset, ']');
            }
            DebugTreeKind::BoxAround(style, ref child) => {
                screen.draw_rect_styled(offset, self.size, style);
                child.render_to(screen, offset + child.offset);
            }
            DebugTreeKind::Empty => {}
            DebugTreeKind::Children(ref children) => children
                .iter()
                .for_each(|child| child.render_to(screen, offset + child.offset)),
        }
    }
}

impl DebugTree {
    pub fn new(size: UVec2, kind: DebugTreeKind) -> Self {
        Self {
            offset: UVec2::ZERO,
            size,
            kind,
        }
    }

    pub fn boxed(mut self, style: RectStyles) -> Self {
        self.offset += UVec2::ONE;
        Self::new(
            self.size + UVec2::splat(2),
            DebugTreeKind::BoxAround(style, Box::new(self)),
        )
    }

    pub fn solid(size: UVec2) -> Self {
        Self::new(size, DebugTreeKind::Solid)
    }

    pub fn empty(size: UVec2) -> Self {
        Self::new(size, DebugTreeKind::Empty)
    }

    pub fn text(string: String) -> Self {
        Self::new(
            UVec2::new(string.chars().count() as u32 + 2, 1),
            DebugTreeKind::Text(string),
        )
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

impl Debugable for EditorTreeSeq {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        let is_cursor_last = self.cursor == self.children.len() && with_cursor;
        let mut nodes = Vec::with_capacity(self.children.len() + is_cursor_last as usize);

        for (index, child) in self.children.iter().enumerate() {
            nodes.push(child.debug(with_cursor && index == self.cursor));
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
        DebugTree::text(if with_cursor {
            let mut string = self.string.clone();
            string.insert(self.byte_cursor(), '█');
            string
        } else {
            self.string.clone()
        })
    }
}

impl Debugable for EditorTreePower {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        DebugTree::horizontal(vec![
            self.base
                .debug(with_cursor && self.cursor == PowerIndex::Base),
            self.power
                .debug(with_cursor && self.cursor == PowerIndex::Power),
        ])
        .boxed(RectStyles::Bold)
    }
}

impl Debugable for EditorTreeFraction {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        let mut vec = vec![DebugTree::vertical(vec![
            self.top
                .debug(with_cursor && self.cursor == FractionIndex::Top),
            self.bottom
                .debug(with_cursor && self.cursor == FractionIndex::Bottom),
        ])];
        if with_cursor && self.cursor == FractionIndex::Left {
            vec.insert(0, DebugTree::solid(UVec2::new(1, vec[0].size.y)));
        }
        DebugTree::horizontal(vec).boxed(RectStyles::Bold)
    }
}

impl Debugable for EditorTree {
    fn debug(&self, with_cursor: bool) -> DebugTree {
        match &self.kind {
            TK::Terminal(term) => term.debug(with_cursor),
            TK::Power(power) => power.debug(with_cursor),
            TK::Fraction(fraction) => fraction.debug(with_cursor),
        }
    }
}
