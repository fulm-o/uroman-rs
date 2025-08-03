//! Command-line interface for uroman-rs.
use clap::Parser;
use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use thiserror::Error;
use uroman::{RomFormat, RomanizationError, Uroman};

#[derive(Error, Debug)]
pub enum UromanError {
    #[error("Failed to open input file '{path}': {source}")]
    InputFileOpen { path: PathBuf, source: io::Error },

    #[error("Failed to create output file '{path}': {source}")]
    OutputFileCreate { path: PathBuf, source: io::Error },

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("Romanization failed: {0}")]
    Romanization(#[from] RomanizationError),
}

#[derive(Parser, Debug)]
#[command(
    author = "fulm-o",
    version,
    about = "A Rust port of uroman for high-speed romanization",
)]
struct Cli {
    /// Direct text input to be romanized.
    #[arg(value_name = "DIRECT_INPUT")]
    direct_input: Vec<String>,

    /// Input file path (default: stdin).
    #[arg(short, long, value_name = "FILE")]
    input_filename: Option<PathBuf>,

    /// Output file path (default: stdout).
    #[arg(short, long, value_name = "FILE")]
    output_filename: Option<PathBuf>,

    /// ISO 639-3 language code (e.g., 'eng').
    #[arg(short = 'l', long)]
    lcode: Option<String>,

    /// Output format of romanization. 'edges' provides offsets.
    #[arg(short = 'f', long, value_enum, default_value_t = RomFormat::default())]
    rom_format: RomFormat,

    /// Limit uroman to the first n lines of a file.
    #[arg(long)]
    max_lines: Option<usize>,

    /// Cache size for romanization (for speed).
    #[arg(short, long, default_value_t = 20000)] // DEFAULT_ROM_MAX_CACHE_SIZE
    cache_size: usize,

    /// Decodes Unicode escape notation, e.g., \\u03B4 to δ.
    #[arg(short = 'd', long, action = clap::ArgAction::Count)]
    decode_unicode: u8,

    /// Run and display a few samples.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    sample: bool,

    /// Suppress progress indicators.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    silent: bool,

    /// Verbose output.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    if let Err(err) = run() {
        if let UromanError::Io(e) = &err
            && e.kind() == io::ErrorKind::BrokenPipe
        {
            return;
        }

        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), UromanError> {
    let cli = Cli::parse();
    let uroman = Uroman::new();

    // if cli.sample {
    //     show_samples(&uroman);
    //     return Ok(());
    // }

    let stream_mode_active = cli.input_filename.is_some() || cli.direct_input.is_empty();

    if !cli.direct_input.is_empty() {
        let mut writer: Box<dyn Write> = if stream_mode_active {
            Box::new(io::stderr())
        } else {
            Box::new(io::stdout())
        };
        process_direct_input(&uroman, &cli, &mut writer)?;
    }

    if stream_mode_active {
        process_stream(&uroman, &cli)?;
    }

    io::stdout().flush()?;

    Ok(())
}

fn process_direct_input(
    uroman: &Uroman,
    cli: &Cli,
    writer: &mut dyn Write,
) -> Result<(), UromanError> {
    for s in &cli.direct_input {
        let result = uroman.romanize_string(s, cli.lcode.as_deref(), Some(&cli.rom_format))?;
        writeln!(writer, "{}", result.to_output_string()?)?;
    }
    Ok(())
}

fn process_stream(uroman: &Uroman, cli: &Cli) -> Result<(), UromanError> {
    let reader = get_reader(&cli.input_filename)?;
    let writer = get_writer(&cli.output_filename)?;
    uroman.romanize_file(
        reader,
        writer,
        cli.lcode.as_deref(),
        &cli.rom_format,
        cli.max_lines,
        cli.silent,
    )?;
    Ok(())
}

fn get_reader(path: &Option<PathBuf>) -> Result<Box<dyn BufRead>, UromanError> {
    match path {
        Some(p) => {
            let file = fs::File::open(p).map_err(|e| UromanError::InputFileOpen {
                path: p.clone(),
                source: e,
            })?;
            Ok(Box::new(BufReader::new(file)))
        }
        None => Ok(Box::new(BufReader::new(io::stdin()))),
    }
}

fn get_writer(path: &Option<PathBuf>) -> Result<Box<dyn Write>, UromanError> {
    match path {
        Some(p) => {
            let file = fs::File::create(p).map_err(|e| UromanError::OutputFileCreate {
                path: p.clone(),
                source: e,
            })?;
            Ok(Box::new(BufWriter::new(file)))
        }
        None => Ok(Box::new(BufWriter::new(io::stdout()))),
    }
}
