use crate::common::error::{Error, Result};
use crate::crawler::{Fetcher, Parser, UrlFrontier, CrawlTask};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{info, warn, error};
use url::Url;
use std::collections::HashMap;

/// Statistics about the crawl
#[derive(Debug, Clone, Default)]
pub struct CrawlStats {
    pub pages_crawled: usize,
    pub pages_failed: usize,
    pub total_links_found: usize,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
}

impl CrawlStats {
    pub fn duration(&self) -> Option<Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

/// Configuration for the crawler
#[derive(Debug, Clone)]
pub struct CrawlerConfig {
    pub max_pages: usize,
    pub max_depth: usize,
    pub max_concurrent: usize,
    pub delay_ms: u64,
    pub user_agent: String,
    pub timeout_seconds: u64,
    pub max_page_size: usize,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            max_pages: 1000,
            max_depth: 5,
            max_concurrent: 10,
            delay_ms: 1000,
            user_agent: "RustCrawler/0.1.0".to_string(),
            timeout_seconds: 30,
            max_page_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Web crawler that coordinates fetching, parsing, and URL management
pub struct Crawler {
    config: CrawlerConfig,
    frontier: UrlFrontier,
    fetcher: Fetcher,
    parser: Parser,
    stats: Arc<Mutex<CrawlStats>>,
    domain_last_access: Arc<Mutex<HashMap<String, Instant>>>,
}

impl Crawler {
    /// Create a new crawler with the given configuration
    pub fn new(config: CrawlerConfig) -> Self {
        let frontier = UrlFrontier::new(config.max_pages * 2);
        let fetcher = Fetcher::new(
            config.user_agent.clone(),
            config.timeout_seconds,
            config.max_page_size,
        );
        let parser = Parser::new();
        
        Self {
            config,
            frontier,
            fetcher,
            parser,
            stats: Arc::new(Mutex::new(CrawlStats::default())),
            domain_last_access: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Add a seed URL to start crawling from
    pub async fn add_seed(&self, url: Url) -> Result<()> {
        if !Fetcher::should_fetch(&url) {
            return Err(Error::InvalidResponse("Invalid seed URL".to_string()));
        }
        
        self.frontier.add(url, 0).await;
        Ok(())
    }
    
    /// Start crawling
    pub async fn crawl(&self) -> Result<CrawlStats> {
        info!("Starting crawl with max {} pages", self.config.max_pages);
        
        // Set start time
        {
            let mut stats = self.stats.lock().await;
            stats.start_time = Some(Instant::now());
        }
        
        // Create concurrent workers
        let mut handles = vec![];
        for worker_id in 0..self.config.max_concurrent {
            let crawler = self.clone_for_worker();
            let handle = tokio::spawn(async move {
                crawler.worker_loop(worker_id).await;
            });
            handles.push(handle);
        }
        
        // Wait for all workers to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        // Set end time and return stats
        let mut stats = self.stats.lock().await;
        stats.end_time = Some(Instant::now());
        Ok(stats.clone())
    }
    
    /// Clone necessary components for a worker
    fn clone_for_worker(&self) -> Self {
        Self {
            config: self.config.clone(),
            frontier: self.frontier.clone(),
            fetcher: Fetcher::new(
                self.config.user_agent.clone(),
                self.config.timeout_seconds,
                self.config.max_page_size,
            ),
            parser: Parser::new(),
            stats: self.stats.clone(),
            domain_last_access: self.domain_last_access.clone(),
        }
    }
    
    /// Worker loop that processes URLs
    async fn worker_loop(&self, worker_id: usize) {
        info!("Worker {} started", worker_id);
        
        loop {
            // Check if we've reached the page limit
            {
                let stats = self.stats.lock().await;
                if stats.pages_crawled >= self.config.max_pages {
                    info!("Worker {} stopping - page limit reached", worker_id);
                    break;
                }
            }
            
            // Get next URL to crawl
            let task = match self.frontier.pop().await {
                Some(task) => task,
                None => {
                    // No more URLs, wait a bit and check again
                    sleep(Duration::from_millis(100)).await;
                    
                    // Check if frontier is still empty
                    if self.frontier.is_empty().await {
                        info!("Worker {} stopping - no more URLs", worker_id);
                        break;
                    }
                    continue;
                }
            };
            
            // Check depth limit
            if task.depth > self.config.max_depth {
                continue;
            }
            
            // Apply rate limiting
            if let Err(e) = self.apply_rate_limit(&task.url).await {
                warn!("Rate limit error: {}", e);
                continue;
            }
            
            // Process the URL
            info!("Worker {} crawling: {} (depth: {})", worker_id, task.url, task.depth);
            if let Err(e) = self.process_url(task).await {
                error!("Error processing URL: {}", e);
            }
        }
        
        info!("Worker {} finished", worker_id);
    }
    
    /// Apply rate limiting for a domain
    async fn apply_rate_limit(&self, url: &Url) -> Result<()> {
        let domain = url.domain()
            .ok_or_else(|| Error::InvalidResponse("No domain in URL".to_string()))?;
        
        let mut last_access = self.domain_last_access.lock().await;
        
        if let Some(last_time) = last_access.get(domain) {
            let elapsed = last_time.elapsed();
            let required_delay = Duration::from_millis(self.config.delay_ms);
            
            if elapsed < required_delay {
                let wait_time = required_delay - elapsed;
                sleep(wait_time).await;
            }
        }
        
        last_access.insert(domain.to_string(), Instant::now());
        Ok(())
    }
    
    /// Process a single URL
    async fn process_url(&self, task: CrawlTask) -> Result<()> {
        // Fetch the page
        let response = match self.fetcher.fetch(&task.url) {
            Ok(resp) => resp,
            Err(e) => {
                self.update_stats_failed().await;
                return Err(e);
            }
        };
        
        // Parse the page
        let parsed = self.parser.parse(&response.body, &response.url)?;
        
        // Extract and filter links
        let filtered_links = self.parser.filter_links(parsed.links);
        
        // Add new links to frontier
        let new_depth = task.depth + 1;
        let new_links: Vec<(Url, usize)> = filtered_links
            .into_iter()
            .map(|url| (url, new_depth))
            .collect();
        
        let links_count = new_links.len();
        self.frontier.add_many(new_links).await;
        
        // Update statistics
        self.update_stats_success(links_count).await;
        
        // Log progress
        if let Some(title) = parsed.title {
            info!("Crawled: {} - {}", task.url, title);
        } else {
            info!("Crawled: {}", task.url);
        }
        
        Ok(())
    }
    
    /// Update statistics for successful crawl
    async fn update_stats_success(&self, links_found: usize) {
        let mut stats = self.stats.lock().await;
        stats.pages_crawled += 1;
        stats.total_links_found += links_found;
    }
    
    /// Update statistics for failed crawl
    async fn update_stats_failed(&self) {
        let mut stats = self.stats.lock().await;
        stats.pages_failed += 1;
    }
    
    /// Get current statistics
    pub async fn get_stats(&self) -> CrawlStats {
        self.stats.lock().await.clone()
    }
}

/// Builder for creating a crawler with custom configuration
pub struct CrawlerBuilder {
    config: CrawlerConfig,
}

impl CrawlerBuilder {
    pub fn new() -> Self {
        Self {
            config: CrawlerConfig::default(),
        }
    }
    
    pub fn max_pages(mut self, max: usize) -> Self {
        self.config.max_pages = max;
        self
    }
    
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = depth;
        self
    }
    
    pub fn max_concurrent(mut self, concurrent: usize) -> Self {
        self.config.max_concurrent = concurrent;
        self
    }
    
    pub fn delay_ms(mut self, delay: u64) -> Self {
        self.config.delay_ms = delay;
        self
    }
    
    pub fn user_agent(mut self, agent: String) -> Self {
        self.config.user_agent = agent;
        self
    }
    
    pub fn build(self) -> Crawler {
        Crawler::new(self.config)
    }
}

impl Default for CrawlerBuilder {
    fn default() -> Self {
        Self::new()
    }
}