# ML -Modeling Language-

[![Crate][crate-badge]][crate] [![travis-badge][]][travis]

This repo is provided as a [Cargo package](http://doc.crates.io/manifest.html) and a [build script](http://doc.crates.io/build-script.html), adjust your `Cargo.toml` to include.
```toml
# Cargo.toml
[build-dependencies.ml]
version = "0.1"
```

And your `build.rs` to generate your uml [graph/dot](http://www.graphviz.org/doc/info/lang.html) and Structured Vector Graphics at **target/dot/{your crate}**.
```rust
extern crate ml;

use std::path::PathBuf;

fn main() {
    ml::src2both(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
                 PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target")
                                                          .join("doc")
                                                          .join(env!("CARGO_PKG_NAME")));
}
```

`src/lib.rs`
```rust
//! ![uml](uml.svg)
```

## Knowledge
This is a reading list of material relevant to *Ml*. It includes prior research that has - at one time or another - influenced the design of *Ml*, as well as publications about *Ml*.
* [Supporting Tool Reuse with Model Transformation](http://www.yusun.io/papers/sede-2009.pdf)
* [Unified Modeling Language Version 2.5](http://www.omg.org/spec/UML/2.5)

## License

`ml` is primarily distributed under the terms of both the [MIT license](https://opensource.org/licenses/MIT) and the [Apache License (Version 2.0)](https://www.apache.org/licenses/LICENSE-2.0), with portions covered by various BSD-like licenses.

See [LICENSE-APACHE](LICENSE-APACHE), and [LICENSE-MIT](LICENSE-MIT) for details.

[crate-badge]: https://img.shields.io/badge/crates.io-v0.1.0-orange.svg?style=flat-square
[crate]: https://crates.io/crates/ml
[travis-badge]: https://travis-ci.org/adjivas/ml.svg?branch=master&style=flat-square
[travis]: https://travis-ci.org/adjivas/ml
