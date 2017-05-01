use std::fmt;
use std::ops::Deref;

use ::syntex_syntax::print::pprust::ty_to_string;
use ::syntex_syntax::{symbol, ast};

use ::dot::escape_html;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Trait<'a> {
    /// Visibility
    pub vis: &'a ast::Visibility,
    pub name: symbol::InternedString,
    pub params: Vec<symbol::InternedString>,
    pub items: Vec<(symbol::InternedString, Vec<String>, String)>,
}

impl <'a>From<(&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::TraitItem>)> for Trait<'a> {
    fn from((item, ty_params, trait_item): (&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::TraitItem>)) -> Trait<'a> {
        Trait {
            vis: &item.vis,
            name: item.ident.name.as_str(),
            params: ty_params.iter()
                             .map(|&ast::TyParam {attrs: _, ident: ast::Ident {name, ..}, ..}| name.as_str())
                             .collect::<Vec<symbol::InternedString>>(),
            items: trait_item.iter()
                             .filter_map(|&ast::TraitItem {id: _, ident: ast::Ident {name, ..}, attrs: _, ref node, ..}|
                                   if let &ast::TraitItemKind::Method(ast::MethodSig { unsafety: _, constness: _, abi: _, ref decl, ..}, _) = node {
                                       if let &ast::FnDecl {ref inputs, output: ast::FunctionRetTy::Ty(ref ty), ..} = decl.deref() {
                                           Some((name.as_str(), inputs.iter().map(|input| ty_to_string(&input.ty)).collect::<Vec<String>>(), ty_to_string(&ty)))
                                       } else {
                                           None
                                       }
                                   } else {
                                       None
                                   }
                             )
                            .collect::<Vec<(symbol::InternedString, Vec<String>, String)>>()
        }
    }
}

impl <'a>fmt::Display for Trait<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "&lt;&lt;&lt;Trait&gt;&gt;&gt;\n{name}|{items}",
           name = self.name,
           items = escape_html(self.items.iter()
                                   .map(|&(ref name, ref ty, ref ret): &(symbol::InternedString, Vec<String>, String)|
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
