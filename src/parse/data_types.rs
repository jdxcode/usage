use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Default, Copy, Clone, EnumString, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum SpecDataTypes {
    #[default]
    Null,
    String,
    Integer,
    Float,
    Boolean,
}
