#![recursion_limit = "256"]

extern crate proc_macro;

use proc_macro2::TokenStream;

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Data, DeriveInput, Expr, Field, Fields, FieldsNamed, Ident, ItemStruct, Token,
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

fn decode_struct_fields(name: &Ident, fields: &mut Fields) -> syn::Result<TokenStream> {
    match fields {
        Fields::Named(fields) => decode_struct_named_fields(name, fields),
        Fields::Unnamed(_) => panic!("`llrp_value` unsupported for tuple structs"),
        Fields::Unit => Ok(quote! {
            if data.len() != 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid length").into());
            }
            Ok((#name, data))
        }),
    }
}

/// Decodes a struct made up of named fields
fn decode_struct_named_fields(name: &Ident, fields: &mut FieldsNamed) -> syn::Result<TokenStream> {
    let remaining_data = Ident::new("__rest", Span::call_site());
    let mut decoding = vec![];
    let mut field_assignments = vec![];

    for field in fields.named.iter_mut() {
        let decoded = decode_field(&remaining_data, field)?;
        let field_name = &field.ident;

        decoding.push(quote! {
            eprintln!("decoding: {}", stringify!(#field_name));
            let (#field_name, #remaining_data) = #decoded;
        });

        field_assignments.push(quote!(#field_name))
    }

    Ok(quote!({
        let #remaining_data = data;

        // Decode each of the fields in order
        #(#decoding)*

        // Build the struct, assigning all decoded fields
        let __result = #name {
            #(#field_assignments,)*
        };

        // Return the struct and any remaining data
        Ok((__result, #remaining_data))
    }))
}

enum FieldAttribute {
    TvParam(syn::Lit),
    HasLength,
    None,
}

impl FieldAttribute {
    fn parse(field: &mut Field) -> syn::Result<FieldAttribute> {
        // Parses a literal value from an attribute
        fn parse_attr_lit(attr: &syn::Attribute) -> syn::Result<syn::Lit> {
            match attr.parse_meta()? {
                syn::Meta::Word(_) | syn::Meta::List(_) => {
                    Err(syn::Error::new(attr.span(), "expected literal"))
                }
                syn::Meta::NameValue(value) => Ok(value.lit),
            }
        }

        for (i, attr) in field.attrs.iter().enumerate() {
            let segment = attr.path.segments.iter().next();
            match segment {
                Some(x) if x.ident == "tv_param" => {
                    let lit = parse_attr_lit(attr)?;
                    drop(segment);

                    field.attrs.remove(i);
                    return Ok(FieldAttribute::TvParam(lit));
                }
                Some(x) if x.ident == "has_length" => match attr.parse_meta()? {
                    syn::Meta::NameValue(_) | syn::Meta::List(_) => {
                        return Err(syn::Error::new(attr.span(), "Unexpected value"));
                    }
                    syn::Meta::Word(_) => {
                        drop(segment);
                        field.attrs.remove(i);
                        return Ok(FieldAttribute::HasLength);
                    }
                },

                Some(_) | None => {}
            }
        }

        Ok(FieldAttribute::None)
    }
}

fn decode_field(data: &Ident, field: &mut Field) -> syn::Result<TokenStream> {
    // Check if this field has an attribute
    let attr = FieldAttribute::parse(field)?;

    // Generate code for decoding the field
    let ty = &field.ty;
    match attr {
        FieldAttribute::TvParam(tv_id) => {
            Ok(quote!(<#ty as crate::TvDecodable>::decode_tv(#data, #tv_id)?))
        }

        FieldAttribute::HasLength => Ok(quote!({
            if #data.len() < 2 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
            }

            let len = u16::from_be_bytes([#data[0], #data[1]]) as usize;
            let mut output = <#ty>::with_capacity(len);
            let mut rest = &#data[2..];

            for _ in 0..len {
                let result = crate::LLRPDecodable::decode(rest)?;
                output.push(result.0);
                rest = result.1;
            }

            (output, rest)
        })),

        FieldAttribute::None => Ok(quote!(<#ty as crate::LLRPDecodable>::decode(#data)?)),
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
    let decode_fields = decode_struct_fields(&struct_name, &mut input.fields).unwrap();

    let expanded = quote! {
        #input

        impl crate::LLRPMessage for #struct_name {
            const ID: u16 = #id;

            fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
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
    let decode_fields = decode_struct_fields(&struct_name, &mut input.fields).unwrap();

    let expanded = quote! {
        #input

        impl crate::TlvDecodable for #struct_name {
            const ID: u16 = #id;

            fn decode_tlv(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                eprintln!("\nparsing: {}", stringify!(#struct_name));

                // Parse the tlv header
                let (param_data, param_len) = crate::parse_tlv_header(data, #id)?;

                // Decode the inner fields for this parameter
                let result: crate::Result<(Self, &[u8])> = {
                    let data = param_data;
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

                Ok((inner, &data[param_len..]))
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

#[proc_macro_derive(LLRPEnum)]
pub fn derive_llrp_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let possible_matches = match input.data {
        Data::Enum(ref inner) => inner.variants.iter().map(|v| {
            let ty = match &v.fields {
                Fields::Unnamed(ref fields) => {
                    if fields.unnamed.len() != 1 {
                        panic!("Invalid varient");
                    }
                    fields.unnamed.iter().next().unwrap()
                }
                _ => panic!("Invalid varient"),
            };

            let name = &v.ident;
            quote!({
                __type if __type == #ty::ID => {
                    let (result, rest) = <#ty as crate::LLRPDecodable>::decode(data)?;
                    Ok((#name(result), rest))
                }
            })
        }),
        _ => panic!("Only supported for enums"),
    };

    let expanded = quote! {
        #input

        impl crate::LLRPDecodable for #name {
            fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                if data.len() < 2 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
                }

                // [6-bit resv, 10-bit message type]
                let __type = u16::from_be_bytes([data[0], data[1]]) & 0b11_1111_1111;
                match value {
                     #(#possible_matches,)*
                     _ => return Err(crate::Error::InvalidType(__type))
                };

            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
