mod params;
mod route;
mod router;

pub use self::{params::*, route::Route, router::Router};
pub type IntoIter<B> = ::router::router::IntoIter<Route<B>>;
pub use ::router::{AsSegments, Segments};
