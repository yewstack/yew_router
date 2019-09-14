# YewRouter
A Routing library for the Yew frontend framework.

I'm currently working towards getting this library in a releasable state.


### Example
```rust
html!{
    <Router>
        <Route matcher=route!("/a/{}") render=component::<AComponent>()) />
        <Route matcher=route!("/b") >
            <BComponent />
        </Route>
        <Route matcher=route!("/c/{capture}" render=Render::new(|matches: &Matches| {
            Some(html!{{matches["capture"}})
        }) />
    </Router>
}
```

### How to use currently
You can use it in your project by adding it to your dependencies like so:
```toml
[dependencies]
yew_router = { git = "https://github.com/yewstack/yew_router" branch="master" }
yew = { git = "https://github.com/yewstack/yew", branch="master" }
```
Currently, this crate relies on unreleased features of yew, and so can't be released itself.


-----
### Contributions/Requests

If you have any questions, suggestions, or want to contribute, please open an Issue or PR and I will get back to you in a timely manner.
