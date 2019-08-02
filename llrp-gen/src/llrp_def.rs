//! Code for parsing LLRP definitions from an XML file

#[derive(Debug, serde::Deserialize)]
#[serde(rename = "llrpdef")]
pub struct LLRPDef {
    #[serde(rename = "$value")]
    pub definitions: Vec<Definition>,
}

#[derive(Debug, serde::Deserialize)]
pub enum Definition {
    #[serde(rename = "messageDefinition")]
    Message(MessageDefinition),

    #[serde(rename = "parameterDefinition")]
    Parameter(ParameterDefinition),

    #[serde(rename = "enumerationDefinition")]
    Enum(EnumerationDefinition),

    #[serde(rename = "choiceDefinition")]
    Choice(ChoiceDefinition),

    #[serde(rename = "namespaceDefinition")]
    Namespace(serde::de::IgnoredAny),
}

#[derive(Debug, serde::Deserialize)]
pub struct MessageDefinition {
    pub name: String,

    #[serde(rename = "typeNum")]
    pub type_num: u16,

    pub required: bool,

    #[serde(rename = "$value")]
    pub fields: Vec<Field>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ParameterDefinition {
    pub name: String,

    #[serde(rename = "typeNum")]
    pub type_num: u16,

    pub required: bool,

    #[serde(rename = "$value")]
    pub fields: Vec<Field>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EnumerationDefinition {
    pub name: String,

    #[serde(rename = "entry")]
    pub entries: Vec<EnumerationVariant>,
}

#[derive(Debug, serde::Deserialize)]
pub struct EnumerationVariant {
    pub value: u16,
    pub name: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChoiceDefinition {
    pub name: String,

    #[serde(rename = "$value")]
    pub fields: Vec<Field>,
}

#[derive(Copy, Clone, Debug, serde::Deserialize)]
pub enum Repeat {
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
pub enum Field {
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

pub fn parse(data: &[u8]) -> Result<LLRPDef, serde_xml_rs::Error> {
    serde_xml_rs::from_reader(data)
}
