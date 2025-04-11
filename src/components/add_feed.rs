use std::str::FromStr;

use chrono::Utc;
use diesel::{dsl::insert_into, QueryDsl, RunQueryDsl, SelectableHelper};
use dioxus::prelude::*;
use rss::Channel;
use url::Url;

use crate::{schema::{self, feed_items}, ChannelFeed, CurrentView, FeedItemRecord, FeedRecord, NewFeedItemRecord, NewFeedRecord, DB};

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
                            let (feeds, feed_items, selected_feed_id) = DB.with_borrow_mut(|conn| {
                                let feed_id = {
                                    use crate::schema::feeds;
                                    use schema::feeds::dsl::*;

                                    match insert_into(feeds::table)
                                        .values(new_feed)
                                        .returning(id)
                                        .get_result(conn) {
                                        Ok(record) => record,
                                        Err(error) => panic!("{:?}", error),
                                    }
                                };


                                {
                                    use crate::schema::feed_items;

                                    // TODO: when inserting, if an item URL is NULL add custom 
                                    // handling that checks if other values are unique
                                    let now = Utc::now();
                                    let items = channel.items.clone().into_iter().map(|item| 
                                        NewFeedItemRecord { 
                                            channel_id: feed_id,
                                            title: item.title,
                                            url: item.link,
                                            description: item.description,
                                            author: item.author,
                                            pub_date: item.pub_date,
                                            create_date: now,
                                            update_date: now,
                                    }).collect::<Vec<_>>();
                                    let items_len = items.len();

                                    let row_count_inserted = match insert_into(feed_items::table).values(items).execute(conn) {
                                        Ok(record) => record,
                                        Err(error) => panic!("{:?}", error),
                                    };
                                    dbg!(items_len, row_count_inserted);
                                }

                                use schema::feeds::dsl::feeds;
                                use schema::feed_items::dsl::feed_items;

                                (feeds
                                    .select(FeedRecord::as_select())
                                    .load::<FeedRecord>(conn)
                                    .unwrap(),
                                feed_items
                                    .select(FeedItemRecord::as_select())
                                    .load::<FeedItemRecord>(conn)
                                    .unwrap(),
                                feed_id
                                )
                            });

                            // safe to unwrap because if we've added a feed, then we want
                            // it to be automatically selected and displayed
                            let selected_index = feeds.iter().position(|val| val.name == channel.title).unwrap();
                            let channel_feed = ChannelFeed {
                                name: feeds[selected_index].name.clone(),
                                channel_id: selected_feed_id,
                                items: feed_items,
                                selected: selected_index 
                            };

                            stored_feeds.set(feeds);
                            current_view.set(Some(CurrentView::SelectedFeed(channel_feed)));

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
