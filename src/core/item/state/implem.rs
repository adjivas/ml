use super::DEFAULT_FUNC;

use std::ops::Deref;
use std::fmt;

use ::syntex_syntax::print::pprust::ty_to_string;
use ::syntex_syntax::symbol::InternedString;
use ::syntex_syntax::ast;

use ::dot::escape_html;

/// The structure `Implem` is a collection of methods and tyes for an abstract element.

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Implem {
    ty: Vec<(InternedString, Vec<String>)>,
    /// method's name, arguments, result.
    method: Vec<(InternedString, Vec<String>, Option<String>)>,
}

impl Implem {
    pub fn is_realization(&self, ty_name: &String) -> bool {
        if let Some(&(ref name, _)) = self.ty.first() {
            name.to_string().eq(ty_name)
        } else {
            false
        }
    }

    pub fn is_association(&self, ty_name: &String) -> bool {
        self.method.iter()
                   .any(|&(_, _, ref result): &(InternedString, Vec<String>, Option<String>)|
                       if let &Some(ref ret) = result {
                           ret.split(|at| "<[(;, )]>".contains(at))
                              .any(|ty| ty.eq(ty_name))
                       } else {
                           false
                       }
                   )
    }

    pub fn is_dependency(&self, _: &String) -> bool {
        false
        /*self.method.iter()
                   .any(|&( _, ref arg, _): &(InternedString, Vec<String>, Option<String>)|
                       arg.iter().any(|ty| ty.ends_with(name)))*/
    }
}

impl From<(Vec<(InternedString, Vec<String>)>, Vec<(InternedString, Vec<String>, Option<String>)>)> for Implem {
    fn from((ty, method): (Vec<(InternedString, Vec<String>)>, Vec<(InternedString, Vec<String>, Option<String>)>)) -> Implem {
        Implem {
            ty: ty,
            method: method,
        }
    }
}

impl <'a> From<(&'a Vec<ast::PathSegment>, &'a Vec<ast::ImplItem>)> for Implem {
    fn from((segments, impl_item): (&'a Vec<ast::PathSegment>, &'a Vec<ast::ImplItem>)) -> Implem {
        Implem::from((segments.iter()
                              .map(|&ast::PathSegment { identifier: ast::Ident {name, ..}, span: _, ref parameters }| {
                                  if let &Some(ref path) = parameters {
                                      if let &ast::PathParameters::AngleBracketed(
                                          ast::AngleBracketedParameterData { lifetimes: _, ref types, .. }
                                      ) = path.deref() {
                                          (name.as_str(), types.iter().map(|ty| ty_to_string(&ty)).collect::<Vec<String>>())
                                      } else {
                                          (name.as_str(), Vec::new())
                                      }
                                  } else {
                                      (name.as_str(), Vec::new())
                                  }
                              })
                              .collect::<Vec<(InternedString, Vec<String>)>>(),
                      impl_item.iter()
                               .filter_map(|&ast::ImplItem { id: _, ident: ast::Ident {name, ..}, vis: _, defaultness: _, attrs: _, ref node, ..}|
                                         if let &ast::ImplItemKind::Method(ast::MethodSig { unsafety: _, constness: _, abi: _, ref decl, .. }, _) = node {
                                             if let ast::FunctionRetTy::Ty(ref ty) = decl.output {
                                                 Some((name.as_str(), decl.inputs.iter().map(|arg| ty_to_string(&arg.ty)).collect::<Vec<String>>(), Some(ty_to_string(&ty))))
                                             } else {
                                                 Some((name.as_str(), decl.inputs.iter().map(|arg| ty_to_string(&arg.ty)).collect::<Vec<String>>(), None))
                                             }
                                         } else {
                                             None
                                         }
                               ).collect::<Vec<(InternedString, Vec<String>, Option<String>)>>()))
    }
}

impl fmt::Display for Implem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{item}",
           item = escape_html(self.method.iter()
                                         .map(|&(ref name, ref args, ref result): &(InternedString, Vec<String>, Option<String>)| {
                                             if let &Some(ref ret) = result {
                                                 format!("{}{}({}) -> {}", DEFAULT_FUNC, name, args.join(", "), ret)
                                             } else {
                                                 format!("{}{}({})", DEFAULT_FUNC, name, args.join(", "))
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
