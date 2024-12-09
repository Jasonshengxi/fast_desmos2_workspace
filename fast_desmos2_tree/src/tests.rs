use crate::tree::{EditorTree as T, EditorTreeSeq as TS, TreeMove};

macro_rules! assert_cursors {
    ($tree:ident, $first_cursor:tt $(, $($cursor:tt),*)?) => {
        {
            let tree = &$tree;
            #[allow(unused_variables)]
            let counter = 0;
            assert_eq!(tree.cursor(), $first_cursor, "Top level incorrect.");
            $($(
                let tree = tree.active_child().unwrap();
                let counter = counter + 1;
                assert_eq!(tree.cursor(), $cursor, "Layer {counter} incorrect: {tree:?}");
            )*)?
            assert!(tree.active_child().is_none(), "Tree does not end where expected to. Got {tree:?}");
        }
    };
}

#[test]
fn seq_move() {
    let mut tree = TS::new(
        0,
        vec![
            T::terminal(0, "1".to_string()),
            T::terminal(0, "23".to_string()),
            T::terminal(0, "3".to_string()),
        ],
    );

    assert_cursors!(tree, 0, 0);

    assert_eq!(tree.apply_move(TreeMove::Right), None);
    assert_cursors!(tree, 1, 0);

    assert_eq!(tree.apply_move(TreeMove::Right), None);
    assert_cursors!(tree, 1, 1);

    assert_eq!(tree.apply_move(TreeMove::Right), None);
    assert_cursors!(tree, 2, 0);

    assert_eq!(tree.apply_move(TreeMove::Right), None);
    assert_cursors!(tree, 3);

    assert_eq!(tree.apply_move(TreeMove::Right), Some(TreeMove::Right));
}

#[test]
fn right_to_left() {
    let mut tree = TS::new(
        3,
        vec![
            T::terminal(0, "1".to_string()),
            T::terminal(0, "2".to_string()),
            T::terminal(0, "3".to_string()),
        ],
    );

    assert_cursors!(tree, 3);

    assert_eq!(tree.apply_move(TreeMove::Left), None);
    assert_cursors!(tree, 2, 0);

    assert_eq!(tree.apply_move(TreeMove::Left), None);
    assert_cursors!(tree, 1, 0);

    assert_eq!(tree.apply_move(TreeMove::Left), None);
    assert_cursors!(tree, 0, 0);

    assert_eq!(tree.apply_move(TreeMove::Left), Some(TreeMove::Left));
    assert_cursors!(tree, 0, 0);
}

#[test]
fn frac_moves() {
    #[rustfmt::skip]
    let mut tree = TS::new(
        0,
        vec![
            T::fraction(
                0, 
                TS::one(T::str("H")), 
                TS::one(T::fraction(
                    0,
                    TS::one(T::str("M")),
                    TS::one(T::str("L"))
                ))
            ),
        ],
    );

    assert_cursors!(tree, 0, 0, 0, 0, 0, 0);

    assert_eq!(tree.apply_move(TreeMove::Right), None);
    assert_cursors!(tree, 0, 0, 0, 0, 1);

    assert_eq!(tree.apply_move(TreeMove::Right), None);
    assert_cursors!(tree, 0, 0, 1);

    assert_eq!(tree.apply_move(TreeMove::Right), None);
    assert_cursors!(tree, 1);

    assert_eq!(tree.apply_move(TreeMove::Right), Some(TreeMove::Right));
    assert_cursors!(tree, 1);

    assert_eq!(tree.apply_move(TreeMove::Left), None);
    assert_cursors!(tree, 0, 0, 1);

    assert_eq!(tree.apply_move(TreeMove::Left), None);
    assert_cursors!(tree, 0, 0, 0, 0, 1);

    assert_eq!(tree.apply_move(TreeMove::Left), None);
    assert_cursors!(tree, 0, 0, 0, 0, 0, 0);
}
