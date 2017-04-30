#![feature(slice_patterns)]
#![feature(never_type)]

extern crate syntex_syntax as syntax;
extern crate syntex_errors;
extern crate itertools;
extern crate dot;

use std::ops::{BitOr, BitAnd};
use std::hash::{Hash, Hasher};
use std::{slice, iter};
use std::path::Path;
use std::rc::Rc;
use std::ops::Deref;
use std::borrow::Cow;
use std::fmt;
use std::vec;

use syntax::codemap::CodeMap;
use syntax::print::pprust::{ty_to_string, arg_to_string};
use syntex_errors::Handler;
use syntex_errors::emitter::ColorConfig;
use syntax::parse::{self, ParseSess};

use itertools::Itertools;

/// The structure Item is a iterable collection of abstract elements.

#[derive(Debug, Clone)]
struct Item <'a> {
    /// Iterator.
    it: iter::Peekable<slice::Iter<'a, syntax::ptr::P<syntax::ast::Item>>>,
}

impl <'a>From<iter::Peekable<slice::Iter<'a, syntax::ptr::P<syntax::ast::Item>>>> for Item<'a> {

    /// The constructor method `from` returns a typed and iterable collection of abstract element.
    fn from(iter: iter::Peekable<slice::Iter<'a, syntax::ptr::P<syntax::ast::Item>>>) -> Item {
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
            let mut list: Vec<&'a syntax::ptr::P<syntax::ast::Item>> = vec!(item);

            list.extend(self.it.peeking_take_while(|ref item| {
                            if let syntax::ast::ItemKind::Impl(_, _, _, _, _, _) = item.node {
                                true
                            } else {    
                                false
                            }
                        })
                        .collect::<Vec<&'a syntax::ptr::P<syntax::ast::Item>>>());
            Some(ItemState::from(list))
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Trait<'a> {
    /// Visibility
    pub vis: &'a syntax::ast::Visibility,
    pub name: syntax::symbol::InternedString,
    pub params: Vec<syntax::symbol::InternedString>,
    pub items: Vec<(syntax::symbol::InternedString, Vec<String>, String)>,
}

impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::TraitItem>)> for Trait<'a> {
    fn from((item, ty_params, trait_item): (&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::TraitItem>)) -> Trait<'a> {
        Trait {
            vis: &item.vis,
            name: item.ident.name.as_str(),
            params: ty_params.iter()
                             .map(|&syntax::ast::TyParam {attrs: _, ident: syntax::ast::Ident {name, ..}, ..}| name.as_str())
                             .collect::<Vec<syntax::symbol::InternedString>>(),
            items: trait_item.iter()
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
        }
    }
}

impl <'a>fmt::Display for Trait<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "&lt;&lt;&lt;Trait&gt;&gt;&gt;\n{name}|{items}",
           name = self.name,
           items = dot::escape_html(self.items.iter()
                                        .map(|&(ref name, ref ty, ref ret): &(syntax::symbol::InternedString, Vec<String>, String)|
                                             format!("{name}({ty}) -&gt; {ret}",
                                                 name = name,
                                                 ty = ty.join(", "),
                                                 ret = ret
                                             ))
                                        .collect::<Vec<String>>()
                                        .join("\n")
                                        .as_str())
        )
    }
}

/// The structure `Struct` is a structure abstract element.

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Struct<'a> {
    pub vis: &'a syntax::ast::Visibility,
    pub name: syntax::symbol::InternedString,
    pub fields: Vec<(&'a syntax::ast::Visibility, syntax::symbol::InternedString, String)>,
}

impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::StructField>)> for Struct<'a> {
    fn from((item, struct_field): (&'a syntax::ast::Item, &'a Vec<syntax::ast::StructField>)) -> Struct<'a> {
        Struct {
            vis: &item.vis,
            name: item.ident.name.as_str(),
            fields: struct_field.iter()
                                .filter_map(|&syntax::ast::StructField { span: _, ident, ref vis, id: _, ref ty, .. }|
                                           ident.and_then(|syntax::ast::Ident {name, ..}| Some((vis, name.as_str(), ty_to_string(&ty)))))
                                .collect::<Vec<(&syntax::ast::Visibility, syntax::symbol::InternedString, String)>>()
        }
    }
}

impl <'a>fmt::Display for Struct<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.fields.is_empty() {
            write!(f, "&lt;&lt;&lt;Structure&gt;&gt;&gt;\n{name}", name = self.name)
        } else {
            write!(f, "&lt;&lt;&lt;Structure&gt;&gt;&gt;\n{name}|{fields}",
                name = self.name,
                fields = dot::escape_html(self.fields.iter()
                                                      .map(|&(ref vis, ref name, ref ty): &(&syntax::ast::Visibility, syntax::symbol::InternedString, String)|
                                                          if syntax::ast::Visibility::Public.eq(vis) {
                                                              format!("+ {name}: {ty}", name = name, ty = ty)
                                                          } else {
                                                              format!("- {name}: {ty}", name = name, ty = ty)
                                                          }
                                                      )
                                                      .collect::<Vec<String>>()
                                                      .join("\n")
                                                      .as_str()),
            )
        }
    }
}

/// The structure `Enum` is a enumerate abstract element.

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Enum<'a> {
    pub vis: &'a syntax::ast::Visibility,
    pub name: syntax::symbol::InternedString,
    pub params: Vec<syntax::symbol::InternedString>,
    pub variants: Vec<(syntax::symbol::InternedString, Vec<String>)>,
}

impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::Variant>)> for Enum<'a> {
    fn from((item, ty_params, variants): (&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::Variant>)) -> Enum<'a> {
        Enum {
            vis: &item.vis,
            name: item.ident.name.as_str(),
            params: ty_params.iter()
                             .map(|&syntax::ast::TyParam {attrs: _, ident: syntax::ast::Ident {name, ..}, ..}| name.as_str())
                             .collect::<Vec<syntax::symbol::InternedString>>(),
            variants: variants.iter()
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
                              .collect::<Vec<(syntax::symbol::InternedString, Vec<String>)>>(),
        }
    }
}

impl <'a>fmt::Display for Enum<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.variants.is_empty() {
            write!(f, "&lt;&lt;&lt;Enumeration&gt;&gt;&gt;\n{name}", name = self.name)
        } else {
            write!(f, "&lt;&lt;&lt;Enumeration&gt;&gt;&gt;\n{name}|{variants}",
                name = self.name,
                variants = dot::escape_html(self.variants.iter()
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
            )
        }
    }
}

/// The structure `Node` is a enumerate for abstract element types or none.

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Node <'a> {
    Trait(Trait<'a>),
    Struct(Struct<'a>),
    Enum(Enum<'a>),
    None,
}

impl <'a> Node <'a> {

    /// The method `as_name` returns the name of the abstract element
    /// or else declare a panic.
    pub fn as_name(&self) -> Result<&syntax::symbol::InternedString, !> {
        match self {
            &Node::Trait(Trait { vis: _, ref name, ..}) => Ok(name),
            &Node::Struct(Struct { vis: _, ref name, ..}) => Ok(name),
            &Node::Enum(Enum { vis: _, ref name, ..}) => Ok(name),
            &Node::None => unreachable!(),
        }
    }
}

impl<'a> IntoIterator for &'a Node<'a> {
    type Item = &'a String;
    type IntoIter = vec::IntoIter<&'a String>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            &Node::Struct(Struct {vis: _, name: _, fields: ref ty_field}) => {
                ty_field.iter()
                        .map(|&(_, _, ref ty): &'a (&'a syntax::ast::Visibility, syntax::symbol::InternedString, std::string::String)| ty)
                        .collect::<Vec<&'a String>>()
                        .into_iter()
            },
            &Node::Enum(Enum {vis: _, name: _, params: _, variants: ref ty_multi_field}) => {
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

impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::TraitItem>)> for Node<'a> {
    fn from(arguments: (&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::TraitItem>)) -> Node<'a> {
        Node::Trait(Trait::from(arguments))
    }
}

impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::StructField>)> for Node<'a> {
    fn from(arguments: (&'a syntax::ast::Item, &'a Vec<syntax::ast::StructField>)) -> Node<'a> {
        Node::Struct(Struct::from(arguments))
    }
}

impl <'a>From<(&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::Variant>)> for Node<'a> {
    fn from(arguments: (&'a syntax::ast::Item, &'a Vec<syntax::ast::TyParam>, &'a Vec<syntax::ast::Variant>)) -> Node<'a> {
        Node::Enum(Enum::from(arguments))
    }
}

impl <'a>fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Node::Struct(ref item) => write!(f, "{}", item),
            &Node::Enum(ref item) => write!(f, "{}", item),
            &Node::Trait(ref item) => write!(f, "{}", item),
            &Node::None => Err(fmt::Error),
        }
    }
}

/// The structure `Method` is a collection of methods from a abstract element.

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
        self.func.iter()
                 .any(|&(_, _, ref arg, _): &(&'a syntax::ast::Visibility, syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)|
                     arg.iter().any(|ty| ty.ends_with(name)))
    }

    pub fn is_relation(&'a self, ty_name: &String) -> bool {
        self.is_association(ty_name).bitor(self.is_dependency(ty_name))
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
        write!(f, "{item}",
            item = dot::escape_html(self.func.iter()
                                             .map(|&(ref vis, ref name, ref inputs, ref ty)|
                                                    match (vis, ty) {
                                                        (&&syntax::ast::Visibility::Public, &Some(ref ty)) => {
                                                            format!("+ {}({}) -&gt; {}", name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "), ty)
                                                        },
                                                        (&&syntax::ast::Visibility::Public, &None) => {
                                                            format!("+ {}({})", name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "))
                                                        },
                                                        (_, &Some(ref ty)) => {
                                                            format!("- {}({}) -&gt; {}", name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "), ty)
                                                        },
                                                        (_, &None) => {
                                                            format!("- {}({})", name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "))
                                                        },
                                                    }
                                                )
                                                .collect::<Vec<String>>()
                                                .join("\n")
                                                .as_str())
        )
    }
}

/// The structure `Implem` is a collection of methods and tyes for an abstract element.

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Implem {
    ty: Vec<(syntax::symbol::InternedString, Vec<String>)>,
    /// method's name, arguments, result.
    method: Vec<(syntax::symbol::InternedString, Vec<String>, Option<String>)>,
}

impl Implem {
    pub fn is_relation(&self, ty_name: &String) -> bool {
        self.is_association(ty_name).bitor(self.is_dependency(ty_name))
    }

    pub fn is_realization(&self, ty_name: &String) -> bool {
        self.ty.iter()
               .any(|&(ref name, _): &(syntax::symbol::InternedString, Vec<String>)|
                    name.to_string().eq(ty_name))
    }

    pub fn is_association(&self, ty_name: &String) -> bool {
        self.method.iter()
                   .any(|&(_, _, ref result): &(syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)|
                       if let &Some(ref ret) = result {
                           ret.split(|at| "<[(;, )]>".contains(at))
                              .any(|ty| ty.eq(ty_name))
                       } else {
                           false
                       }
                   )
    }

    pub fn is_dependency(&self, name: &String) -> bool {
        false
        /*self.method.iter()
                   .any(|&( _, ref arg, _): &(syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)|
                       arg.iter().any(|ty| ty.ends_with(name)))*/
    }
}

impl From<(Vec<(syntax::symbol::InternedString, Vec<String>)>, Vec<(syntax::symbol::InternedString, Vec<String>, Option<String>)>)> for Implem {
    fn from((ty, method): (Vec<(syntax::symbol::InternedString, Vec<String>)>, Vec<(syntax::symbol::InternedString, Vec<String>, Option<String>)>)) -> Implem {
        Implem {
            ty: ty,
            method: method,
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

impl fmt::Display for Implem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{item}",
           item = dot::escape_html(self.method.iter()
                                       .map(|&(ref name, ref args, ref result): &(syntax::symbol::InternedString, std::vec::Vec<String>, Option<String>)| {
                                           if let &Some(ref ret) = result {
                                               format!("{}({}) -&gt; {}", name, args.join(", "), ret)
                                           } else {
                                               format!("{}({})", name, args.join(", "))
                                           }
                                       })
                                       .collect::<Vec<String>>()
                                       .join("\n")
                                       .as_str()))
        /*if let Some(&(ref name, ref template)) = self.ty.last() {
            if template.is_empty() {
                write!(f, "{name}", name = name.to_string())
            } else {
                write!(f, "{name}&lt;{item}&gt;",
                   name = name.to_string(),
                   item = dot::escape_html(template.join(", ")
                                                   .as_str()))
           }
        } else {
            Ok(())
        }*/
    }
}

/// The structure `ItemState` describes an abstract element with a collections of methodes
/// and implementations.
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
                          .bitor(rhs.implem.iter()
                                           .any(|implem| implem.is_association(&ty_name)))

            }
        }
    }

    pub fn is_dependency(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let ref ty_name: String = name.to_string();

                rhs.method.iter()
                          .any(|method| method.is_dependency(&ty_name))
                          .bitor(self.implem.iter()
                                            .any(|implem| implem.is_dependency(&ty_name)))
            }
        }
    }

    pub fn is_aggregation(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let mut ty_name_mut: String = String::from("*mut ");
                let mut ty_name_const: String = String::from("*const ");
                
                ty_name_mut.push_str(&name);
                ty_name_const.push_str(&name);
                rhs.node.into_iter()
                        .any(|attribut: &String|
                              attribut.split(|at| "<[(;,)]>".contains(at))
                                      .any(|ty| ty_name_mut.eq(ty).bitor(ty_name_const.eq(ty))))
            }
        }
    }

    pub fn is_composition(&self, rhs: &ItemState<'a>) -> bool {
        match self.as_name() {
            Ok(name) => {
                let ty_name: String = name.to_string();

                rhs.node.into_iter()
                        .any(|attribut: &String|
                              attribut.split(|at| "<[(;,)]>".contains(at))
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
        self.is_association(rhs)
            .bitor(self.is_dependency(rhs))
            .bitor(self.is_aggregation(rhs))
            .bitor(self.is_composition(rhs))
            .bitor(self.is_realization(rhs))
    }

    pub fn as_name(&self) -> Result<&syntax::symbol::InternedString, !> {
        self.node.as_name()
    }

    pub fn as_arrow(&self, rhs: &ItemState<'a>) -> Relation {
        Relation::from((self, rhs))
    }
}

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
    pub fn as_style(&self) -> dot::ArrowShape {
        match self {
            &Relation::Association => dot::ArrowShape::Vee(dot::Side::Both),
            &Relation::Dependency => dot::ArrowShape::Vee(dot::Side::Both),
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
                .filter_map(|item: (&&'a syntax::ptr::P<syntax::ast::Item>)|
                    if let syntax::ast::ItemKind::Impl(_, _, _, None, _, ref impl_item) = item.node {
                        Some(Method::from(impl_item))
                    } else {
                        None
                    }
                )
                .collect::<Vec<Method>>(),
            implem: properties.iter()
                .filter_map(|item: (&&'a syntax::ptr::P<syntax::ast::Item>)|
                    if let syntax::ast::ItemKind::Impl(_, _, _, Some(syntax::ast::TraitRef {path: syntax::ast::Path {span: _, ref segments}, ..}), _, ref impl_item) = item.node {
                        Some(Implem::from((segments, impl_item)))
                    } else {
                        None
                    }
                )
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

impl<'a> dot::Labeller<'a, ItemState<'a>, Segment<'a>> for ListItem<'a> {
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

    fn edge_end_arrow(&'a self, ref seg: &Segment<'a>) -> dot::Arrow {
        match (
            seg.left.as_arrow(&seg.right),
            seg.right.as_arrow(&seg.left)
        ) {
            (Relation::Association, Relation::Association) => dot::Arrow::none(),
            (edge_left, _) => dot::Arrow::from_arrow(edge_left.as_style()),
        }
    }

    fn edge_style(&'a self, ref seg: &Segment<'a>) -> dot::Style {
        if seg.left.is_realization(&seg.right).bitor(seg.left.is_dependency(&seg.right)) {
            dot::Style::Dashed
        } else {
            dot::Style::None
        }
    }
}

impl<'a> dot::GraphWalk<'a, ItemState<'a>, Segment<'a>> for ListItem<'a> {
    fn nodes(&'a self) -> dot::Nodes<'a, ItemState<'a>> {
        Cow::Owned(self.clone().collect::<Vec<ItemState<'a>>>())
    }
    
    fn edges(&'a self) -> dot::Edges<'a, Segment<'a>> {
        let items = self.clone().collect::<Vec<ItemState<'a>>>();

        Cow::Owned(items.iter()
                        .map(|item| items.iter()
                                         .filter(|rhs| item.ne(rhs))
                                         .filter(|rhs| item.is_relation(rhs))
                                         .map(|rhs| Segment::from((item.clone(), rhs.clone())))
                                         .collect::<Vec<Segment<'a>>>())
                        .collect::<Vec<Vec<Segment<'a>>>>()
                        .concat()
                        .into_iter()
                        .unique()
                        .collect::<Vec<Segment<'a>>>())
    }

    fn source(&self, seg: &Segment<'a>) -> ItemState<'a> { seg.left.clone() }

    fn target(&self, seg: &Segment<'a>) -> ItemState<'a> { seg.right.clone() }
}

pub fn from_file<'a>(path: &Path) -> Option<Vec<u8>> {
    let codemap = Rc::new(CodeMap::new());
    let tty_handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(codemap.clone()));
    let parse_session: ParseSess = ParseSess::with_span_handler(tty_handler, codemap.clone());
    let parse = parse::parse_crate_from_file(&path, &parse_session);

    match parse {
        Err(_) => None,
        Ok(parse) => {
            let it: ListItem = ListItem::from(Item::from(parse.module.items.iter().peekable()));
            let mut f = Vec::new();

            dot::render(&it, &mut f).unwrap();
            Some(f)
        },
    }
}
