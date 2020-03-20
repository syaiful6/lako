table! {
    email_tokens (id) {
        id -> Int4,
        email_id -> Int4,
        token -> Varchar,
        created_at -> Timestamp,
    }
}

table! {
    emails (id) {
        id -> Int4,
        user_id -> Int4,
        email -> Varchar,
        verified -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        hashed_password -> Varchar,
        role -> Int2,
        profile_name -> Varchar,
        profile_image -> Varchar,
        joined_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(email_tokens -> emails (email_id));
joinable!(emails -> users (user_id));

allow_tables_to_appear_in_same_query!(
    email_tokens,
    emails,
    users,
);
