use quote::quote;

use proc_macro2::{Literal, Span, TokenStream};
use std::io::Write;
use syn::Ident;

const LLRP_DEF: &[u8] = include_bytes!("../llrp-1x1-def.xml");

fn message_name(name: &str) -> Ident {
    use heck::CamelCase;
    Ident::new(&name.to_camel_case(), Span::call_site())
}

fn type_name_str(name: &str) -> String {
    match name {
        "u1" => "bool".to_owned(),
        "u1v" => "BitArray".to_owned(),
        "u8" => "u8".to_owned(),
        "u8v" => "Vec<u8>".to_owned(),
        "u16" => "u16".to_owned(),
        "u16v" => "Vec<u16>".to_owned(),
        "u32" => "u32".to_owned(),
        "u32v" => "Vec<u32>".to_owned(),
        "u64" => "u64".to_owned(),
        "u64v" => "Vec<u64>".to_owned(),
        "s8" => "i8".to_owned(),
        "s16" => "i16".to_owned(),
        "s32" => "i32".to_owned(),
        "s64" => "i64".to_owned(),
        "utf8v" => "String".to_owned(),
        "bytesToEnd" => "Vec<u8>".to_owned(),
        other => other.to_owned(),
    }
}

fn type_name(name: &str) -> TokenStream {
    syn::parse_str(&type_name_str(&name)).unwrap()
}

fn type_with_repeat(name: &str, repeat: &Repeat) -> TokenStream {
    let base_type = type_name(name);
    match repeat {
        Repeat::One => quote!(#base_type),
        Repeat::ZeroOrOne => quote!(Option<#base_type>),
        Repeat::ZeroToN => quote!(Vec<#base_type>),
        Repeat::OneToN => quote!(Vec<#base_type>),
    }
}

fn field_name_str(name: &str) -> String {
    use heck::SnakeCase;

    match name {
        "Match" => "match_".to_owned(),
        "NumGPIs" => "num_gpis".to_owned(),
        "NumGPOs" => "num_gpos".to_owned(),
        "AntennaIDs" => "antenna_ids".to_owned(),
        "LLRPStatus" => "status".to_owned(),
        other => other.to_snake_case(),
    }
}

fn field_name(name: &str) -> Ident {
    Ident::new(&field_name_str(&name), Span::call_site())
}

fn enum_name(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

fn enum_variant_name(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename = "llrpdef")]
struct LLRPDef {
    #[serde(rename = "$value")]
    definitions: Vec<Definition>,
}

#[derive(Debug, serde::Deserialize)]
enum Definition {
    #[serde(rename = "parameterDefinition")]
    Parameter(ParameterDefinition),

    #[serde(rename = "enumerationDefinition")]
    Enum(EnumerationDefinition),

    #[serde(rename = "messageDefinition")]
    Message(MessageDefinition),

    #[serde(rename = "choiceDefinition")]
    Choice(ChoiceDefinition),

    #[serde(rename = "namespaceDefinition")]
    Namespace(serde::de::IgnoredAny),
}

#[derive(Debug, serde::Deserialize)]
struct MessageDefinition {
    name: String,

    #[serde(rename = "typeNum")]
    type_num: u16,

    required: bool,

    #[serde(rename = "$value")]
    items: Vec<Item>,
}

impl MessageDefinition {
    fn gen_code(&self) -> TokenStream {
        let name = message_name(&self.name);
        let type_num = self.type_num;

        let field_defs = gen_field_defs(&self.items);

        let data = Ident::new("__rest", Span::call_site());
        let decode_fields = valid_fields(&self.items).map(Item::field_decode);
        let field_names = valid_fields(&self.items).map(Item::field_name);

        quote! {
            #[derive(Debug, Eq, PartialEq)]
            pub struct #name {
                #(#field_defs,)*
            }

            impl crate::LLRPMessage for #name {
                const ID: u16 = #type_num;

                fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                    let #data = data;
                    #(#decode_fields)*
                    let __result = #name {
                        #(#field_names,)*
                    };

                    Ok((__result, #data))
                }
            }
        }
    }
}

impl std::fmt::Display for MessageDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.gen_code())
    }
}

#[derive(Debug, serde::Deserialize)]
struct ParameterDefinition {
    name: String,

    #[serde(rename = "typeNum")]
    type_num: u16,

    required: bool,

    #[serde(rename = "$value")]
    items: Vec<Item>,
}

impl ParameterDefinition {
    fn gen_code(&self) -> TokenStream {
        let name = type_name(&self.name);
        let type_num = self.type_num;
        let field_defs = gen_field_defs(&self.items);

        let data = Ident::new("__rest", Span::call_site());
        let decode_fields = valid_fields(&self.items).map(Item::field_decode);
        let field_names = valid_fields(&self.items).map(Item::field_name);

        quote! {
            #[derive(Debug, Eq, PartialEq)]
            pub struct #name {
                #(#field_defs,)*
            }

            impl crate::TlvDecodable for #name {
                const ID: u16 = #type_num;

                fn decode_tlv(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                    let (param_data, param_len) = crate::parse_tlv_header(data, #type_num)?;

                    let #data = param_data;
                    #(#decode_fields)*
                    let __result = #name {
                        #(#field_names,)*
                    };

                    // Ensure that all bytes were consumed when parsing the struct fields
                    if #data.len() != 0 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Did not use consume all bytes in the parameter when inner parameters",
                        )
                        .into());
                    }

                    Ok((__result, &data[param_len..]))
                }
            }
        }
    }
}

impl std::fmt::Display for ParameterDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.gen_code())
    }
}

#[derive(Debug, serde::Deserialize)]
struct EnumerationDefinition {
    name: String,

    #[serde(rename = "entry")]
    entries: Vec<EnumEntry>,
}

impl EnumerationDefinition {
    fn decode(&self) -> TokenStream {
        let name = enum_name(&self.name);

        let (type_, type_len): (Ident, usize) = {
            let (t_name, len) = match self.entries.iter().map(|x| x.value).max().unwrap() {
                x if x > 0xFF => ("u16", 2),
                _ => ("u8", 1),
            };
            (Ident::new(t_name, Span::call_site()), len)
        };

        let mut variants = vec![];
        let mut matches = vec![];

        for entry in &self.entries {
            let variant_ident = enum_variant_name(&entry.name);
            let value = Literal::u16_unsuffixed(entry.value);

            variants.push(quote!(#variant_ident = #value));
            matches.push(quote!(#value => #name::#variant_ident));
        }

        quote! {
            #[derive(Debug, Eq, PartialEq)]
            pub enum #name {
                #(#variants,)*
            }

            impl crate::LLRPDecodable for #name {
                fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                    if data.len() < #type_len {
                        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid length").into());
                    }

                    let value = match <#type_>::from_be_bytes(data[..#type_len].try_into().unwrap()) {
                        #(#matches,)*
                        other => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid variant: {}", other)).into())
                    };

                    Ok((value, &data[#type_len..]))
                }
            }
        }
    }
}

impl std::fmt::Display for EnumerationDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.decode())
    }
}

#[derive(Debug, serde::Deserialize)]
struct EnumEntry {
    value: u16,
    name: String,
}

#[derive(Debug, serde::Deserialize)]
struct ChoiceDefinition {
    name: String,

    #[serde(rename = "$value")]
    items: Vec<Item>,
}

impl ChoiceDefinition {
    fn gen_code(&self) -> TokenStream {
        let name = type_name(&self.name);
        quote! {
            #[derive(Debug , Eq , PartialEq)]
            pub enum #name {

            }

            impl crate::TlvDecodable for #name {
                fn decode_tlv(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                    unimplemented!()
                }
            }
        }
    }
}

impl std::fmt::Display for ChoiceDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.gen_code())
    }
}

#[derive(Debug, serde::Deserialize)]
enum Repeat {
    #[serde(rename = "1")]
    One,

    #[serde(rename = "0-1")]
    ZeroOrOne,

    #[serde(rename = "1-N")]
    OneToN,

    #[serde(rename = "0-N")]
    ZeroToN,
}

impl Default for Repeat {
    fn default() -> Self {
        Repeat::ZeroOrOne
    }
}

#[derive(Debug, serde::Deserialize)]
enum Item {
    #[serde(rename = "annotation")]
    Annotation(serde::de::IgnoredAny),

    #[serde(rename = "choice")]
    Choice {
        repeat: Repeat,

        #[serde(rename = "type")]
        type_: String,
    },

    #[serde(rename = "field")]
    Field {
        #[serde(rename = "type")]
        type_: String,
        name: String,
        format: Option<String>,
        enumeration: Option<String>,
    },

    #[serde(rename = "parameter")]
    Parameter {
        #[serde(default)]
        repeat: Repeat,

        #[serde(rename = "type")]
        type_: String,
    },

    #[serde(rename = "reserved")]
    Reserved {
        #[serde(rename = "bitCount")]
        bit_count: usize,
    },
}

impl Item {
    fn field_def(&self) -> TokenStream {
        match self {
            Item::Annotation(_) => panic!("Annotations should be stripped"),

            Item::Choice { repeat, type_ } | Item::Parameter { repeat, type_ } => {
                let name = field_name(&type_);
                let type_ = type_with_repeat(&type_, repeat);

                quote!(pub #name: #type_)
            }

            Item::Field { type_, name, format, enumeration } => {
                let name = field_name(&name);
                let type_ = type_name(&type_);

                if let Some(enumeration) = &enumeration {
                    let enum_ident = enum_name(&enumeration);
                    return quote!(pub #name: #enum_ident);
                }

                match format.as_ref() {
                    _=> quote!(pub #name: #type_),
                }
            }

            Item::Reserved { bit_count } => {
                let type_ = Ident::new(&format!("u{}", bit_count), Span::call_site());
                quote!(pub __reserved: #type_)
            }
        }
    }

    fn field_name(&self) -> Ident {
        match self {
            Item::Annotation(_) => panic!("Annotations should be stripped"),
            Item::Choice { type_, .. } | Item::Parameter { type_, .. } => field_name(&type_),
            Item::Field { name, .. } => field_name(&name),
            Item::Reserved { .. } => Ident::new("__reserved", Span::call_site()),
        }
    }

    fn field_decode(&self) -> TokenStream {
        let data = Ident::new("__rest", Span::call_site());

        match self {
            Item::Annotation(_) => panic!("Annotations should be stripped"),

            Item::Choice { type_, repeat } => {
                let name = field_name(&type_);
                let type_ = type_with_repeat(&type_, repeat);

                quote! {
                    let (#name, #data) = <#type_ as crate::LLRPDecodable>::decode(#data)?;
                }
            }

            Item::Parameter { type_, repeat } => {
                let name = field_name(&type_);
                let type_ = type_with_repeat(&type_, repeat);

                quote! {
                    let (#name, #data) = <#type_ as crate::LLRPDecodable>::decode(#data)?;
                }
            }

            Item::Field { type_, name, format, enumeration } => {
                let name = field_name(&name);
                let type_ = type_name(&type_);

                if let Some(enumeration) = &enumeration {
                    let enum_ident = enum_name(&enumeration);

                    return quote! {
                        let (#name, #data) = <#enum_ident as crate::LLRPDecodable>::decode(#data)?;
                    };
                }

                match format.as_ref().map(String::as_str) {
                    // Some("Hex") => {
                    //     quote! {
                    //         let (#name, #data) = {
                    //             if #data.len() < 2 {
                    //                 return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid length").into());
                    //             }

                    //             let len = u16::from_be_bytes([#data[0], #data[1]]) as usize;
                    //             let mut output = <#type_>::with_capacity(len);
                    //             let mut rest = &#data[2..];

                    //             for _ in 0..len {
                    //                 let result = crate::LLRPDecodable::decode(rest)?;
                    //                 output.push(result.0);
                    //                 rest = result.1;
                    //             }

                    //             (output, rest)
                    //         };
                    //     }
                    // }
                    _ => {
                        quote! {
                            let (#name, #data) = <#type_ as crate::LLRPDecodable>::decode(#data)?;
                        }
                    }
                }
            }

            Item::Reserved { bit_count } => {
                let type_ = Ident::new(&format!("u{}", bit_count), Span::call_site());
                quote! {
                    let (__reserved, #data) = <#type_ as crate::LLRPDecodable>::decode(#data)?;
                }
            }
        }
    }
}

fn valid_fields(items: &[Item]) -> impl Iterator<Item = &Item> {
    items.iter().filter(|i| match i {
        Item::Annotation(_) => false,
        _ => true,
    })
}

fn gen_field_defs(items: &[Item]) -> Vec<TokenStream> {
    valid_fields(items).map(Item::field_def).collect()
}

const LIB_CONTENT: &[u8] = include_bytes!("../base/lib.rs");
const COMMON_CONTENT: &[u8] = include_bytes!("../base/common.rs");

fn main() {
    let def: LLRPDef = serde_xml_rs::from_reader(LLRP_DEF).unwrap();

    let out_dir = std::path::Path::new("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let file_writer = |name| {
        let file = std::fs::File::create(out_dir.join(name)).unwrap();
        std::io::BufWriter::new(file)
    };

    file_writer("lib.rs").write_all(LIB_CONTENT).unwrap();
    file_writer("common.rs").write_all(COMMON_CONTENT).unwrap();

    let mut messages_out = file_writer("messages.rs");
    let mut params_out = file_writer("parameters.rs");
    let mut enums_out = file_writer("enumerations.rs");
    let mut choices_out = file_writer("choices.rs");

    writeln!(
        messages_out,
        "use std::io;\nuse crate::{{common::*, parameters::*, enumerations::*, choices::*}};"
    )
    .unwrap();
    writeln!(params_out, "use std::io;\nuse crate::{{common::*, enumerations::*, choices::*}};")
        .unwrap();
    writeln!(enums_out, "use std::{{io, convert::TryInto}};").unwrap();

    for item in def.definitions {
        let item: Definition = item;
        match item {
            Definition::Parameter(def) => writeln!(params_out, "{}", def).unwrap(),
            Definition::Enum(def) => writeln!(enums_out, "{}", def).unwrap(),
            Definition::Message(def) => writeln!(messages_out, "{}", def).unwrap(),
            Definition::Choice(def) => writeln!(choices_out, "{}", def).unwrap(),
            Definition::Namespace(_) => (),
        }
    }
}
