#![allow(dead_code)]
extern crate ml;

struct A {
}

impl A {
    fn b() -> B {
        B {
        }
    }
}

struct Ab {
}

impl Ab {
    fn b() -> B {
        B {
        }
    }
}

struct B {
}

impl B {
    fn a() -> Ab {
        Ab {
        }
    }
}

#[test]
fn test_association() {
    assert_eq!(
        String::from_utf8(ml::rs2dot("tests/association.rs").unwrap()).unwrap(),
        r#"digraph ml {
    ndA[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nA|- b() -&amp;gt; B}"][shape="record"];
    ndAb[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nAb|- b() -&amp;gt; B}"][shape="record"];
    ndB[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nB|- a() -&amp;gt; Ab}"][shape="record"];
    ndAb -> ndB[label=""][arrowhead="none"];
    ndB -> ndA[label=""][arrowhead="vee"];
}
"#);
}
