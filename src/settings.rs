use crate::calculator::Angle;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: Theme,
    pub scientific: bool,
    pub history_open: bool,
    pub angle: Angle,
}
