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

joinable!(email_tokens -> emails (email_id));

allow_tables_to_appear_in_same_query!(
    email_tokens,
    emails,
);
