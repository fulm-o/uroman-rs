//! Command-line interface for uroman-rs.

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use clap::Parser;
use std::fs;
use std::io::{self, BufRead, BufReader, BufWriter, IsTerminal, Write};
use std::path::PathBuf;
use thiserror::Error;
use clap::ValueEnum;
use uroman::{RomFormat, RomanizationError, Uroman};

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
enum CliRomFormat {
    #[default]
    Str,
    Edges,
    Alts,
    Lattice,
}

impl From<CliRomFormat> for RomFormat {
    fn from(cli_format: CliRomFormat) -> Self {
        match cli_format {
            CliRomFormat::Str => RomFormat::Str,
            CliRomFormat::Edges => RomFormat::Edges,
            CliRomFormat::Alts => RomFormat::ALTS,
            CliRomFormat::Lattice => RomFormat::Lattice,
        }
    }
}

#[derive(Error, Debug)]
enum UromanError {
    #[error("Failed to open input file '{path}': {source}")]
    InputFileOpen { path: PathBuf, source: io::Error },

    #[error("Failed to create output file '{path}': {source}")]
    OutputFileCreate { path: PathBuf, source: io::Error },

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("REPL error: {0}")]
    Repl(#[from] ReadlineError),

    #[error("Romanization failed: {0}")]
    Romanization(#[from] RomanizationError),
}

#[derive(Parser, Debug)]
#[command(
    author = "fulm-o",
    version,
    about,
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
    #[arg(short = 'f', long, value_enum, default_value_t = CliRomFormat::default())]
    rom_format: CliRomFormat,

    /// Limit uroman to the first n lines of a file.
    #[arg(long)]
    max_lines: Option<usize>,

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

    if cli.direct_input.is_empty() && cli.input_filename.is_none()
        && std::io::stdin().is_terminal() {
            run_repl(&uroman, &cli)?;
            return Ok(());
        }

    let mut writer = get_writer(&cli.output_filename)?;

    if !cli.direct_input.is_empty() {
        process_direct_input(&uroman, &cli, &mut writer)?;
    }

    if cli.input_filename.is_some() || cli.direct_input.is_empty() {
        process_stream(&uroman, &cli, &mut writer)?;
    }

    writer.flush()?;

    Ok(())
}

fn process_direct_input(
    uroman: &Uroman,
    cli: &Cli,
    writer: &mut dyn Write,
) -> Result<(), UromanError> {
    for s in &cli.direct_input {
        let result = uroman.romanize_string(
            s,
            cli.lcode.as_deref(),
            Some(&cli.rom_format.into())
        )?;
        writeln!(writer, "{}", result.to_output_string()?)?;
    }
    Ok(())
}

fn process_stream(
    uroman: &Uroman,
    cli: &Cli,
    writer: &mut dyn Write,
) -> Result<(), UromanError> {
    let reader = get_reader(&cli.input_filename)?;

    uroman.romanize_file(
        reader,
        writer,
        cli.lcode.as_deref(),
        &cli.rom_format.into(),
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


fn run_repl(uroman: &Uroman, cli: &Cli) -> Result<(), UromanError> {
    let mut rl = DefaultEditor::new()?;

    let history_path = || -> Option<std::path::PathBuf> {
        let mut path = dirs::cache_dir()?;
        path.push("uroman-rs");
        std::fs::create_dir_all(&path).ok()?;
        path.push("history.txt");
        Some(path)
    };

    if let Some(path) = history_path()
        && rl.load_history(&path).is_err() {
        }

    let lcode = cli.lcode.as_deref();
    let rom_format: RomFormat = cli.rom_format.into();

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                rl.add_history_entry(&line)?;

                if line.trim() == ":exit" || line.trim() == ":quit" {
                    break;
                }

                if line.trim().is_empty() {
                    continue;
                }

                match uroman.romanize_string(&line, lcode, Some(&rom_format)) {
                    Ok(result) => match result.to_output_string() {
                        Ok(output) => println!("{}", output),
                        Err(e) => eprintln!("Error formatting output: {}", e),
                    },
                    Err(e) => eprintln!("Romanization error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted. To exit, press Ctrl-D or type :exit.");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting.");
                break;
            }
            Err(err) => {
                eprintln!("REPL Error: {}", err);
                break;
            }
        }
    }

    if let Some(path) = history_path()
        && let Err(err) = rl.save_history(&path) {
            eprintln!("Warning: could not save history to {:?}: {}", path, err);
        }

    Ok(())
}