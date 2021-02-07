#[allow(clippy::all)]
pub mod game_capnp {
    include!(concat!(env!("OUT_DIR"), "/src/schema/game_capnp.rs"));
}

pub mod vector_capnp {
    include!(concat!(env!("OUT_DIR"), "/src/schema/vector_capnp.rs"));
}