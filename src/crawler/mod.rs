pub mod frontier;
pub mod fetcher;

pub use frontier::{UrlFrontier, CrawlTask};
pub use fetcher::{Fetcher, FetchResponse};