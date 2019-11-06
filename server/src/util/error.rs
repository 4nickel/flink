use std;
use diesel;
use base64;

pub type ApiResult<T> = Result<T, Error>;

#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "invariant violated: {}", message)]
    SqlInvariantError {
        message: String,
    },
    #[fail(display = "data guard failed: {}", name)]
    DataGuardError {
        name: String,
    },
    #[fail(display = "input/output failure: {}", error)]
    IoError {
        error: std::io::Error,
    },
    #[fail(display = "unicode error: {}", error)]
    Utf8Error {
        error: std::str::Utf8Error,
    },
    #[fail(display = "base64 error: {}", error)]
    Base64Error {
        error: base64::DecodeError,
    },
    #[fail(display = "serialization error: {}", error)]
    SerializationError {
        error: serde_json::error::Error,
    },
    #[fail(display = "database error: {}", error)]
    DatabaseError {
        error: diesel::result::Error,
    },
}

#[derive(Debug, Fail)]
pub enum ClientError {
    #[fail(display = "session: {}", error)]
    SessionError {
        error: crate::model::user::session::SessionError,
    },
    #[fail(display = "registration: {}", error)]
    RegistrationError {
        error: crate::model::user::user::RegistrationError,
    },
    #[fail(display = "authentication: {}", error)]
    AuthenticationError {
        error: crate::model::user::user::AuthenticationError,
    },
    #[fail(display = "multipart: {}", error)]
    MultipartError {
        error: crate::util::multipart::MultipartError,
    },
    #[fail(display = "file: {}", error)]
    FileError {
        error: crate::api::app::files::FileError,
    }
}

#[derive(Debug)]
pub enum Error {
    ClientError(ClientError),
    ServerError(ServerError),
}

impl From<ServerError> for Error {
    fn from(error: ServerError) -> Self
    { Error::ServerError(error) }
}

impl From<ClientError> for Error {
    fn from(error: ClientError) -> Self
    { Error::ClientError(error) }
}

use crate::model::user::session::SessionError;
impl From<SessionError> for Error {
    fn from(error: SessionError) -> Self
    { Error::ClientError(error.into()) }
}
impl From<SessionError> for ClientError {
    fn from(error: SessionError) -> Self
    { ClientError::SessionError{ error } }
}

use crate::model::user::user::RegistrationError;
impl From<RegistrationError> for Error {
    fn from(error: RegistrationError) -> Self
    { Error::ClientError(error.into()) }
}
impl From<RegistrationError> for ClientError {
    fn from(error: RegistrationError) -> Self
    { ClientError::RegistrationError { error } }
}

use crate::model::user::user::AuthenticationError;
impl From<AuthenticationError> for Error {
    fn from(error: AuthenticationError) -> Self
    { Error::ClientError(error.into()) }
}
impl From<AuthenticationError> for ClientError {
    fn from(error: AuthenticationError) -> Self
    { ClientError::AuthenticationError { error } }
}

use crate::api::app::files::FileError;
impl From<FileError> for Error {
    fn from(error: FileError) -> Self
    { Error::ClientError(error.into()) }
}
impl From<FileError> for ClientError {
    fn from(error: FileError) -> Self
    { ClientError::FileError { error } }
}

use crate::util::multipart::MultipartError;
impl From<MultipartError> for Error {
    fn from(error: MultipartError) -> Self
    { Error::ClientError(error.into()) }
}
impl From<MultipartError> for ClientError {
    fn from(error: MultipartError) -> Self
    { ClientError::MultipartError { error } }
}

use diesel::result::{Error as DieselError};
impl From<DieselError> for Error {
    fn from(error: DieselError) -> Self
    { Error::ServerError(error.into()) }
}
impl From<DieselError> for ServerError {
    fn from(error: DieselError) -> Self
    { ServerError::DatabaseError { error } }
}

impl From<std::string::FromUtf8Error> for ServerError {
    fn from(error: std::string::FromUtf8Error) -> Self
    { ServerError::Utf8Error{ error: error.utf8_error() } }
}
impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self
    { Error::ServerError(error.into()) }
}

impl From<base64::DecodeError> for Error {
    fn from(error: base64::DecodeError) -> Self
    { Error::ServerError(error.into()) }
}
impl From<base64::DecodeError> for ServerError {
    fn from(error: base64::DecodeError) -> Self
    { ServerError::Base64Error { error } }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self
    { Error::ServerError(error.into()) }
}

impl From<serde_json::error::Error> for ServerError {
    fn from(error: serde_json::error::Error) -> Self
    { ServerError::SerializationError{ error } }
}

impl From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> Self
    { ServerError::IoError{ error } }
}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self
    { Error::ServerError ( error.into() ) }
}

trait ErrorInfo {
    fn info(&self) -> (i32, Status);
}

impl ErrorInfo for SessionError {
    #[allow(unused_variables)]
    fn info(&self) -> (i32, Status)
    {
        match self {
            SessionError::CookieNotFound { }          => { (110, Status::NotFound) },
            SessionError::RecordNotFound { token }    => { (111, Status::NotFound) }
        }
    }
}

impl ErrorInfo for RegistrationError {
    #[allow(unused_variables)]
    fn info(&self) -> (i32, Status)
    {
        match self {
            RegistrationError::DuplicateUsername { username }                   => { (120, Status::Conflict) },
            RegistrationError::PasswordMismatch { password_one, password_two, } => { (121, Status::UnprocessableEntity) }
        }
    }
}

impl ErrorInfo for AuthenticationError {
    #[allow(unused_variables)]
    fn info(&self) -> (i32, Status)
    {
        match self {
            AuthenticationError::InvalidUsername { username, password } => (130, Status::UnprocessableEntity),
            AuthenticationError::InvalidPassword { username, password } => (130, Status::UnprocessableEntity),
        }
    }
}

impl ErrorInfo for MultipartError {
    #[allow(unused_variables)]
    fn info(&self) -> (i32, Status)
    {
        match self {
            MultipartError::RequestError { message }    => { (140, Status::LengthRequired) },
            MultipartError::KeyError { key }            => { (141, Status::BadRequest) },
            MultipartError::ValueError { key, val }     => { (142, Status::BadRequest) }
        }
    }
}

impl ErrorInfo for FileError {
    #[allow(unused_variables)]
    fn info(&self) -> (i32, Status)
    {
        match self {
            FileError::PermissionDenied { file, user }  => { (150, Status::Forbidden) },
            FileError::InvalidDuration { string }       => { (151, Status::UnprocessableEntity) },
        }
    }
}

use rocket::http::Status;
impl ErrorInfo for Error {
    #[allow(unused_variables)]
    fn info(&self) -> (i32, Status)
    {
        match self {
            Error::ClientError(c) => match c {
                ClientError::SessionError { error }         => { error.info() },
                ClientError::RegistrationError { error }    => { error.info() },
                ClientError::AuthenticationError { error }  => { error.info() },
                ClientError::MultipartError { error }       => { error.info() },
                ClientError::FileError { error }            => { error.info() },
            },
            Error::ServerError(_) => (100, Status::InternalServerError),
        }
    }
}

use rocket::data::Outcome;
pub fn failure<T, E>(error: E) -> Outcome<T, Error>
where
    E: Into<Error>
{
    let e: Error = error.into();
    let (_, status) = e.info();
    return Outcome::Failure((status, e))
}

// }}}
