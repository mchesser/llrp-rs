use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;

use crate::repr::{Container, Definition, EnumVariant, Field};

pub struct GeneratedCode {
    pub(crate) messages: Vec<TokenStream>,
    pub(crate) message_enum: TokenStream,
    pub(crate) parameters: Vec<TokenStream>,
    pub(crate) enumerations: Vec<TokenStream>,
    pub(crate) choices: Vec<TokenStream>,
}

impl std::fmt::Display for GeneratedCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(include_str!("../base/common.rs"))?;

        let messages = &self.messages;
        let message_enum = &self.message_enum;
        let parameters = &self.parameters;
        let enumerations = &self.enumerations;
        let choices = &self.choices;

        let body = quote! {
            #[allow(bad_style, unused_imports, unused_mut, unreachable, dead_code, unused_variables)]
            pub mod messages {
                use super::{*, parameters::*, enumerations::*, choices::*};
                #(#messages)*

                #message_enum
            }

            #[allow(bad_style, unused_imports, unused_mut, unreachable, dead_code, unused_variables)]
            pub mod parameters {
                use super::{*, enumerations::*, choices::*};
                #(#parameters)*
            }

            #[allow(bad_style, unused_imports, unused_mut, unreachable, dead_code, unused_variables)]
            pub mod enumerations {
                use super::{*, parameters::*, choices::*};
                #(#enumerations)*
            }

            #[allow(bad_style, unused_imports, unused_mut, unreachable, dead_code, unused_variables)]
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
    let mut message_names = vec![];
    let mut message_matches = vec![];
    for d in &definitions {
        match d {
            Definition::Message { id, ident, .. } => {
                message_names.push(ident);
                message_matches.push(quote! {
                    #id => Ok(Self::#ident(#ident::decode(payload)?.0))
                });
            }
            _ => (),
        }
    }

    let message_enum = quote! {
        pub enum Message {
            #(#message_names(#message_names),)*
        }

        impl Message {
            pub fn decode(message_id: u32, payload: &[u8]) -> crate::Result<Message> {
                match message_id as u16 {
                    #(#message_matches,)*
                    _ => Err(crate::Error::UnknownMessageId(message_id))
                }
            }
        }
    };

    let mut messages = vec![];
    let mut parameters = vec![];
    let mut enumerations = vec![];
    let mut choices = vec![];

    for d in definitions {
        match d {
            Definition::Message { id, ident, fields } => {
                messages.push(define_message(id, ident, &fields));
            }
            Definition::Parameter { id, ident, fields } => {
                parameters.push(define_parameter(id, ident, &fields));
            }
            Definition::TvParameter { id, ident, fields } => {
                parameters.push(define_tv_parameter(id, ident, &fields));
            }
            Definition::Enum { ident, variants } => {
                enumerations.push(define_enum(ident, &variants));
            }
            Definition::Choice { ident, choices: entries } => {
                choices.push(define_choice(ident, &entries));
            }
        }
    }
    GeneratedCode { messages, message_enum, parameters, enumerations, choices }
}

fn define_message(id: u16, ident: Ident, fields: &[Field]) -> TokenStream {
    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);

    let decoder = Ident::new("decoder", Span::call_site());
    let decode_fields = fields.iter().map(|field| {
        let ident = &field.ident;
        let decode = decode_field(field, &decoder);
        quote!(let #ident = #decode?;)
    });

    let encoder = Ident::new("encoder", Span::call_site());
    let encode_fields = fields.iter().map(|field| {
        let ident = &field.ident;
        let encode = encode_field(field, &encoder);
        quote! {
            let #ident = &self.#ident;
            #encode;
        }
    });

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

            fn encode(&self, buffer: &mut Vec<u8>) {
                let mut #encoder = Encoder::new(buffer);
                #(#encode_fields)*
            }
        }
    }
}

fn define_parameter(id: u16, ident: Ident, fields: &[Field]) -> TokenStream {
    let field_defs = fields.iter().map(define_field);
    let field_names = fields.iter().map(|field| &field.ident);

    let decoder = Ident::new("decoder", Span::call_site());
    let decode_fields = fields.iter().map(|field| {
        let ident = &field.ident;
        let encode = decode_field(field, &decoder);
        quote!(let #ident = #encode?;)
    });

    let encoder = Ident::new("encoder", Span::call_site());
    let encode_fields = fields.iter().map(|field| {
        let ident = &field.ident;
        let encode = encode_field(field, &encoder);
        quote! {
            let #ident = &self.#ident;
            #encode;
        }
    });

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub struct #ident {
            #(#field_defs,)*
        }

        impl crate::TlvParameter for #ident {
            const ID: u16 = #id;
        }

        impl crate::LLRPValue for #ident {
            fn decode(decoder: &mut Decoder) -> crate::Result<Self> {
                decoder.tlv_param(#id, |decoder| {
                    #(#decode_fields)*

                    Ok(#ident {
                        #(#field_names,)*
                    })
                })
            }

            fn encode(&self, encoder: &mut Encoder) {
                encoder.tlv_param(#id, |encoder| {
                    #(#encode_fields)*
                });
            }

            fn can_decode_type(type_num: u16) -> bool {
                type_num == #id
            }
        }
    }
}

fn define_tv_parameter(id: u8, ident: Ident, fields: &[Field]) -> TokenStream {
    if let [Field { ty, .. }] = fields {
        // If there is only one field, then just use a typedef
        return quote!(pub type #ident = #ty;);
    }

    // Otherwise define a new struct
    let decoder = Ident::new("decoder", Span::call_site());
    let field_defs = fields.iter().map(define_field);
    let decode_fields = fields.iter().map(|field| {
        let ident = &field.ident;
        let decoded = decode_field(field, &decoder);
        quote!(#ident: #decoded?)
    });

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub struct #ident {
            #(#field_defs,)*
        }

        impl crate::LLRPValue for #ident {
            fn decode(decoder: &mut Decoder) -> crate::Result<Self> {
                Ok(#ident {
                    #(#decode_fields,)*
                })
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
    let mut decode_matches = vec![];
    let mut encode_matches = vec![];

    for entry in variants {
        let variant_ident = &entry.ident;
        let value = Literal::u16_unsuffixed(entry.value);

        variant_defs.push(quote!(#variant_ident = #value));
        decode_matches.push(quote!(#value => Self::#variant_ident));
        encode_matches.push(quote!(Self::#variant_ident => #value as u32));
    }

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub enum #ident {
            #(#variant_defs,)*
        }

        impl crate::LLRPEnumeration for #ident {
            fn from_value<T: Into<u32>>(value: T) -> crate::Result<Self> {
                let result = match value.into() {
                    #(#decode_matches,)*
                    other => return Err(crate::Error::InvalidVariant(other)),
                };

                Ok(result)
            }

            fn to_value<T: Bits>(&self) -> T {
                T::from_bits(match self {
                    #(#encode_matches,)*
                })
            }
        }
    }
}

fn define_choice(ident: Ident, choices: &[Field]) -> TokenStream {
    let ident = &ident;

    let mut tv_variants = vec![];
    let mut tv_ids = vec![];
    let mut decode_tv_params = vec![];
    let mut encode_tv_params = vec![];

    let mut tlv_variants = vec![];

    for choice in choices {
        let ty = match &choice.ty {
            Container::Option(choice_ty) | Container::Raw(choice_ty) => choice_ty,
            _ => panic!("Invalid choice container type"),
        };

        match choice.encoding {
            crate::repr::Encoding::TvParameter { tv_id } => {
                let tv_id_u16 = tv_id as u16;

                decode_tv_params.push(quote! {
                    #tv_id_u16 => Ok(Self::#ty(decoder.read_tv(#tv_id)?))
                });
                encode_tv_params.push(quote! {
                    Self::#ty(value) => value.encode_tv(encoder, #tv_id)
                });

                tv_variants.push(ty);
                tv_ids.push(tv_id_u16);
            }
            _ => tlv_variants.push(ty),
        }
    }

    quote! {
        #[derive(Debug, Eq, PartialEq)]
        pub enum #ident {
            #(#tlv_variants(#tlv_variants),)*
            #(#tv_variants(#tv_variants),)*
        }

        impl crate::LLRPValue for #ident {
            fn can_decode_type(type_num: u16) -> bool {
                [#(#tlv_variants::ID,)* #(#tv_ids,)*].contains(&type_num)
            }

            fn decode(decoder: &mut Decoder) -> Result<Self> {
                let type_num = decoder.peek_param_type()?.as_u16();
                match type_num {
                    #(#decode_tv_params,)*

                    #(
                        type_num if type_num == #tlv_variants::ID => {
                            Ok(Self::#tlv_variants(decoder.read()?))
                        },
                    )*
                    _ => Err(crate::Error::InvalidType(type_num)),
                }
            }

            fn encode(&self, encoder: &mut Encoder) {
                match self {
                    #(#encode_tv_params,)*
                    #(Self::#tlv_variants(value) => value.encode(encoder),)*
                }
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

fn decode_field(field: &Field, decoder: &Ident) -> TokenStream {
    use crate::repr::Encoding;

    let ty = &field.ty;
    match &field.encoding {
        Encoding::RawBits { num_bits } => quote!(#decoder.read_from_bits::<#ty>(#num_bits)),
        Encoding::TlvParameter => quote!(#decoder.read::<#ty>()),
        Encoding::TvParameter { tv_id } => quote!(#decoder.read_tv::<#ty>(#tv_id)),
        Encoding::ArrayOfT { inner } => {
            let decode_inner = decode_field(&inner, decoder);
            quote!(#decoder.array(|#decoder| #decode_inner))
        }
        Encoding::Enum { inner } => {
            let decode_inner = decode_field(&inner, decoder);

            match &inner.encoding {
                Encoding::ArrayOfT { inner: array_element } => {
                    let element_ty = &array_element.ty;
                    quote!(LLRPEnumeration::from_vec::<#element_ty>(#decode_inner?))
                }
                _ => {
                    let inner_ty = &inner.ty;
                    quote!(LLRPEnumeration::from_value::<#inner_ty>(#decode_inner?))
                }
            }
        }
        Encoding::Manual => quote!(#decoder.read::<#ty>()),
    }
}

fn encode_field(field: &Field, encoder: &Ident) -> TokenStream {
    use crate::repr::Encoding;

    let ty = &field.ty;
    let ident = &field.ident;

    match &field.encoding {
        Encoding::RawBits { num_bits } => quote!(#encoder.write_to_bits(#ident, #num_bits)),
        Encoding::TlvParameter => quote!(#encoder.write(#ident)),
        Encoding::TvParameter { tv_id } => quote!(#encoder.write_tv(#ident, #tv_id)),
        Encoding::ArrayOfT { inner } => {
            let encode_inner = encode_field(&inner, encoder);
            let inner_ident = &inner.ident;
            quote!(#encoder.array(#ident, |#encoder, #inner_ident| #encode_inner))
        }
        Encoding::Enum { inner } => {
            let encode_inner = encode_field(&inner, encoder);
            let inner_ident = &inner.ident;

            match &inner.encoding {
                Encoding::ArrayOfT { inner: array_element } => {
                    let element_ty = &array_element.ty;
                    quote!(unimplemented!())
                }
                _ => {
                    let inner_ty = &inner.ty;
                    quote!({
                        let tmp = #ident.to_value::<#inner_ty>();
                        let #inner_ident = &tmp;
                        #encode_inner
                    })
                }
            }
        }
        Encoding::Manual => quote!(#encoder.write(#ident)),
    }
}
