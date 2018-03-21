// Yeah, fuck this shit, custom code inlining it is
/*
#![feature(concat_idents)]


macro_rules! initialize_plugins_internal {
    ($($plugin_name:expr), *) => {
        $(
            concat_idents!("let mut ", $plugin_name) = concat_idents!("plugins::", $plugin_name, "::Plugin::new()");
        )*;
   };
}


macro_rules! initialize_plugins {
    () => {
        initialize_plugins_internal!("command_runner", "alive", "file_checker", "process_counter", "system_monitor");
    };
}
*/
