use super::DEFAULT_FUNC;

use std::ops::Deref;
use std::fmt;

use ::syntex_syntax::print::pprust::{ty_to_string, arg_to_string};
use ::syntex_syntax::symbol::InternedString;
use ::syntex_syntax::ast;

use ::dot::escape_html;

/// The structure `Method` is a collection of methods from a abstract element.

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Method <'a> {
    /// visibility, method's name, arguments, result.
    func: Vec<(&'a ast::Visibility, InternedString, Vec<String>, Option<String>)>
}

impl <'a> Method <'a> {
    pub fn is_association(&self, ty_name: &String) -> bool {
        self.func.iter()
                 .any(|&(_, _, _, ref result): &(&'a ast::Visibility, InternedString, Vec<String>, Option<String>)|
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
                 .any(|&(_, _, ref arg, _): &(&'a ast::Visibility, InternedString, Vec<String>, Option<String>)|
                     arg.iter().any(|ty| ty.ends_with(name)))
    }
}

impl <'a> From<Vec<(&'a ast::Visibility, InternedString, Vec<String>, Option<String>)>> for Method<'a> {
    fn from(func: Vec<(&'a ast::Visibility, InternedString, Vec<String>, Option<String>)>) -> Method<'a> {
        Method {
            func: func,
        }
    }
}

impl <'a> From<&'a Vec<ast::ImplItem>> for Method<'a> {
    fn from(impl_item: &'a Vec<ast::ImplItem>) -> Method<'a> {
        Method::from(impl_item.iter()
                              .filter_map(|&ast::ImplItem {id: _, ident: ast::Ident { name, ..}, ref vis, defaultness: _, attrs: _, ref node, .. }| {
                                     if let &ast::ImplItemKind::Method(ast::MethodSig {unsafety: _, constness: _, abi: _, ref decl, ..}, _) = node {
                                         if let &ast::FnDecl {ref inputs, output: ast::FunctionRetTy::Ty(ref ty), ..} = decl.deref() {
                                             Some((vis, name.as_str(), inputs.iter().map(|ref arg| arg_to_string(&arg)).collect::<Vec<String>>(), Some(ty_to_string(&ty))))
                                         } else if let &ast::FnDecl {ref inputs, output: ast::FunctionRetTy::Default(_), ..} = decl.deref() {
                                             Some((vis, name.as_str(), inputs.iter().map(|ref arg| arg_to_string(&arg)).collect::<Vec<String>>(), None))
                                         } else {
                                             None
                                         }
                                     } else {
                                         None
                                     }
                               })
                               .collect::<Vec<(&'a ast::Visibility, InternedString, Vec<String>, Option<String>)>>())
    }
}

impl <'a>fmt::Display for Method<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{item}",
            item = escape_html(self.func.iter()
                                        .map(|&(ref vis, ref name, ref inputs, ref ty)|
                                               match (vis, ty) {
                                                   (&&ast::Visibility::Public, &Some(ref ty)) => {
                                                       format!("+{}{}({}) -> {}", DEFAULT_FUNC, name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "), ty)
                                                   },
                                                   (&&ast::Visibility::Public, &None) => {
                                                       format!("+{}{}({})", DEFAULT_FUNC, name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "))
                                                   },
                                                   (_, &Some(ref ty)) => {
                                                       format!("-{}{}({}) -> {}", DEFAULT_FUNC, name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "), ty)
                                                   },
                                                   (_, &None) => {
                                                       format!("-{}{}({})", DEFAULT_FUNC, name, inputs.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", "))
                                                   },
                                               }
                                           )
                                           .collect::<Vec<String>>()
                                           .join("\n")
                                           .as_str())
        )
    }
}
