//! Weather warnings (nowcast) fetching and parsing.

mod cache;
mod cap;
mod fetch;
mod rss;

pub use cuaca::warnings::Warning;
pub use fetch::fetch_warnings;
