use std::{env, ffi::OsString, fs};

use anyhow::{Result, bail};
use tes5edit_rust_json::{
    parse_file, read_json_input, to_json_pretty, validate_plugin, write_file, write_json_pack,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    if args.len() != 3 {
        bail!("usage: tes5edit-rust-json-cli [to-json|to-json-pack|from-json] INPUT OUTPUT");
    }
    match args[0].to_string_lossy().as_ref() {
        "to-json" => {
            let plugin = parse_file(&args[1])?;
            fs::write(&args[2], to_json_pretty(&plugin, false)?)?;
        }
        "to-json-pack" => {
            let plugin = parse_file(&args[1])?;
            write_json_pack(&plugin, &args[2])?;
        }
        "from-json" => {
            let plugin = read_json_input(&args[1])?;
            validate_plugin(&plugin)?;
            write_file(&plugin, &args[2])?;
        }
        command => bail!("unknown command {command:?}"),
    }
    Ok(())
}
