use crate::tree::{EditorTree, EditorTreeSeq, TreeMov};

#[test]
fn seq_move() {
    let mut tree = EditorTreeSeq::new(0, vec![
        EditorTree::terminal(0, "1".to_string()),
        EditorTree::terminal(0, "23".to_string()),
        EditorTree::terminal(0, "3".to_string()),
    ]);

    assert_eq!(tree.cursor(), 0);
    assert_eq!(tree.active_child().unwrap().cursor(), 0);

    assert_eq!(tree.apply_movement(TreeMov::Right), None);
    assert_eq!(tree.cursor(), 1);
    assert_eq!(tree.active_child().unwrap().cursor(), 0);

    assert_eq!(tree.apply_movement(TreeMov::Right), None);
    assert_eq!(tree.cursor(), 1);
    assert_eq!(tree.active_child().unwrap().cursor(), 1);

    assert_eq!(tree.apply_movement(TreeMov::Right), None);
    assert_eq!(tree.cursor(), 2);
    assert_eq!(tree.active_child().unwrap().cursor(), 0);

    assert_eq!(tree.apply_movement(TreeMov::Right), None);
    assert_eq!(tree.cursor(), 3);
    assert_eq!(tree.active_child(), None);

    assert_eq!(tree.apply_movement(TreeMov::Right), Some(TreeMov::Right));
}
