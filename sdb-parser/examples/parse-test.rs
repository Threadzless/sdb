use sdb_parser::*;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let raw = args.get(1).unwrap();

    let mut parser = TreeParser::new(raw);
    let tree = parser.parse();
    println!("{tree:#?}")
}