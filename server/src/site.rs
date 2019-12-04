use rocket::response::NamedFile;
use rocket_contrib::json::JsonValue;
use std::path::{Path, PathBuf};

const SITE_DIR: &'static str = "assets";

#[get("/", rank = 4)]
pub fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new(SITE_DIR).join("index.html")).ok()
}

#[get("/<file..>", rank = 4)]
pub fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(SITE_DIR).join(file)).ok()
}

#[catch(401)]
pub fn json_401() -> JsonValue {
    json!({"error": "Unauthorized" })
}
#[catch(404)]
pub fn json_404() -> JsonValue {
    json!({"error": "NotFound" })
}
#[catch(403)]
pub fn json_403() -> JsonValue {
    json!({"error": "Forbidden" })
}
#[catch(500)]
pub fn json_500() -> JsonValue {
    json!({"error": "InternalServerError" })
}
