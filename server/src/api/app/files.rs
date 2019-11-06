use crate::db;
use crate::model::{User, File, FileInsert};
use crate::util::{error::{Error as ApiError, ApiResult}, random::random_ascii, upload::FileUpload, download::FileDownload};

use rocket::{http::Status, response::{status, NamedFile}};
use rocket_contrib::json::{JsonValue};
use std::path::{PathBuf, Path};
use diesel::prelude::*;
use chrono::prelude::*;

const VAR_DIR: &'static str = "var";
const STORE_DIR: &'static str = "store";
const SPOOL_DIR: &'static str = "spool";
const FILE_KEY_LEN: usize = 32;

pub fn user_store(user_id: i32) -> PathBuf
{ Path::new(VAR_DIR).join(STORE_DIR).join(user_id.to_string()) }

pub fn user_spool(user_id: i32) -> PathBuf
{ Path::new(VAR_DIR).join(SPOOL_DIR).join(user_id.to_string()) }

pub fn user_store_file(user_id: i32, key: &str) -> PathBuf
{ user_store(user_id).join(key) }

#[derive(Debug, Fail)]
pub enum FileError {
    #[fail(display = "permission denied: user[{}] -> file[{}]", user, file)]
    PermissionDenied {
        file: File,
        user: User,
    },
    #[fail(display = "invalid duration: {}", string)]
    InvalidDuration {
        string: String
    },
}

// {{{ Upload

#[post("/", data = "<data>", format = "multipart/form-data")]
pub fn upload_http(u: User, data: FileUpload, c: db::Connection) -> ApiResult<status::Created<JsonValue>>
{
    use std::fs;
    use crate::util::date::UtcDateTime;

    assert!(Path::new(VAR_DIR).is_dir());
    assert!(Path::new(VAR_DIR).join(STORE_DIR).is_dir());
    assert!(Path::new(VAR_DIR).join(SPOOL_DIR).is_dir());

    let key = random_ascii(FILE_KEY_LEN);

    if !user_store(u.id).is_dir() {
        if let Err(e) = fs::create_dir(user_store(u.id)) {
            println!("[file] failed to create user store directory");
            println!("[file] {:?}", e);
            return Err(e.into())
        }
    }
    if !user_spool(u.id).is_dir() {
        if let Err(e) = fs::create_dir(user_spool(u.id)) {
            println!("[file] failed to create user spool directory");
            println!("[file] {:?}", e);
            return Err(e.into())
        }
    }

    let duration = match data.meta.as_str() {
        "d" => chrono::Duration::days(1),
        "w" => chrono::Duration::weeks(1),
        "m" => chrono::Duration::weeks(4),
        "q" => chrono::Duration::weeks(4*3),
        "y" => chrono::Duration::weeks(4*12),
        s => return Err(FileError::InvalidDuration { string: s.into() }.into()),
    };

    if let Err(e) = fs::rename(data.file, user_store_file(u.id, &key)) {
        println!("[file] failed to move file from download location");
        println!("[file] {:?}", e);
        return Err(e.into())
    }

    let now = Utc::now();
    let end = now+duration;

    let file = File::create(&FileInsert {
        user_id: u.id,
        val: data.name.clone(),
        key: key,
        upload_date: UtcDateTime(now).into(),
        delete_date: UtcDateTime(end).into(),
        downloads: 0,
        bytes: data.size as i64,
    }, &c)?;

    Ok(status::Created(file.url(), Some(JsonValue(serde_json::to_value(&file)?))))
}

#[post("/", data = "<_data>", format = "multipart/form-data", rank = 3)]
pub fn upload_forbidden(_data: FileUpload) -> Status
{ Status::Forbidden }

// }}}
// {{{ Lookup

#[get("/<key>")]
pub fn lookup(key: String, c: db::Connection) -> ApiResult<FileDownload>
{
    c.transaction::<_, ApiError, _>(|| {
        let mut file = File::by_key(&key, &c)?;
        match NamedFile::open(user_store_file(file.user_id, &file.key)) {
            Ok(named_file) => {
                file.downloads += 1;
                file.update(&c)?;
                Ok(FileDownload(named_file, file.val))
            },
            Err(error) => {
                println!("[file] failed to open named file: {}", key);
                Err(error.into())
            }
        }
    })
}

// }}}
// {{{ Delete

#[delete("/<key>")]
pub fn delete(u: User, key: String, c: db::Connection) -> ApiResult<JsonValue>
{
    let file = File::by_key(&key, &c)?;

    if file.user_id != u.id {
        return Err(FileError::PermissionDenied {
            file: file,
            user: u,
        }.into())
    }

    if let Err(error) = std::fs::remove_file(user_store_file(file.user_id, &file.key)) {
        File::delete(file.id, &c)?;
        Err(error.into())
    } else {
        File::delete(file.id, &c)?;
        Ok(json!({"key": file.key}))
    }
}

// }}}
// {{{ Query

#[get("/")]
pub fn query(u: User, c: db::Connection) -> ApiResult<JsonValue>
{
    let files = File::table().filter(File::with_user(u.id)).get_results::<File>(&*c)?;
    Ok(JsonValue(serde_json::to_value(&files)?))
}

#[get("/", rank = 3)]
pub fn query_forbidden() -> Status
{ Status::Forbidden }

// }}}
