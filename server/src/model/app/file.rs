use crate::util::{self, error::Res};
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

impl File {

    pub fn by_id(id: i32, c: &db::Connection) -> Res<Self> {
        Ok(files::table.filter(files::id.eq(id)).first(&**c)?)
    }

    pub fn by_key(key: &str, c: &db::Connection) -> Res<Self> {
        Ok(files::table.filter(files::key.eq(key)).first(&**c)?)
    }

    pub fn insert_one(values: &FileInsert, c: &db::Connection) -> Res<Self> {
        diesel::insert_into(files::table)
            .values(values)
            .execute(&**c)?;

        Ok(files::table.filter(
            util::sql::with_rowid(util::sql::last_insert_rowid(c))).first(&**c)?
        )
    }

    pub fn create(values: &FileInsert, c: &db::Connection) -> Res<File> {
        Self::insert_one(values, c)
    }

    pub fn delete(id: i32, c: &db::Connection) -> Res<usize> {
        Ok(diesel::delete(files::table.filter(files::id.eq(id))).execute(&**c)?)
    }

    pub fn update(&self, c: &db::Connection) -> Res<()> {
        self.save_changes::<File>(&**c)?;
        Ok(())
    }

    pub fn is_duplicate(key: &str, c: &db::Connection) -> Res<bool> {
        let count = files::table.select(diesel::dsl::count(files::key.eq(key))).execute(&**c)?;
        return Ok(count > 1);
    }

    pub fn collection_url() -> String {
        use crate::api;
        api::collection_url(api::RES_FILE)
    }

    pub fn url(&self) -> String {
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
