//! Route based on enums.
use crate::{route::Route, RouteState};
use std::fmt::Write;

/// Routing trait for enums.
///
/// # Note
/// Don't try to implement this yourself, rely on the derive macro.
///
/// # Example
/// ```
/// use yew_router::{route::Route, Switch};
/// #[derive(Debug, Switch, PartialEq)]
/// enum TestEnum {
///     #[to = "/test/route"]
///     TestRoute,
///     #[to = "/capture/string/{path}"]
///     CaptureString { path: String },
///     #[to = "/capture/number/{num}"]
///     CaptureNumber { num: usize },
///     #[to = "/capture/unnamed/{doot}"]
///     CaptureUnnamed(String),
/// }
///
/// assert_eq!(
///     TestEnum::switch(Route::<()>::from("/test/route")),
///     Some(TestEnum::TestRoute)
/// );
/// assert_eq!(
///     TestEnum::switch(Route::<()>::from("/capture/string/lorem")),
///     Some(TestEnum::CaptureString {
///         path: "lorem".to_string()
///     })
/// );
/// assert_eq!(
///     TestEnum::switch(Route::<()>::from("/capture/number/22")),
///     Some(TestEnum::CaptureNumber { num: 22 })
/// );
/// assert_eq!(
///     TestEnum::switch(Route::<()>::from("/capture/unnamed/lorem")),
///     Some(TestEnum::CaptureUnnamed("lorem".to_string()))
/// );
/// ```
pub trait Switch: Sized {
    /// Based on a route, possibly produce an itself.
    fn switch<T: RouteState>(route: Route<T>) -> Option<Self> {
        Self::from_route_part(route).0
    }

    /// Get self from a part of the state
    fn from_route_part<T: RouteState>(part: Route<T>) -> (Option<Self>, Option<T>);

    /// Build part of a route from itself.
    fn build_route_section<T>(self, route: &mut String) -> Option<T>;

    /// Called when the key (the named capture group) can't be located. Instead of failing outright,
    /// a default item can be provided instead.
    ///
    /// Its primary motivation for existing is to allow implementing Switch for Option.
    /// This doesn't make sense at the moment because this only works for the individual key section
    /// - any surrounding literals are pretty much guaranteed to make the parse step fail.
    /// because of this, this functionality might be removed in favor of using a nested Switch enum,
    /// or multiple variants.
    fn key_not_available() -> Option<Self> {
        None
    }
}

/// Builds a route from a switch.
pub fn build_route_from_switch<T: Switch, U>(switch: T) -> Route<U> {
    let mut buf = String::with_capacity(50); // TODO, play with this to maximize perf/size.

    let state: Option<U> = None;
    let state = state.or(switch.build_route_section(&mut buf));
    Route { route: buf, state }
}

/// Wrapper that requires that an implementor of Switch must start with a `/`.
///
/// This is needed for any non-derived type provided by yew-router to be used by itself.
///
/// This is because route strings will almost always start with `/`, so in order to get a std type
/// with the `rest` attribute, without a specified leading `/`, this wrapper is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LeadingSlash<T>(pub T);
impl<U: Switch> Switch for LeadingSlash<U> {
    fn from_route_part<T: RouteState>(part: Route<T>) -> (Option<Self>, Option<T>) {
        if part.route.starts_with('/') {
            let route = Route {
                route: part.route[1..].to_string(),
                state: part.state,
            };
            let (inner, state) = U::from_route_part(route);
            (inner.map(LeadingSlash), state)
        } else {
            (None, None)
        }
    }

    fn build_route_section<T>(self, route: &mut String) -> Option<T> {
        write!(route, "/").ok()?;
        self.0.build_route_section(route)
    }
}

macro_rules! impl_switch_for_from_to_str {
    ($($SelfT: ty),*) => {
        $(
        impl Switch for $SelfT {
            fn from_route_part<T: RouteState>(part: Route<T>) -> (Option<Self>, Option<T>) {
                (
                    ::std::str::FromStr::from_str(&part.route).ok(),
                    part.state
                )
            }

            fn build_route_section<T>(self, f: &mut String) -> Option<T> {
                write!(f, "{}", self).expect("Writing to string should never fail.");
                None
            }
        }
        )*
    };
}

impl_switch_for_from_to_str! {
    String,
    bool,
    f64,
    f32,
    usize,
    u128,
    u64,
    u32,
    u16,
    u8,
    isize,
    i128,
    i64,
    i32,
    i16,
    i8,
    std::num::NonZeroU128,
    std::num::NonZeroU64,
    std::num::NonZeroU32,
    std::num::NonZeroU16,
    std::num::NonZeroU8,
    std::num::NonZeroI128,
    std::num::NonZeroI64,
    std::num::NonZeroI32,
    std::num::NonZeroI16,
    std::num::NonZeroI8
}

#[test]
fn isize_build_route() {
    let mut route = "/".to_string();
    let mut _state: Option<String> = None;
    _state = _state.or((-432isize).build_route_section(&mut route));
    assert_eq!(route, "/-432".to_string());
}

// TODO add implementations for Dates - with various formats, UUIDs
