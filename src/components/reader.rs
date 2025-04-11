use dioxus::prelude::*;
use crate::CurrentView;


#[component]
pub fn Feed(current_view: Signal<Option<CurrentView>> ) -> Element {
    rsx! {
        match &*current_view.read() {
            Some(view) => rsx!{
                match view {
                    CurrentView::SelectedFeed(channel, ..) => rsx!{
                        h1 { "All Articles" }
                        ul {
                            for item in channel.items.iter() {
                                // li { key: "{item.guid}",
                                li {
                                    div { class: "card card-border bg-base-100 min-w-[45ch] max-w-[80ch] shadow-sm",
                                        div { class: "card-body w-full",
                                            h2 { class: "card-title",
                                        a {
                                            href: item.url.clone().unwrap(),
                                            {item.title.clone().unwrap()}
                                        }
                                    }
                                    p {
                                        {item.description.clone().unwrap()}
                                    }
                                        }
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
                                    div { class: "card card-border bg-base-100 min-w-[45ch] max-w-[80ch] shadow-sm",
                                        div { class: "card-body",
                                            h2 { class: "card-title",
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
                            }
                        }
                    },
                    },
                },
                None => rsx!{},
            },
        }   
}
