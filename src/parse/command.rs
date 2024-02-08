use crate::parse::arg::Arg;
use crate::parse::flag::Flag;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub hide: bool,
    pub full_cmd: Vec<String>,
    pub usage: String,
    pub commands: IndexMap<String, Command>,
    pub args: Vec<Arg>,
    pub flags: Vec<Flag>,
}
