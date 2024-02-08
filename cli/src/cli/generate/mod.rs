use std::path::PathBuf;
use usage::error::UsageErr;

use usage::Spec_old;

mod completion;
mod markdown;

#[derive(clap::Args)]
#[clap(visible_alias = "g")]
pub struct Generate {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    Completion(completion::Completion),
    Markdown(markdown::Markdown),
}

impl Generate {
    pub fn run(&self) -> miette::Result<()> {
        match &self.command {
            Command::Completion(cmd) => cmd.run(),
            Command::Markdown(cmd) => cmd.run(),
        }
    }
}

pub fn file_or_spec(file: &Option<PathBuf>, spec: &Option<String>) -> Result<Spec_old, UsageErr> {
    if let Some(file) = file {
        let (spec, _) = Spec_old::parse_file(file)?;
        Ok(spec)
    } else {
        Spec_old::parse_spec(spec.as_ref().unwrap())
    }
}
