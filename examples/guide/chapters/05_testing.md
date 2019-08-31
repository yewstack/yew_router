# Testing

To make sure that your router works reliably, you will want to test your `FromMatches` implementations, as well as the output of your `route!` macros.


## FromMatches
Testing implementors of is simple enough.

Just provide a `&Matches` (an alias of `HashMap<'str, String>`) to your prop's `from_matches()` method and test the expected results.

### Example
```rust

#[test]
fn creates_props() {
    let mut matches: Matches = HashMap::new();
    matches.insert("key", "value");
    assert!(Props::from_matches(matches).is_some())
}
```

## `route!`
Testing this is often less than ideal, since you often will want to keep the macro in-line with the `Route` so you have better readability.
The best solution at the moment is to just copy + paste the `route!` macros as you see them into the tests. 

### Example
```rust

#[test]
fn matcher_rejects_unexpected_route() {
    let matcher = route!("/a/b");
    matcher.match_path("/a/b").expect("should match");
    matcher.match_path("/a/c").expect("should reject");
}