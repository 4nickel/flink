use crate::util::error::{Error as E, Res};

use multipart::server::save::SaveResult::*;
use multipart::server::save::{Entries, SavedData};
use multipart::server::Multipart;
use rocket::{
    data::{self, FromDataSimple},
    http::ContentType,
    Data, Outcome, Request,
};
use std::path::{Path, PathBuf};

pub const SIZE_LIMIT: u64 = 5368709120;

pub struct MultipartForm {
    pub entries: Entries,
    pub partial: Option<String>,
    pub failure: Option<String>,
}

#[derive(Debug, Fail)]
pub enum MultipartError {
    #[fail(display = "request error: {}", message)]
    RequestError { message: String },
    #[fail(display = "key error: {}", key)]
    KeyError { key: String },
    #[fail(display = "value error: {} -> {}", key, val)]
    ValueError { key: String, val: String },
}

impl MultipartForm {
    pub fn read_boundary<'a>(request: &'a Request) -> Res<&'a str> {
        let content_type = match request.guard::<&ContentType>() {
            Outcome::Success(value) => value,
            _ => {
                let message = "content-type not set";
                println!("[multipart] error: {}", message);
                return Err(MultipartError::RequestError {
                    message: message.into(),
                }
                .into());
            }
        };
        match content_type
            .params()
            .find(|&(k, _)| k == "boundary")
            .ok_or_else(|| String::from("boundary not set"))
        {
            Ok((_, boundary)) => Ok(boundary),
            Err(message) => {
                println!("[multipart] error: {}", message);
                Err(MultipartError::RequestError { message }.into())
            }
        }
    }

    pub fn from_bounded_data(data: Data, boundary: &str, path: &Path) -> Res<Self> {
        match Multipart::with_body(data.open(), boundary)
            .save()
            .size_limit(SIZE_LIMIT)
            .with_dir(path)
        {
            Full(entries) => {
                println!("[multipart] read full form");
                let form = Self {
                    entries: entries,
                    partial: None,
                    failure: None,
                };
                Ok(form)
            }
            Partial(partial, reason) => {
                println!("[multipart] read partial form");
                let partial_name = match partial.partial {
                    Some(field) => {
                        let name = format!("{:?}", field.source.headers);
                        println!("[multipart] name: {}", name);
                        Some(name)
                    }
                    _ => None,
                };
                let reason = format!("{:?}", reason);
                println!("[multipart] reason: {}", reason);
                let form = Self {
                    entries: partial.entries,
                    partial: partial_name,
                    failure: Some(reason),
                };
                Ok(form)
            }
            Error(e) => {
                println!("[multipart] error: {:?}", e);
                return Err(e.into());
            }
        }
    }

    pub fn from_request(request: &Request, data: Data, path: &Path) -> Res<Self> {
        Ok(Self::from_bounded_data(
            data,
            Self::read_boundary(request)?,
            path,
        )?)
    }

    pub fn get_opt<'a>(&'a self, key: &str) -> Option<&'a SavedData> {
        if let Some(field) = self.entries.fields.get(key) {
            if let Some(value) = field.get(0) {
                return Some(&value.data);
            }
        }
        return None;
    }

    pub fn get<'a>(&'a self, key: &str) -> Result<&'a SavedData, MultipartError> {
        if let Some(field) = self.entries.fields.get(key) {
            if let Some(value) = field.get(0) {
                return Ok(&value.data);
            }
        }
        return Err(MultipartError::KeyError { key: key.into() });
    }

    pub fn get_text<'a>(&'a self, key: &str) -> Result<String, MultipartError> {
        match self.get(key)? {
            SavedData::Text(val) => Ok(val.clone()),
            _ => Err(MultipartError::ValueError {
                key: key.into(),
                val: String::from("Text"),
            }),
        }
    }
    pub fn get_file<'a>(&'a self, key: &str) -> Result<(PathBuf, usize), MultipartError> {
        match self.get(key)? {
            SavedData::File(val, len) => Ok((val.clone(), *len as usize)),
            _ => Err(MultipartError::ValueError {
                key: key.into(),
                val: String::from("File"),
            }),
        }
    }
}

impl FromDataSimple for MultipartForm {
    type Error = E;
    fn from_data(request: &Request, data: Data) -> data::Outcome<Self, Self::Error> {
        use crate::util::error::failure;
        match MultipartForm::from_request(request, data, Path::new("/tmp/multipart")) {
            Ok(form) => data::Outcome::Success(form),
            Err(error) => failure(error),
        }
    }
}
