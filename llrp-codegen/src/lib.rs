mod codegen;
mod llrp_def;
mod repr;

pub use crate::{codegen::GeneratedCode, repr::Definition};

const LLRP_DEF: &[u8] = include_bytes!("../llrp-1x1-def.xml");

pub fn load_definitions() -> Vec<Definition> {
    let def = llrp_def::parse(LLRP_DEF).unwrap();
    repr::parse_definitions(def)
}

pub fn generate_code(definitions: Vec<Definition>) -> GeneratedCode {
    codegen::generate(definitions)
}
