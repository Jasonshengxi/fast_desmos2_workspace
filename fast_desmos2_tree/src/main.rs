use fast_desmos2_tree::tree::{
    debug::Debugable as _, Direction, EditorTree as T, EditorTreeSeq as TS, FractionIndex, Motion,
    TreeAction, TreeMovable,
};
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

fn make_stdout() -> RawTerminal<Stdout> {
    std::io::stdout()
        .into_raw_mode()
        .expect("Can enter raw mode")
    // std::io::stdout()
}

enum EditorMode {
    Normal,
    Insert,
}

impl Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorMode::Normal => write!(f, "NORMAL"),
            EditorMode::Insert => write!(f, "INSERT"),
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut stdout = make_stdout();

    #[rustfmt::skip]
    let mut tree = TS::new(
        0,
        vec![
            T::terminal('T'),
            T::fraction(
                FractionIndex::Bottom,
                TS::new(
                    0,
                    vec![
                        T::terminal('f'),
                        T::fraction(
                            FractionIndex::Bottom,
                            TS::one(T::terminal('A')),
                            TS::one(T::terminal('B')),
                        ),
                    ],
                ),
                TS::one(T::fraction(
                    FractionIndex::Bottom,
                    TS::one(T::terminal('M')),
                    TS::one(T::terminal('L')),
                )),
            ),
        ],
    );
    // #[rustfmt::skip]
    // let mut tree = TS::new(
    //     3,
    //     vec![
    //         T::str("a"),
    //         T::str("b"),
    //         T::str("c"),
    //     ]
    // );

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

        let t = &mut tree;
        match mode {
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
        screen.display_raw(&mut stdout, UVec2::new(0, 1))?;
        stdout.flush()?;
    }

    Ok(())
}
