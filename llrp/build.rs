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


    let config = rustfmt_nightly::Config::default();
    rustfmt_nightly::Session::new(config, Some(&mut std::io::sink()))
        .format(rustfmt_nightly::Input::File(out_path))
        .unwrap();
}
