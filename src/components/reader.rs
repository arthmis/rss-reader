use dioxus::prelude::*;
use crate::CurrentView;


#[component]
pub fn Feed(current_view: Signal<Option<CurrentView>> ) -> Element {
    rsx! {
        match &*current_view.read() {
            Some(view) => rsx!{
                match view {
                    CurrentView::NewFeed(channel) => rsx!{
                        h1 { "All Articles" }
                        ul {
                            for item in channel.items.iter() {
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
                    CurrentView::AllFeeds(articles)=> rsx!{
                        h1 { "All Articles" }
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
                    CurrentView::SelectedFeed(channel) => rsx!{},
                    },
                },
                None => rsx!{},
            },
        }   
}
