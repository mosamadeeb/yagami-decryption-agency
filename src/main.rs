use std::{convert::TryInto, fs, io::Write, path::PathBuf};

use clap::{CommandFactory, Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use spinners::{Spinner, Spinners};

const CHARA_KEY: &[u8; 512] = include_bytes!("keys/chara_key.bin");
const CHARA2_KEY: &[u8; 512] = include_bytes!("keys/chara2_key.bin");

#[derive(Parser)]
#[clap(name = "yagami-decryption-agency")]
#[clap(author = "SutandoTsukai181")]
#[clap(version = "0.1.0")]
#[clap(about = "Decrypts/encrypts Judgment and Lost Judgment PC chara.par archives", long_about = None)]
struct Args {
    /// Path to input file.
    #[clap(value_parser)]
    input: PathBuf,

    /// Path to output file. Defaults to input with ".decrypted.par" as the extension.
    #[clap(value_parser)]
    output: Option<PathBuf>,

    /// Operation mode.
    #[clap(arg_enum, value_parser, default_value = "auto")]
    mode: Mode,

    /// Type of the encrypted PAR file.
    #[clap(arg_enum, value_parser, default_value = "auto")]
    par_type: ParType,

    /// Overwrite files without asking.
    #[clap(short, long, action)]
    overwrite: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Mode {
    /// Automatically select mode based on input file name.
    Auto,

    /// Decrypt file.
    Decrypt,

    /// Encrypt file.
    Encrypt,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum ParType {
    /// Automatically select PAR type based on its contents.
    Auto,

    /// chara.par.
    Chara,

    /// chara2.par (Lost Judgment only).
    Chara2,
}

fn xor(data: &mut Vec<u8>, key: &[u8]) {
    let mut key = key.iter().cycle();

    println!("Performing XOR...");
    let bar = ProgressBar::new(data.len() as u64)
        .with_style(ProgressStyle::with_template("{bar:50.cyan/blue} [{percent}%]").unwrap());

    data.iter_mut().enumerate().for_each(|(i, b)| {
        if i % (1024 * 1024) == 0 {
            bar.inc(1024 * 1024);
        }
        *b ^= key.next().unwrap();
    });

    bar.finish();
    println!();
    println!();
}

fn rotate(data: &mut Vec<u8>, left: bool) {
    let rotate_dir = if left {
        |value: u64, i: u32| value.rotate_left(i)
    } else {
        |value: u64, i: u32| value.rotate_right(i)
    };

    println!("Rotating bits...");
    let bar = ProgressBar::new(data.len() as u64)
        .with_style(ProgressStyle::with_template("{bar:50.cyan/blue} [{percent}%]").unwrap());

    for (i, c) in data.chunks_mut(8).enumerate() {
        if (i % (1024 * 1024 / 8)) == 0 {
            bar.inc(1024 * 1024);
        }

        let mut value = u64::from_le_bytes(c.try_into().unwrap());
        value = rotate_dir(value, (i % 64) as u32);
        c.copy_from_slice(&value.to_le_bytes());
    }

    bar.finish();
    println!();
    println!();
}

fn pad(data: &mut Vec<u8>) {
    let rem = data.len() % 8;
    if rem != 0 {
        data.write_all(&vec![0; 8 - rem]).unwrap();
    }
}

fn decrypt(mut data: Vec<u8>, key: &[u8]) -> Vec<u8> {
    println!("Decrypting...");
    println!();

    xor(&mut data, key);
    pad(&mut data);
    rotate(&mut data, true);
    data
}

fn encrypt(mut data: Vec<u8>, key: &[u8]) -> Vec<u8> {
    println!("Encrypting...");
    println!();

    pad(&mut data);
    rotate(&mut data, false);
    xor(&mut data, key);
    data
}

fn main() {
    let mut args = Args::parse();

    // Print header
    print!("{}", Args::command().render_version());
    println!("{}", Args::command().get_author().unwrap());
    println!();

    if let Mode::Auto = args.mode {
        let file_name = args
            .input
            .file_name()
            .expect("Invalid path")
            .to_str()
            .unwrap_or_default();

        if file_name.ends_with(".decrypted.par") {
            args.mode = Mode::Encrypt;
        } else if file_name.ends_with(".par") {
            args.mode = Mode::Decrypt;
        } else {
            println!("Unable to determine operation mode.");
            println!("Select a mode:");
            args.mode = match dialoguer::Select::new()
                .items(&["Encrypt", "Decrypt"])
                .clear(false)
                .interact()
                .expect("Operation mode needs to be selected")
            {
                0 => Mode::Encrypt,
                1 => Mode::Decrypt,
                _ => panic!("Unexpected selection."),
            };
        }
    }

    let mut sp = Spinner::new(Spinners::Line, "Reading file...".into());
    let par = fs::read(&args.input).expect("Could not read file");
    sp.stop_with_newline();

    let key = match args.par_type {
        ParType::Auto => {
            // let magic: Vec<&u8> = par.iter().take(4).collect();

            match &par[0..4] {
                b"\xAC\xC5\x8B\x99" => CHARA_KEY,
                b"\x01\x6E\x58\xE4" => CHARA2_KEY,
                _ => {
                    println!();
                    println!("Unable to determine PAR type.");
                    println!("Select a type:");
                    match dialoguer::Select::new()
                        .items(&["chara.par", "chara2.par"])
                        .clear(false)
                        .interact()
                        .expect("PAR type needs to be selected")
                    {
                        0 => CHARA_KEY,
                        1 => CHARA2_KEY,
                        _ => panic!("Unexpected selection."),
                    }
                }
            }
        }
        ParType::Chara => CHARA_KEY,
        ParType::Chara2 => CHARA2_KEY,
    };

    let output_extension: &str;
    let result = match args.mode {
        Mode::Decrypt => {
            output_extension = "decrypted.par";
            decrypt(par, key)
        }
        Mode::Encrypt => {
            output_extension = "par";
            encrypt(par, key)
        }
        _ => unreachable!(),
    };

    let mut output: PathBuf;
    if let Some(output_path) = args.output {
        output = output_path;
    } else {
        output = args.input.clone();

        if args.mode == Mode::Encrypt && output.extension().is_some() {
            output.set_file_name(
                output
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(".decrypted.par", ".par"),
            );
        }

        output.set_extension(output_extension);
    }

    println!("Writing file to {:?}", &output);

    if !args.overwrite
        && output.is_file()
        && !dialoguer::Confirm::new()
            .with_prompt("File already exists. Overwrite?")
            .interact()
            .unwrap_or(false)
    {
        println!("Aborting.");
        return;
    }

    println!();

    let mut sp = Spinner::new(Spinners::Line, "Writing file...".into());
    fs::write(&output, result).expect("Could not write file");
    sp.stop_with_newline();

    println!();
    println!("Finished.");

    press_btn_continue::wait("Press any key to continue...").unwrap();
}
