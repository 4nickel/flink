use crate::util::{self, error::{Error as ApiError, ApiResult}, random::random_ascii};
use crate::db::{self, schema::*};
use crate::model::{User};
use diesel::{self, prelude::*, dsl::*};
use rocket::{http::{Cookie, Cookies}};

const SESSION_TOKEN_KEY: &'static str = "__session_token";
const SESSION_TOKEN_LEN: usize = 32;

#[derive(Identifiable, Insertable, Queryable, Associations, Serialize, PartialEq, Debug)]
#[belongs_to(User)]
#[table_name="sessions"]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
}

#[derive(Debug, Fail)]
pub enum SessionError {
    #[fail(display = "session cookie not found")]
    CookieNotFound { },
    #[fail(display = "session record not found: {}", token)]
    RecordNotFound {
        token: String
    },
}

// {{{ Types

#[derive(Insertable)]
#[table_name="sessions"]
pub struct SessionInsert {
    pub user_id: i32,
    pub token: String,
}

type AllColumns = (
    sessions::id,
    sessions::user_id,
    sessions::token,
);

const ALL_COLUMNS : AllColumns = (
    sessions::id,
    sessions::user_id,
    sessions::token,
);

type All = Select<sessions::table, AllColumns>;
type WithId = Eq<sessions::id, i32>;
type WithIds<'a> = EqAny<sessions::id, &'a Vec<i32>>;
type WithUser = Eq<sessions::user_id, i32>;
type WithUsers<'a> = EqAny<sessions::user_id, &'a Vec<i32>>;
type WithToken<'a> = Eq<sessions::token, &'a str>;
type WithTokens<'a> = EqAny<sessions::token, &'a Vec<&'a str>>;

// }}}

impl Session {

    // {{{ Data Access

    pub fn insert_one(values: &SessionInsert, c: &db::Connection) -> ApiResult<Self>
    {
        diesel::insert_into(Self::table())
            .values(values)
            .execute(&**c)?;

        Ok(Self::table().filter(
            util::sql::with_rowid(util::sql::last_insert_rowid(c)))
                .first(&**c)?
        )
    }

    pub fn delete(token: &str, c: &db::Connection) -> ApiResult<usize>
    {
        let result =
            diesel::delete(
                Self::table().filter(Self::with_token(token))
            ).execute(&**c)?;
        Ok(result)
    }

    pub fn table() -> sessions::table
    { sessions::table }

    pub fn all() -> All
    { Self::table().select(ALL_COLUMNS) }

    pub fn with_id(id: i32) -> WithId
    { sessions::id.eq(id) }

    pub fn with_ids(ids: &Vec<i32>) -> WithIds
    { sessions::id.eq_any(ids) }

    pub fn with_user(user_id: i32) -> WithUser
    { sessions::user_id.eq(user_id) }

    pub fn with_users(user_ids: &Vec<i32>) -> WithUsers
    { sessions::user_id.eq_any(user_ids) }

    pub fn with_token(token: &str) -> WithToken
    { sessions::token.eq(token) }

    pub fn with_tokens<'a>(tokens: &'a Vec<&'a str>) -> WithTokens
    { sessions::token.eq_any(tokens) }

    // }}}
    // {{{ Cookie Access

    pub fn set_cookie(&self, cookies: &mut Cookies)
    {
        use base64::encode;
        let cookie =
            Cookie::build(SESSION_TOKEN_KEY, encode(&self.token))
                .path("/")
                .http_only(true)
                .finish();
        cookies.add(cookie);
    }

    pub fn del_cookie(cookies: &mut Cookies)
    {
        cookies.remove(Cookie::named(SESSION_TOKEN_KEY));
    }

    pub fn get_cookie(cookies: &Cookies) -> ApiResult<Option<String>>
    {
        use base64::decode;
        match cookies.get(SESSION_TOKEN_KEY) {
            Some(token) => Ok(Some(String::from_utf8(decode(token.value())?)?)),
            None => Ok(None)
        }
    }

    pub fn from_cookie(cookies: &mut Cookies, c: &db::Connection) -> ApiResult<Option<Self>>
    {
        match Self::get_cookie(cookies) {
            Ok(token)  => {
                match token {
                    Some(value) => {
                        println!("[session {}] cookie found", value);
                        let result = Self::table().filter(Self::with_token(&value)).first::<Session>(&**c)?;
                        Ok(Some(result))
                    },
                    None => {
                        println!("[session] no cookie found");
                        Ok(None)
                    }
                }
            },
            /* Delete the cookie if there was an error. */
            Err(error) => {
                println!("[session] cookie error: deleting session cookie");
                Self::del_cookie(cookies);
                Err(error)
            },
        }
    }

    // }}}
    // {{{ Utility

    pub fn is_duplicate(token: &str, c: &db::Connection) -> ApiResult<bool>
    {
        let count = Self::table().select(diesel::dsl::count(Self::with_token(token))).execute(&**c)?;
        return Ok(count > 1)
    }

    pub fn token() -> String
    {
        random_ascii(SESSION_TOKEN_LEN)
    }

    // }}}
    // {{{ API

    pub fn create(user_id: i32, c: &db::Connection, cookies: &mut Cookies) -> ApiResult<Session>
    {
        c.transaction::<_, ApiError, _>(|| {

            println!("[uid {}] creating session", user_id);

            /* generate a token. */
            let mut token = Self::token();
            while Self::is_duplicate(&token, c)? {
                token = Self::token();
            }

            /* store new session. */
            let session = Session::insert_one(&SessionInsert {
                user_id: user_id,
                token: token,
            }, c)?;


            println!("[uid {}] started session [{}]", user_id, session.token);
            session.set_cookie(cookies);
            Ok(session)
        })
    }

    pub fn login(user_id: i32, c: &db::Connection, cookies: &mut Cookies) -> ApiResult<Session>
    {
        use diesel::result::Error::NotFound;

        c.transaction::<_, ApiError, _>(|| {

            /* Find the stored session. */
            let query = Self::table()
                .filter(Self::with_user(user_id))
                .first::<Session>(&**c);

            match query {
                Ok(session) => {
                    /* A session was found, tell the client to store a cookie. */
                    println!("[uid {}] setting session cookie", user_id);
                    session.set_cookie(cookies);
                    Ok(session)
                }
                Err(error)  => match error {
                    /* If no session was found we need to create a new one. */
                    NotFound => {
                        println!("[uid {}] creating new session", user_id);
                        Self::create(user_id, c, cookies)
                    },
                    _ => {
                        println!("[uid {}] error during login: {:?}", user_id, error);
                        Err(error.into())
                    }
                }
            }
        })
    }

    pub fn logout(c: &db::Connection, cookies: &mut Cookies) -> ApiResult<usize>
    {
        use crate::util::error::ServerError;

        let result = match Self::get_cookie(cookies) {
            Ok(Some(token)) => {
                let deleted = Self::delete(&token, &c)?;
                match deleted {
                    1 => {
                        println!("[session {}] logout successfull", token);
                        Ok(1)
                    },
                    0 => {
                        println!("[session {}] logout unsuccessfull: session not found", token);
                        Err(SessionError::RecordNotFound { token: token.into() }.into())
                    },
                    _ => {
                        let message = String::from("DELETE: affected too many rows");
                        println!("[session {}] critical error: {}", token, message);
                        Err(ServerError::SqlInvariantError { message }.into() )
                    },
                }
            }
            Ok(None) => {
                println!("[session] session cookie not found");
                Err(SessionError::CookieNotFound { }.into())
            },
            Err(error) => {
                println!("[session] error while reading cookie");
                Err(error)
            },
        };
        Self::del_cookie(cookies);
        result
    }

    // }}}
}
