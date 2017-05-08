//! The purpose of this module is to provide reexports of core traits so that they can be then
//! glob-imported all at once:

pub use ::DEFAULT_NAME_DOT;
pub use ::DEFAULT_NAME_PNG;
pub use ::core::segment::Segment;
pub use ::core::item::Item;
pub use ::core::item::relation::Relation;
pub use ::core::item::state::ItemState;
pub use ::core::item::state::method::Method;
pub use ::core::item::state::implem::Implem;
pub use ::core::item::state::abstraction::Abstract;
pub use ::core::item::state::abstraction::extend::Trait;
pub use ::core::item::state::abstraction::structure::Struct;
pub use ::core::item::state::abstraction::enumerate::Enum;
