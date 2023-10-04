use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Lit, LitChar, LitStr, Token,
};

struct Delimiter {
    marker: LitChar,
    left: LitStr,
    right: Option<LitStr>,
}

impl Parse for Delimiter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(input.error("Missing left delimiter"));
        }

        let (marker, left) = match input.parse::<Lit>()? {
            Lit::Str(left) => {
                let marker_ch = left
                    .value()
                    .chars()
                    .next()
                    .expect("Left delimiter cannot be empty");

                let marker = LitChar::new(marker_ch, left.span());

                (marker, left)
            }
            Lit::Char(marker) => {
                let left = LitStr::new(&marker.value().to_string(), marker.span());
                (marker, left)
            }
            lit => {
                return Err(syn::Error::new(
                    lit.span(),
                    "Expected a string or a character",
                ));
            }
        };

        if input.is_empty() {
            return Ok(Self {
                marker,
                left,
                right: None,
            });
        }

        input.parse::<Token![,]>()?;

        if input.is_empty() {
            return Ok(Self {
                marker,
                left,
                right: None,
            });
        }

        let right = match input.parse::<Lit>()? {
            Lit::Str(s) => s,
            Lit::Char(c) => LitStr::new(&c.value().to_string(), c.span()),
            lit => {
                return Err(syn::Error::new(
                    lit.span(),
                    "Expected a string or a character",
                ));
            }
        };

        if !input.is_empty() {
            return Err(input.error("Unexpected token"));
        }

        Ok(Self {
            marker,
            left,
            right: Some(right),
        })
    }
}

#[proc_macro_derive(InlineDelimiter, attributes(delimiter))]
pub fn delimiter_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_inline_delimiter(&ast)
}

fn impl_inline_delimiter(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    for attr in &ast.attrs {
        if !attr.meta.path().is_ident("delimiter") {
            continue;
        }

        let Delimiter {
            marker,
            left,
            right,
        } = attr.meta.require_list().unwrap().parse_args().unwrap();

        let right = right.unwrap_or_else(|| left.clone());

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
