use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::repr::{Container, Definition, EnumVariant, Field};

pub struct GeneratedCode {
    pub(crate) messages: Vec<TokenStream>,
    pub(crate) parameters: Vec<TokenStream>,
    pub(crate) enumerations: Vec<TokenStream>,
    pub(crate) choices: Vec<TokenStream>,
}

impl std::fmt::Display for GeneratedCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(include_str!("../base/common.rs"))?;

        let messages = &self.messages;
        let parameters = &self.parameters;
        let enumerations = &self.enumerations;
        let choices = &self.choices;
        let body = quote! {
            #[allow(bad_style, unused_imports, unused_mut)]
            pub mod messages {
                use super::{*, parameters::*, enumerations::*, choices::*};
                #(#messages)*
            }

            #[allow(bad_style, unused_imports, unused_mut)]
            pub mod parameters {
                use super::{*, enumerations::*, choices::*};
                #(#parameters)*
            }

            #[allow(bad_style, unused_imports, unused_mut)]
            pub mod enumerations {
                use super::{*, parameters::*, choices::*};
                #(#enumerations)*
            }

            #[allow(bad_style, unused_imports, unused_mut)]
            pub mod choices {
                use super::{*, parameters::*, enumerations::*};
                #(#choices)*
            }
        };
        write!(f, "{}", body)?;

        Ok(())
    }
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
                code.messages.push(define_message(id, ident, &fields, true));
            }
            Definition::Parameter { id, ident, fields } => {
                code.parameters.push(define_parameter(id, ident, &fields, true));
            }
            Definition::Enum { ident, variants } => {
                code.enumerations.push(define_enum(ident, &variants));
            }
            Definition::Choice { ident, choices } => {
                code.choices.push(define_choice(ident, &choices, true));
            }
        }
    }
    code
}

fn define_message(id: u16, ident: Ident, fields: &[Field], trace: bool) -> TokenStream {
    let data = Ident::new("__rest", Span::call_site());

    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);

    let decode_fields = fields.iter().map(|field| decode_field(field, &data, trace));

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

fn define_parameter(id: u16, ident: Ident, fields: &[Field], trace: bool) -> TokenStream {
    let data = Ident::new("__rest", Span::call_site());

    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);

    let decode_fields = fields.iter().map(|field| decode_field(field, &data, trace));

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

fn define_choice(ident: Ident, choices: &[Field], trace: bool) -> TokenStream {
    let ident = &ident;

    let variants: Vec<_> = choices
        .iter()
        .map(|choice| match &choice.ty {
            Container::Option(choice_ty) | Container::Raw(choice_ty) => choice_ty,
            _ => panic!("Invalid choice container type"),
        })
        .collect();

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub enum #ident {
            #(#variants(#variants),)*
        }

        impl crate::TlvDecodable for #ident {
            fn decode_tlv(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                let message_type = crate::get_tlv_message_type(data)?;
                let (value, rest) = match message_type {
                    #(
                        message_type if message_type == #variants::ID => {
                            let (value, rest) = crate::TlvDecodable::decode_tlv(data)?;
                            (#ident::#variants(value), rest)
                        },
                    )*
                    _ => return Err(crate::Error::InvalidType(message_type)),
                };
                Ok((value, rest))
            }

            fn check_type(ty: u16) -> bool {
                [#(#variants::ID,)*].contains(&ty)
            }
        }

        #(
            impl From<#variants> for #ident {
                fn from(value: #variants) -> #ident {
                    #ident::#variants(value)
                }
            }
        )*
    }
}

fn define_field(field: &Field) -> TokenStream {
    let ident = &field.ident;
    let ty = &field.ty;
    quote!(pub #ident: #ty)
}

fn decode_field(field: &Field, data: &Ident, trace: bool) -> TokenStream {
    use crate::repr::Encoding;

    let ident = &field.ident;
    let ty = &field.ty;

    let trace_stmt = gen_trace_stmt(&quote!(#ident), trace);

    match &field.encoding {
        Encoding::RawBits { num_bits } => {
            quote! {
                #trace_stmt
                let (bits, #data) = reader.read_bits(#data, #num_bits)?;
                let #ident = <#ty>::from_bits(bits);
            }
        }

        Encoding::TlvParameter => quote! {
            #trace_stmt
            let (#ident, #data) = crate::TlvDecodable::decode_tlv(#data)?;
        },

        Encoding::TvParameter { tv_id } => quote! {
            #trace_stmt
            let (#ident, #data) = crate::TvDecodable::decode_tv(#data, #tv_id)?;
        },

        Encoding::ArrayOfT { inner } => {
            let decode_inner = decode_field(&inner, data, trace);
            let inner_ident = &inner.ident;

            quote! {
                #trace_stmt
                let (len, #data) = u16::decode(#data)?;
                let mut #ident = <#ty>::with_capacity(len as usize);
                let mut tmp = #data;
                for _ in 0..len {
                    #decode_inner
                    #ident.push(#inner_ident);
                    tmp = #data;
                }
                let #data = tmp;
            }
        }

        Encoding::Enum { inner } => {
            let decode_inner = decode_field(&inner, data, trace);
            let inner_ident = &inner.ident;

            match &inner.encoding {
                Encoding::ArrayOfT { inner: array_element } => {
                    let element_ty = &array_element.ty;
                    quote! {
                        #decode_inner
                        let #ident = LLRPEnumeration::from_vec::<#element_ty>(#inner_ident)?;
                    }
                }
                _ => {
                    let inner_ty = &inner.ty;
                    quote! {
                        #decode_inner
                        let #ident = LLRPEnumeration::from_value::<#inner_ty>(#inner_ident)?;
                    }
                }
            }
        }

        Encoding::Manual => quote! {
            #trace_stmt
            let (#ident, #data) = crate::LLRPDecodable::decode(#data)?;
        },
    }
}

fn gen_trace_stmt(ident: &TokenStream, enabled: bool) -> Option<TokenStream> {
    match enabled {
        true => Some(quote!(eprintln!("Decoding: {}", stringify!(#ident));)),
        false => None,
    }
}
