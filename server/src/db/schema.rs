table! {
    files (id) {
        id -> Integer,
        user_id -> Integer,
        key -> Text,
        val -> Text,
        upload_date -> Timestamp,
        delete_date -> Timestamp,
        downloads -> Integer,
        bytes -> BigInt,
    }
}

table! {
    passwords (user_id) {
        user_id -> Integer,
        hash -> Binary,
        salt -> Text,
    }
}

table! {
    sessions (id) {
        id -> Integer,
        user_id -> Integer,
        token -> Text,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

joinable!(files -> users (user_id));
joinable!(passwords -> users (user_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(files, passwords, sessions, users,);
