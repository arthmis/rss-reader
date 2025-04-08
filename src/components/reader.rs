use dioxus::prelude::*;
use rss::Channel;
use url::Url;

use crate::Article;

pub struct CurrentFeed {
    feed_url: Url,
    channel: Channel,
}

#[component]
pub fn Feed(articles: ReadOnlySignal<Option<Vec<Article>>> ) -> Element {
    rsx! {
            h1 { "All Articles" }
            match &*articles.read_unchecked() {
                Some(articles) => rsx! { 
                    ul {
                        for item in articles.iter() {
                            // li { key: "{item.guid}",
                            li {
                                h2 {
                                    a {
                                        href: item.link.clone().unwrap(),
                                        {item.title.clone().unwrap()}
                                    }
                                }
                                p {
                                    {item.description.clone().unwrap()}
                                }
                            }
                        }
                    }
                },
                None => rsx! { "Loading..." }
            }
        }   
}
