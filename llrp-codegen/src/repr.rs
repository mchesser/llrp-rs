//! Code for constructing an internal representation of the LLRP definition which is closer to
//! structure needed for code generation

use std::collections::HashMap;

use heck::CamelCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::llrp_def::{self, Repeat};

#[derive(Debug, Clone)]
pub enum Definition {
    Message { id: u16, ident: Ident, fields: Vec<Field> },
    Parameter { id: u16, ident: Ident, fields: Vec<Field> },
    TvParameter { id: u8, ident: Ident, fields: Vec<Field> },
    Enum { ident: Ident, variants: Vec<EnumVariant> },
    Choice { ident: Ident, choices: Vec<Field> },
}

#[derive(Debug, Clone)]
pub enum Encoding {
    /// Represents a fixed number of bits
    RawBits { num_bits: u8 },

    /// Represents a TLV encoded parameter
    TlvParameter,

    /// Represents a TV encoded parameter
    TvParameter { tv_id: u8 },

    /// Represents an array of values with a prefixed length
    ArrayOfT { inner: Box<Field> },

    /// Represents an enum value encoded as a number of bits
    Enum { inner: Box<Field> },

    /// Represents types that must be manually decoded
    Manual,
}

#[derive(Debug, Clone)]
pub enum Container {
    Raw(TokenStream),
    Box(TokenStream),
    Option(TokenStream),
    OptionBox(TokenStream),
    Vec(TokenStream),
}

impl quote::ToTokens for Container {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::TokenStreamExt;

        tokens.append_all(match self {
            Container::Raw(ty) => quote!(#ty),
            Container::Box(ty) => quote!(Box<#ty>),
            Container::Option(ty) => quote!(Option<#ty>),
            Container::OptionBox(ty) => quote!(Option<Box<#ty>>),
            Container::Vec(ty) => quote!(Vec<#ty>),
        });
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    /// The identifier (name) of this field
    pub ident: Ident,

    /// The type of the field
    pub ty: Container,

    /// Represents how the field is encoded
    pub encoding: Encoding,
}

#[derive(Debug, Clone)]
pub struct TvField {
    pub id: u8,
    pub ty: TokenStream,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// The identifier for this variant
    pub ident: Ident,

    /// The value of the variant as a u16
    pub value: u16,
}

pub fn parse_definitions(def: llrp_def::LLRPDef) -> Vec<Definition> {
    let mut definitions = vec![];

    // First define TV parameters (since these can change how regular parameters are defined)
    let mut tv_params = HashMap::new();
    for definiton in &def.definitions {
        if let llrp_def::Definition::Parameter(param_def) = definiton {
            // Check if this parameter lies within the TV parameter range
            if param_def.type_num > 127 {
                continue;
            }

            let name = &param_def.name;
            let type_num = param_def.type_num;
            let required = param_def.required;
            let mut fields = &param_def.fields[..];

            // Skip annotation field if it exists
            if let Some(llrp_def::Field::Annotation(_)) = fields.get(0) {
                fields = &fields[1..];
            }


            let ident = Ident::new(name, Span::call_site());

            let ty = quote!(#ident);
            tv_params.insert(name.clone(), TvField { id: type_num as u8, ty, required });

            let fields = parse_fields(fields, &HashMap::new());
            definitions.push(Definition::TvParameter { id: type_num as u8, ident, fields });
        }
    }

    // Then parse all other definitions
    for definiton in &def.definitions {
        definitions.push(match definiton {
            llrp_def::Definition::Message(def) => Definition::Message {
                id: def.type_num,
                ident: Ident::new(&def.name.to_camel_case(), Span::call_site()),
                fields: parse_fields(&def.fields, &tv_params),
            },

            llrp_def::Definition::Parameter(def) => {
                if def.type_num <= 127 {
                    // This is a TV parameter (handled above)
                    continue;
                }
                assert!(def.type_num < 2048, "type num should be less than 2048 ({:?})", def);

                Definition::Parameter {
                    id: def.type_num,
                    ident: Ident::new(&def.name, Span::call_site()),
                    fields: parse_fields(&def.fields, &tv_params),
                }
            }

            llrp_def::Definition::Enum(def) => Definition::Enum {
                ident: Ident::new(&def.name, Span::call_site()),
                variants: def
                    .entries
                    .iter()
                    .map(|x| EnumVariant {
                        ident: Ident::new(&x.name, Span::call_site()),
                        value: x.value,
                    })
                    .collect(),
            },

            llrp_def::Definition::Choice(def) => Definition::Choice {
                ident: Ident::new(&def.name, Span::call_site()),
                choices: parse_fields(&def.fields, &tv_params),
            },

            llrp_def::Definition::Namespace(_) => continue,
        })
    }

    definitions
}

fn parse_fields(fields: &[llrp_def::Field], tv_params: &HashMap<String, TvField>) -> Vec<Field> {
    let mut output = vec![];

    for field in fields {
        output.push(match field {
            llrp_def::Field::Annotation(_) => continue,

            llrp_def::Field::Choice { repeat, type_ }
            | llrp_def::Field::Parameter { repeat, type_ } => {
                map_field(type_, type_, *repeat, tv_params)
            }

            llrp_def::Field::Field { type_, name, format: _, enumeration } => {
                match enumeration.as_ref() {
                    Some(enumeration) => {
                        let enum_ident = Ident::new(enumeration, Span::call_site());
                        let inner = inner_field(type_);

                        let ty = match &inner.encoding {
                            Encoding::ArrayOfT { .. } => Container::Vec(quote!(#enum_ident)),
                            _ => Container::Raw(quote!(#enum_ident)),
                        };

                        Field { ident: field_ident(name), ty, encoding: Encoding::Enum { inner } }
                    }
                    None => map_field(name, type_, Repeat::One, &tv_params),
                }
            }

            llrp_def::Field::Reserved { bit_count } => {
                let type_name = format!("u{}", bit_count);
                map_field("__reserved", &type_name, Repeat::One, &tv_params)
            }
        });
    }

    output
}

#[rustfmt::skip]
fn type_of(type_name: &str) -> (TokenStream, Encoding) {
    use Encoding::*;

    let (mapped_name, encoding) = match type_name {
        // Values encoded with a fixed number of bits
        "u1"  => ("bool", RawBits { num_bits: 1 }),
        "u2"  => ("u8",   RawBits { num_bits: 2 }),
        "u3"  => ("u8",   RawBits { num_bits: 3 }),
        "u4"  => ("u8",   RawBits { num_bits: 4 }),
        "u5"  => ("u8",   RawBits { num_bits: 5 }),
        "u6"  => ("u8",   RawBits { num_bits: 6 }),
        "u7"  => ("u8",   RawBits { num_bits: 7 }),
        "u9"  => ("u16",  RawBits { num_bits: 9 }),
        "u10" => ("u16",  RawBits { num_bits: 10 }),
        "u11" => ("u16",  RawBits { num_bits: 11 }),
        "u12" => ("u16",  RawBits { num_bits: 12 }),
        "u13" => ("u16",  RawBits { num_bits: 13 }),
        "u14" => ("u16",  RawBits { num_bits: 14 }),
        "u15" => ("u16",  RawBits { num_bits: 15 }),

        // Values with manual implementations
        "u8"    => ("u8",       Manual),
        "u16"   => ("u16",      Manual),
        "u32"   => ("u32",      Manual),
        "u64"   => ("u64",      Manual),
        "u96"   => ("[u8; 12]", Manual),
        "s8"    => ("i8",       Manual),
        "s16"   => ("i16",      Manual),
        "s32"   => ("i32",      Manual),
        "s64"   => ("i64",      Manual),
        "u1v"   => ("BitArray", Manual),
        "utf8v" => ("String",   Manual),

        // Arrays of values
        "u8v" | "bytesToEnd" => ("Vec<u8>", ArrayOfT { inner: inner_field("u8") }),
        "u16v" => ("Vec<u16>", ArrayOfT { inner: inner_field("u16") }),
        "u32v" => ("Vec<u32>", ArrayOfT { inner: inner_field("u32") }),
        "u64v" => ("Vec<u64>", ArrayOfT { inner: inner_field("u64") }),

        // Tlv encoded parameters
        other => (other, TlvParameter),
    };

    (syn::parse_str(mapped_name).unwrap(), encoding)
}

#[rustfmt::skip]
fn field_ident(name: &str) -> Ident {
    use heck::SnakeCase;

    match name {
        "Match"      => Ident::new("match_", Span::call_site()),
        "NumGPIs"    => Ident::new("num_gpis", Span::call_site()),
        "NumGPOs"    => Ident::new("num_gpos", Span::call_site()),
        "AntennaIDs" => Ident::new("antenna_ids", Span::call_site()),
        "LLRPStatus" => Ident::new("status", Span::call_site()),
        other        => Ident::new(&other.to_snake_case(), Span::call_site()),
    }
}

fn inner_field(type_name: &str) -> Box<Field> {
    let (ty, encoding) = type_of(type_name);
    let ident = Ident::new(&format!("__{}_item", type_name), Span::call_site());
    Box::new(Field { ident, ty: Container::Raw(ty), encoding })
}

fn map_field(
    name: &str,
    type_name: &str,
    repeat: Repeat,
    tv_params: &HashMap<String, TvField>,
) -> Field {
    let ident = field_ident(name);

    let (base_type, encoding) = match tv_params.get(type_name) {
        Some(tv_field) => (tv_field.ty.clone(), Encoding::TvParameter { tv_id: tv_field.id }),
        None => type_of(type_name),
    };

    let is_recursive = match type_name {
        "ParameterError" => true,
        _ => false,
    };
    let ty = match (repeat, is_recursive) {
        (Repeat::One, false) => Container::Raw(base_type),
        (Repeat::One, true) => Container::Box(base_type),
        (Repeat::ZeroOrOne, false) => Container::Option(base_type),
        (Repeat::ZeroOrOne, true) => Container::OptionBox(base_type),
        (Repeat::ZeroToN, _) => Container::Vec(base_type),
        (Repeat::OneToN, _) => Container::Vec(base_type),
    };

    Field { ident, ty, encoding }
}
