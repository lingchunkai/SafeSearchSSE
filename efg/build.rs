extern crate capnpc;
extern crate regex;
use std::env;
use std::fs::File;
use regex::Regex;
use std::io::{Write, Read};

fn make_schema(name: std::string::String) {
    assert!(name.len() > 0, "schema name must not be empty");

    let out_dir = env::var("OUT_DIR").unwrap();

    ::capnpc::CompilerCommand::new().file(format!("./src/schema/{}.capnp", name)).run().expect("compiling schema");

    // Since we are placing this in schema instead of root, we will need to replace all instances 
    // of ::game_capnp with ::schema::game_capnp.
    let regex = Regex::new(format!("::{}_capnp", name).as_str()).unwrap();
    let schema_file_name = out_dir.clone() + format!("/src/schema/{}_capnp.rs", name).as_str();

    let new_contents = {
        // Read schema file and get existing contents.
        let mut schema_file = File::open(&schema_file_name).expect("something went wrong opening the schema file");
        let schema_contents = {
            let mut contents = String::new();
            schema_file.read_to_string(&mut contents).expect("something went wrong reading the schema file");
            contents
        };

        // Perform replacement using regex matching. 
        // Note that since we are using edition=2018 in the .toml file, 
        // but compiled with a new version of capnpc, we will need to 
        // replace by specify crate::schema::game_capnp rather than ::schema::game_capnp.
        // 
        // See [https://users.rust-lang.org/t/error-e0433-failed-to-resolve-could-not-find-outermost-in-root/23220/2]
        // for more information
        let out = regex.replace_all(&schema_contents, format!("crate::schema::{}_capnp", name).as_str());
        out.to_string()
    };

    // (Over)-Write new contents to file.
    write!(File::create(&schema_file_name).unwrap(), "{}", new_contents).unwrap();
}

/// Runs capnp and places it into OUT_DIR, while replacing 
/// all instances of ::game_capnp in the generated code 
/// to ::schema::game_capnp.
/// Adapted from Gabriele Farina: gfarina@cs.cmu.edu
fn main() {
    make_schema(String::from("game"));
    make_schema(String::from("vector"));
}