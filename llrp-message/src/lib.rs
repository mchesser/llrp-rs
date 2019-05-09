extern crate proc_macro;

use self::proc_macro::TokenStream;

use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Expr, Ident, ItemStruct, Token};

struct Args {
    id: Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        if ident != "id" {
            return Err(syn::Error::new(ident.span(), "expected `id`"));
        }

        input.parse::<Token![=]>()?;
        let id = input.parse::<Expr>()?;

        Ok(Args {
            id,
        })
    }
}

#[proc_macro_attribute]
pub fn llrp_message(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let args = parse_macro_input!(args as Args);

    let name = input.ident.clone();
    let id = args.id;

    let expanded = quote! {
        #input

        impl llrp_common::DecodableMessage for #name {
            const ID: u16 = #id;

            fn decode(data: &[u8]) -> std::io::Result<Self> {
                unimplemented!()
            }
        }
    };

    TokenStream::from(expanded)
}
