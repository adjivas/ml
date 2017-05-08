use std::fmt;

use ::syntex_syntax::print::pprust::ty_to_string;
use ::syntex_syntax::{codemap, symbol, ast};

use ::dot::escape_html;

/// The structure `Enum` is a enumerate abstract element.

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Enum<'a> {
    pub vis: &'a ast::Visibility,
    pub name: symbol::InternedString,
    pub params: Vec<symbol::InternedString>,
    pub variants: Vec<(symbol::InternedString, Vec<String>)>,
}

impl <'a>From<(&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::Variant>)> for Enum<'a> {
    fn from((item, ty_params, variants): (&'a ast::Item, &'a Vec<ast::TyParam>, &'a Vec<ast::Variant>)) -> Enum<'a> {
        Enum {
            vis: &item.vis,
            name: item.ident.name.as_str(),
            params: ty_params.iter()
                             .map(|&ast::TyParam {attrs: _, ident: ast::Ident {name, ..}, ..}| name.as_str())
                             .collect::<Vec<symbol::InternedString>>(),
            variants: variants.iter()
                              .map(|&codemap::Spanned {node: ast::Variant_ {name: ast::Ident {name, ..}, attrs: _, ref data, ..}, ..}| {
                                   if let &ast::VariantData::Tuple(ref struct_field, _) = data {
                                       (name.as_str(),
                                        struct_field.iter()
                                                    .filter_map(|&ast::StructField { span: _, ident: _, vis: _, id: _, ref ty, .. }| Some(ty_to_string(&ty)))
                                                    .collect::<Vec<String>>())
                                   } else {
                                       (name.as_str(), Vec::new())
                                   }
                              })
                              .collect::<Vec<(symbol::InternedString, Vec<String>)>>(),
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
                variants = escape_html(self.variants.iter()
                                           .map(|&(ref name, ref struct_field): &(symbol::InternedString, Vec<String>)|
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


