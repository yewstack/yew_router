# YewRouter
A Routing library for the Yew frontend framework.

I'm currently working towards getting this library in a releasable state.


### Example
```rust
#[derive(Switch, Debug)]
pub enum AppRoute {
    #[to = "/profile/{id}"]
    Profile(u32),
    #[to = "/forum"]
    #[rest]
    Forum(ForumRoute),
    #[to = "/"]
    Index,
}

#[derive(Switch, Debug)]
pub enum ForumRoute {
    #[to = "/{subforum}/{thread_slug}"]
    SubForumAndThread{subforum: String, thread_slug: String}
    #[to = "/{subforum}"]
    SubForum{subforum: String}
}

html! {
    <Router<AppRoute, ()>
        render = Router::render(|switch: Option<&AppRoute>| {
            match switch {
                Some(AppRoute::Profile(id)) => html!{<ProfileComponent id = id/>},
                Some(AppRoute::Index) => html!{<IndexComponent/>},
                Some(AppRoute::Forum(forum_route)) => html!{<ForumComponent route = forum_route/>},
                None => html!{"404"}
            }
        })
    />
}
```

### How to use currently
You can use it in your project by adding it to your dependencies like so:
```toml
[dependencies]
yew_router = { git = "https://github.com/yewstack/yew_router", branch="master" }
yew = "0.9.0"
```
Currently, this crate is ready to release, but is waiting on determining a namespace it can use.


-----
### Contributions/Requests

If you have any questions, suggestions, or want to contribute, please open an Issue or PR and I will get back to you in a timely manner.
