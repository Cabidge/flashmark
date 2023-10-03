use proc_macro::TokenStream;
use syn::{Expr, Lit, LitChar, LitStr};

#[proc_macro_derive(InlineDelimiter, attributes(delimiter))]
pub fn delimiter_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_inline_delimiter(&ast)
}

fn delim_and_marker(s: &LitStr) -> (LitChar, LitStr) {
    let marker_ch = s.value().chars().next().unwrap();
    let marker = LitChar::new(marker_ch, s.span());

    (marker, s.to_owned())
}

fn impl_inline_delimiter(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    for attr in &ast.attrs {
        if !attr.meta.path().is_ident("delimiter") {
            continue;
        }

        let Ok(meta) = attr.meta.require_name_value() else { continue };

        let (marker, left, right) = match &meta.value {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Char(c) => {
                    let delimiter = LitStr::new(&c.value().to_string(), c.span());
                    (c.to_owned(), delimiter.clone(), delimiter)
                }
                Lit::Str(s) => {
                    let (marker, left) = delim_and_marker(s);

                    let right: String = left.value().chars().rev().collect();
                    let right = LitStr::new(&right, left.span());

                    (marker, left, right)
                }
                _ => panic!(),
            },
            _ => panic!(),
        };

        return quote::quote! {
            impl InlineDelimiter for #name {
                const MARKER: char = #marker;
                const LEFT_DELIM: &'static str = #left;
                const RIGHT_DELIM: &'static str = #right;
            }
        }
        .into();
    }

    panic!("No delimiter attribute found")
}
