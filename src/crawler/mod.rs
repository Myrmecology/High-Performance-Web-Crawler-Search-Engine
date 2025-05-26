pub mod frontier;
pub mod fetcher;
pub mod parser;
pub mod crawler;

pub use frontier::{UrlFrontier, CrawlTask};
pub use fetcher::{Fetcher, FetchResponse};
pub use parser::{Parser, ParsedPage};
pub use crawler::{Crawler, CrawlerBuilder, CrawlStats};