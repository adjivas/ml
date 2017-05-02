#![allow(dead_code)]

extern crate mml;

struct A {
    b: B,
}

struct B {
}

#[test]
fn test_composition() {
    assert_eq!(
        String::from_utf8(mml::rs2dot("tests/composition.rs").unwrap()).unwrap(),
        r#"digraph ml {
    ndA[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nA|- b: B}"][shape="record"];
    ndB[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nB}"][shape="record"];
    ndB -> ndA[label=""][arrowhead="diamond"];
}
"#);
}
