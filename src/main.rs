use std::{
    cell::RefCell,
    str::FromStr,
};

use chrono::{DateTime, Utc};
use diesel::{
    backend::Backend,
    deserialize::FromSql,
    expression::AsExpression,
    serialize::ToSql,
    sql_types, Connection, SqliteConnection,
};
use dioxus::prelude::*;

use components::Feed;
use rss::{Channel, Guid};
use url::Url;

mod components;
mod models;
mod schema;

use diesel::prelude::*;

thread_local! {
    pub static DB: RefCell<SqliteConnection> = {
        let database_url = "./testdb.db";
        // Return the connection
        RefCell::new(SqliteConnection::establish(database_url).unwrap())
    };
}

#[derive(AsExpression, Debug, Clone)]
#[diesel(sql_type = sql_types::Text)]
pub struct StringTime(String);

impl From<DateTime<Utc>> for StringTime {
    fn from(val: DateTime<Utc>) -> Self {
        Self(val.to_rfc2822())
    }
}

impl From<StringTime> for DateTime<Utc> {
    fn from(value: StringTime) -> Self {
        DateTime::parse_from_rfc2822(&value.0).unwrap().into()
    }
}

impl<Text, DB> Queryable<Text, DB> for StringTime
where
    DB: Backend,
    Text: diesel::sql_types::SingleValue,
    String: FromSql<Text, DB>,
{
    type Row = String;

    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        Ok(StringTime(row))
    }
}

impl<DB> ToSql<sql_types::Text, DB> for StringTime
where
    DB: Backend,
    String: ToSql<sql_types::Text, DB>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, DB>,
    ) -> diesel::serialize::Result {
        self.0.to_sql(out)
    }
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = schema::feeds)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FeedRecord {
    pub id: i32,
    pub url: String,
    pub feed_url: String,
    #[diesel(deserialize_as = StringTime)]
    pub create_date: DateTime<Utc>,
    #[diesel(deserialize_as = StringTime)]
    pub update_date: DateTime<Utc>,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = schema::feeds)]
pub struct NewFeedRecord {
    pub url: String,
    pub feed_url: String,
    #[diesel(serialize_as = StringTime)]
    pub create_date: DateTime<Utc>,
    #[diesel(serialize_as = StringTime)]
    pub update_date: DateTime<Utc>,
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");

#[derive(Debug, Clone, PartialEq)]
pub struct Article {
    // id: Uuid,
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    author: Option<String>,
    guid: Option<Guid>,
    pub_date: Option<DateTime<Utc>>,
    channel_title: String,
    channel_link: Url,
}

async fn load_all_feeds() -> Vec<Article> {
    let feed_urls = DB.with_borrow_mut(|conn| {
        use schema::feeds::dsl::*;

        feeds
            .select(FeedRecord::as_select())
            .load::<FeedRecord>(conn)
            .unwrap()
    });

    let mut articles = Vec::new();
    for record in feed_urls.iter() {
        let content = reqwest::get(&record.feed_url)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        let channel = Channel::read_from(&content[..]);
        match channel {
            Ok(channel) => {
                for item in channel.items {
                    let pub_date = if let Some(date) = item.pub_date {
                        if let Ok(time) = DateTime::parse_from_rfc2822(&date) {
                            Some(time.into())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let article = Article {
                        // id: Uuid::new_v4(),
                        title: item.title,
                        link: item.link,
                        description: item.description,
                        author: item.author,
                        guid: item.guid,
                        pub_date,
                        channel_title: channel.title.clone(),
                        channel_link: Url::from_str(&channel.link).unwrap(),
                    };
                    articles.push(article);
                }
            }
            Err(err) => {
                panic!("{:?}", err);
            }
        };
    }
    articles.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));
    articles.dedup_by(|a, b| {
        if let (Some(a), Some(b)) = (&a.link, &b.link) {
            return a.eq_ignore_ascii_case(b);
        }

        false
    });
    articles
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut added_new_feed = use_signal(|| false);
    let articles = use_resource(move || async move {
        added_new_feed.set(false);
        load_all_feeds().await
    });

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        // AddFeed { current_feed_url, current_feed }
        Feed { articles: articles.value(), }
    }
}
