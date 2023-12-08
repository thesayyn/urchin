
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    helpers: Vec<Helper>,
    pipelines: Vec<Pipeline>
}

#[derive(Serialize, Deserialize, Debug)]
struct Helper {
    name: String,
    from: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Pipeline {
    name: String,
    expr: String,
}
