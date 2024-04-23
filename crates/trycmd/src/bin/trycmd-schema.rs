use std::io::prelude::*;

fn main() {
    let schema = schemars::schema_for!(trycmd::schema::OneShot);
    let schema = serde_json::to_string_pretty(&schema).unwrap();
    std::io::stdout().write_all(schema.as_bytes()).unwrap();
}
