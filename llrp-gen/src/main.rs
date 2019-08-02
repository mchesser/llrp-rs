use quote::quote;

use proc_macro2::{Literal, Span, TokenStream};
use std::io::Write;
use syn::Ident;

const LLRP_DEF: &[u8] = include_bytes!("../llrp-1x1-def.xml");

fn message_name(name: &str) -> Ident {
    use heck::CamelCase;
    Ident::new(&name.to_camel_case(), Span::call_site())
}

fn type_name(type_name: &str) -> TokenStream {
    let mapped_name = match type_name {
        "u1" => "bool",
        "u8" => "u8",
        "u16" => "u16",
        "u32" => "u32",
        "u64" => "u64",
        "s8" => "i8",
        "s16" => "i16",
        "s32" => "i32",
        "s64" => "i64",
        "u1v" => "BitArray",
        "u8v" => "Vec<u8>",
        "u16v" => "Vec<u16>",
        "u32v" => "Vec<u32>",
        "u64v" => "Vec<u64>",
        "utf8v" => "String",
        "bytesToEnd" => "Vec<u8>",
        other => other,
    };
    syn::parse_str(mapped_name).unwrap()
}

fn field_encoding(type_name: &str) -> Encoding {
    match type_name {
        "u1" => Encoding::BitPacket { num_bits: 1 },
        "AccessSpecState" => Encoding::BitPacket { num_bits: 1 },
        "u2" => Encoding::BitPacket { num_bits: 2 },
        "u3" => Encoding::BitPacket { num_bits: 3 },
        "u4" => Encoding::BitPacket { num_bits: 4 },
        "u5" => Encoding::BitPacket { num_bits: 5 },
        "u6" => Encoding::BitPacket { num_bits: 6 },
        "u7" => Encoding::BitPacket { num_bits: 7 },
        "u8v" => Encoding::ArrayWithLength,
        "u16v" => Encoding::ArrayWithLength,
        "u32v" => Encoding::ArrayWithLength,
        "u64v" => Encoding::ArrayWithLength,
        "bytesToEnd" => Encoding::ArrayWithLength,
        _ => Encoding::TlvParameter,
    }
}

fn type_with_repeat(name: &str, repeat: &Repeat) -> TokenStream {
    let base_type = type_name(name);

    let is_recursive = match name {
        "ParameterError" => true,
        _ => false,
    };

    match (repeat, is_recursive) {
        (Repeat::One, false) => quote!(#base_type),
        (Repeat::One, true) => quote!(Box<#base_type>),
        (Repeat::ZeroOrOne, false) => quote!(Option<#base_type>),
        (Repeat::ZeroOrOne, true) => quote!(Option<Box<#base_type>>),
        (Repeat::ZeroToN, _) => quote!(Vec<#base_type>),
        (Repeat::OneToN, _) => quote!(Vec<#base_type>),
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

        let fields = parse_items(&self.items, true);

        let data = Ident::new("__rest", Span::call_site());

        let field_defs = fields.iter().map(|field| field.field_def());
        let field_names = fields.iter().map(|field| &field.ident);

        let mut remaining_bits = 0;
        let decode_fields =
            fields.iter().map(|field| field.field_decode(&data, &mut remaining_bits));

        let token_stream = quote! {
            #[derive(Debug, Eq, PartialEq)]
            pub struct #name {
                #(#field_defs,)*
            }

            impl crate::LLRPMessage for #name {
                const ID: u16 = #type_num;

                fn decode(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                    let #data = data;
                    let mut __spare_bits: u32 = 0;
                    #(#decode_fields)*
                    let __result = #name {
                        #(#field_names,)*
                    };

                    debug_assert_eq!(__spare_bits, 0, "Bits were remaining");

                    Ok((__result, #data))
                }
            }
        };

        assert_eq!(remaining_bits, 0, "{} spare bits were remaining for {}", remaining_bits, name);
        token_stream
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

        let fields = parse_items(&self.items, false);

        let data = Ident::new("__rest", Span::call_site());

        let field_defs = fields.iter().map(|field| field.field_def());
        let field_names = fields.iter().map(|field| &field.ident);

        let mut remaining_bits = 0;
        let decode_fields =
            fields.iter().map(|field| field.field_decode(&data, &mut remaining_bits));

        let token_stream = quote! {
            #[derive(Debug, Eq, PartialEq)]
            pub struct #name {
                #(#field_defs,)*
            }

            impl crate::TlvDecodable for #name {
                const ID: u16 = #type_num;

                fn decode_tlv(data: &[u8]) -> crate::Result<(Self, &[u8])> {
                    let (param_data, rest) = crate::parse_tlv_header(data, #type_num)?;

                    let #data = param_data;
                    let mut __spare_bits: u32 = 0;

                    #(#decode_fields)*
                    let __result = #name {
                        #(#field_names,)*
                    };

                    debug_assert_eq!(__spare_bits, 0, "Bits were remaining");
                    crate::validate_consumed(#data)?;

                    Ok((__result, rest))
                }
            }
        };

        assert_eq!(remaining_bits, 0, "{} spare bits were remaining for {}", remaining_bits, name);
        token_stream
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

        let max_variant = self.entries.iter().map(|x| x.value).max().unwrap();
        let required_bits = usize::next_power_of_two(max_variant as usize);

        let (type_, type_len): (Ident, usize) = {
            let (t_name, len) = match required_bits {
                x if x > 8 => ("u16", 2),
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

        let matches = &matches;

        quote! {
            #[derive(Debug, Eq, PartialEq)]
            pub enum #name {
                #(#variants,)*
            }

            impl crate::LLRPEnumeration for #name {
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

fn parse_items(items: &[Item], is_message: bool) -> Vec<MessageOrParameterField> {
    let mut output = vec![];

    for item in items {
        let field = match item {
            Item::Annotation(_) => continue,

            Item::Choice { repeat, type_ } => MessageOrParameterField {
                encoding: Encoding::TlvParameter,
                ident: field_name(&type_),
                source_type: type_.clone(),
                type_: type_with_repeat(&type_, repeat),
            },

            Item::Parameter { repeat, type_ } => MessageOrParameterField {
                encoding: Encoding::TlvParameter,
                ident: field_name(&type_),
                source_type: type_.clone(),
                type_: type_with_repeat(&type_, repeat),
            },

            Item::Field { type_, name, format, enumeration } => {
                let ident = field_name(&name);

                match enumeration.as_ref() {
                    Some(enumeration) => {
                        let enum_ident = enum_name(&enumeration);
                        MessageOrParameterField {
                            encoding: Encoding::Enum { source_type: type_.clone() },
                            ident,
                            source_type: enumeration.clone(),
                            type_: if type_.ends_with("v") {
                                quote!(Vec<#enum_ident>)
                            }
                            else {
                                quote!(#enum_ident)
                            },
                        }
                    }
                    None => MessageOrParameterField {
                        encoding: field_encoding(&type_),
                        ident,
                        source_type: type_.clone(),
                        type_: type_name(&type_),
                    },
                }
            }

            Item::Reserved { bit_count } => {
                let source_type = format!("u{}", bit_count);
                let type_ = Ident::new(&source_type, Span::call_site());

                MessageOrParameterField {
                    encoding: Encoding::BitPacket { num_bits: *bit_count as u8 },
                    ident: Ident::new("__reserved", Span::call_site()).into(),
                    source_type,
                    type_: quote!(#type_),
                }
            }
        };

        output.push(field);
    }

    output
}

#[derive(Clone)]
enum Encoding {
    TlvParameter,
    TvParameter,
    ArrayWithLength,
    BitPacket { num_bits: u8 },
    Enum { source_type: String },
}

struct MessageOrParameterField {
    encoding: Encoding,
    ident: Ident,
    source_type: String,
    type_: TokenStream,
}

impl MessageOrParameterField {
    fn field_def(&self) -> TokenStream {
        let ident = &self.ident;
        let ty = &self.type_;
        quote!(pub #ident: #ty)
    }

    fn field_decode(&self, data: &Ident, remaining_bits: &mut u8) -> TokenStream {
        let ident = &self.ident;
        let type_ = &self.type_;

        match &self.encoding {
            Encoding::TlvParameter => quote! {
                let (#ident, #data) = crate::LLRPDecodable::decode(#data)?;
            },

            Encoding::TvParameter => {
                let tv_id = 0_u8;
                quote! {
                    let (#ident, #data) = crate::TvDecodable::decode_tv(#data, #tv_id)?;
                }
            }

            Encoding::ArrayWithLength => quote! {
                let (#ident, #data) = {
                    let (len_bytes, mut rest) = split_at_checked(#data, 2)?;
                    let len = u16::from_be_bytes([len_bytes[0], len_bytes[1]]);

                    let mut output = <#type_>::with_capacity(len as usize);
                    for _ in 0..len {
                        let result = crate::LLRPDecodable::decode(rest)?;
                        output.push(result.0);
                        rest = result.1;
                    }

                    (output, rest)
                };
            },

            Encoding::Enum { source_type } => {
                let tmp_ident = Ident::new("__enum_tmp", Span::call_site());
                let source_ty = type_name(&source_type);
                let is_vec = source_type.ends_with("v");

                let source = MessageOrParameterField {
                    encoding: field_encoding(&source_type),
                    ident: tmp_ident.clone(),
                    source_type: source_type.clone(),
                    type_: source_ty.clone(),
                };

                let source_decode = source.field_decode(data, remaining_bits);

                if is_vec {
                    quote! {
                        #source_decode
                        let #ident = LLRPEnumeration::from_vec::<u8>(#tmp_ident)?;
                    }
                }
                else {
                    quote! {
                        #source_decode
                        let #ident = LLRPEnumeration::from_value::<#source_ty>(#tmp_ident)?;
                    }
                }
            }

            &Encoding::BitPacket { num_bits } => {
                let get_more_bits = match *remaining_bits {
                    x if x >= num_bits => {
                        *remaining_bits = *remaining_bits - num_bits;
                        quote! {}
                    }

                    x if x + 8 >= num_bits => {
                        *remaining_bits = (*remaining_bits + 8) - num_bits;
                        quote! {
                            let (split_byte, #data) = crate::split_at_checked(#data, 1)?;
                            __spare_bits = (__spare_bits << 8) | (split_byte[0] as u32);
                        }
                    }

                    _ => panic!("Too many bits in packed struct"),
                };

                quote! {
                    #get_more_bits
                    let #ident = <#type_>::from_bits(__spare_bits & ((1 << #num_bits) - 1));
                    __spare_bits = __spare_bits >> #num_bits;
                }
            }
        }
    }
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

    writeln!(messages_out, "use crate::{{common::*, parameters::*, enumerations::*, choices::*}};")
        .unwrap();
    writeln!(params_out, "use crate::{{common::*, enumerations::*, choices::*}};").unwrap();
    writeln!(enums_out, "use std::convert::TryInto; use crate::{{common::*}};").unwrap();

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
