use std::io::Write;

use fast_desmos2_eval::IdentStorer;
use fast_desmos2_tree::tree::EditorTreeSeqNormal;
use fast_desmos2_tree_parser::parse;

fn main() {
    let mut line = String::new();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    loop {
        print!("=> ");
        stdout.flush().unwrap();
        line.clear();
        stdin.read_line(&mut line).unwrap();
        execute(line.trim());
    }
}

fn execute(source: &str) {
    let source = EditorTreeSeqNormal::str(source);
    let parsed = parse(&source, &IdentStorer::default()).unwrap();

    let func_ptr = fast_desmos2_cranelift::compile(&parsed);

    let output = func_ptr();
    println!("output = {output}");
}
