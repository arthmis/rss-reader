use std::{cell::RefCell, str::FromStr};

use chrono::{DateTime, Utc};
use components::{AddFeed, Feed};
use diesel::{
    backend::Backend, deserialize::FromSql, expression::AsExpression, serialize::ToSql, sql_types,
    Connection, SqliteConnection,
};
use dioxus::prelude::*;

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
    pub name: String,
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
    pub name: String,
    #[diesel(serialize_as = StringTime)]
    pub create_date: DateTime<Utc>,
    #[diesel(serialize_as = StringTime)]
    pub update_date: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = schema::feed_items)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FeedItemRecord {
    pub id: i32,
    pub channel_id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub pub_date: Option<String>,
    #[diesel(deserialize_as = StringTime)]
    pub create_date: DateTime<Utc>,
    #[diesel(deserialize_as = StringTime)]
    pub update_date: DateTime<Utc>,
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = schema::feed_items)]
pub struct NewFeedItemRecord {
    pub channel_id: i32,
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub pub_date: Option<String>,
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

async fn load_all_feeds() -> (Vec<FeedRecord>, Vec<Article>) {
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
    (feed_urls, articles)
}

fn main() {
    dioxus::launch(App);
}

enum CurrentView {
    AllFeeds(Vec<Article>),
    // SelectedFeed(Channel, usize),
    SelectedFeed(ChannelFeed),
}

pub struct ChannelFeed {
    name: String,
    id: i32,
    items: Vec<FeedItemRecord>,
    selected: usize,
}

#[component]
fn App() -> Element {
    let mut current_view: Signal<Option<CurrentView>> = use_signal(|| None);
    let mut stored_feeds: Signal<Vec<FeedRecord>> = use_signal(Vec::new);

    let selected_feed_index = use_memo(move || match &*current_view.read() {
        Some(CurrentView::SelectedFeed(ChannelFeed { selected, .. })) => Some(*selected),
        _ => None,
    });

    use_effect(move || {
        spawn(async move {
            let (feed_urls, feeds) = load_all_feeds().await;
            current_view.set(Some(CurrentView::AllFeeds(feeds)));
            stored_feeds.set(feed_urls);
        });
    });

    rsx! {
        // Global app resources
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
         document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css")
        }

        div { class: "drawer md:drawer-open",
            input { id: "my-drawer-2", r#type: "checkbox", class: "drawer-toggle"}
            div { class: "drawer-content flex flex-col items-center justify-center",
                label { for: "my-drawer-2", class: "btn btn-primary drawer-button md:hidden",
                    "Open drawer"
                }
                AddFeed { current_view, stored_feeds }
                Feed{ current_view }
            }
            div { class: "drawer-side",
                label { for: "my-drawer-2", aria_label: "close sidebar", class: "drawer-overlay",
                }
                ul { class: "menu bg-base-200 text-base-content min-h-full w-80",
                    for (i, record) in stored_feeds.iter().enumerate() {
                        li { onmounted: move |element| async move { 
                            // scroll the selected feed into view
                            if let Some(index) = &*selected_feed_index.read() {
                                if *index == i {
                                    let _ = element.scroll_to(ScrollBehavior::Smooth).await;
                                }
                            }
                        },
                        class: if selected_feed_index.read().is_some() && selected_feed_index.read().unwrap() == i {"active-feed"},
                            {
                                let (record_id, channel_name) = (record.id, record.name.clone());
                                rsx!{
                                    a { onclick: move |_| {
                                    let items = DB.with_borrow_mut(move |conn| {
                                        use schema::feed_items::dsl::*;

                                        // select by channel id
                                        feed_items
                                            .filter(channel_id.eq(record_id))
                                            .limit(40)
                                            .select(FeedItemRecord::as_select())
                                            .load::<FeedItemRecord>(conn)
                                            .unwrap()
                                    });
                                    let channel_feed = ChannelFeed {
                                        name: channel_name.clone(),
                                        id: record_id,
                                        items,
                                        selected: i,
                                    };

                                    current_view.set(Some(CurrentView::SelectedFeed(channel_feed)));

                                    },
                                        {record.name.clone()}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
