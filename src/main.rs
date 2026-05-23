use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use logmix::merger::merge;
use logmix::output::{write_record, Format};
use logmix::parser::sniff_and_parse;

#[derive(Parser)]
#[command(name = "logmix", version, about)]
struct Cli {
    files: Vec<PathBuf>,
    #[arg(long, value_enum, default_value = "passthrough")]
    format: OutputFormat,
    #[arg(long)]
    verbose: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Passthrough,
    Jsonl,
    Tagged,
}

impl From<OutputFormat> for Format {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Passthrough => Format::Passthrough,
            OutputFormat::Jsonl => Format::Jsonl,
            OutputFormat::Tagged => Format::Tagged,
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let format: Format = cli.format.into();
    let mut sources = Vec::new();

    for path in &cli.files {
        let source = path.to_string_lossy().into_owned();
        let file = File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
        let records: Vec<_> = BufReader::new(file)
            .lines()
            .map(|line| {
                let line = line?;
                Ok(sniff_and_parse(&line, &source))
            })
            .collect::<Result<_, io::Error>>()?;
        if cli.verbose {
            eprintln!("{}: {} records", path.display(), records.len());
        }
        sources.push(records.into_iter());
    }

    let mut stdout = io::stdout().lock();
    for record in merge(sources) {
        write_record(&mut stdout, &record, format)?;
    }
    stdout.flush()?;
    Ok(())
}
