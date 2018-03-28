#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub sender: String,
    pub ts: i64,
    pub message: String,
    pub plugin_name: String,
}
