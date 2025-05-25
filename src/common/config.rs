use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub crawler: CrawlerConfig,
    pub storage: StorageConfig,
    pub search: SearchConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CrawlerConfig {
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
    
    /// Maximum crawl depth
    pub max_depth: usize,
    
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    
    /// User agent string
    pub user_agent: String,
    
    /// Default delay between requests to the same domain (milliseconds)
    pub default_delay_ms: u64,
    
    /// Maximum retries for failed requests
    pub max_retries: u32,
    
    /// Maximum page size in bytes
    pub max_page_size: usize,
    
    /// Number of worker threads
    pub num_workers: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    /// Storage directory path
    pub storage_path: String,
    
    /// Index directory path
    pub index_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchConfig {
    /// Maximum results per query
    pub max_results: usize,
    
    /// Default result limit
    pub default_limit: usize,
    
    /// Enable snippet generation
    pub enable_snippets: bool,
    
    /// Snippet length
    pub snippet_length: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiConfig {
    /// API server host
    pub host: String,
    
    /// API server port
    pub port: u16,
    
    /// Enable CORS
    pub enable_cors: bool,
    
    /// API rate limit (requests per minute)
    pub rate_limit: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            crawler: CrawlerConfig {
                max_concurrent_requests: 100,
                max_depth: 5,
                timeout_seconds: 30,
                user_agent: "RustCrawler/0.1.0".to_string(),
                default_delay_ms: 1000,
                max_retries: 3,
                max_page_size: 10 * 1024 * 1024, // 10MB
                num_workers: 8,
            },
            storage: StorageConfig {
                storage_path: "./data/storage".to_string(),
                index_path: "./data/index".to_string(),
            },
            search: SearchConfig {
                max_results: 1000,
                default_limit: 10,
                enable_snippets: true,
                snippet_length: 200,
            },
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                enable_cors: true,
                rate_limit: 100,
            },
        }
    }
}

impl Config {
    /// Load configuration from files and environment
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // For now, just return default config
        // We'll implement file loading later
        Ok(Config::default())
    }
}