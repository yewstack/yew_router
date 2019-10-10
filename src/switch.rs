//! Route based on enums.
use crate::route_info::RouteInfo;
use crate::RouteState;
use std::str::FromStr;

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
    fn switch<T: RouteState>(route: RouteInfo<T>) -> Option<Self>;

    /// If the key isn't available, this will be called.
    /// This allows an implementation to provide a default when matching fails instead of outright failing the parse.
    fn key_not_available() -> Option<Self> {
        None
    }
}


impl<U: Switch> Switch for Option<U> {
    fn switch<T: RouteState>(route: RouteInfo<T>) -> Option<Self> {
        Some(Some(Switch::switch(route)?))
    }

    /// This will cause the derivation of `from_matches` to not fail if the key can't be located
    fn key_not_available() -> Option<Self> {
        Some(None)
    }
}

impl<U, E> Switch for Result<U, E>
    where
        U: FromStr<Err = E>,
{
    fn switch<T: RouteState>(route: RouteInfo<T>) -> Option<Self> {
        Some(U::from_str(&route.route))
    }
}


macro_rules! impl_switch_for_from_str {
    ($($SelfT: ty),*) => {
        $(
        impl Switch for $SelfT {
            fn switch<T>(route: RouteInfo<T>) -> Option<Self> {
                std::str::FromStr::from_str(&route.route).ok()
            }
        }
        )*
    };
}


use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::PathBuf;

// TODO add implementations for Dates - with various formats, UUIDs
impl_switch_for_from_str! {
    String,
    PathBuf,
    bool,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddr,
    SocketAddrV4,
    SocketAddrV6,
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

