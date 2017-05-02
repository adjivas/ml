extern crate mml;

use std::path::PathBuf;

fn main() {
    let _ = mml::src2both(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
                          PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target")
                                                                   .join("doc")
                                                                   .join(env!("CARGO_PKG_NAME")));
}
