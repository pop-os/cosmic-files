use serde::{Deserialize, Serialize};

use crate::config::{DesktopConfig, DesktopState};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DesktopPos {
    pub display: String,
    pub row: usize,
    pub col: usize,
    pub rows: usize,
    pub cols: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesktopLayout {
    pub primary_output_name: Option<String>,
    pub config: DesktopConfig,
    pub state: DesktopState,
}
