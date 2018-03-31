pub trait AgentPlugin {
    fn name(&self) -> String;
    fn gather(&mut self) -> Result<String, String>;
    fn ready(&self) -> bool;
}
