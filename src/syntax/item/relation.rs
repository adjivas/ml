use super::ItemState;

use ::dot::{Fill, ArrowShape, Side};

/// The enumeration `Relation` is the relationship specification from [UML 2.5](http://www.omg.org/spec/UML/2.5) without generalization.
pub enum Relation {
    Association,
    Aggregation,
    Composition,
    Realization,
    Dependency,
    None,
}

impl Relation {

    /// The method `as_style` returns a stylized arrow (See *Table B.2 UML Edges* from [UML 2.5](http://www.omg.org/spec/UML/2.5).
    pub fn as_style(&self) -> ArrowShape {
        match self {
            &Relation::Association => ArrowShape::Vee(Side::Both),
            &Relation::Dependency => ArrowShape::Vee(Side::Both),
            &Relation::Aggregation => ArrowShape::Diamond(Fill::Open, Side::Both),
            &Relation::Composition => ArrowShape::Diamond(Fill::Filled, Side::Both),
            &Relation::Realization => ArrowShape::Normal(Fill::Open, Side::Both),
            &Relation::None => ArrowShape::NoArrow,
        }
    }
}

impl <'a>From<(&'a ItemState<'a>, &'a ItemState<'a>)> for Relation {
    fn from((left, right): (&'a ItemState<'a>, &'a ItemState<'a>)) -> Relation {
        if left.is_composition(right) {
            Relation::Composition
        } else if left.is_aggregation(right) {
            Relation::Aggregation
        } else if left.is_dependency(right) {
            Relation::Dependency
        } else if left.is_association(right) {
            Relation::Association
        } else if left.is_realization(right) {
            Relation::Realization
        } else {
            Relation::None
        }
    }
}

