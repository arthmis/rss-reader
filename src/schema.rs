// @generated automatically by Diesel CLI.

diesel::table! {
    feeds (id) {
        id -> Integer,
        url -> Text,
        feed_url -> Text,
        name -> Text,
        create_date -> Text,
        update_date -> Text,
    }
}
