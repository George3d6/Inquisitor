use super::plugin_interface::AgentPlugin;

#[macro_use]
mod plugin_initialization;

plugins!(alive, file_checker);
