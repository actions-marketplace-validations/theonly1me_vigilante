use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use vigilante::{Error, Format, Lock, ReviewRequest, ScanRequest, Source, scan_json, scan_text};

#[derive(Parser)]
#[command(
    name = "vigilante",
    version,
    about = "Shows the new powers that a pull request gives to code, CI, build, and AI agents"
)]
struct Cli {
    #[arg(long, global = true, default_value = ".", help = "The repository root")]
    repository: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Print every power in the repository")]
    Scan {
        #[arg(long, help = "Scan this commit instead of the working directory")]
        commit: Option<String>,
        #[arg(long, value_enum, default_value_t = ScanFormat::Text)]
        format: ScanFormat,
    },
    #[command(about = "Write vigilante.lock from the working directory")]
    Lock,
    #[command(about = "Report the powers that the head adds since its merge base with the base")]
    Diff {
        #[arg(long, help = "The branch or commit that the pull request merges into")]
        base: String,
        #[arg(long, default_value = "HEAD", help = "The pull request commit")]
        head: String,
        #[arg(long, value_enum, default_value_t = ReportFormat::Markdown)]
        format: ReportFormat,
    },
    #[command(about = "Fail when vigilante.lock does not match the repository")]
    Check {
        #[arg(long, help = "Check this commit instead of the working directory")]
        commit: Option<String>,
        #[arg(long, value_enum, default_value_t = ReportFormat::Markdown)]
        format: ReportFormat,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum ScanFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum ReportFormat {
    Markdown,
    Json,
    Sarif,
}

impl From<ReportFormat> for Format {
    fn from(format: ReportFormat) -> Self {
        match format {
            ReportFormat::Markdown => Self::Markdown,
            ReportFormat::Json => Self::Json,
            ReportFormat::Sarif => Self::Sarif,
        }
    }
}

enum Outcome {
    Pass,
    Fail,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(Outcome::Pass) => ExitCode::SUCCESS,
        Ok(Outcome::Fail) => ExitCode::from(1),
        Err(error) => {
            eprintln!("vigilante: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<Outcome, Error> {
    let repository = cli.repository;
    match cli.command {
        Command::Scan { commit, format } => {
            let report = vigilante::scan_report(ScanRequest {
                repository,
                source: source_of(commit),
            })?;
            match format {
                ScanFormat::Text => print!("{}", scan_text(&report)),
                ScanFormat::Json => print!("{}", scan_json(&report)),
            }
            Ok(Outcome::Pass)
        }
        Command::Lock => {
            let map = vigilante::scan(ScanRequest {
                repository: repository.clone(),
                source: Source::WorkingDirectory,
            })?;
            let lock = Lock::of(&map);
            let target = repository.join(Lock::FILE);
            fs::write(&target, lock.render()).map_err(|error| {
                Error::new(format!("cannot write {}: {error}", target.display()))
            })?;
            let power_count: usize = lock.powers.values().map(|powers| powers.len()).sum();
            println!(
                "Wrote {} with {} in {}.",
                Lock::FILE,
                counted(Counted {
                    count: power_count,
                    noun: "power"
                }),
                counted(Counted {
                    count: lock.powers.len(),
                    noun: "scope"
                })
            );
            Ok(Outcome::Pass)
        }
        Command::Diff { base, head, format } => report(Report {
            request: ReviewRequest {
                repository,
                base: Some(base),
                head: Source::Commit(head),
            },
            format,
        }),
        Command::Check { commit, format } => report(Report {
            request: ReviewRequest {
                repository,
                base: None,
                head: source_of(commit),
            },
            format,
        }),
    }
}

struct Report {
    request: ReviewRequest,
    format: ReportFormat,
}

fn report(report: Report) -> Result<Outcome, Error> {
    let review = vigilante::review(report.request)?;
    print!("{}", review.render(report.format.into()));
    Ok(if review.passed() {
        Outcome::Pass
    } else {
        Outcome::Fail
    })
}

fn source_of(commit: Option<String>) -> Source {
    commit.map_or(Source::WorkingDirectory, Source::Commit)
}

struct Counted {
    count: usize,
    noun: &'static str,
}

fn counted(counted: Counted) -> String {
    let suffix = if counted.count == 1 { "" } else { "s" };
    format!("{} {}{suffix}", counted.count, counted.noun)
}
