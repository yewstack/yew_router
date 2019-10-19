# YewRouter
A routing library for the [Yew](https://github.com/yewstack/yew) frontend framework.

This project has just joined efforts with [saschagrunert/yew-router](https://github.com/saschagrunert/yew-router) and is working towards a new design.


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
        render = Router::render(|switch: Option<AppRoute>| {
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

### How to Include
You can use the in-development version in your project by adding it to your dependencies like so:
```toml
[dependencies]
yew_router = { git = "https://github.com/yewstack/yew_router", branch="master" }
yew = "0.9.2"
```
Or if you want to use the prior version before the projects merged:
```toml
[dependencies]
yew-router = "0.5.0"
yew = "0.9.2"
```

-----
### Contributions/Requests

If you have any questions, suggestions, or want to contribute, please open an Issue or PR and we will get back to you in a timely manner.
