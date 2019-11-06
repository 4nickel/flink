use crate::db;
use crate::model::{Session, User};
use crate::util::error::ApiResult;

use rocket::{request::Form, http::{Cookies, Status}, response::status};
use rocket_contrib::json::{Json, JsonValue};

// {{{ Login

#[derive(FromForm, Serialize, Deserialize, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
}

pub fn login(login: Login, c: db::Connection, mut cookies: Cookies) -> ApiResult<JsonValue>
{
    let session = User::login(
        &login.username,
        &login.password,
        &c,
        &mut cookies
    )?;
    Ok(json!({ "token": session.token }))
}

#[post("/", data = "<data>", format = "application/json")]
pub fn login_json(data: Json<Login>, c: db::Connection, cookies: Cookies) -> ApiResult<JsonValue>
{ login(data.into_inner(), c, cookies) }

#[post("/", data = "<data>", format = "application/x-www-form-urlencoded")]
pub fn login_http(data: Form<Login>, c: db::Connection, cookies: Cookies) -> ApiResult<JsonValue>
{ login(data.into_inner(), c, cookies) }

// }}}
// {{{ Logout

#[delete("/")]
pub fn logout(c: db::Connection, mut cookies: Cookies) -> ApiResult<JsonValue>
{
    match Session::logout(&c, &mut cookies) {
        Ok(_) => Ok(json!({})),
        Err(error) => Err(error)
    }
}

// }}}
// {{{ Register

#[derive(FromForm, Serialize, Deserialize, Debug)]
pub struct Register {
    pub username:     String,
    pub password_one: String,
    pub password_two: String,
}

pub fn register(register: Register, c: db::Connection, mut cookies: Cookies) -> ApiResult<status::Created<JsonValue>>
{
    match User::register(
        &register.username,
        &register.password_one,
        &register.password_two,
        &c,
        &mut cookies
    ) {
        Ok((user, _, session)) => Ok(status::Created(user.url(), Some(json!({ "token": session.token })))),
        Err(error) => Err(error)
    }
}

#[post("/", data = "<data>", format = "application/json")]
pub fn register_json(data: Json<Register>, c: db::Connection, cookies: Cookies) -> ApiResult<status::Created<JsonValue>>
{ register(data.into_inner(), c, cookies) }

#[post("/", data = "<data>", format = "application/x-www-form-urlencoded")]
pub fn register_http(data: Form<Register>, c: db::Connection, cookies: Cookies) -> ApiResult<status::Created<JsonValue>>
{ register(data.into_inner(), c, cookies) }

// }}}
// {{{ Query

#[get("/")]
pub fn query(u: User) -> ApiResult<JsonValue>
{
    Ok(json!({"name": u.name}))
}

#[get("/", rank = 3)]
pub fn query_forbidden() -> Status
{ Status::Forbidden }

// }}}
