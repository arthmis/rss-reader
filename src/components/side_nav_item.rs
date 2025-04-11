use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use dioxus::prelude::*;

use crate::{schema, ChannelFeed, CurrentView, FeedItemRecord, DB};

#[derive(Clone, Debug, PartialEq)]
pub struct FeedNameId {
    pub id: i32,
    pub channel_name: String,
}

#[component]
pub fn SideNavItem(
    mut current_view: Signal<Option<CurrentView>>,
    selected_feed_index: Memo<Option<usize>>,
    feed_metadata: FeedNameId,
    index: usize,
) -> Element {
    rsx! {

        li { key: "{feed_metadata.id}", onmounted: move |element| async move {
            // scroll the selected feed into view
            if let Some(selected_index) = &*selected_feed_index.read() {
                if *selected_index == index {
                    let _ = element.scroll_to(ScrollBehavior::Smooth).await;
                }
            }
        },
        class: if selected_feed_index.read().is_some() && selected_feed_index.read().unwrap() == index {"active-feed"},
            {
                rsx!{
                    a { onclick: move |_| {
                    let items = DB.with_borrow_mut(move |conn| {
                        use schema::feed_items::dsl::*;

                        // select by channel id
                        feed_items
                            .filter(channel_id.eq(feed_metadata.id))
                            .select(FeedItemRecord::as_select())
                            .load::<FeedItemRecord>(conn)
                            .unwrap()
                    });
                    let channel_feed = ChannelFeed {
                        name: feed_metadata.channel_name.clone(),
                        channel_id: feed_metadata.id,
                        items,
                        selected: index,
                    };

                    current_view.set(Some(CurrentView::SelectedFeed(channel_feed)));

                    },
                        {feed_metadata.channel_name.clone()}
                    }
                }
            }
        }
    }
}
