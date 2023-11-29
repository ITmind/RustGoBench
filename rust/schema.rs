// @generated automatically by Diesel CLI.

diesel::table! {
    users (email) {
        email -> Text,
        first -> Text,
        last -> Text,
        city -> Text,
        county -> Text,
        age -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(users,);
