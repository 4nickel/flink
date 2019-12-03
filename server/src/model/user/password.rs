use crate::db::{self, schema::*};
use crate::model::{User};
use crate::util::{self, error::Res, random::random_ascii};
use diesel::{self, prelude::*};
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

impl Password {

    pub fn insert_one(values: &Password, c: &db::Connection) -> Res<Self> {
        diesel::insert_into(passwords::table)
            .values(values)
            .execute(&**c)?;
        Ok(passwords::table.filter(
            util::sql::with_rowid(util::sql::last_insert_rowid(c))).first(&**c)?
        )
    }

    pub fn salt() -> String {
        random_ascii(SALT_LEN)
    }

    pub fn hash(password: &str, salt: &str) -> Vec<u8> {
        argon2i_simple(password, &(salt.to_owned() + PEANUTS)).to_vec()
    }

    pub fn is_valid(password: &str, hash: &Vec<u8>, salt: &str) -> bool {
        Self::hash(password, salt) == *hash
    }
}
