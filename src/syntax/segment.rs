use std::ops::{BitOr, BitAnd};
use std::hash::{Hash, Hasher};

use super::ItemState;

/// The structure `Segment` contents two nodes.

#[derive(Debug, Clone, Eq)]
pub struct Segment<'a> {
    pub left: ItemState<'a>,
    pub right: ItemState<'a>,
}

impl <'a> From<(ItemState<'a>, ItemState<'a>)> for Segment <'a> {
    fn from((left, right): (ItemState<'a>, ItemState<'a>)) -> Segment<'a> {
        Segment {
            left: left,
            right: right,
        }
    }
}

impl <'a> Hash for Segment <'a> {
    fn hash<H: Hasher>(&self, _: &mut H) {
    }
}

impl <'a> PartialEq for Segment <'a> {
    fn eq(&self, rhs: &Segment) -> bool {
        self.left.eq(&rhs.left)
                 .bitand(self.right.eq(&rhs.right))
                                   .bitor(self.left.eq(&rhs.right)
                                                   .bitand(self.right.eq(&rhs.left)))
    }
}
