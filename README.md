# YewRouter
A Routing library for the Yew frontend framework.

I'm currently working towards getting this library in a releasable state.


### Example
```rust
html!{
    <Router>
        <Route path=route!("/a/{}" => AComponent) />
        <Route path=route!("/b") >
            <BComponent />
        </Route>
        <Route path=route!("/c/{capture}" => |matches| {
            Some(html!{{matches["capture"}})
        }) />
    </Router>
}
```

### How to use currently
You can use it in your project by adding it to your dependencies like so:
```toml
[dependencies]
yew_router = { git = "https://github.com/hgzimmerman/YewRouter" branch="master" }
```
-----
### Disclaimer
The API surface will be changing in the near future as I experiment with ways to accomplish the goal of creating an easy to use router.
Specifically, the `Route` component may take a render function specified outside of the `route!()` macro in the near future, as well as small changes to the syntax for the route matcher string.

-----
### Contributions/Requests

If you have any questions, suggestions, or want to contribute, please open an Issue or PR and I will get back to you in a timely manner.
