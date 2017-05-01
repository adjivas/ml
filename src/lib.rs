extern crate syntex_syntax;
extern crate syntex_errors;
extern crate itertools;
extern crate walkdir;
extern crate dot;

pub mod prelude;
pub mod syntax;

use std::process::{Command, Stdio};
use std::io::{Write, Read};
use std::ffi::OsStr;
use std::path::Path;
use std::rc::Rc;

use syntex_errors::emitter::ColorConfig;
use syntex_errors::Handler;

use syntex_syntax::codemap::CodeMap;
use syntex_syntax::parse::{self, ParseSess};
use syntex_syntax::{ast, ptr};

use walkdir::WalkDir;
use syntax::ListItem;

pub const DEFAULT_NAME_DOT: &'static str = "uml-2.5.dot";
pub const DEFAULT_NAME_PNG: &'static str = "uml-2.5.png";

/// The function `from_file` returns a syntex module.
fn from_file<P: AsRef<Path>>(path: P) -> Option<ast::Crate> {
    let codemap = Rc::new(CodeMap::new());
    let tty_handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(codemap.clone()));
    let parse_session: ParseSess = ParseSess::with_span_handler(tty_handler, codemap.clone());
    let parse = parse::parse_crate_from_file(path.as_ref(), &parse_session);

    parse.ok()
}

/// The function `from_items` returns a graph formated for *Graphiz/Dot*.
fn from_items(list: Vec<ptr::P<ast::Item>>) -> Option<Vec<u8>> {
    let mut f: Vec<u8> = Vec::new();
    let it: ListItem = ListItem::from(list.iter().peekable());

    dot::render(&it, &mut f).ok().and_then(|()| Some(f))
}

/// The function `rs2dot` returns graphed file module.
pub fn rs2dot<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    from_file(path).and_then(|parse: ast::Crate| from_items(parse.module.items))
}

/// The function `src2dot` returns graphed repository of modules.
pub fn src2dot<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    from_items(WalkDir::new(path).into_iter()
                                 .filter_map(|entry: Result<walkdir::DirEntry, _>| entry.ok())
                                 .filter(|entry| entry.file_type().is_file())
                                 .filter_map(|entry: walkdir::DirEntry| {
                                     let path: &Path = entry.path();

                                     if path.extension().eq(&Some(OsStr::new("rs"))) {
                                         from_file(path).and_then(|parse| Some(parse.module.items))
                                     } else {
                                         None
                                     }
                                 })
                                 .collect::<Vec<Vec<ptr::P<ast::Item>>>>()
                                 .concat())
}

/// The function `src2png` returns pnged repository of modules.
pub fn src2png<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    src2dot(path).and_then(|buf|
        Command::new("dot").arg("-Tpng")
                           .stdin(Stdio::piped()).stdout(Stdio::piped())
                           .spawn()
                           .ok()
                           .and_then(|child| {
                                let mut ret = vec![];

                                child.stdin.unwrap().write_all(buf.as_slice()).unwrap();
                                child.stdout.unwrap().read_to_end(&mut ret).unwrap();
                                Some(ret)
                           })
    )
}
