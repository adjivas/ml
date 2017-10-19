pub mod abstraction;
pub mod implem;
pub mod method;

#[cfg(not(feature = "fn-emilgardis"))]
const DEFAULT_FUNC: &'static str = " ";
#[cfg(feature = "fn-emilgardis")]
const DEFAULT_FUNC: &'static str = " fn ";

use self::abstraction::Abstract;
use self::implem::Implem;
use self::method::Method;

use super::relation::Relation;

use std::ops::BitOr;
use std::fmt;

use ::syntex_syntax::symbol::InternedString;
use ::syntex_syntax::{ptr, ast};

/// The structure `ItemState` describes an abstract element with a collections of methodes
/// and implementations.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ItemState<'a> {
    /// Data Type.
    node: Abstract<'a>,
    /// Implementation of Method.
    method: Vec<Method<'a>>,
    /// Implementation of Trait.
    implem: Vec<Implem>,
}

impl <'a> ItemState <'a> {

    pub fn is_none(&self) -> bool {
        self.node.eq(&Abstract::None)
    }

    pub fn is_association(&self, rhs: &ItemState<'a>) -> bool {
        if let Some(ref name) = self.as_name() {
            let ref ty_name: String = name.to_string();

            rhs.method.iter()
                      .any(|func| func.is_association(ty_name))
                      .bitor(rhs.implem.iter()
                                       .any(|implem| implem.is_association(&ty_name)))
        } else {
            false
        }
    }

    pub fn is_dependency(&self, rhs: &ItemState<'a>) -> bool {
        if let Some(ref name) = self.as_name() {
            let ref ty_name: String = name.to_string();

            rhs.method.iter()
                      .any(|method| method.is_dependency(&ty_name))
                      .bitor(self.implem.iter()
                                        .any(|implem| implem.is_dependency(&ty_name)))
        } else {
            false
        }
    }

    pub fn is_aggregation(&self, rhs: &ItemState<'a>) -> bool {
        if let Some(ref name) = self.as_name() {
            let mut ty_name_mut: String = String::from("*mut ");
            let mut ty_name_const: String = String::from("*const ");
            
            ty_name_mut.push_str(&name);
            ty_name_const.push_str(&name);
            rhs.node.into_iter()
                    .any(|attribut: &String|
                          attribut.split(|at| "<[(;,)]>".contains(at))
                                  .any(|ty| ty_name_mut.eq(ty).bitor(ty_name_const.eq(ty))))
        } else {
            false
        }
    }

    pub fn is_composition(&self, rhs: &ItemState<'a>) -> bool {
        if let Some(ref name) = self.as_name() {
            let ty_name: String = name.to_string();

            rhs.node.into_iter()
                    .any(|attribut: &String|
                          attribut.split(|at| "<[(;,)]>".contains(at))
                                  .any(|ty| ty.eq(&ty_name)))
        } else {
            false
        }
    }

    pub fn is_realization(&self, rhs: &ItemState<'a>) -> bool {
        if let Some(ref name) = self.as_name() {
            let ty_name: String = name.to_string();

            rhs.implem.iter()
                      .any(|implem| implem.is_realization(&ty_name))
        } else {
            false
        }
    }

    pub fn is_relation(&self, rhs: &ItemState<'a>) -> bool {
        self.is_association(rhs)
            .bitor(self.is_dependency(rhs))
            .bitor(self.is_aggregation(rhs))
            .bitor(self.is_composition(rhs))
            .bitor(self.is_realization(rhs))
    }

    pub fn as_name(&self) -> Option<&InternedString> {
        self.node.as_name()
    }

    pub fn as_arrow(&self, rhs: &ItemState<'a>) -> Relation {
        Relation::from((self, rhs))
    }
}

impl <'a>From<(Abstract<'a>, Vec<&'a ptr::P<ast::Item>>)> for ItemState<'a> {
    fn from((node, properties): (Abstract<'a>, Vec<&'a ptr::P<ast::Item>>)) -> ItemState<'a> {
        ItemState {
            node: node,
            method: properties.iter()
                .filter_map(|item: (&&'a ptr::P<ast::Item>)|
                    if let ast::ItemKind::Impl(_, _, _, _, None, _, ref impl_item) = item.node {
                        Some(Method::from(impl_item))
                    } else {
                        None
                    }
                )
                .collect::<Vec<Method>>(),
            implem: properties.iter()
                .filter_map(|item: (&&'a ptr::P<ast::Item>)|
                    if let ast::ItemKind::Impl(_, _, _, _, Some(ast::TraitRef {path: ast::Path {span: _, ref segments}, ..}), _, ref impl_item) = item.node {
                        Some(Implem::from((segments, impl_item)))
                    } else {
                        None
                    }
                )
                .collect::<Vec<Implem>>()
        }
    }
}

impl <'a>From<Vec<&'a ptr::P<ast::Item>>> for ItemState<'a> {
    fn from(state: Vec<&'a ptr::P<ast::Item>>) -> ItemState<'a> {
        state.split_first().and_then(|(ref item, properties): (&&'a ptr::P<ast::Item>, &[&'a ptr::P<ast::Item>])| {
            match &item.node {
                /// Trait.
                &ast::ItemKind::Trait(_, ast::Generics {lifetimes: _, ref ty_params, ..}, _, ref trait_item) => {
                    let kind: (&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::TraitItem>) = (item, ty_params, trait_item);
                    let kind: (Abstract, Vec<&'a ptr::P<ast::Item>>) = (Abstract::from(kind), properties.to_vec());
                    Some(ItemState::from(kind))
                },
                /// Structure with variables.
                &ast::ItemKind::Struct(ast::VariantData::Struct(ref struct_field, _), ..) => {
                    let kind: (&'a ast::Item, &'a Vec<ast::StructField>) = (item, struct_field);
                    let kind: (Abstract, Vec<&'a ptr::P<ast::Item>>) = (Abstract::from(kind), properties.to_vec());
                    Some(ItemState::from(kind))
                },
                /// Enumeration with variables.
                &ast::ItemKind::Enum(ast::EnumDef {ref variants}, ast::Generics {lifetimes: _, ref ty_params, ..}) => {
                    let kind: (&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::Variant>) = (item, ty_params, variants);
                    let kind: (Abstract, Vec<&'a ptr::P<ast::Item>>) = (Abstract::from(kind), properties.to_vec());
                    Some(ItemState::from(kind))
                },
                _ => None,
            }
        }).unwrap_or_default()
    }
}

impl <'a>fmt::Display for ItemState<'a> {

    #[cfg(feature = "implem")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{{node}|{method}|{implem}}}",
            node = self.node,
            method = self.method.iter()
                                .map(|ref methods| format!("{}", methods))
                                .collect::<Vec<String>>().join("\n").as_str(),
            implem = self.implem.iter()
                                .map(|ref implem| format!("{}", implem))
                                .collect::<Vec<String>>().join("\n").as_str())
    }

    #[cfg(not(feature = "implem"))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.method.is_empty() {
            write!(f, "{{{node}}}", node = self.node)
        } else {
            write!(f, "{{{node}|{method}}}",
                node = self.node,
                method = self.method.iter()
                                    .map(|ref methods| format!("{}", methods))
                                    .collect::<Vec<String>>().join("\n").as_str())
        }
    }
}
