use std::path::{Path, PathBuf};

pub const HTTP_PORT: u16 = 8080;
pub const WS_PORT: u16 = 8081;
pub const WS_TICKRATE: u32 = 30;
pub const PHYSICS_TICKRATE: u32 = 60;

pub fn get_public_directory() -> PathBuf {
    Path::new("./public/").canonicalize().unwrap()
}
