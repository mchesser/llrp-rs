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
    Enum { ident: Ident, variants: Vec<EnumVariant> },
    Choice,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Container {
    None,
    Box,
    Option,
    OptionBox,
    Vec,
}

#[derive(Debug, Clone)]
pub struct Field {
    /// The identifier (name) of this field
    pub ident: Ident,

    /// The type of the field
    pub ty: TokenStream,

    /// Represents how the field is encoded
    pub encoding: Encoding,
}

impl Field {
    pub fn is_vec(&self) -> bool {
        match self.encoding {
            Encoding::ArrayOfT { .. } => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TvField {
    pub id: u8,
    pub ty: TokenStream,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// The identifier for this variant
    pub ident: Ident,

    /// The value of the variant as a u16
    pub value: u16,
}

pub fn parse_definitions(def: llrp_def::LLRPDef) -> Vec<Definition> {
    // First identify any TV parameters
    let mut tv_params = HashMap::new();
    for definiton in &def.definitions {
        if let llrp_def::Definition::Parameter(param_def) = definiton {
            let mut fields = &param_def.fields[..];

            // Skip annotation field if it exists
            if let Some(llrp_def::Field::Annotation(_)) = fields.get(0) {
                fields = &fields[1..];
            }

            if fields.len() != 1 {
                continue;
            }
            if let llrp_def::Field::Field { name, type_, .. } = &fields[0] {
                if name == &param_def.name {
                    let (ty, _) = type_of(type_);
                    tv_params.insert(name.clone(), TvField { id: param_def.type_num as u8, ty });
                }
            }
        }
    }

    // Then parse all other definitions
    let mut definitions = vec![];
    for definiton in &def.definitions {
        definitions.push(match definiton {
            llrp_def::Definition::Message(def) => Definition::Message {
                id: def.type_num,
                ident: Ident::new(&def.name.to_camel_case(), Span::call_site()),
                fields: parse_fields(&def.fields, &tv_params),
            },

            llrp_def::Definition::Parameter(def) => Definition::Parameter {
                id: def.type_num,
                ident: Ident::new(&def.name, Span::call_site()),
                fields: parse_fields(&def.fields, &tv_params),
            },

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

            llrp_def::Definition::Choice(_) | llrp_def::Definition::Namespace(_) => continue,
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

            llrp_def::Field::Field { type_, name, format, enumeration } => {
                match enumeration.as_ref() {
                    Some(enumeration) => {
                        let ident = field_ident(name);
                        let enum_ident = Ident::new(enumeration, Span::call_site());
                        let inner = inner_field("__item", type_);

                        let ty = match inner.is_vec() {
                            true => quote!(Vec<#enum_ident>),
                            false => quote!(#enum_ident),
                        };

                        Field { ident, ty, encoding: Encoding::Enum { inner } }
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

fn get_container(type_name: &str, repeat: Repeat) -> Container {
    let recursive = match type_name {
        "ParameterError" => true,
        _ => false,
    };

    match (repeat, recursive) {
        (Repeat::One, true) => Container::Box,
        (Repeat::One, false) => Container::None,
        (Repeat::ZeroOrOne, true) => Container::OptionBox,
        (Repeat::ZeroOrOne, false) => Container::Option,
        (Repeat::ZeroToN, _) | (Repeat::OneToN, _) => Container::Vec,
    }
}

#[rustfmt::skip]
fn type_of(type_name: &str) -> (TokenStream, Encoding) {
    use Encoding::*;

    let (ty, encoding) = match type_name {
        "u1"    => ("bool",      RawBits { num_bits: 1 }),
        "u2"    => ("u8",        RawBits { num_bits: 2 }),
        "u3"    => ("u8",        RawBits { num_bits: 3 }),
        "u4"    => ("u8",        RawBits { num_bits: 4 }),
        "u5"    => ("u8",        RawBits { num_bits: 5 }),
        "u6"    => ("u8",        RawBits { num_bits: 6 }),
        "u7"    => ("u8",        RawBits { num_bits: 7 }),
        "u8"    => ("u8",        Manual),
        "u9"    => ("u16",       RawBits { num_bits: 9 }),
        "u10"   => ("u16",       RawBits { num_bits: 10 }),
        "u11"   => ("u16",       RawBits { num_bits: 11 }),
        "u12"   => ("u16",       RawBits { num_bits: 12 }),
        "u13"   => ("u16",       RawBits { num_bits: 13 }),
        "u14"   => ("u16",       RawBits { num_bits: 14 }),
        "u15"   => ("u16",       RawBits { num_bits: 15 }),
        "u16"   => ("u16",       Manual),
        "u32"   => ("u32",       Manual),
        "u64"   => ("u64",       Manual),
        "u96"   => ("[u8; 12]",  Manual),
        "s8"    => ("i8",        Manual),
        "s16"   => ("i16",       Manual),
        "s32"   => ("i32",       Manual),
        "s64"   => ("i64",       Manual),
        "u1v"   => ("BitArray",  Manual),
        "u8v"   => ("Vec<u8>",   ArrayOfT { inner: inner_field("__item", "u8") }),
        "u16v"  => ("Vec<u16>",  ArrayOfT { inner: inner_field("__item", "u16") }),
        "u32v"  => ("Vec<u32>",  ArrayOfT { inner: inner_field("__item", "u32") }),
        "u64v"  => ("Vec<u64>",  ArrayOfT { inner: inner_field("__item", "u64") }),
        "utf8v" => ("String",    Manual),
        "bytesToEnd" => ("Vec<u8>", ArrayOfT { inner: inner_field("__item", "u8") }),
        other => (other, TlvParameter),
    };
    (syn::parse_str(ty).unwrap(), encoding)
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

fn inner_field(name: &str, type_name: &str) -> Box<Field> {
    let (ty, encoding) = type_of(type_name);
    let ident = Ident::new(name, Span::call_site());
    Box::new(Field { ident, ty, encoding })
}

fn map_field(
    name: &str,
    type_name: &str,
    repeat: Repeat,
    tv_params: &HashMap<String, TvField>,
) -> Field {
    let ident = field_ident(name);

    let (base_type, mut encoding) = type_of(type_name);

    let ty = match get_container(type_name, repeat) {
        Container::None => quote!(#base_type),
        Container::Box => quote!(Box<#base_type>),

        Container::Option => match tv_params.get(name) {
            Some(tv_field) => {
                encoding = Encoding::TvParameter { tv_id: tv_field.id };
                let ty = &tv_field.ty;
                quote!(Option<#ty>)
            }
            None => quote!(Option<#base_type>),
        },
        Container::OptionBox => quote!(Option<Box<#base_type>>),

        Container::Vec => quote!(Vec<#base_type>),
    };

    Field { ident, ty, encoding }
}
