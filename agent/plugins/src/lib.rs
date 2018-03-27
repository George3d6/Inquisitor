extern crate agent_lib;
use agent_lib::AgentPlugin;

#[macro_use]
mod plugin_initialization;

plugins!(alive, command_runner, filechecker, process_counter, system_monitor);
