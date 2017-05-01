#![allow(dead_code)]

extern crate ml;

struct A {
    b: B,
}

struct B {
}

#[test]
fn test_composition() {
    assert_eq!(
        String::from_utf8(ml::rs2dot("tests/composition.rs").unwrap()).unwrap(),
        r#"digraph ml {
    ndA[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nA|- b: B}"][shape="record"];
    ndB[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nB}"][shape="record"];
    ndB -> ndA[label=""][arrowhead="diamond"];
}
"#);
}
