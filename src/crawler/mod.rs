pub mod frontier;
pub mod fetcher;
pub mod parser;

pub use frontier::{UrlFrontier, CrawlTask};
pub use fetcher::{Fetcher, FetchResponse};
pub use parser::{Parser, ParsedPage};