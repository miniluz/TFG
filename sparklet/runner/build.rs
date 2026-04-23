use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=Config.toml");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_config.rs");
    let mut writer = BufWriter::new(File::create(dest_path).unwrap());

    let toml_str = fs::read_to_string("Config.toml").expect("Config.toml not found");
    let config: toml::Value = toml::from_str(&toml_str).expect("Invalid TOML");

    let params = config.get("parameters").unwrap();
    let polyphony = params.get("polyphony").unwrap().as_integer().unwrap() as usize;

    let init = config.get("initial_config").unwrap();
    let get_u8 = |k| init.get(k).unwrap().as_integer().unwrap() as u8;

    let attack = get_u8("attack");
    let sustain = get_u8("sustain");
    let decay_release = get_u8("decay_release"); // Maps TOML decay_release -> release
    let osc_type = get_u8("oscilator_type");
    let f250 = get_u8("f250hz");
    let f500 = get_u8("f500hz");
    let f1000 = get_u8("f1000hz");
    let f2000 = get_u8("f2000hz");
    let f4000 = get_u8("f4000hz");
    let f8000 = get_u8("f8000hz");

    write!(
        writer,
        r#"
pub struct InitialConfig {{
    pub attack: u8,
    pub sustain: u8,
    pub decay_release: u8,
    pub oscilator_type: u8,
    pub f250hz: u8,
    pub f500hz: u8,
    pub f1000hz: u8,
    pub f2000hz: u8,
    pub f4000hz: u8,
    pub f8000hz: u8,
}}

pub struct Parameters {{
    pub polyphony: usize,
}}

pub struct BuildConfig {{
    pub initial_config: InitialConfig,
    pub parameters: Parameters,
}}

pub const BUILD_CONFIG: BuildConfig = BuildConfig {{
    initial_config: InitialConfig {{
        attack: {attack},
        sustain: {sustain},
        decay_release: {decay_release},
        oscilator_type: {osc_type},
        f250hz: {f250},
        f500hz: {f500},
        f1000hz: {f1000},
        f2000hz: {f2000},
        f4000hz: {f4000},
        f8000hz: {f8000},
    }},
    parameters: Parameters {{
        polyphony: {polyphony},
    }},
}};
"#
    )
    .unwrap();
}
