use crate::util::{self, error::{Error as ApiError, ApiResult}};
use crate::db::{self, schema::*};
use crate::model::{Password, Session};
//use std::ops::Deref;
use diesel::{self, SaveChangesDsl, prelude::*, dsl::*, result::QueryResult};
use rocket::{Outcome, http::{Cookies}, request::{self, Request, FromRequest}};
use rocket::http::Status;

#[derive(Identifiable, AsChangeset, Queryable, Associations, Serialize, PartialEq, Debug)]
#[table_name="users"]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct UserInsert {
    pub name: String,
}

#[derive(Debug, Fail)]
pub enum RegistrationError {
    #[fail(display = "duplicate username: '{}'", username)]
    DuplicateUsername {
        username: String
    },
    #[fail(display = "password mismatch: '{}' != '{}'", password_one, password_two)]
    PasswordMismatch {
        password_one: String,
        password_two: String
    },
}

#[derive(Debug, Fail)]
pub enum AuthenticationError {
    #[fail(display = "invalid username: '{}' -> '{}'", username, password)]
    InvalidUsername {
        username: String,
        password: String
    },
    #[fail(display = "invalid password: '{}' -> '{}'", username, password)]
    InvalidPassword {
        username: String,
        password: String
    },
}

type AllColumns = (
    users::id,
    users::name,
);

const ALL_COLUMNS : AllColumns = (
    users::id,
    users::name,
);

type All = Select<users::table, AllColumns>;
type WithId = Eq<users::id, i32>;
type WithIds<'a> = EqAny<users::id, &'a Vec<i32>>;
type WithName<'a> = Eq<users::name, &'a str>;
type WithNames<'a> = EqAny<users::name, &'a Vec<&'a str>>;

impl User {

    pub fn insert_one(values: &UserInsert, c: &db::Connection) -> ApiResult<Self>
    {
        diesel::insert_into(Self::table())
            .values(values)
            .execute(&**c)?;

        Ok(Self::table().filter(
            util::sql::with_rowid(util::sql::last_insert_rowid(c))).first(&**c)?
        )
    }

    pub fn table() -> users::table
    { users::table }

    pub fn all() -> All
    { Self::table().select(ALL_COLUMNS) }

    pub fn collection_url() -> String
    {
        use crate::api;
        api::collection_url(api::RES_USER)
    }

    pub fn url(&self) -> String
    {
        use crate::api;
        api::resource_url(api::RES_USER, self.id)
    }

    pub fn with_id(id: i32) -> WithId
    { users::id.eq(id) }

    pub fn with_ids(ids: &Vec<i32>) -> WithIds
    { users::id.eq_any(ids) }

    pub fn with_name(name: &str) -> WithName
    { users::name.eq(name) }

    pub fn with_names<'a>(names: &'a Vec<&'a str>) -> WithNames
    { users::name.eq_any(names) }

    pub fn by_id(id: i32, c: &db::Connection) -> ApiResult<Self>
    {
        let user = Self::table()
            .filter(Self::with_id(id))
            .first(&**c)?;
        Ok(user)
    }

    pub fn from_cookie(cookies: &mut Cookies, c: &db::Connection) -> ApiResult<Option<Self>>
    {
        match Session::from_cookie(cookies, c) {
            Ok(result) => {
                match result {
                    Some(s) => {
                        println!("[user {}] session found [{}]", s.user_id, s.token);
                        Ok(Some(Self::by_id(s.user_id, c)?))
                    }
                    None => {
                        println!("[user] session not found");
                        Ok(None)
                    }
                }
            }
            Err(error) => {
                // TODO: handle this correctly..
                println!("[user] session error: {:?}", error);
                Session::del_cookie(cookies);
                Ok(None)
            }
        }
    }

    pub fn create(values: &UserInsert, password: &str, c: &db::Connection) -> ApiResult<(User, Password)>
    {
        c.transaction::<_, ApiError, _>(|| {
            // Check for duplicate names.
            if Self::is_duplicate(&values.name, c)? {
                println!("[user {}]: duplicate username", values.name);
                return Err(RegistrationError::DuplicateUsername { username: values.name.clone() }.into())
            }
            // Create the user and password records.
            let user = Self::insert_one(values, c)?;
            let salt = Password::salt();
            let hash = Password::hash(password, &salt);
            let pass = Password::insert_one(&Password {
                user_id: user.id,
                hash:    hash,
                salt:    salt,
            }, c)?;
            Ok((user, pass))
        })
    }

    pub fn delete(id: i32, c: &db::Connection) -> ApiResult<usize>
    {
        Ok(diesel::delete(Self::table().filter(Self::with_id(id))).execute(&**c)?)
    }

    pub fn update(user: &User, c: &db::Connection) -> ApiResult<()>
    {
        user.save_changes::<User>(&**c)?;
        Ok(())
    }

    pub fn is_duplicate(name: &str, c: &db::Connection) -> QueryResult<bool>
    {
        let count = Self::table().select(diesel::dsl::count(Self::with_name(name))).execute(&**c)?;
        Ok(count > 1)
    }

    pub fn register(name: &str, password_one: &str, password_two: &str, c: &db::Connection, cookies: &mut Cookies) -> ApiResult<(User, Password, Session)>
    {
        if password_one != password_two {
            println!("[register {}]: password mismatch", name);
            return Err(RegistrationError::PasswordMismatch {
                password_one: password_one.into(),
                password_two: password_two.into()
            }.into())
        }
        c.transaction::<_, ApiError, _>(|| {
            println!("[register {}]: creating user", name);
            let (user, pass) = Self::create(&UserInsert { name: name.to_string() }, password_one, c)?;
            println!("[register {}]: creating session", name);
            let sess = Session::create(user.id, c, cookies)?;
            Ok((user, pass, sess))
        })
    }

    pub fn login(username: &str, password: &str, c: &db::Connection, cookies: &mut Cookies) -> ApiResult<Session>
    {
        use diesel::result::Error::NotFound;
        type Login = (i32, Vec<u8>, String);

        c.transaction::<_, ApiError, _>(|| {

            let login = Password::table()
                    .inner_join(User::table())
                    .filter(
                        User::with_name(username)
                    ).select((
                        users::id,
                        passwords::hash,
                        passwords::salt
                    )).first::<Login>(&**c);

            match login {
                Ok((user, hash, salt)) => {
                    match Password::is_valid(password, &hash, &salt) {
                        true  => {
                            println!("[login {}] password valid", username);
                            Ok(Session::login(user, c, cookies)?)
                        },
                        false => {
                            println!("[login {}] password invalid", username);
                            Err(AuthenticationError::InvalidPassword {
                                username: username.into(),
                                password: password.into()
                            }.into())
                        }
                    }
                }
                Err(error) => match error {
                    NotFound => {
                        println!("[login {}] username invalid", username);
                        Err(AuthenticationError::InvalidUsername {
                            username: username.into(),
                            password: password.into()
                        }.into())
                    }
                    _ => {
                        println!("[login {}] password error: {:?}", username, error);
                        Err(error.into())
                    }
                }
            }
        })
    }
}

// {{{ User

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ApiError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error>
    {
        use crate::util::error::ServerError;

        let connection = match request.guard::<db::Connection>() {
            Outcome::Success(value) => value,
            _ => return Outcome::Failure((Status::InternalServerError, ServerError::DataGuardError { name: String::from("Connection") }.into() )),
        };

        let mut cookies = match request.guard::<Cookies>() {
            Outcome::Success(value) => value,
            _ => return Outcome::Failure((Status::InternalServerError, ServerError::DataGuardError { name: String::from("Cookies") }.into() )),
        };

        match User::from_cookie(&mut cookies, &connection) {
            Ok(Some(u)) => {
                println!("[user {}] authenticated", u.id);
                Outcome::Success(u)
            },
            Ok(None) => {
                println!("[user] not authenticated");
                Outcome::Forward(())
            }
            Err(error) => {
                println!("[user] authentication error: {:?}", error);
                Outcome::Failure((Status::InternalServerError, error))
            }
        }
    }
}

use core::fmt::{Display, Formatter, Error as FmtError};
impl Display for User {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "User[{}] {}", self.id, self.name)
    }
}

// }}}
