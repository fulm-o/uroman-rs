extern crate uroman;

use std::{
    env,
    fs::File,
    io::{self, BufReader},
    process,
};

use uroman::{RomFormat, Uroman};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run --example process_file -- <path/to/file.txt>");
        eprintln!("\nFor example, try:");
        eprintln!("  cargo run --example process_file -- examples/sample.txt");
        process::exit(1);
    }

    let filepath = &args[1];
    println!("Processing file: {filepath}\n");

    let uroman = Uroman::new();

    let file = match File::open(filepath) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error: Failed to open file '{filepath}': {e}");
            process::exit(1);
        }
    };

    let reader = BufReader::new(file);

    if let Err(e) = uroman.romanize_file(
        reader,
        io::stdout().lock(),
        None,
        RomFormat::Str,
        None,  // max_lines
        true,  // decode_unicode
        false, // silent
    ) {
        eprintln!("{e}")
    };
}
