use std::string::String;


pub trait AgentPlugin {
    fn new() -> Self;
    fn name(&self) -> String;
    fn gather(&mut self) -> Result<String, String>;
    fn ready(&self) -> bool;
}
