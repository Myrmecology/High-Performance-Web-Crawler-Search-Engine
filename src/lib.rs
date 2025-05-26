//! High-Performance Web Crawler & Search Engine
//!
//! A blazing-fast web crawler and search engine built with Rust,
//! featuring concurrent crawling, full-text search, and distributed capabilities.

pub mod api;
pub mod common;
pub mod crawler;
pub mod indexer;
pub mod search;
pub mod storage;

pub use common::{config::Config, error::Result};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::common::error::{Error, Result};
    pub use crate::common::config::Config;
    pub use crate::crawler::{Crawler, CrawlerBuilder};
}