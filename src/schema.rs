table! {
    clients (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Varchar,
        email -> Varchar,
        company_name -> Varchar,
        phone_number -> Varchar,
        company_website -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    emails (id) {
        id -> Int4,
        user_id -> Int4,
        email -> Varchar,
        token -> Text,
        verified -> Bool,
        token_generated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        role -> Int2,
        username -> Varchar,
        hashed_password -> Varchar,
        profile_name -> Varchar,
        profile_image -> Varchar,
        joined_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

joinable!(clients -> users (user_id));
joinable!(emails -> users (user_id));

allow_tables_to_appear_in_same_query!(
    clients,
    emails,
    users,
);
