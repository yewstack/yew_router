//! Route based on enums.
use crate::route_info::RouteInfo;

/// Routing trait for enums
///
/// # Example
/// ```
/// use yew_router::Switch;
/// use yew_router::route_info::RouteInfo;
/// #[derive(Debug, Switch, PartialEq)]
/// enum TestEnum {
///     #[to = "/test/route"]
///     TestRoute,
///     #[to = "/capture/string/{path}"]
///     CaptureString{path: String},
///     #[to = "/capture/number/{num}"]
///     CaptureNumber{num: usize},
///     #[to = "/capture/unnamed/{doot}"]
///     CaptureUnnamed(String),
///     #[to = "{*}/skip/"]
///     Skip
/// }
///
/// assert_eq!(TestEnum::switch(RouteInfo::<()>::from("/test/route")), Some(TestEnum::TestRoute));
/// assert_eq!(TestEnum::switch(RouteInfo::<()>::from("/capture/string/lorem")), Some(TestEnum::CaptureString{path: "lorem".to_string()}));
/// assert_eq!(TestEnum::switch(RouteInfo::<()>::from("/capture/number/22")), Some(TestEnum::CaptureNumber{num: 22}));
/// assert_eq!(TestEnum::switch(RouteInfo::<()>::from("/capture/unnamed/lorem")), Some(TestEnum::CaptureUnnamed("lorem".to_string())));
/// ```
///
pub trait Switch: Sized {
    /// Based on a route, possibly produce an itself.
    fn switch<T>(route: RouteInfo<T>) -> Option<Self>;
}


