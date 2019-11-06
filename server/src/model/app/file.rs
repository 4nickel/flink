use crate::util::{self, error::ApiResult};
use crate::db::{self, schema::*};
use crate::model::{User};
use diesel::{self, SaveChangesDsl, prelude::*, dsl::*};
use chrono::prelude::*;

#[derive(Identifiable, AsChangeset, Queryable, Associations, Serialize, PartialEq, Debug)]
#[belongs_to(User)]
#[table_name="files"]
pub struct File {
    pub id: i32,
    pub user_id: i32,
    pub key: String,
    pub val: String,
    pub upload_date: NaiveDateTime,
    pub delete_date: NaiveDateTime,
    pub downloads: i32,
    pub bytes: i64,
}

#[derive(Insertable)]
#[table_name="files"]
pub struct FileInsert {
    pub user_id: i32,
    pub key: String,
    pub val: String,
    pub upload_date: NaiveDateTime,
    pub delete_date: NaiveDateTime,
    pub downloads: i32,
    pub bytes: i64,
}

type AllColumns = (
    files::id,
    files::user_id,
    files::key,
    files::val,
    files::upload_date,
    files::delete_date,
    files::downloads,
    files::bytes,
);

const ALL_COLUMNS : AllColumns = (
    files::id,
    files::user_id,
    files::key,
    files::val,
    files::upload_date,
    files::delete_date,
    files::downloads,
    files::bytes,
);

type All = Select<files::table, AllColumns>;
type WithId = Eq<files::id, i32>;
type WithIds<'a> = EqAny<files::id, &'a Vec<i32>>;
type WithUser = Eq<files::user_id, i32>;
type WithUsers<'a> = EqAny<files::user_id, &'a Vec<i32>>;
type WithKey<'a> = Eq<files::key, &'a str>;
type WithKeys<'a> = EqAny<files::key, &'a Vec<&'a str>>;
type WithVal<'a> = Eq<files::val, &'a str>;
type WithVals<'a> = EqAny<files::val, &'a Vec<&'a str>>;

impl File {

    pub fn by_id(id: i32, c: &db::Connection) -> ApiResult<Self>
    { Ok(Self::table().filter(Self::with_id(id)).first(&**c)?) }

    pub fn by_key(key: &str, c: &db::Connection) -> ApiResult<Self>
    { Ok(Self::table().filter(Self::with_key(key)).first(&**c)?) }

    pub fn table() -> files::table
    { files::table }

    pub fn all() -> All
    { Self::table().select(ALL_COLUMNS) }

    pub fn with_id(id: i32) -> WithId
    { files::id.eq(id) }

    pub fn with_ids(ids: &Vec<i32>) -> WithIds
    { files::id.eq_any(ids) }

    pub fn with_user(user_id: i32) -> WithUser
    { files::user_id.eq(user_id) }

    pub fn with_users(user_ids: &Vec<i32>) -> WithUsers
    { files::user_id.eq_any(user_ids) }

    pub fn with_key(key: &str) -> WithKey
    { files::key.eq(key) }

    pub fn with_keys<'a>(keys: &'a Vec<&'a str>) -> WithKeys
    { files::key.eq_any(keys) }

    pub fn with_val(val: &str) -> WithVal
    { files::val.eq(val) }

    pub fn with_vals<'a>(vals: &'a Vec<&'a str>) -> WithVals
    { files::val.eq_any(vals) }

    pub fn insert_one(values: &FileInsert, c: &db::Connection) -> ApiResult<Self>
    {
        diesel::insert_into(Self::table())
            .values(values)
            .execute(&**c)?;

        Ok(Self::table().filter(
            util::sql::with_rowid(util::sql::last_insert_rowid(c))).first(&**c)?
        )
    }

    pub fn create(values: &FileInsert, c: &db::Connection) -> ApiResult<File>
    {
        Self::insert_one(values, c)
    }

    pub fn delete(id: i32, c: &db::Connection) -> ApiResult<usize>
    {
        Ok(diesel::delete(Self::table().filter(Self::with_id(id))).execute(&**c)?)
    }

    pub fn update(&self, c: &db::Connection) -> ApiResult<()>
    {
        self.save_changes::<File>(&**c)?;
        Ok(())
    }

    pub fn is_duplicate(key: &str, c: &db::Connection) -> ApiResult<bool>
    {
        let count = Self::table().select(diesel::dsl::count(Self::with_key(key))).execute(&**c)?;
        return Ok(count > 1);
    }

    pub fn collection_url() -> String
    {
        use crate::api;
        api::collection_url(api::RES_FILE)
    }

    pub fn url(&self) -> String
    {
        use crate::api;
        api::resource_url(api::RES_FILE, self.id)
    }
}

use core::fmt::{Display, Formatter, Error as FmtError};
impl Display for File {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "File[{}] @{} {}", self.id, self.key, self.val)
    }
}
