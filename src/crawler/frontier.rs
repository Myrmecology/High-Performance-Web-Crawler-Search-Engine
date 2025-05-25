use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

/// URL Frontier manages the queue of URLs to be crawled
#[derive(Clone)]
pub struct UrlFrontier {
    /// Queue of URLs to crawl
    queue: Arc<Mutex<VecDeque<CrawlTask>>>,
    /// Set of seen URLs to avoid duplicates
    seen: Arc<Mutex<HashSet<String>>>,
    /// Maximum queue size
    max_size: usize,
}

#[derive(Debug, Clone)]
pub struct CrawlTask {
    pub url: Url,
    pub depth: usize,
    pub retry_count: u32,
}

impl UrlFrontier {
    pub fn new(max_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            seen: Arc::new(Mutex::new(HashSet::new())),
            max_size,
        }
    }
    
    /// Add a URL to the frontier
    pub async fn add(&self, url: Url, depth: usize) -> bool {
        let url_str = url.as_str().to_string();
        
        let mut seen = self.seen.lock().await;
        if seen.contains(&url_str) {
            return false;
        }
        
        let mut queue = self.queue.lock().await;
        if queue.len() >= self.max_size {
            return false;
        }
        
        seen.insert(url_str);
        queue.push_back(CrawlTask {
            url,
            depth,
            retry_count: 0,
        });
        
        true
    }
    
    /// Add multiple URLs
    pub async fn add_many(&self, urls: Vec<(Url, usize)>) {
        for (url, depth) in urls {
            self.add(url, depth).await;
        }
    }
    
    /// Get the next URL to crawl
    pub async fn pop(&self) -> Option<CrawlTask> {
        let mut queue = self.queue.lock().await;
        queue.pop_front()
    }
    
    /// Get the current queue size
    pub async fn size(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }
    
    /// Check if the frontier is empty
    pub async fn is_empty(&self) -> bool {
        let queue = self.queue.lock().await;
        queue.is_empty()
    }
    
    /// Check if a URL has been seen
    pub async fn has_seen(&self, url: &Url) -> bool {
        let seen = self.seen.lock().await;
        seen.contains(url.as_str())
    }
    
    /// Re-add a failed task with incremented retry count
    pub async fn retry(&self, mut task: CrawlTask) -> bool {
        task.retry_count += 1;
        let mut queue = self.queue.lock().await;
        if queue.len() < self.max_size {
            queue.push_back(task);
            true
        } else {
            false
        }
    }
    
    /// Get statistics about the frontier
    pub async fn stats(&self) -> FrontierStats {
        let queue = self.queue.lock().await;
        let seen = self.seen.lock().await;
        
        FrontierStats {
            queue_size: queue.len(),
            seen_count: seen.len(),
            max_size: self.max_size,
        }
    }
}

#[derive(Debug)]
pub struct FrontierStats {
    pub queue_size: usize,
    pub seen_count: usize,
    pub max_size: usize,
}