extern crate ml;

use std::fs::File;
use std::io::Write;

fn main() {
    let content: Vec<u8> = ml::src2dot("src").unwrap();
    let mut file = File::create(ml::DEFAULT_NAME).unwrap();

    file.write_all(content.as_slice()).unwrap();
}
