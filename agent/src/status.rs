use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub sender: String,
    pub ts: i64,
    pub message: String,
    pub plugin_name: String,
}
