use std::rc::Rc;
use std::path::PathBuf;
use std::ffi::OsString;
use std::vec;

use syntex_syntax::{ast, ptr};

#[derive(Default, Debug, Clone)]
pub struct Module {
    pub list: Vec<ptr::P<ast::Item>>,
    pub path: Vec<OsString>,
}

impl From<(Vec<ptr::P<ast::Item>>, PathBuf)> for Module {
    fn from((list, mut path): (Vec<ptr::P<ast::Item>>, PathBuf)) -> Module {
        path.set_extension("");
        Module {
            list: list,
            path: path.components()
                      .skip(1)
                      .map(|comp| comp.as_os_str().to_os_string())
                      .collect::<Vec<OsString>>(),
        }
    }
}

impl IntoIterator for Module {
    type Item = (ptr::P<ast::Item>, Rc<Vec<OsString>>);
    type IntoIter = vec::IntoIter<(ptr::P<ast::Item>, Rc<Vec<OsString>>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ref rc: Rc<Vec<OsString>> = Rc::new(self.path);
        self.list.into_iter()
                 .map(|item| (item, Rc::clone(rc)))
                 .collect::<Vec<(ptr::P<ast::Item>, Rc<Vec<OsString>>)>>()
                 .into_iter()
    }
}
