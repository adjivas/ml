extern crate ml;

use std::path::Path;

fn main() {
    println!("{}",
             String::from_utf8(ml::from_file(Path::new("src/lib.rs")).unwrap()).unwrap()
            );
}
