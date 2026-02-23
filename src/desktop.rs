use crate::config::{DesktopConfig, DesktopState};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesktopLayout {
    pub primary_output_name: Option<String>,
    pub config: DesktopConfig,
    pub state: DesktopState,
}
