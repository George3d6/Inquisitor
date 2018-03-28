pub mod utils;

pub trait AgentPlugin {
    fn name(&self) -> String;
    fn gather(&mut self) -> Result<String, String>;
    fn ready(&self) -> bool {
        self.when_ready() < 0
    }
    fn when_ready(&self) -> i64;
}
