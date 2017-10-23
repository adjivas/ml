pub mod extend;
pub mod structure;
pub mod enumerate;

use std::fmt;
use std::vec;
use std::rc::Rc;

use ::syntex_syntax::symbol;
use ::core::ast;

use ::module::path::ModulePath;

use self::extend::Trait;
use self::structure::Struct;
use self::enumerate::Enum;

/// The structure `Abstract` is a enumerate for abstract element types or none.

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Abstract <'a> {
    Trait(Trait<'a>),
    Struct(Struct<'a>),
    Enum(Enum<'a>),
    None,
}

impl <'a> Abstract <'a> {

    /// The method `as_name` returns the name of the abstract element
    /// or else declare a panic.
    pub fn as_name(&self) -> Option<&symbol::InternedString> {
        match self {
            &Abstract::Trait(Trait { vis: _, ref name, ..}) => Some(name),
            &Abstract::Struct(Struct { vis: _, ref name, ..}) => Some(name),
            &Abstract::Enum(Enum { vis: _, ref name, ..}) => Some(name),
            &Abstract::None => None,
        }
    }
}

impl<'a> IntoIterator for &'a Abstract<'a> {
    type Item = &'a String;
    type IntoIter = vec::IntoIter<&'a String>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            &Abstract::Struct(Struct {path: _, vis: _, name: _, fields: ref ty_field}) => {
                ty_field.iter()
                        .map(|&(_, _, ref ty): &'a (&'a ast::Visibility, symbol::InternedString, String)| ty)
                        .collect::<Vec<&'a String>>()
                        .into_iter()
            },
            &Abstract::Enum(Enum {path: _, vis: _, name: _, params: _, variants: ref ty_multi_field}) => {
                ty_multi_field.iter()
                              .map(|&(_, ref ty_field): &'a (symbol::InternedString, Vec<String>)| 
                                   ty_field.iter()
                                           .map(|ty: &'a String| ty)
                                           .collect::<Vec<&'a String>>())
                              .collect::<Vec<Vec<&'a String>>>()
                              .concat()
                              .into_iter()
            },
            _ => {
                Vec::default().into_iter()
            },
        }
    }
}

impl <'a> Default for Abstract<'a> {
    fn default() -> Abstract<'a> {
        Abstract::None
    }
}

impl <'a>From<((&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::TraitItem>), Rc<ModulePath>)> for Abstract<'a> {
    fn from(arguments: ((&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::TraitItem>), Rc<ModulePath>)) -> Abstract<'a> {
        Abstract::Trait(Trait::from(arguments))
    }
}

impl <'a>From<((&'a ast::Item, &'a Vec<ast::StructField>), Rc<ModulePath>)> for Abstract<'a> {
    fn from(arguments: ((&'a ast::Item, &'a Vec<ast::StructField>), Rc<ModulePath>)) -> Abstract<'a> {
        Abstract::Struct(Struct::from(arguments))
    }
}

impl <'a>From<((&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::Variant>), Rc<ModulePath>)> for Abstract<'a> {
    fn from(arguments: ((&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::Variant>), Rc<ModulePath>)) -> Abstract<'a> {
        Abstract::Enum(Enum::from(arguments))
    }
}

impl <'a>fmt::Display for Abstract<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Abstract::Struct(ref item) => write!(f, "{}", item),
            &Abstract::Enum(ref item) => write!(f, "{}", item),
            &Abstract::Trait(ref item) => write!(f, "{}", item),
            &Abstract::None => Err(fmt::Error),
        }
    }
}
