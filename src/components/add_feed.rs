use std::str::FromStr;

use chrono::Utc;
use diesel::{dsl::insert_into, RunQueryDsl};
use dioxus::prelude::*;
use rss::Channel;
use url::Url;

use crate::{NewFeedRecord, DB};

// https://feeds.arstechnica.com/arstechnica/index
#[component]
pub fn AddFeed(current_feed_url: Signal<Option<Url>>, current_feed: Signal<Channel>) -> Element {
    rsx! {
        form { onsubmit:  move |event| {
                let form = event.data.values();
                let url = form.get("feed").unwrap();
                let url = Url::from_str(&url.as_value()).unwrap();
                        dbg!("hello");
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
                                use crate::schema::feeds;
                                let new_feed = NewFeedRecord { url: url.clone().to_string(), feed_url: url.clone().to_string(), create_date: Utc::now(), update_date: Utc::now() };
                                DB.with_borrow_mut(|conn| {
                                    match insert_into(feeds::table).values(new_feed).execute(conn) {
                                        Ok(_) => dbg!("added feed"),
                                        Err(error) => panic!("{:?}", error)
                                    };
                                });
                                current_feed_url.set(Some(url));
                                current_feed.set(channel);

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
            label {
                "Feed URL"
                input { name: "feed", r#type: "url", required: true }
            }
            button {
                "Add Feed"
            }
        },
    }
}
