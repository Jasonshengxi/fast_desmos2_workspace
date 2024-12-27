use crate::tree::{
    CombinedCursor::Terminal as TM, EditorTree as T, EditorTreeSeq as TS, Motion, TreeMovable as _,
};

macro_rules! assert_cursors {
    ($tree:ident, $first_cursor:expr $(, $($cursor:expr),*)?) => {
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
fn right_to_left() {
    let mut tree = TS::new(
        3,
        vec![T::terminal('1'), T::terminal('2'), T::terminal('3')],
    );

    assert_cursors!(tree, 3);

    assert_eq!(tree.apply_move(Motion::Left), None);
    assert_cursors!(tree, 2, TM);

    assert_eq!(tree.apply_move(Motion::Left), None);
    assert_cursors!(tree, 1, TM);

    assert_eq!(tree.apply_move(Motion::Left), None);
    assert_cursors!(tree, 0, TM);

    assert_eq!(tree.apply_move(Motion::Left), Some(Motion::Left));
    assert_cursors!(tree, 0, TM);
}
