use std::rc::Rc;
use std::path::PathBuf;
use std::ffi::OsString;
use std::vec;

use syntex_syntax::{ast, ptr};

pub mod path;

use self::path::ModulePath;

#[derive(Default, Debug, Clone)]
pub struct Module {
    pub list: Vec<ptr::P<ast::Item>>,
    pub path: ModulePath,
}

impl From<(Vec<ptr::P<ast::Item>>, PathBuf)> for Module {
    fn from((list, mut path): (Vec<ptr::P<ast::Item>>, PathBuf)) -> Module {
        path.set_extension("");
        Module {
            list: list,
            path: ModulePath {
                path: path.components()
                          .skip(1)
                          .map(|comp| comp.as_os_str().to_os_string())
                          .collect::<Vec<OsString>>(),
            },
        }
    }
}

impl IntoIterator for Module {
    type Item = (ptr::P<ast::Item>, Rc<ModulePath>);
    type IntoIter = vec::IntoIter<(ptr::P<ast::Item>, Rc<ModulePath>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ref rc: Rc<ModulePath> = Rc::new(self.path);
        self.list.into_iter()
                 .map(|item| (item, Rc::clone(rc)))
                 .collect::<Vec<(ptr::P<ast::Item>, Rc<ModulePath>)>>()
                 .into_iter()
    }
}
