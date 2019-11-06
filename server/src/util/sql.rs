use crate::db;
use diesel::{self, prelude::*};

// {{{ Sqlite

pub fn last_insert_rowid(c: &db::Connection) -> i32
{
    no_arg_sql_function!(last_insert_rowid, diesel::sql_types::Integer);
    match diesel::select(last_insert_rowid).first(&**c) {
        Ok(value) => value,
        _ => 0,
    }
}

use diesel::sql_types::{Bool};
use diesel::expression::sql_literal::{SqlLiteral, sql};
pub fn with_rowid(oid: i32) -> SqlLiteral<Bool>
{
    sql::<Bool>(&format!("OID = {}", oid))
}

// }}}
