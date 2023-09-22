mod decorated;
mod params;
mod route;
mod router;
mod routing;

pub use self::{
    decorated::DecoratedRouter, params::*, route::Route, router::Router, routing::Routing,
};
pub type IntoIter<B> = ::router::router::IntoIter<Route<B>>;
pub use ::router::{AsSegments, Segments};
