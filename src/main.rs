#![feature(slice_patterns)]
#![feature(never_type)]

extern crate syntex_syntax as syntax;
extern crate syntex_errors;
extern crate itertools;
extern crate dot;

use std::{slice, iter, fmt};
use std::path::Path;
use std::rc::Rc;
use std::ops::Deref;
use std::ops::Not;
use std::borrow::Cow;
use std::fs::File;

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

#[derive(Debug, PartialEq, Clone)]
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
                      .filter_map(|&syntax::ast::TraitItem {id: _, ident: syntax::ast::Ident {name, ..}, attrs: _, ref node, ..}| {
                            if let &syntax::ast::TraitItemKind::Method(syntax::ast::MethodSig { unsafety: _, constness: _, abi: _, ref decl, ..}, _) = node {
                                if let &syntax::ast::FnDecl {ref inputs, output: syntax::ast::FunctionRetTy::Ty(ref ty), ..} = decl.deref() {
                                    Some((name.as_str(), inputs.iter().map(|input| ty_to_string(&input.ty)).collect::<Vec<String>>(), ty_to_string(&ty)))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
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
                            ident.and_then(|syntax::ast::Ident {name, ..}| Some((vis, name.as_str(), ty_to_string(&ty)))))
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
                                .map(|&(ref vis, ref name, ref ty): &(&syntax::ast::Visibility, syntax::symbol::InternedString, String)| {
                                    if syntax::ast::Visibility::Public.eq(vis) {
                                        format!("+ {}&lt;{}&gt;", name, ty)
                                    } else {
                                        format!("- {}&lt;{}&gt;", name, ty)
                                    }
                                })
                                .collect::<Vec<String>>()
                                .join("\n")
                                .as_str()),
            ),
            &Node::Enum(_, ref name, _, ref variants) => write!(f, "&lt;&lt;&lt;Enumeration&gt;&gt;&gt;|{name}|{variants}",
                name = name,
                variants = dot::escape_html(
                    variants.iter()
                            .map(|&(ref name, ref struct_field): &(syntax::symbol::InternedString, Vec<String>)| {
                                 if struct_field.is_empty() {
                                     format!("{}", name)
                                 } else {
                                     format!("{} : {}", name, struct_field.concat())
                                 }
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                            .as_str()),
            ),
            &Node::None => Err(fmt::Error),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ItemState<'a> {
    /// Data Type.
    node: Node<'a>,
    /// Implementation of Method.
    method: Vec<Vec<(&'a syntax::ast::Visibility, syntax::symbol::InternedString, Option<String>, std::vec::Vec<String>)>>,
    /// Implementation of Trait.
    implem: Vec<Vec<(syntax::symbol::InternedString, Vec<String>)>>,
}

impl <'a> ItemState <'a> {
    pub fn is_none(&self) -> bool {
        self.node.eq(&Node::None)
    }

    pub fn as_name(&self) -> Result<&syntax::symbol::InternedString, !> {
        self.node.as_name()
    }
}

impl <'a>From<(Node<'a>, Vec<&'a syntax::ptr::P<syntax::ast::Item>>)> for ItemState<'a> {
    fn from((node, properties): (Node<'a>, Vec<&'a syntax::ptr::P<syntax::ast::Item>>)) -> ItemState<'a> {
        ItemState {
            node: node,
            method: properties.iter()
                .filter_map(|item: (&&'a syntax::ptr::P<syntax::ast::Item>)| {
                    if let syntax::ast::ItemKind::Impl(_, _, _, None, _, ref impl_item) = item.node {
                        Some(impl_item.iter()
                             .filter_map(|&syntax::ast::ImplItem {id: _, ident: syntax::ast::Ident { name, ..}, ref vis, defaultness: _, attrs: _, ref node, .. }| {
                                    if let &syntax::ast::ImplItemKind::Method(syntax::ast::MethodSig {unsafety: _, constness: _, abi: _, ref decl, ..}, _) = node {
                                        if let &syntax::ast::FnDecl {ref inputs, output: syntax::ast::FunctionRetTy::Ty(ref ty), ..} = decl.deref() {
                                            Some((vis, name.as_str(), Some(ty_to_string(&ty)), inputs.iter().map(|ref arg| arg_to_string(&arg)).collect::<Vec<String>>()))
                                        } else if let &syntax::ast::FnDecl {ref inputs, output: syntax::ast::FunctionRetTy::Default(_), ..} = decl.deref() {
                                            Some((vis, name.as_str(), None, inputs.iter().map(|ref arg| arg_to_string(&arg)).collect::<Vec<String>>()))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                              })
                              .collect::<Vec<(&syntax::ast::Visibility, syntax::symbol::InternedString, Option<String>, std::vec::Vec<String>)>>())
                    } else {
                        None
                    }
                })
                .collect::<Vec<Vec<(&'a syntax::ast::Visibility, syntax::symbol::InternedString, Option<String>, std::vec::Vec<String>)>>>(),
            implem: properties.iter()
                .filter_map(|item: (&&'a syntax::ptr::P<syntax::ast::Item>)| {
                    if let syntax::ast::ItemKind::Impl(_, _, _, Some(syntax::ast::TraitRef {path: syntax::ast::Path {span: _, ref segments}, ..}), ..) = item.node {
                        Some(segments.iter()
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
                            .collect::<Vec<(syntax::symbol::InternedString, Vec<String>)>>())
                    } else {
                        None
                    }
                })
                .collect::<Vec<Vec<(syntax::symbol::InternedString, Vec<String>)>>>(),
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
            write!(f, "{{{}}}", self.node)
        } else {
            write!(f, "{{{}|{}}}", self.node,
                self.method.iter().map(|ref methods| {
                        methods.iter().map(|&(ref vis, ref name, ref ty, ref inputs)| {
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
                        })
                        .collect::<Vec<String>>().join("\n")
                })
                .collect::<Vec<String>>().join("\n"))
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
        while let Some(state) = self.parse.next() {
            if state.is_none().not() {
                return Some(state);
            }
        }
        None
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
}

impl<'a> dot::GraphWalk<'a, ItemState<'a>, (ItemState<'a>, ItemState<'a>)> for ListItem<'a> {
    fn nodes(&'a self) -> dot::Nodes<'a, ItemState<'a>> {
        Cow::Owned(self.clone().collect::<Vec<ItemState<'a>>>())
    }
    
    fn edges(&'a self) -> dot::Edges<'a, (ItemState<'a>, ItemState<'a>)> {
        Cow::Borrowed(&[])
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
