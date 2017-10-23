use std::ffi::OsString;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ModulePath {
    pub path: Vec<OsString>,
}
