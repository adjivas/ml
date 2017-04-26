#![allow(dead_code)]
extern crate ml;

use std::path::Path;

struct Amut {
    b: *mut B,
}

struct Aconst {
    b: *const B,
}

struct B {
}

#[test]
fn test_aggregation() {
    let source = Path::new("tests/aggregation");

    assert_eq!(
        String::from_utf8(ml::from_file(&source.with_extension("rs")).unwrap()).unwrap(),
        r#"digraph ml {
    ndAmut[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nAmut|- b: *mut B}"][shape="record"];
    ndAconst[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nAconst|- b: *const B}"][shape="record"];
    ndB[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nB}"][shape="record"];
    ndB -> ndAmut[label=""][arrowhead="odiamond"];
    ndB -> ndAconst[label=""][arrowhead="odiamond"];
}
"#);
}
