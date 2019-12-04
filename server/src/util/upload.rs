use crate::model::User;
use crate::util::error::{failure, Error as ApiError};
use crate::util::multipart::MultipartForm;

use rocket::{
    data::{self, FromDataSimple},
    Data, Outcome, Request,
};
use std::path::PathBuf;

use crate::api::app::files::user_spool;

pub struct FileUpload {
    pub form: MultipartForm,
    pub name: String,
    pub meta: String,
    pub file: PathBuf,
    pub size: usize,
}

impl FromDataSimple for FileUpload {
    type Error = ApiError;

    fn from_data(request: &Request, data: Data) -> data::Outcome<Self, Self::Error> {
        use crate::util::error::ServerError;

        let user = match request.guard::<User>() {
            Outcome::Success(value) => value,
            _ => {
                return failure(ServerError::DataGuardError {
                    name: String::from("User"),
                })
            }
        };

        let form = match MultipartForm::from_request(request, data, &user_spool(user.id)) {
            Ok(success) => success,
            Err(error) => return failure(error),
        };

        let meta = match form.get_text("meta") {
            Ok(success) => success,
            Err(error) => return failure(error),
        };
        let name = match form.get_text("name") {
            Ok(success) => success,
            Err(error) => return failure(error),
        };
        let (file, size) = match form.get_file("file") {
            Ok(success) => success,
            Err(error) => return failure(error),
        };

        let upload = FileUpload {
            form: form,
            name: name,
            meta: meta,
            file: file,
            size: size,
        };
        data::Outcome::Success(upload)
    }
}
