use std::{env, fs};

use decl::parse;

fn main() {
    let path = env::args()
        .nth(1)
        .expect("missing argument: path to ELF binary");

    let file_contents = fs::read(path).expect("reading file");

    let data = parse(&file_contents).expect("parsing file");

    println!("{data:#?}");
}
