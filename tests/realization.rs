#![allow(dead_code, unused_variables)]
extern crate ml;

use std::fmt::Debug;

#[derive(Debug)]
struct A<T> where T: Debug {
    a: T,
}

impl<T> A<T> where T: Debug {
    fn a(a: T) -> Self {
        A {
            a: a,
        }
    }
}

impl <T>B<T> for A<T> where T: Debug {
    fn a(&self) -> Option<T> {
        None
    }
}

trait B<T> : Debug where T: Debug {
    fn a(&self) -> Option<T>;
}

impl <T>B<T> {
    fn a(&self) -> Option<T> {
        None
    }
}

#[test]
fn test_realization() {
    assert_eq!(String::from_utf8(ml::rs2dot("tests/realization.rs").unwrap()).unwrap(),
        r#"digraph ml {
    ndA[label="{&lt;&lt;&lt;Structure&gt;&gt;&gt;\nA|- a: T|- a(a: T) -&amp;gt; Self}"][shape="record"];
    ndB[label="{&lt;&lt;&lt;Trait&gt;&gt;&gt;\nB|a(&amp;Self) -&amp;gt; Option&lt;T&gt;|- a(&amp;self) -&amp;gt; Option&lt;T&gt;}"][shape="record"];
    ndB -> ndA[label=""][style="dashed"][arrowhead="onormal"];
}
"#);
}
