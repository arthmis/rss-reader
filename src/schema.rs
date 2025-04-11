// @generated automatically by Diesel CLI.

diesel::table! {
    feed_items (id) {
        id -> Integer,
        channel_id -> Integer,
        title -> Nullable<Text>,
        url -> Nullable<Text>,
        description -> Nullable<Text>,
        author -> Nullable<Text>,
        pub_date -> Nullable<Text>,
        create_date -> Text,
        update_date -> Text,
    }
}

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

diesel::joinable!(feed_items -> feeds (channel_id));

diesel::allow_tables_to_appear_in_same_query!(
    feed_items,
    feeds,
);
