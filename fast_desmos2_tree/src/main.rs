use fast_desmos2_tree::tree::{EditorTree as T, EditorTreeSeq as TS, TreeMove};
use glam::UVec2;
use std::io::{Stdout, Write};
use termion::{
    clear, cursor,
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

fn main() -> Result<(), std::io::Error> {
    let mut stdout = make_stdout();

    #[rustfmt::skip]
    let mut tree = TS::new(
        0,
        vec![
            T::str("Text"),
            T::fraction(
                T::FRACTION_BOTTOM, 
                TS::one(T::str("H")), 
                TS::one(T::fraction(
                    T::FRACTION_BOTTOM, 
                    TS::one(T::str("M")),
                    TS::one(T::str("L"))
                ))
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

    let debug_tree = tree.debug(true);
    let screen = debug_tree.render();
    write!(stdout, "{}", clear::All)?;
    screen.display_raw(&mut stdout, UVec2::ZERO)?;
    stdout.flush()?;

    for key in std::io::stdin().keys() {
        let key = key.unwrap();
        match key {
            Key::Ctrl('c') => break,
            Key::Char('h') => tree.apply_move(TreeMove::Left),
            Key::Char('j') => tree.apply_move(TreeMove::Down),
            Key::Char('k') => tree.apply_move(TreeMove::Up),
            Key::Char('l') => tree.apply_move(TreeMove::Right),
            _ => None,
        };

        let debug_tree = tree.debug(true);
        let screen = debug_tree.render();
        write!(stdout, "{}", clear::All)?;
        screen.display_raw(&mut stdout, UVec2::ZERO)?;
        stdout.flush()?;
    }

    Ok(())
}
