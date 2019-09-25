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
                code.messages.push(define_message(id, ident, &fields, false));
            }
            Definition::Parameter { id, ident, fields } => {
                code.parameters.push(define_parameter(id, ident, &fields, false));
            }
            Definition::TvParameter { id, ident, fields } => {
                code.parameters.push(define_tv_parameter(id, ident, &fields, false));
            }
            Definition::Enum { ident, variants } => {
                code.enumerations.push(define_enum(ident, &variants));
            }
            Definition::Choice { ident, choices } => {
                code.choices.push(define_choice(ident, &choices, false));
            }
        }
    }
    code
}

fn define_message(id: u16, ident: Ident, fields: &[Field], trace: bool) -> TokenStream {
    let decoder = Ident::new("decoder", Span::call_site());

    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);

    let decode_fields = fields.iter().map(|field| decode_field(field, &decoder, trace));

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub struct #ident {
            #(#field_defs,)*
        }

        impl crate::LLRPMessage for #ident {
            const ID: u16 = #id;

            fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                let mut #decoder = Decoder::new(data);

                #(#decode_fields)*

                let __result = #ident {
                    #(#field_names,)*
                };

                Ok((__result, #decoder.bytes))
            }
        }
    }
}

fn define_parameter(id: u16, ident: Ident, fields: &[Field], trace: bool) -> TokenStream {
    let decoder = Ident::new("decoder", Span::call_site());

    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);

    let decode_fields = fields.iter().map(|field| decode_field(field, &decoder, trace));

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub struct #ident {
            #(#field_defs,)*
        }

        impl crate::TlvDecodable for #ident {
            const ID: u16 = #id;
        }

        impl crate::LLRPDecodable for #ident {
            fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                let mut base = Decoder::new(data);
                let mut #decoder = base.tlv_param_decoder(#id)?;

                #(#decode_fields)*
                let __result = #ident {
                    #(#field_names,)*
                };

                #decoder.validate_consumed()?;

                Ok((__result, base.bytes))
            }

            fn can_decode_type(type_num: u16) -> bool {
                type_num == #id
            }
        }
    }
}

fn define_tv_parameter(id: u8, ident: Ident, fields: &[Field], trace: bool) -> TokenStream {
    if let [Field { ty, .. }] = fields {
        // If there is only one field, then just use a typedef
        return quote! {
            pub type #ident = #ty;
        };
    }

    // Otherwise define a new struct
    let decoder = Ident::new("decoder", Span::call_site());
    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);
    let decode_fields = fields.iter().map(|field| decode_field(field, &decoder, trace));

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub struct #ident {
            #(#field_defs,)*
        }

        impl crate::LLRPDecodable for #ident {
            fn decode(data: &[u8]) -> Result<(Self, &[u8])> {
                let mut #decoder = Decoder::new(data);

                #(#decode_fields)*

                let __result = #ident {
                    #(#field_names,)*
                };

                Ok((__result, #decoder.bytes))
            }

            fn can_decode_type(type_num: u16) -> bool {
                type_num == #id as u16
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
                    other => return Err(crate::Error::InvalidVariant(other)),
                };

                Ok(result)
            }
        }
    }
}

fn define_choice(ident: Ident, choices: &[Field], trace: bool) -> TokenStream {
    let ident = &ident;

    let mut tv_variants = vec![];
    let mut tv_ids = vec![];
    let mut decode_tv_params = vec![];

    let mut tlv_variants = vec![];

    for choice in choices {
        let ty = match &choice.ty {
            Container::Option(choice_ty) | Container::Raw(choice_ty) => choice_ty,
            _ => panic!("Invalid choice container type"),
        };
        match choice.encoding {
            crate::repr::Encoding::TvParameter { tv_id } => {
                let tv_id = tv_id as u16;
                tv_variants.push(ty.clone());
                tv_ids.push(tv_id);
                decode_tv_params.push(quote! {
                    #tv_id => {
                        let (value, rest) = crate::LLRPDecodable::decode_tv(data, #tv_id as u8)?;
                        (#ident::#ty(value), rest)
                    }
                });
            }
            _ => tlv_variants.push(ty.clone()),
        }
    }

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub enum #ident {
            #(#tlv_variants(#tlv_variants),)*
            #(#tv_variants(#tv_variants),)*
        }

        impl crate::TlvDecodable for #ident {
        }

        impl crate::LLRPDecodable for #ident {
            fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                let message_type = crate::get_message_type(data)?;
                let (value, rest) = match message_type {
                    #(#decode_tv_params,)*

                    #(
                        message_type if message_type == #tlv_variants::ID => {
                            let (value, rest) = crate::LLRPDecodable::decode(data)?;
                            (#ident::#tlv_variants(value), rest)
                        },
                    )*
                    _ => return Err(crate::Error::InvalidType(message_type)),
                };
                Ok((value, rest))
            }

            fn can_decode_type(type_num: u16) -> bool {
                [#(#tlv_variants::ID,)* #(#tv_ids,)*].contains(&type_num)
            }
        }

        #(
            impl From<#tlv_variants> for #ident {
                fn from(value: #tlv_variants) -> #ident {
                    #ident::#tlv_variants(value)
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

fn decode_field(field: &Field, decoder: &Ident, trace: bool) -> TokenStream {
    use crate::repr::Encoding;

    let ident = &field.ident;
    let ty = &field.ty;

    let trace_stmt = gen_trace_stmt(&quote!(#ident), trace);

    match &field.encoding {
        Encoding::RawBits { num_bits } => {
            quote! {
                #trace_stmt
                let #ident = <#ty>::from_bits(#decoder.read_bits(#num_bits)?);
            }
        }

        Encoding::TlvParameter => quote! {
            #trace_stmt
            let #ident = #decoder.read()?;
        },

        Encoding::TvParameter { tv_id } => quote! {
            #trace_stmt
            let #ident = #decoder.read_tv(#tv_id)?;
        },

        Encoding::ArrayOfT { inner } => {
            let decode_inner = decode_field(&inner, decoder, trace);
            let inner_ident = &inner.ident;

            quote! {
                #trace_stmt
                let len = #decoder.read::<u16>()?;
                let mut #ident = <#ty>::with_capacity(len as usize);
                for _ in 0..len {
                    #decode_inner
                    #ident.push(#inner_ident);
                }
            }
        }

        Encoding::Enum { inner } => {
            let decode_inner = decode_field(&inner, decoder, trace);
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
            let #ident = #decoder.read()?;
        },
    }
}

fn gen_trace_stmt(ident: &TokenStream, enabled: bool) -> Option<TokenStream> {
    match enabled {
        true => Some(quote!(eprintln!("Decoding: {}", stringify!(#ident));)),
        false => None,
    }
}
