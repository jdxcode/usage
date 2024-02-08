use std::fmt::{Display, Formatter};
use std::iter::once;
use std::path::Path;

use kdl::{KdlDocument, KdlEntry, KdlNode};
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use xx::file;

use crate::error::UsageErr;
use crate::parse::cmd::SpecCommand_old;
use crate::parse::command::Command;
use crate::parse::config::SpecConfig;
use crate::parse::context::ParsingContext;
use crate::parse::helpers::NodeHelper;
use crate::{SpecArg_old, SpecFlag_old};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CLI {
    pub name: String,
    pub bin_name: String,
    pub version: Option<String>,
    pub author: Option<String>,

    pub config: SpecConfig,
    pub command: Command,
}

impl CLI {
    pub fn parse_file(file: &Path) -> Result<(CLI, String), UsageErr> {
        let (spec, body) = split_script(file)?;
        let spec = serde_json::from_str(&spec)?;
        Ok((spec, body))
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Spec_old {
    pub name: String,
    pub bin: String,
    pub cmd: SpecCommand_old,
    pub config: SpecConfig,
    pub version: Option<String>,
    pub usage: String,

    pub about: Option<String>,
    pub long_about: Option<String>,
}

impl Spec_old {
    pub fn parse_file(file: &Path) -> Result<(Spec_old, String), UsageErr> {
        let (spec, body) = split_script(file)?;
        let ctx = ParsingContext::new(file, &spec);
        let mut schema = Self::parse(&ctx, &spec)?;
        if schema.bin.is_empty() {
            schema.bin = file.file_name().unwrap().to_str().unwrap().to_string();
        }
        if schema.name.is_empty() {
            schema.name = schema.bin.clone();
        }
        Ok((schema, body))
    }
    pub fn parse_spec(input: &str) -> Result<Spec_old, UsageErr> {
        Self::parse(&Default::default(), input)
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
            && self.bin.is_empty()
            && self.usage.is_empty()
            && self.cmd.is_empty()
            && self.config.is_empty()
    }

    pub(crate) fn parse(ctx: &ParsingContext, input: &str) -> Result<Spec_old, UsageErr> {
        let kdl: KdlDocument = input
            .parse()
            .map_err(|err: kdl::KdlError| UsageErr::KdlError(err))?;
        let mut schema = Self {
            ..Default::default()
        };
        for node in kdl.nodes().iter().map(|n| NodeHelper::new(ctx, n)) {
            match node.name() {
                "name" => schema.name = node.arg(0)?.ensure_string()?,
                "bin" => schema.bin = node.arg(0)?.ensure_string()?,
                "version" => schema.version = Some(node.arg(0)?.ensure_string()?),
                "about" => schema.about = Some(node.arg(0)?.ensure_string()?),
                "long_about" => schema.long_about = Some(node.arg(0)?.ensure_string()?),
                "usage" => schema.usage = node.arg(0)?.ensure_string()?,
                "arg" => schema.cmd.args.push(SpecArg_old::parse(ctx, &node)?),
                "flag" => schema.cmd.flags.push(SpecFlag_old::parse(ctx, &node)?),
                "cmd" => {
                    let node: SpecCommand_old = SpecCommand_old::parse(ctx, &node)?;
                    schema.cmd.subcommands.insert(node.name.to_string(), node);
                }
                "config" => schema.config = SpecConfig::parse(ctx, &node)?,
                "include" => {
                    let file = node
                        .props()
                        .get("file")
                        .map(|v| v.ensure_string())
                        .transpose()?
                        .ok_or_else(|| ctx.build_err("missing file".into(), node.span()))?;
                    let file = Path::new(&file);
                    let file = match file.is_relative() {
                        true => ctx.file.parent().unwrap().join(file),
                        false => file.to_path_buf(),
                    };
                    info!("include: {}", file.display());
                    let (other, _) = Self::parse_file(&file)?;
                    schema.merge(other);
                }
                k => bail_parse!(ctx, node.span(), "unsupported spec key {k}"),
            }
        }
        set_subcommand_ancestors(&mut schema.cmd, &[]);
        Ok(schema)
    }

    fn merge(&mut self, other: Spec_old) {
        if !other.name.is_empty() {
            self.name = other.name;
        }
        if !other.bin.is_empty() {
            self.bin = other.bin;
        }
        if !other.usage.is_empty() {
            self.usage = other.usage;
        }
        if other.about.is_some() {
            self.about = other.about;
        }
        if other.long_about.is_some() {
            self.long_about = other.long_about;
        }
        if !other.config.is_empty() {
            self.config.merge(&other.config);
        }
        self.cmd.merge(other.cmd);
    }
}

fn split_script(file: &Path) -> Result<(String, String), UsageErr> {
    let full = file::read_to_string(file)?;
    let schema = full.strip_prefix("#!/usr/bin/env usage\n").unwrap_or(&full);
    let (schema, body) = schema.split_once("\n#!").unwrap_or((&schema, ""));
    let schema = schema.trim().to_string();
    let body = format!("#!{}", body);
    Ok((schema, body))
}

fn set_subcommand_ancestors(cmd: &mut SpecCommand_old, ancestors: &[String]) {
    if cmd.usage.is_empty() {
        cmd.usage = cmd.usage();
    }
    let ancestors = ancestors.to_vec();
    for subcmd in cmd.subcommands.values_mut() {
        subcmd.full_cmd = ancestors
            .clone()
            .into_iter()
            .chain(once(subcmd.name.clone()))
            .collect();
        set_subcommand_ancestors(subcmd, &subcmd.full_cmd.clone());
    }
}

impl Display for Spec_old {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut doc = KdlDocument::new();
        let nodes = &mut doc.nodes_mut();
        if !self.name.is_empty() {
            let mut node = KdlNode::new("name");
            node.push(KdlEntry::new(self.name.clone()));
            nodes.push(node);
        }
        if !self.bin.is_empty() {
            let mut node = KdlNode::new("bin");
            node.push(KdlEntry::new(self.bin.clone()));
            nodes.push(node);
        }
        if let Some(version) = &self.version {
            let mut node = KdlNode::new("version");
            node.push(KdlEntry::new(version.clone()));
            nodes.push(node);
        }
        if let Some(about) = &self.about {
            let mut node = KdlNode::new("about");
            node.push(KdlEntry::new(about.clone()));
            nodes.push(node);
        }
        if let Some(long_about) = &self.long_about {
            let mut node = KdlNode::new("long_about");
            node.push(KdlEntry::new(long_about.clone()));
            nodes.push(node);
        }
        if !self.usage.is_empty() {
            let mut node = KdlNode::new("usage");
            node.push(KdlEntry::new(self.usage.clone()));
            nodes.push(node);
        }
        for flag in self.cmd.flags.iter() {
            nodes.push(flag.into());
        }
        for arg in self.cmd.args.iter() {
            nodes.push(arg.into());
        }
        for cmd in self.cmd.subcommands.values() {
            nodes.push(cmd.into())
        }
        if !self.config.is_empty() {
            nodes.push((&self.config).into());
        }
        write!(f, "{}", doc)
    }
}

#[cfg(feature = "clap")]
impl From<&clap::Command> for Spec_old {
    fn from(cmd: &clap::Command) -> Self {
        Spec_old {
            name: cmd.get_name().to_string(),
            bin: cmd.get_bin_name().unwrap_or(cmd.get_name()).to_string(),
            cmd: cmd.into(),
            version: cmd.get_version().map(|v| v.to_string()),
            about: cmd.get_about().map(|a| a.to_string()),
            long_about: cmd.get_long_about().map(|a| a.to_string()),
            usage: cmd.clone().render_usage().to_string(),
            ..Default::default()
        }
    }
}

#[cfg(feature = "clap")]
impl From<&Spec_old> for clap::Command {
    fn from(schema: &Spec_old) -> Self {
        let mut cmd = clap::Command::new(&schema.name);
        for flag in schema.cmd.flags.iter() {
            cmd = cmd.arg(flag);
        }
        for arg in schema.cmd.args.iter() {
            let a = clap::Arg::new(&arg.name).required(arg.required);
            cmd = cmd.arg(a);
        }
        for scmd in schema.cmd.subcommands.values() {
            cmd = cmd.subcommand(scmd);
        }
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let spec = Spec_old::parse(
            &Default::default(),
            r#"
name "Usage CLI"
bin "usage"
arg "arg1"
flag "-f,--force" global=true
cmd "config" {
  cmd "set" {
    arg "key" help="Key to set"
    arg "value"
  }
}
        "#,
        )
        .unwrap();
        assert_display_snapshot!(spec, @r###"
        name "Usage CLI"
        bin "usage"
        flag "-f,--force" global=true
        arg "<arg1>"
        cmd "config" {
            cmd "set" {
                arg "<key>" help="Key to set"
                arg "<value>"
            }
        }
        "###);
    }

    #[test]
    #[cfg(feature = "clap")]
    fn test_clap() {
        let cmd = clap::Command::new("test");
        assert_display_snapshot!(Spec_old::from(&cmd), @r###"
        name "test"
        bin "test"
        usage "Usage: test"
        "###);
    }
}
