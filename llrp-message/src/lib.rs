#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::TokenStream;

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Data, DeriveInput, Expr, Fields, Ident, ItemStruct, Token,
};

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

fn decode_fields(name: &Ident, fields: &mut Fields) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let rest_ident = Ident::new("__rest", Span::call_site());

            let mut parse_entries = vec![];
            let mut struct_body = vec![];

            for field in fields.named.iter_mut() {
                let tv_param_attr = {
                    let pos = field.attrs.iter().position(|x| {
                        x.path.segments.iter().next().map_or(false, |x| x.ident == "tv_param")
                    });
                    pos.map(|i| field.attrs.remove(i))
                };

                let field_name = &field.ident;
                let ty = &field.ty;

                match tv_param_attr {
                    Some(attr) => {
                        let id = match attr.parse_meta().unwrap() {
                            syn::Meta::Word(_) | syn::Meta::List(_) => {
                                panic!("expected `tv_param = id`")
                            }
                            syn::Meta::NameValue(value) => value.lit,
                        };
                        parse_entries.push(quote! {
                            let (#field_name, #rest_ident) =
                                <#ty as llrp_common::TvDecodable>::decode_tv(#rest_ident, #id)?;
                        });
                    }
                    None => {
                        parse_entries.push(quote! {
                            eprintln!("parsing: {}", stringify!(#field_name));
                            let (#field_name, #rest_ident) =
                                <#ty as llrp_common::LLRPDecodable>::decode(#rest_ident)?;
                        });
                    }
                }

                struct_body.push(quote!(#field_name));
            }

            quote! {
                let #rest_ident = data;
                #(#parse_entries)*
                Ok((#name { #(#struct_body,)* }, #rest_ident))
            }
        }
        Fields::Unnamed(_) => panic!("`llrp_value` unsupported for tuple structs"),
        Fields::Unit => {
            quote! {
                if data.len() != 0 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid length").into());
                }
                Ok((#name, data))
            }
        }
    }
}

#[proc_macro_attribute]
pub fn llrp_message(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input: ItemStruct = parse_macro_input!(input as ItemStruct);

    let args = parse_macro_input!(args as Args);
    let id = args.id;

    let struct_name = input.ident.clone();
    let decode_fields = decode_fields(&struct_name, &mut input.fields);

    let expanded = quote! {
        #input

        impl llrp_common::LLRPDecodable for #struct_name {
            const ID: u16 = #id;

            fn decode(data: &[u8]) -> llrp_common::Result<(Self, &[u8])> {
                #decode_fields
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn llrp_parameter(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input: ItemStruct = parse_macro_input!(input as ItemStruct);

    let args = parse_macro_input!(args as Args);
    let id = args.id;

    let struct_name = input.ident.clone();
    let decode_fields = decode_fields(&struct_name, &mut input.fields);

    let expanded = quote! {
        #input

        impl llrp_common::LLRPDecodable for #struct_name {
            const ID: u16 = #id;

            fn decode(data: &[u8]) -> llrp_common::Result<(Self, &[u8])> {
                eprintln!("\nparsing: {}", stringify!(#struct_name));

                if data.len() < 2 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
                }

                eprintln!("data = {:02x?}", data);
                // [6-bit resv, 10-bit message type]
                let __type = u16::from_be_bytes([data[0], data[1]]) & 0b11_1111_1111;
                eprintln!("type = {}", __type);
                if __type != Self::ID {
                    return Err(llrp_common::Error::InvalidType(__type))
                }

                // 16-bit length
                let __len = u16::from_be_bytes([data[2], data[3]]) as usize;
                eprintln!("len = {} (data.len() = {})", __len, data.len());

                if __len > data.len() {
                    // Length was larger than the remaining data
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid length").into());
                }

                // Decode the inner fields for this parameter
                let result: llrp_common::Result<(Self, &[u8])> = {
                    let data = &data[4..__len];
                    #decode_fields
                };
                let (inner, __rest) = result?;

                // Ensure that all bytes were consumed when parsing the struct fields
                if __rest.len() != 0 {
                    eprintln!("__rest.len() = {}", __rest.len());
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Did not use consume all bytes in the parameter when inner parameters",
                    )
                    .into());
                }

                Ok((inner, &data[__len..]))
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(TryFromU16)]
pub fn derive_try_from_u16(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let possible_matches = match input.data {
        Data::Enum(ref inner) => inner.variants.iter().map(|v| {
            let name = &v.ident;
            quote!(__value if __value == #name as u16 => #name)
        }),
        _ => panic!("Only supported for enums"),
    };

    let expanded = quote! {
        impl std::convert::TryFrom<u16> for #name {
            type Error = u16;
            fn try_from(value: u16) -> Result<#name, u16> {
                use #name::*;
                let result = match value {
                     #(#possible_matches,)*
                     _ => return Err(value)
                };
                Ok(result)
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
