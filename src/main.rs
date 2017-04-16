#![feature(slice_patterns)]
#![feature(never_type)]

extern crate syntex_syntax as syntax;
extern crate syntex_errors;
extern crate itertools;
extern crate dot;

use std::{slice, iter};
use std::path::Path;
use std::rc::Rc;
use std::ops::Deref;
use std::borrow::Cow;
use std::fs::File;
use std::ops::BitOr;
use std::ops::Not;
use std::fmt;
use std::vec;

use syntax::codemap::CodeMap;
use syntax::print::pprust::{ty_to_string, arg_to_string};
use syntex_errors::Handler;
use syntex_errors::emitter::ColorConfig;
use syntax::parse::{self, ParseSess};

use itertools::Itertools;

#[derive(Debug, Clone)]
struct Item <'a> {
    it: iter::Peekable<slice::Iter<'a, syntax::ptr::P<syntax::ast::Item>>>,
}

impl <'a>From<iter::Peekable<slice::Iter<'a, syntax::ptr::P<syntax::ast::Item>>>> for Item<'a> {
    fn from(iter: iter::Peekable<slice::Iter<'a, syntax::ptr::P<syntax::ast::Item>>>) -> Item {
        Item {
            it: iter,
        }
    }
}

impl <'a>Iterator for Item<'a> {
    type Item = ItemState<'a>;

    fn next(&mut self) -> Option<ItemState<'a>> {
        self.it.next().and_then(|item| {
            let mut list: Vec<&'a syntax::ptr::P<syntax::ast::Item>> = vec!(item);

            list.extend(
                self.it.peeking_take_while(|ref item| {
                    if let syntax::ast::ItemKind::Impl(_, _, _, _, _, _) = item.node {
                        true
                    } else {    
                        false
                    }
                })
                .collect::<Vec<&'a syntax::ptr::P<syntax::ast::Item>>>()
            );
            Some(ItemState::from(list))
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum Node <'a> {
    Trait(&'a syntax::ast::Visibility, syntax::symbol::InternedString, Vec<syntax::symbol::InternedString>, Vec<(syntax::symbol::InternedString, Vec<String>, String)>),
    Struct(&'a syntax::ast::Visibility, syntax::symbol::InternedString, Vec<(&'a syntax::ast::Visibility, syntax::symbol::InternedString, String)>),
    Enum(&'a syntax::ast::Visibility, syntax::symbol::InternedString, Vec<syntax::symbol::InternedString>, Vec<(syntax::symbol::InternedString, Vec<String>)>),
    None,
}

impl <'a> Node <'a> {
    pub fn as_name(&self) -> Result<&syntax::symbol::InternedString, !> {
        match self {
            &Node::Trait(_, ref name, _, _) => Ok(name),
            &Node::Struct(_, ref name, _) => Ok(name),
            &Node::Enum(_, ref name, _, _) => Ok(name),
            &Node::None => unreachable!(),
        }
    }
    
    pub fn is_trait(&self) -> bool {
        match self {
            &Node::Trait(_, _, _, _) => true,
            _ => false,
        }
    }
}

impl<'a> IntoIterator for &'a Node<'a> {
    type Item = &'a String;
    type IntoIter = vec::IntoIter<&'a String>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            &Node::Struct(_, _, ref ty_field) => {
                    ty_field.iter()
                        .map(|&(_, _, ref ty): &'a (&'a syntax::ast::Visibility, syntax::symbol::InternedString, std::string::String)| ty)
                        .collect::<Vec<&'a String>>()
                        .into_iter()
            },
            &Node::Enum(_, _, _, ref ty_multi_field) => {
                    ty_multi_field.iter()
                        .map(|&(_, ref ty_field): &'a (syntax::symbol::InternedString, Vec<String>)| 
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

impl <'a> Default for Node<'a> {
    fn default() -> Node<'a> {
        Node::None
    }
}

/// Trait
impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::TraitItem>)> for Node<'a> {
    fn from((item, ty_params, trait_item): (&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::TraitItem>)) -> Node<'a> {
        Node::Trait(
            &item.vis,
            item.ident.name.as_str(),
            ty_params.iter()
                     .map(|&syntax::ast::TyParam {attrs: _, ident: syntax::ast::Ident {name, ..}, ..}| name.as_str())
                     .collect::<Vec<syntax::symbol::InternedString>>(),
            trait_item.iter()
                      .filter_map(|&syntax::ast::TraitItem {id: _, ident: syntax::ast::Ident {name, ..}, attrs: _, ref node, ..}|
                            if let &syntax::ast::TraitItemKind::Method(syntax::ast::MethodSig { unsafety: _, constness: _, abi: _, ref decl, ..}, _) = node {
                                if let &syntax::ast::FnDecl {ref inputs, output: syntax::ast::FunctionRetTy::Ty(ref ty), ..} = decl.deref() {
                                    Some((name.as_str(), inputs.iter().map(|input| ty_to_string(&input.ty)).collect::<Vec<String>>(), ty_to_string(&ty)))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                      )
                     .collect::<Vec<(syntax::symbol::InternedString, Vec<String>, String)>>()
        )
    }
}

/// Structure
impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::StructField>)> for Node<'a> {
    fn from((item, struct_field): (&'a syntax::ast::Item, &'a Vec<syntax::ast::StructField>)) -> Node<'a> {
        Node::Struct(
            &item.vis,
            item.ident.name.as_str(),
            struct_field.iter()
                        .filter_map(|&syntax::ast::StructField { span: _, ident, ref vis, id: _, ref ty, .. }|
                                ident.and_then(|syntax::ast::Ident {name, ..}|
                                    Some((vis, name.as_str(), ty_to_string(&ty)))))
                        .collect::<Vec<(&syntax::ast::Visibility, syntax::symbol::InternedString, String)>>()
        )
    }
}

/// Enumeration
impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::Variant>)> for Node<'a> {
    fn from((item, ty_params, variants): (&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::Variant>)) -> Node<'a> {
        Node::Enum(
            &item.vis,
            item.ident.name.as_str(),
            ty_params.iter()
                     .map(|&syntax::ast::TyParam {attrs: _, ident: syntax::ast::Ident {name, ..}, ..}| name.as_str())
                     .collect::<Vec<syntax::symbol::InternedString>>(),
            variants.iter()
               .map(|&syntax::codemap::Spanned {node: syntax::ast::Variant_ {name: syntax::ast::Ident {name, ..}, attrs: _, ref data, ..}, ..}| {
                    if let &syntax::ast::VariantData::Tuple(ref struct_field, _) = data {
                        (name.as_str(),
                         struct_field.iter()
                                     .filter_map(|&syntax::ast::StructField { span: _, ident: _, vis: _, id: _, ref ty, .. }| Some(ty_to_string(&ty)))
                                     .collect::<Vec<String>>())
                    } else {
                        (name.as_str(), Vec::new())
                    }
               })
               .collect::<Vec<(syntax::symbol::InternedString, Vec<String>)>>()
        )
    }
}

impl <'a>fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Node::Trait(_, _, _, _) => write!(f, ""),
            &Node::Struct(_, ref name, ref struct_field) => write!(f, "&lt;&lt;&lt;Structure&gt;&gt;&gt;|{name}|{struct_field}",
                name = name,
                struct_field = dot::escape_html(
                    struct_field.iter()
                                .map(|&(ref vis, ref name, ref ty): &(&syntax::ast::Visibility, syntax::symbol::InternedString, String)|
                                    if syntax::ast::Visibility::Public.eq(vis) {
                                        format!("+ {}: {}", name, ty)
                                    } else {
                                        format!("- {}: {}", name, ty)
                                    }
                                )
                                .collect::<Vec<String>>()
                                .join("\n")
                                .as_str()),
            ),
            &Node::Enum(_, ref name, _, ref variants) => write!(f, "&lt;&lt;&lt;Enumeration&gt;&gt;&gt;|{name}|{variants}",
                name = name,
                variants = dot::escape_html(
                    variants.iter()
                            .map(|&(ref name, ref struct_field): &(syntax::symbol::InternedString, Vec<String>)|
                                 if struct_field.is_empty() {
                                     format!("{}", name)
                                 } else {
                                     format!("{}({})", name, struct_field.join(", "))
                                 }
                            )
                            .collect::<Vec<String>>()
                            .join("\n")
                            .as_str()),
            ),
            &Node::None => Err(fmt::Error),
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Method <'a> {
    /// visibility, method's name, arguments, result.
    func: Vec<(&'a syntax::ast::Visibility, syntax::symbol::InternedString, Vec<String>, Option<String>)>
}

impl <'a> Method <'a> {
    pub fn is_association(&self, ty_name: &String) -> bool {
        self.func.iter()
                 .any(|&(_, _, _, ref result): &(&'a syntax::ast::Visibility, syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)|
                     if let &Some(ref ret) = result {
                         ret.split(|at| "<[(;, )]>".contains(at))
                            .any(|ty| ty.eq(ty_name))
                     } else {
                         false
                     }
                 )
    }

    pub fn is_dependency(&self, name: &String) -> bool {
        let mut ty_name: String = String::from(" ");
        
        ty_name.push_str(name);
        self.func.iter()
                 .any(|&(_, _, _, ref result): &(&'a syntax::ast::Visibility, syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)|
                     if let &Some(ref ret) = result {
                         ret.split(|at| "<[(;,)]>".contains(at))
                            .any(|ty| ty.ends_with(&ty_name))
                     } else {
                         false
                     }
                 )
    }

    pub fn is_relation(&'a self, ty_name: &String) -> bool {
        self.func.iter()
                 .any(|&(_, _, _, ref result): &(&'a syntax::ast::Visibility, syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)|
                     if let &Some(ref ret) = result {
                         ret.split(|at| "<[(;, )]>".contains(at))
                            .any(|ty| ty_name.eq(ty))
                     } else {
                         false
                     }
                 )
    }
}

impl <'a> From<Vec<(&'a syntax::ast::Visibility, syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)>> for Method<'a> {
    fn from(func: Vec<(&'a syntax::ast::Visibility, syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)>) -> Method<'a> {
        Method {
            func: func,
        }
    }
}

impl <'a> From<&'a Vec<syntax::ast::ImplItem>> for Method<'a> {
    fn from(impl_item: &'a Vec<syntax::ast::ImplItem>) -> Method<'a> {
        Method::from(impl_item.iter()
                              .filter_map(|&syntax::ast::ImplItem {id: _, ident: syntax::ast::Ident { name, ..}, ref vis, defaultness: _, attrs: _, ref node, .. }| {
                                     if let &syntax::ast::ImplItemKind::Method(syntax::ast::MethodSig {unsafety: _, constness: _, abi: _, ref decl, ..}, _) = node {
                                         if let &syntax::ast::FnDecl {ref inputs, output: syntax::ast::FunctionRetTy::Ty(ref ty), ..} = decl.deref() {
                                             Some((vis, name.as_str(), inputs.iter().map(|ref arg| arg_to_string(&arg)).collect::<Vec<String>>(), Some(ty_to_string(&ty))))
                                         } else if let &syntax::ast::FnDecl {ref inputs, output: syntax::ast::FunctionRetTy::Default(_), ..} = decl.deref() {
                                             Some((vis, name.as_str(), inputs.iter().map(|ref arg| arg_to_string(&arg)).collect::<Vec<String>>(), None))
                                         } else {
                                             None
                                         }
                                     } else {
                                         None
                                     }
                               })
                               .collect::<Vec<(&'a syntax::ast::Visibility, syntax::symbol::InternedString, Vec<String>, Option<String>)>>())
    }
}

impl <'a>fmt::Display for Method<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}",
            self.func.iter().map(|&(ref vis, ref name, ref inputs, ref ty)|
                match (vis, ty) {
                    (&&syntax::ast::Visibility::Public, &Some(ref ty)) => {
                        format!("+ fn {}({}) -&gt; {}", name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "), ty)
                    },
                    (&&syntax::ast::Visibility::Public, &None) => {
                        format!("+ fn {}({})", name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "))
                    },
                    (_, &Some(ref ty)) => {
                        format!("- fn {}({}) -&gt; {}", name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "), ty)
                    },
                    (_, &None) => {
                        format!("- fn {}({})", name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "))
                    },
                }
            )
            .collect::<Vec<String>>().join("\n")
        )
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Implem {
    ty: Vec<(syntax::symbol::InternedString, Vec<String>)>,
    /// method's name, arguments, result.
    func: Vec<(syntax::symbol::InternedString, Vec<String>, Option<String>)>,
}

impl Implem {
    pub fn is_realization(&self, ty_name: &String) -> bool {
        self.func.iter()
                 .any(|&(_, _, ref result): &(syntax::symbol::InternedString, Vec<String>, Option<String>)|
                      if let &Some(ref ret) = result {
                          ret.split(|at| "<[(;, )]>".contains(at))
                             .any(|ty| ty_name.eq(ty))
                      } else {
                          false
                      }
                  )
    }
}

impl From<(Vec<(syntax::symbol::InternedString, Vec<String>)>, Vec<(syntax::symbol::InternedString, Vec<String>, Option<String>)>)> for Implem {
    fn from((ty, func): (Vec<(syntax::symbol::InternedString, Vec<String>)>, Vec<(syntax::symbol::InternedString, Vec<String>, Option<String>)>)) -> Implem {
        Implem {
            ty: ty,
            func: func,
        }
    }
}

impl <'a> From<(&'a Vec<syntax::ast::PathSegment>, &'a Vec<syntax::ast::ImplItem>)> for Implem {
    fn from((segments, impl_item): (&'a Vec<syntax::ast::PathSegment>, &'a Vec<syntax::ast::ImplItem>)) -> Implem {
        Implem::from((segments.iter()
                              .map(|&syntax::ast::PathSegment { identifier: syntax::ast::Ident {name, ..}, ref parameters }| {
                                  if let &Some(ref path) = parameters {
                                      if let &syntax::ast::PathParameters::AngleBracketed(
                                          syntax::ast::AngleBracketedParameterData { lifetimes: _, ref types, .. }
                                      ) = path.deref() {
                                          (name.as_str(), types.iter().map(|ty| ty_to_string(&ty)).collect::<Vec<String>>())
                                      } else {
                                          (name.as_str(), Vec::new())
                                      }
                                  } else {
                                      (name.as_str(), Vec::new())
                                  }
                              })
                              .collect::<Vec<(syntax::symbol::InternedString, Vec<String>)>>(),
                              impl_item.iter()
                                       .filter_map(|&syntax::ast::ImplItem { id: _, ident: syntax::ast::Ident {name, ..}, vis: _, defaultness: _, attrs: _, ref node, ..}|
                                                 if let &syntax::ast::ImplItemKind::Method(syntax::ast::MethodSig { unsafety: _, constness: _, abi: _, ref decl, .. }, _) = node {
                                                     if let syntax::ast::FunctionRetTy::Ty(ref ty) = decl.output {
                                                         Some((name.as_str(), decl.inputs.iter().map(|arg| ty_to_string(&arg.ty)).collect::<Vec<String>>(), Some(ty_to_string(&ty))))
                                                     } else {
                                                         Some((name.as_str(), decl.inputs.iter().map(|arg| ty_to_string(&arg.ty)).collect::<Vec<String>>(), None))
                                                     }
                                                 } else {
                                                     None
                                                 }
                              ).collect::<Vec<(syntax::symbol::InternedString, Vec<String>, Option<String>)>>()))
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ItemState<'a> {
    /// Data Type.
    node: Node<'a>,
    /// Implementation of Method.
    method: Vec<Method<'a>>,
    /// Implementation of Trait.
    implem: Vec<Implem>,
}

impl <'a> ItemState <'a> {

    pub fn is_none(&self) -> bool {
        self.node.eq(&Node::None)
    }

    pub fn is_association(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let ref ty_name: String = name.to_string();

                rhs.method.iter()
                          .any(|func| func.is_association(ty_name))
            }
        }
    }

    pub fn is_dependency(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let ref ty_name: String = name.to_string();

                rhs.method.iter()
                          .any(|func| func.is_dependency(ty_name))
            }
        }
    }

    pub fn is_aggregation(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let mut ty_name: String = String::from("* ");
                
                ty_name.push_str(&name);
                rhs.node.into_iter()
                        .any(|attribut: &String|
                              attribut.split(|at| "<[(;,)]>".contains(at))
                                      .filter(|ty| ty.is_empty().not())
                                      .any(|ty| ty_name.ends_with(ty)))
            }
        }
    }

    pub fn is_composition(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let ty_name: String = name.to_string();

                rhs.node.into_iter()
                        .any(|attribut: &String|
                              attribut.split(|at| "<[(;, )]>".contains(at))
                                      .any(|ty| ty.eq(&ty_name)))
            }
        }
    }

    pub fn is_realization(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let ty_name: String = name.to_string();
        
                rhs.implem.iter()
                          .any(|implem| implem.is_realization(&ty_name))
            }
        }
    }

    pub fn is_relation(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let ty_name: String = name.to_string();
        
                rhs.node.into_iter()
                        .any(|attribut: &String|
                              attribut.split(|at| "<[(;, )]>".contains(at))
                                      .any(|ty| ty_name.eq(ty)))
                        .bitor(rhs.method.iter()
                                  .any(|func| func.is_relation(&ty_name)))
                        .bitor(rhs.implem.iter()
                                  .any(|implem| implem.is_realization(&ty_name)))
            }
        }
    }

    pub fn as_name(&self) -> Result<&syntax::symbol::InternedString, !> {
        self.node.as_name()
    }

    pub fn as_arrow(&self, rhs: &ItemState<'a>) -> Relation {
        Relation::from((self, rhs))
    }

    pub fn as_style(&self) -> dot::Style {
        if self.node.is_trait() {
            dot::Style::Dashed
        } else {
            dot::Style::Solid
        }
    }
}

pub enum Relation {
    Association,
    Aggregation,
    Composition,
    Realization,
    Dependency,
    None,
}

impl Relation {
    pub fn as_style(&self) -> dot::ArrowShape {
        match self {
            &Relation::Association => dot::ArrowShape::Vee(dot::Side::Both),
            &Relation::Dependency => dot::ArrowShape::Normal(dot::Fill::Filled, dot::Side::Both),
            &Relation::Aggregation => dot::ArrowShape::Diamond(dot::Fill::Open, dot::Side::Both),
            &Relation::Composition => dot::ArrowShape::Diamond(dot::Fill::Filled, dot::Side::Both),
            &Relation::Realization => dot::ArrowShape::Normal(dot::Fill::Open, dot::Side::Both),
            &Relation::None => dot::ArrowShape::NoArrow,
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

impl <'a>From<(Node<'a>, Vec<&'a syntax::ptr::P<syntax::ast::Item>>)> for ItemState<'a> {
    fn from((node, properties): (Node<'a>, Vec<&'a syntax::ptr::P<syntax::ast::Item>>)) -> ItemState<'a> {
        ItemState {
            node: node,
            method: properties.iter()
                .filter_map(|item: (&&'a syntax::ptr::P<syntax::ast::Item>)| {
                    if let syntax::ast::ItemKind::Impl(_, _, _, None, _, ref impl_item) = item.node {
                        Some(Method::from(impl_item))
                    } else {
                        None
                    }
                })
                .collect::<Vec<Method>>(),
            implem: properties.iter()
                .filter_map(|item: (&&'a syntax::ptr::P<syntax::ast::Item>)| {
                    if let syntax::ast::ItemKind::Impl(_, _, _, Some(syntax::ast::TraitRef {path: syntax::ast::Path {span: _, ref segments}, ..}), _, ref impl_item) = item.node {
                        Some(Implem::from((segments, impl_item)))
                    } else {
                        None
                    }
                })
                .collect::<Vec<Implem>>()
        }
    }
}

impl <'a>From<Vec<&'a syntax::ptr::P<syntax::ast::Item>>> for ItemState<'a> {
    fn from(state: Vec<&'a syntax::ptr::P<syntax::ast::Item>>) -> ItemState<'a> {
        state.split_first().and_then(|(ref item, properties): (&&'a syntax::ptr::P<syntax::ast::Item>, &[&'a syntax::ptr::P<syntax::ast::Item>])| {
            match &item.node {
                /// Trait.
                &syntax::ast::ItemKind::Trait(_, syntax::ast::Generics {lifetimes: _, ref ty_params, ..}, _, ref trait_item) => {
                    let kind: (&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::TraitItem>) = (item, ty_params, trait_item);
                    let kind: (Node, Vec<&'a syntax::ptr::P<syntax::ast::Item>>) = (Node::from(kind), properties.to_vec());
                    Some(ItemState::from(kind))
                },
                /// Structure with variables.
                &syntax::ast::ItemKind::Struct(syntax::ast::VariantData::Struct(ref struct_field, _), ..) => {
                    let kind: (&'a syntax::ast::Item, &'a Vec<syntax::ast::StructField>) = (item, struct_field);
                    let kind: (Node, Vec<&'a syntax::ptr::P<syntax::ast::Item>>) = (Node::from(kind), properties.to_vec());
                    Some(ItemState::from(kind))
                },
                /// Enumeration with variables.
                &syntax::ast::ItemKind::Enum(syntax::ast::EnumDef {ref variants}, syntax::ast::Generics {lifetimes: _, ref ty_params, ..}) => {
                    let kind: (&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::Variant>) = (item, ty_params, variants);
                    let kind: (Node, Vec<&'a syntax::ptr::P<syntax::ast::Item>>) = (Node::from(kind), properties.to_vec());
                    Some(ItemState::from(kind))
                },
                _ => None,
            }
        }).unwrap_or_default()
    }
}

impl <'a>fmt::Display for ItemState<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.method.is_empty() {
            write!(f, "{{{node}}}", node = self.node)
        } else {
            write!(f, "{{{node}|{method}}}",
                node = self.node,
                method = dot::escape_html(
                    self.method.iter()
                               .map(|ref methods| format!("{}", methods))
                               .collect::<Vec<String>>().join("\n").as_str()
                )
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListItem <'a> {
    parse: Item<'a>,
}

impl <'a> From<Item<'a>> for ListItem <'a> {
    fn from(parse: Item<'a>) -> ListItem <'a> {
        ListItem {
            parse: parse,
        }
    }
}

impl <'a>Iterator for ListItem<'a> {
    type Item = ItemState<'a>;

    fn next(&mut self) -> Option<ItemState<'a>> {
        self.parse.by_ref().skip_while(|state| state.is_none()).next()
    }
}

impl<'a> dot::Labeller<'a, ItemState<'a>, (ItemState<'a>, ItemState<'a>)> for ListItem<'a> {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("ml").unwrap()
    }

    fn node_id(&'a self, state: &ItemState<'a>) -> dot::Id<'a> {
        match state.as_name() {
            Ok(name) => dot::Id::new(format!("nd{}", name)).unwrap(),
        }
    }

    fn node_shape(&'a self, _node: &ItemState<'a>) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::LabelStr(Cow::from(format!("record"))))
    }

    fn node_label(&'a self, state: &ItemState<'a>) -> dot::LabelText<'a> {
        dot::LabelText::LabelStr(format!("{}", state).into())
    }

    fn node_style(&'a self, edge: &ItemState<'a>) -> dot::Style {
        edge.as_style()
    }

    fn edge_end_arrow(&'a self, &(ref edge_left, ref edge_right): &(ItemState<'a>, ItemState<'a>)) -> dot::Arrow {
        match (
            edge_left.as_arrow(edge_right),
            edge_right.as_arrow(edge_left)
        ) {
            (Relation::Association, Relation::Association) => dot::Arrow::none(),
            (edge_left, _) => dot::Arrow::from_arrow(edge_left.as_style()),
        }
    }
}

impl<'a> dot::GraphWalk<'a, ItemState<'a>, (ItemState<'a>, ItemState<'a>)> for ListItem<'a> {
    fn nodes(&'a self) -> dot::Nodes<'a, ItemState<'a>> {
        Cow::Owned(self.clone().collect::<Vec<ItemState<'a>>>())
    }
    
    fn edges(&'a self) -> dot::Edges<'a, (ItemState<'a>, ItemState<'a>)> {
        let items = self.clone().collect::<Vec<ItemState<'a>>>();

        Cow::Owned(items.iter()
                        .map(|item|
                             items.iter()
                                  .filter(|rhs| item.ne(rhs))
                                  .filter(|rhs| item.is_relation(rhs))
                                  .map(|rhs| (item.clone(), rhs.clone()))
                                  .collect::<Vec<(ItemState<'a>, ItemState<'a>)>>())
                        .collect::<Vec<Vec<(ItemState<'a>, ItemState<'a>)>>>()
                        .concat())
    }

    fn source(&self, e: &(ItemState<'a>, ItemState<'a>)) -> ItemState<'a> { let &(ref s, _) = e; s.clone() }

    fn target(&self, e: &(ItemState<'a>, ItemState<'a>)) -> ItemState<'a> { let &(_, ref t) = e; t.clone() }
}

fn main() {
    let codemap = Rc::new(CodeMap::new());
    let tty_handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(codemap.clone()));
    let parse_session = ParseSess::with_span_handler(tty_handler, codemap.clone());

    let path = Path::new("src/main.rs");
    let result: Result<syntax::ast::Crate, _> = parse::parse_crate_from_file(&path, &parse_session);

    if let Ok(parse) = result {
        let it: ListItem = ListItem::from(Item::from(parse.module.items.iter().peekable()));
        let mut f = File::create("example1.dot").unwrap();

        dot::render(&it, &mut f).unwrap()
    }
}
