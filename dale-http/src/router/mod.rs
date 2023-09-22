mod decorated;
mod params;
mod route;
mod router;
mod routing;

pub use self::{params::*, route::Route, router::Router};
pub type IntoIter<B> = ::router::router::IntoIter<Route<B>>;
pub use ::router::{AsSegments, Segments};
