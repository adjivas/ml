pub mod relation;
pub mod state;

pub use self::state::ItemState;

use std::{slice, iter};
use std::rc::Rc;

use ::syntex_syntax::{ptr, ast};
use ::itertools::Itertools;

use ::module::path::ModulePath;

/// The structure Item is a iterable collection of abstract elements.

#[derive(Debug, Clone)]
pub struct Item <'a> {
    /// Iterator.
    it: iter::Peekable<slice::Iter<'a, (ptr::P<ast::Item>, Rc<ModulePath>)>>,
}

impl <'a>From<iter::Peekable<slice::Iter<'a, (ptr::P<ast::Item>, Rc<ModulePath>)>>> for Item<'a> {

    /// The constructor method `from` returns a typed and iterable collection of abstract element.
    fn from(iter: iter::Peekable<slice::Iter<'a, (ptr::P<ast::Item>, Rc<ModulePath>)>>) -> Item {
        Item {
            it: iter,
        }
    }
}

impl <'a>Iterator for Item<'a> {
    type Item = ItemState<'a>;

    /// The method `next` will returns the first abstract elements defined like a structure,
    /// enumeration or trait.
    fn next(&mut self) -> Option<ItemState<'a>> {
        self.it.next().and_then(|item| {
            let mut list: Vec<&'a (ptr::P<ast::Item>, Rc<ModulePath>)> = vec!(item);

            list.extend(self.it.peeking_take_while(|&&(ref item, _): (&&'a (ptr::P<ast::Item>, Rc<ModulePath>))| {
                            if let ast::ItemKind::Impl(..) = item.node {
                                true
                            } else {    
                                false
                            }
                        })
                        .collect::<Vec<&'a (ptr::P<ast::Item>, Rc<ModulePath>)>>());
            Some(ItemState::from(list))
        })
    }
}
