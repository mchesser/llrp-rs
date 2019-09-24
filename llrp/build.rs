use std::fmt::Write as FmtWrite;
use std::io::Write;

fn main() {
    let definitions = llrp_codegen::load_definitions();
    let code = llrp_codegen::generate_code(definitions);

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_path = std::path::Path::new(&out_dir).join("llrp_generated.rs");

    let mut output = std::fs::File::create(&out_path).unwrap();
    write!(output, "{}", code).unwrap();
    output.flush().unwrap();
    drop(output);

    let fmt_out_path = std::path::Path::new(&out_dir).join("llrp_generated_fmt.rs");
    let mut fmt_output = std::fs::File::create(&fmt_out_path).unwrap();

    let config = rustfmt_nightly::Config::default();
    let result = rustfmt_nightly::Session::new(config, Some(&mut fmt_output))
        .format(rustfmt_nightly::Input::File(out_path));

    let mut tmp = std::fs::File::create("tmp.txt").unwrap();
    match result {
        Ok(report) => {
            writeln!(tmp, "ok: {}", report).unwrap();
        }
        Err(err) => {
            writeln!(tmp, "err: {}", err).unwrap();
        }
    }

}
