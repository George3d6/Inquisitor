pub mod utils;

pub trait AgentPlugin {
    fn name(&self) -> String;
    fn gather(&mut self) -> Result<String, String>;
    fn ready(&self) -> bool;
    fn when_ready(&self) -> i64;
}
