use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use clap::{Parser, ValueEnum};

use logmix::merger::merge;
use logmix::output::{write_record, Format};
use logmix::parser::sniff_and_parse;
use logmix::record::Record;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Passthrough,
    Jsonl,
    Tagged,
}

impl From<OutputFormat> for Format {
    fn from(f: OutputFormat) -> Self {
        match f {
            OutputFormat::Passthrough => Format::Passthrough,
            OutputFormat::Jsonl => Format::Jsonl,
            OutputFormat::Tagged => Format::Tagged,
        }
    }
}

#[derive(Parser)]
#[command(name = "logmix", about = "Merge structured log streams by timestamp")]
struct Cli {
    #[arg(long, value_enum, default_value_t = OutputFormat::Passthrough)]
    format: OutputFormat,

    files: Vec<String>,
}

fn main() {
    if let Err(code) = run() {
        std::process::exit(code);
    }
}

fn run() -> Result<(), i32> {
    let cli = Cli::parse();
    if cli.files.is_empty() {
        eprintln!("logmix: no input files specified");
        return Err(1);
    }

    let mut sources: Vec<Box<dyn Iterator<Item = Record>>> = Vec::new();
    for path in &cli.files {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("logmix: cannot open {}: {e}", path);
                return Err(1);
            }
        };
        let source = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();
        let lines = BufReader::new(file).lines();
        sources.push(Box::new(lines.filter_map(move |line| {
            let line = line.ok()?;
            if line.is_empty() {
                return None;
            }
            Some(sniff_and_parse(&line, &source))
        })));
    }

    let format = cli.format.into();
    let mut stdout = io::stdout().lock();
    for record in merge(sources) {
        write_record(&mut stdout, &record, format).map_err(|e| {
            eprintln!("logmix: write error: {e}");
            1
        })?;
    }
    stdout.flush().map_err(|_| 1)?;
    Ok(())
}
