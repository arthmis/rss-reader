use std::str::FromStr;

use chrono::Utc;
use diesel::{dsl::insert_into, QueryDsl, RunQueryDsl, SelectableHelper};
use dioxus::prelude::*;
use rss::Channel;
use url::Url;

use crate::{schema, CurrentView, FeedRecord, NewFeedRecord, DB};

// https://feeds.arstechnica.com/arstechnica/index
#[component]
pub fn AddFeed(
    current_view: Signal<Option<CurrentView>>,
    stored_feeds: Signal<Vec<FeedRecord>>,
) -> Element {
    rsx! {
        form { onsubmit:  move |event| {
                let form = event.data.values();
                let url = form.get("feed").unwrap();
                let url = Url::from_str(&url.as_value()).unwrap();
                spawn(async move {
                        // if there is a url then retrieve its feed otherwise get latest from all feeds from the database
                        let content = reqwest::get(url.clone())
                        .await.unwrap()
                        .bytes()
                        .await.unwrap();
                        let channel = Channel::read_from(&content[..]);
                        match channel {
                            Ok(channel) => {
                                // save to database
                                let new_feed = NewFeedRecord {
                                    url: url.clone().to_string(),
                                    feed_url: url.clone().to_string(),
                                    name: channel.title.clone(),
                                    create_date: Utc::now(),
                                    update_date: Utc::now() };
                                let feeds = DB.with_borrow_mut(|conn| {
                                    use crate::schema::feeds;
                                    use schema::feeds::dsl::*;
                                    if let Err(error) = insert_into(feeds::table).values(new_feed).execute(conn) {
                                        panic!("{:?}", error)
                                    }

                                    feeds
                                        .select(FeedRecord::as_select())
                                        .load::<FeedRecord>(conn)
                                        .unwrap()
                                });

                                let selected_index = feeds.iter().position(|val| val.name == channel.title);
                                stored_feeds.set(feeds);
                                current_view.set(Some(CurrentView::SelectedFeed(channel, selected_index.unwrap())));

                            },
                            Err(err) => {
                                dbg!(err);
                            },
                        };
                    });
                },
                // extract the url from the feed
                // add the feed to the database
                // only add feed if successful when retrieving feed
                // try the multiple potential feeds
            label { class: "input",
                span { class: "label", "Feed URL"}
                input { class: "input", name: "feed", r#type: "url", required: true }
            }
            button { class: "btn",
                "Add Feed"
            }
        },
    }
}
