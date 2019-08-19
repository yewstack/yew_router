#[cfg(test)]
mod tests {
    use yew_router_derive::FromMatches;
    use std::collections::HashMap;

    #[test]
    fn derive_works() {

        #[derive(FromMatches)]
        #[allow(dead_code)]
        struct S {
            hello: String,
            there: String,
        }
    }

    #[test]
    fn derive_is_implemented_correctly() {
        #[derive(FromMatches)]
        #[allow(dead_code)]
        struct S {
            hello: String,
            there: String,
        }
        let mut hm = HashMap::new();
        hm.insert("hello".to_string(), "yeet".to_string());
        hm.insert("there".to_string(), "yote".to_string());
        let x = S::from_matches(&hm).expect("should create from hash map.");
        assert_eq!(x.hello, "yeet".to_string());
        assert_eq!(x.there, "yote".to_string());
    }
}

