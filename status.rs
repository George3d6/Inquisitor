use std::vec::Vec;
use std::string::String;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Status {
    pub sender:         String,
    pub ts:             i64,
    pub message:        String,
    pub plugin_name:    String,
}
