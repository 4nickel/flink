use crate::db::{self, schema::*};
use crate::model::{User};
use crate::util::{self, error::ApiResult, random::random_ascii};
use diesel::{self, prelude::*, dsl::*};
use argon2rs::{argon2i_simple};

const SALT_LEN: usize = 16;
const PEANUTS: &'static str = "peanut-butter-jelly-time!";

#[derive(Identifiable, Insertable, Queryable, Associations, Serialize, PartialEq, Debug)]
#[primary_key(user_id)]
#[belongs_to(User)]
#[table_name="passwords"]
pub struct Password {
    pub user_id: i32,
    pub hash: Vec<u8>,
    pub salt: String,
}

type AllColumns = (
    passwords::user_id,
    passwords::hash,
    passwords::salt,
);

const ALL_COLUMNS : AllColumns = (
    passwords::user_id,
    passwords::hash,
    passwords::salt,
);

type All = Select<passwords::table, AllColumns>;
type WithUser = Eq<passwords::user_id, i32>;
type WithUsers<'a> = EqAny<passwords::user_id, &'a Vec<i32>>;

impl Password {

    pub fn insert_one(values: &Password, c: &db::Connection) -> ApiResult<Self>
    {
        diesel::insert_into(Self::table())
            .values(values)
            .execute(&**c)?;

        Ok(Self::table().filter(
            util::sql::with_rowid(util::sql::last_insert_rowid(c))).first(&**c)?
        )
    }

    pub fn salt() -> String
    {
        random_ascii(SALT_LEN)
    }

    pub fn hash(password: &str, salt: &str) -> Vec<u8>
    {
        argon2i_simple(password, &(salt.to_owned() + PEANUTS)).to_vec()
    }

    pub fn is_valid(password: &str, hash: &Vec<u8>, salt: &str) -> bool
    {
        Self::hash(password, salt) == *hash
    }

    pub fn table() -> passwords::table
    { passwords::table }

    pub fn all() -> All
    { Self::table().select(ALL_COLUMNS) }

    pub fn with_user(user_id: i32) -> WithUser
    { passwords::user_id.eq(user_id) }

    pub fn with_users(user_ids: &Vec<i32>) -> WithUsers
    { passwords::user_id.eq_any(user_ids) }
}
