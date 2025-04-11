use chrono::Utc;
use diesel::{result::Error as DieselError, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use dioxus::prelude::*;
use rss::Channel;

use crate::{schema, ChannelFeed, CurrentView, FeedItemRecord, FeedRecord, NewFeedItemRecord, DB};

#[component]
pub fn RefreshButton(
    current_view: Signal<Option<CurrentView>>,
    stored_feeds: Signal<Vec<FeedRecord>>,
    selected_feed_index: Memo<Option<usize>>,
) -> Element {
    let refresh_button_click_handler = move |_| async move {
        match &mut *current_view.write() {
            Some(view) => {
                match view {
                    CurrentView::AllFeeds(articles) => todo!(),
                    CurrentView::SelectedFeed(channel_feed) => {
                        // TODO store the feed URL in ChannelFeed struct
                        let url = {
                            let feeds = stored_feeds.read();
                            let selected = selected_feed_index.read().unwrap();
                            feeds[selected].feed_url.clone()
                        };

                        let content = reqwest::get(url.clone())
                            .await
                            .unwrap()
                            .bytes()
                            .await
                            .unwrap();

                        let channel = Channel::read_from(&content[..]);
                        match channel {
                            Ok(channel) => {
                                let (feeds, feed_items) = DB.with_borrow_mut(|conn| {
                                    {
                                        use crate::schema::feed_items;
                                        use diesel::dsl::insert_into;

                                        // TODO: when inserting, if an item URL is NULL add custom
                                        // handling that checks if other values are unique
                                        let now = Utc::now();
                                        let items = channel
                                            .items
                                            .clone()
                                            .into_iter()
                                            .map(|item| NewFeedItemRecord {
                                                channel_id: channel_feed.channel_id,
                                                title: item.title,
                                                url: item.link,
                                                description: item.description,
                                                author: item.author,
                                                pub_date: item.pub_date,
                                                create_date: now,
                                                update_date: now,
                                            })
                                            .collect::<Vec<_>>();
                                        let items_len = items.len();

                                        for item in items.into_iter() {
                                            let row_count_inserted =
                                                match insert_into(feed_items::table)
                                                    .values(item)
                                                    .execute(conn)
                                                {
                                                    Ok(record) => record,
                                                    Err(error) => {
                                                        match error {
                                                            // TODO: if insert fails then try an update in future
                                                            DieselError::DatabaseError(..) => {
                                                                continue;
                                                            }
                                                            _ => todo!(),
                                                        }
                                                    }
                                                };
                                            dbg!(items_len, row_count_inserted);
                                        }
                                    }

                                    use schema::feed_items::dsl::feed_items;
                                    use schema::feeds::dsl::feeds;

                                    (
                                        feeds
                                            .select(FeedRecord::as_select())
                                            .load::<FeedRecord>(conn)
                                            .unwrap(),
                                        feed_items
                                            .filter(
                                                schema::feed_items::dsl::channel_id
                                                    .eq(channel_feed.channel_id),
                                            )
                                            .select(FeedItemRecord::as_select())
                                            .load::<FeedItemRecord>(conn)
                                            .unwrap(),
                                    )
                                });

                                // safe to unwrap because if we've added a feed, then we want
                                // it to be automatically selected and displayed
                                let selected_index = feeds
                                    .iter()
                                    .position(|val| val.name == channel.title)
                                    .unwrap();
                                let new_channel_feed = ChannelFeed {
                                    name: feeds[selected_index].name.clone(),
                                    channel_id: channel_feed.channel_id,
                                    items: feed_items,
                                    selected: selected_index,
                                };

                                stored_feeds.set(feeds);
                                *view = CurrentView::SelectedFeed(new_channel_feed);
                            }
                            Err(err) => {
                                dbg!(err);
                                panic!();
                            }
                        };
                    }
                }
            }
            None => todo!(),
        };
    };
    rsx! {
        button { class: "btn btn-primary", onclick: refresh_button_click_handler,
            "Get New Articles"
        }
    }
}
