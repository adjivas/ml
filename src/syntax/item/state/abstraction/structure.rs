use std::fmt;

use ::syntex_syntax::print::pprust::ty_to_string;
use ::syntex_syntax::{symbol, ast};

use ::dot::escape_html;

/// The structure `Struct` is a structure abstract element.

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Struct<'a> {
    pub vis: &'a ast::Visibility,
    pub name: symbol::InternedString,
    pub fields: Vec<(&'a ast::Visibility, symbol::InternedString, String)>,
}

impl <'a>From<(&'a ast::Item, &'a Vec<ast::StructField>)> for Struct<'a> {
    fn from((item, struct_field): (&'a ast::Item, &'a Vec<ast::StructField>)) -> Struct<'a> {
        Struct {
            vis: &item.vis,
            name: item.ident.name.as_str(),
            fields: struct_field.iter()
                                .filter_map(|&ast::StructField { span: _, ident, ref vis, id: _, ref ty, .. }|
                                           ident.and_then(|ast::Ident {name, ..}| Some((vis, name.as_str(), ty_to_string(&ty)))))
                                .collect::<Vec<(&ast::Visibility, symbol::InternedString, String)>>()
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
                fields = escape_html(self.fields.iter()
                                                .map(|&(ref vis, ref name, ref ty): &(&ast::Visibility, symbol::InternedString, String)|
                                                    if ast::Visibility::Public.eq(vis) {
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
