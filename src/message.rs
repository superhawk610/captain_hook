use ::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub action: Hook,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "__type")]
pub enum Hook {
    #[serde(rename = "spawn")]
    Spawn { arg: String },
    #[serde(rename = "connect")]
    Connect { id: String },
    #[serde(rename = "log")]
    Log { text: String },
    #[serde(rename = "done")]
    Done {},
}
