mod codegen;
mod llrp_def;
mod repr;

use std::io::Write;

const LLRP_DEF: &[u8] = include_bytes!("../llrp-1x1-def.xml");
const LIB_CONTENT: &[u8] = include_bytes!("../base/lib.rs");
const COMMON_CONTENT: &[u8] = include_bytes!("../base/common.rs");

fn main() {
    let def = llrp_def::parse(LLRP_DEF).unwrap();
    let definitions = repr::parse_definitions(def);
    let code = codegen::generate(definitions);

    let out_dir = std::path::Path::new("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let file_writer = |name| {
        let file = std::fs::File::create(out_dir.join(name)).unwrap();
        std::io::BufWriter::new(file)
    };

    file_writer("lib.rs").write_all(LIB_CONTENT).unwrap();
    file_writer("common.rs").write_all(COMMON_CONTENT).unwrap();

    let mut messages_out = file_writer("messages.rs");
    writeln!(messages_out, "use crate::{{common::*, parameters::*, enumerations::*,choices::*}};")
        .unwrap();

    for message in code.messages {
        writeln!(messages_out, "{}", message).unwrap();
    }

    let mut params_out = file_writer("parameters.rs");
    writeln!(params_out, "use crate::{{common::*, enumerations::*, choices::*}};").unwrap();

    for param in code.parameters {
        writeln!(params_out, "{}", param).unwrap();
    }

    let mut enums_out = file_writer("enumerations.rs");
    writeln!(enums_out, "use crate::{{common::*}};").unwrap();

    for enumeration in code.enumerations {
        writeln!(enums_out, "{}", enumeration).unwrap();
    }

    let mut choices_out = file_writer("choices.rs");
    writeln!(choices_out, "use crate::{{parameters::*}};").unwrap();

    for choice in code.choices {
        writeln!(choices_out, "{}", choice).unwrap();
    }
}
