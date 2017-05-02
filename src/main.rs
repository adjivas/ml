extern crate mml;

fn main() {
    let _ = mml::src2both("src", concat!("target/doc/", env!("CARGO_PKG_NAME")));
}
