extern crate agent_lib;
    use agent_lib::plugin_interface::AgentPlugin;

    #[macro_use]
    mod plugin_initialization;

    plugins!(alive, filechecker);