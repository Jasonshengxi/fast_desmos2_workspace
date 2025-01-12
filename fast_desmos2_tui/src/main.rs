#![allow(unused)]
use draw::{CharScreen, Debugable as _};
use fast_desmos2_cranelift::compile;
use fast_desmos2_eval::IdentStorer;
use fast_desmos2_tree::tree::{
    Direction, EditorTree as T, EditorTreeSeqNormal as TS, FractionIndex, Motion, TreeAction,
    TreeMovable,
};
use fast_desmos2_tree_parser as tree_parser;
use glam::UVec2;
use std::{
    fmt::Display,
    io::{Stdout, Write},
};
use termion::{
    clear, color, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
};

mod draw;

fn make_stdout() -> RawTerminal<Stdout> {
    std::io::stdout()
        .into_raw_mode()
        .expect("Can enter raw mode")
    // std::io::stdout()
}

enum EditorMode {
    Normal,
    Insert,
    Leader,
}

impl Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorMode::Normal => write!(f, "NORMAL"),
            EditorMode::Insert => write!(f, "INSERT"),
            EditorMode::Leader => write!(f, "LEADER"),
        }
    }
}

pub fn display_raw(
    screen: &CharScreen,
    to: &mut termion::raw::RawTerminal<std::io::Stdout>,
    offset: UVec2,
) -> std::io::Result<()> {
    use termion::cursor;
    for y in 0..screen.height() {
        let row_start = offset.with_y(offset.y + y as u32);
        write!(
            to,
            "{}",
            cursor::Goto(row_start.x as u16 + 1, row_start.y as u16 + 1)
        )?;
        for x in 0..screen.width() {
            write!(to, "{}", screen.read(UVec2::new(x as u32, y as u32)))?;
        }
    }
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    let mut stdout = make_stdout();

    #[rustfmt::skip]
    let mut tree = TS::empty();

    let mut mode = EditorMode::Normal;

    for key in std::iter::once(Ok(Key::Esc)).chain(std::io::stdin().keys()) {
        let key = key.unwrap();
        if let Key::Ctrl('c') = key {
            break;
        }

        fn apply_action(tree: &mut TS, action: TreeAction) {
            tree.apply_action(action);
        }

        fn apply_move(tree: &mut TS, movement: Motion) {
            tree.apply_move(movement);
        }

        let mut extra_text = String::new();

        let t = &mut tree;
        match mode {
            EditorMode::Leader => {
                match key {
                    Key::Char('e') => {
                        let idents = IdentStorer::default();
                        let parsed = tree_parser::parse(&tree, &idents);
                        match parsed {
                            Ok(node) => {
                                let func = compile(&node);
                                let output = func();
                                extra_text = format!("{output:#?}");
                            }
                            Err(err) => extra_text = format!("{err:#?}"),
                        }
                    }
                    Key::Char('q') => extra_text = format!("{tree:#?}"),
                    Key::Char('p') => {
                        let idents = IdentStorer::default();
                        let parsed = tree_parser::parse(&tree, &idents);
                        match parsed {
                            Ok(node) => extra_text = format!("{node:#?}"),
                            Err(err) => extra_text = format!("{err:#?}"),
                        }
                    }
                    Key::Char('t') => {
                        extra_text = format!("Some test text\nhehe");
                    }
                    _ => {}
                }
                mode = EditorMode::Normal;
            }
            EditorMode::Normal => match key {
                Key::Char('h') => apply_move(t, Motion::Left),
                Key::Char('j') => apply_move(t, Motion::Down),
                Key::Char('k') => apply_move(t, Motion::Up),
                Key::Char('l') => apply_move(t, Motion::Right),

                Key::Char('w') => apply_move(t, Motion::Word),
                Key::Char('b') => apply_move(t, Motion::Back),
                Key::Char('$') => apply_move(t, Motion::Last),
                Key::Char('^') => apply_move(t, Motion::First),

                Key::Char('0') => t.enter_from(Direction::Left),

                Key::Char('i') => mode = EditorMode::Insert,
                Key::Char(' ') => mode = EditorMode::Leader,
                // Key::Char('x') => apply_action(tree, TreeAction::Delete),
                _ => {}
            },
            EditorMode::Insert => match key {
                Key::Esc => mode = EditorMode::Normal,

                Key::Backspace => apply_action(t, TreeAction::Delete),
                Key::Char(c) => apply_action(t, TreeAction::from_char(c)),

                Key::Left => apply_move(t, Motion::Left),
                Key::Down => apply_move(t, Motion::Down),
                Key::Up => apply_move(t, Motion::Up),
                Key::Right => apply_move(t, Motion::Right),
                _ => {}
            },
        }

        let debug_tree = tree.debug(true);
        let screen = debug_tree.render();
        write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1))?;
        write!(
            stdout,
            "{}-- {mode} --{}",
            color::Fg(color::Green),
            color::Fg(color::Reset)
        )?;
        display_raw(&screen, &mut stdout, UVec2::new(0, 1))?;
        write!(stdout, "{}", cursor::Goto(0, 3 + screen.height() as u16))?;

        drop(stdout);
        println!("{extra_text}");
        stdout = make_stdout();

        stdout.flush()?;
    }

    Ok(())
}
