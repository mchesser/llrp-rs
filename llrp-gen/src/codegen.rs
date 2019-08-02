use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::repr::{Definition, EnumVariant, Field};

pub struct GeneratedCode {
    pub messages: Vec<TokenStream>,
    pub parameters: Vec<TokenStream>,
    pub enumerations: Vec<TokenStream>,
    pub choices: Vec<TokenStream>,
}

pub fn generate(definitions: Vec<Definition>) -> GeneratedCode {
    let mut code = GeneratedCode {
        messages: vec![],
        parameters: vec![],
        enumerations: vec![],
        choices: vec![],
    };
    for d in definitions {
        match d {
            Definition::Message { id, ident, fields } => {
                code.messages.push(define_message(id, ident, &fields));
            }
            Definition::Parameter { id, ident, fields } => {
                code.parameters.push(define_parameter(id, ident, &fields));
            }
            Definition::Enum { ident, variants } => {
                code.enumerations.push(define_enum(ident, &variants));
            }
            Definition::Choice => {}
        }
    }
    code
}

fn define_message(id: u16, ident: Ident, fields: &[Field]) -> TokenStream {
    let data = Ident::new("__rest", Span::call_site());

    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);

    let decode_fields = fields.iter().map(|field| decode_field(field, &data));

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub struct #ident {
            #(#field_defs,)*
        }

        impl crate::LLRPMessage for #ident {
            const ID: u16 = #id;

            fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                let #data = data;
                let mut reader = BitContainer::default();

                #(#decode_fields)*

                let __result = #ident {
                    #(#field_names,)*
                };

                debug_assert_eq!(reader.valid_bits, 0, "Bits were remaining");

                Ok((__result, #data))
            }
        }
    }
}

fn define_parameter(id: u16, ident: Ident, fields: &[Field]) -> TokenStream {
    let data = Ident::new("__rest", Span::call_site());

    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);

    let decode_fields = fields.iter().map(|field| decode_field(field, &data));

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub struct #ident {
            #(#field_defs,)*
        }

        impl crate::TlvDecodable for #ident {
            const ID: u16 = #id;

            fn decode_tlv(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                let (param_data, rest) = crate::parse_tlv_header(data, #id)?;

                let #data = param_data;
                let mut reader = BitContainer::default();

                #(#decode_fields)*
                let __result = #ident {
                    #(#field_names,)*
                };

                debug_assert_eq!(reader.valid_bits, 0, "Bits were remaining");
                crate::validate_consumed(#data)?;

                Ok((__result, rest))
            }
        }
    }
}

fn define_enum(ident: Ident, variants: &[EnumVariant]) -> TokenStream {
    let ident = &ident;

    let mut variant_defs = vec![];
    let mut matches = vec![];

    for entry in variants {
        let variant_ident = &entry.ident;
        let value = Literal::u16_unsuffixed(entry.value);

        variant_defs.push(quote!(#variant_ident = #value));
        matches.push(quote!(#value => #ident::#variant_ident));
    }

    let matches = &matches;

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub enum #ident {
            #(#variant_defs,)*
        }

        impl crate::LLRPEnumeration for #ident {
            fn from_value<T: Into<u32>>(value: T) -> crate::Result<Self> {
                let result = match value.into() {
                    #(#matches,)*
                    other => return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Invalid variant: {}", other)
                        ).into()
                    )
                };

                Ok(result)
            }
        }
    }
}

fn define_field(field: &Field) -> TokenStream {
    let ident = &field.ident;
    let ty = &field.ty;
    quote!(pub #ident: #ty)
}

fn decode_field(field: &Field, data: &Ident) -> TokenStream {
    use crate::repr::Encoding;

    let ident = &field.ident;
    let ty = &field.ty;

    match &field.encoding {
        Encoding::RawBits { num_bits } => {
            quote! {
                let (bits, #data) = reader.read_bits(#data, #num_bits)?;
                let #ident = <#ty>::from_bits(bits);
            }
        }

        Encoding::TlvParameter => quote! {
            let (#ident, #data) = crate::TlvDecodable::decode_tlv(#data)?;
        },

        Encoding::TvParameter { tv_id } => quote! {
            let (#ident, #data) = crate::TvDecodable::decode_tv(#data, #tv_id)?;
        },

        Encoding::ArrayOfT { inner } => {
            let decode_inner = decode_field(&inner, data);
            quote! {
                let (len, #data) = u16::decode(#data)?;
                let mut #ident = <#ty>::with_capacity(len as usize);
                for _ in 0..len {
                    #decode_inner
                    #ident.push(__item);
                }
            }
        }

        Encoding::Enum { inner } => {
            let decode_inner = decode_field(&inner, data);
            let inner_ty = &inner.ty;
            if field.is_vec() {
                quote! {
                    #decode_inner
                    let #ident = LLRPEnumeration::from_vec::<#inner_ty>(__item)?;
                }
            }
            else {
                quote! {
                    #decode_inner
                    let #ident = LLRPEnumeration::from_value::<#inner_ty>(__item)?;
                }
            }
        }

        Encoding::Manual => quote! {
            let (#ident, #data) = crate::LLRPDecodable::decode(#data)?;
        },
    }
}
