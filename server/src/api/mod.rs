pub mod app;
pub mod authentication;
pub use app::*;

pub const API_PROT: &'static str = "https";
pub const API_HOST: &'static str = "flink.com";
pub const API_BASE: &'static str = "api";
pub const RES_USER: &'static str = "user";
pub const RES_FILE: &'static str = "file";

pub fn collection_url(res: &str) -> String {
    format!("{}://{}/{}/{}", API_PROT, API_HOST, API_BASE, res)
}

pub fn resource_url(res: &str, id: i32) -> String {
    format!("{}/{}", collection_url(res), id)
}
