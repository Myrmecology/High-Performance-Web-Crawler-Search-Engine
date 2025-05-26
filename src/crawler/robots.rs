use crate::common::error::{Error, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use url::Url;
use std::sync::Arc;
use tracing::{info, warn};

/// Cache entry for robots.txt data
#[derive(Clone, Debug)]
struct RobotsCache {
    rules: RobotsRules,
    fetched_at: Instant,
}

/// Parsed robots.txt rules for a domain
#[derive(Clone, Debug)]
struct RobotsRules {
    disallowed_paths: Vec<String>,
    allowed_paths: Vec<String>,
    crawl_delay: Option<Duration>,
    sitemap: Option<String>,
}

impl Default for RobotsRules {
    fn default() -> Self {
        Self {
            disallowed_paths: Vec::new(),
            allowed_paths: Vec::new(),
            crawl_delay: None,
            sitemap: None,
        }
    }
}

/// Robots.txt checker with caching
#[derive(Clone)]
pub struct RobotsChecker {
    cache: Arc<Mutex<HashMap<String, RobotsCache>>>,
    cache_duration: Duration,
    user_agent: String,
}

impl RobotsChecker {
    /// Create a new robots checker
    pub fn new(user_agent: String) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_duration: Duration::from_secs(3600), // Cache for 1 hour
            user_agent,
        }
    }
    
    /// Check if a URL is allowed to be crawled
    pub async fn is_allowed(&self, url: &Url) -> Result<bool> {
        let domain = url.domain()
            .ok_or_else(|| Error::InvalidResponse("No domain in URL".to_string()))?;
        
        // Get robots.txt rules for this domain
        let rules = self.get_rules(url).await?;
        
        // Check if the path is disallowed
        let path = url.path();
        
        // First check allowed paths (they override disallowed)
        for allowed in &rules.allowed_paths {
            if path.starts_with(allowed) {
                return Ok(true);
            }
        }
        
        // Then check disallowed paths
        for disallowed in &rules.disallowed_paths {
            if path.starts_with(disallowed) {
                info!("Robots.txt disallows crawling: {}", url);
                return Ok(false);
            }
        }
        
        // If no rules match, it's allowed
        Ok(true)
    }
    
    /// Get crawl delay for a domain
    pub async fn get_crawl_delay(&self, url: &Url) -> Result<Option<Duration>> {
        let rules = self.get_rules(url).await?;
        Ok(rules.crawl_delay)
    }
    
    /// Get robots.txt rules for a domain (with caching)
    async fn get_rules(&self, url: &Url) -> Result<RobotsRules> {
        let domain = url.domain()
            .ok_or_else(|| Error::InvalidResponse("No domain in URL".to_string()))?;
        
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(domain) {
                if cached.fetched_at.elapsed() < self.cache_duration {
                    return Ok(cached.rules.clone());
                }
            }
        }
        
        // Fetch and parse robots.txt
        let robots_url = Url::parse(&format!("{}://{}/robots.txt", url.scheme(), domain))
            .map_err(|e| Error::UrlParseError(e))?;
        
        info!("Fetching robots.txt from {}", robots_url);
        
        let rules = match self.fetch_and_parse(&robots_url).await {
            Ok(rules) => rules,
            Err(e) => {
                warn!("Failed to fetch robots.txt for {}: {}. Allowing crawl.", domain, e);
                // If we can't fetch robots.txt, we allow crawling (standard practice)
                RobotsRules::default()
            }
        };
        
        // Cache the rules
        {
            let mut cache = self.cache.lock().await;
            cache.insert(
                domain.to_string(),
                RobotsCache {
                    rules: rules.clone(),
                    fetched_at: Instant::now(),
                },
            );
        }
        
        Ok(rules)
    }
    
    /// Fetch and parse robots.txt
    async fn fetch_and_parse(&self, robots_url: &Url) -> Result<RobotsRules> {
        // Create a new fetcher for this request
        let fetcher = crate::crawler::Fetcher::new(
            self.user_agent.clone(),
            10, // 10 second timeout
            1024 * 1024, // 1MB max
        );
        
        // Use tokio to run the blocking fetch operation
        let url = robots_url.clone();
        let response = tokio::task::spawn_blocking(move || {
            fetcher.fetch(&url)
        }).await
            .map_err(|e| Error::Unknown(format!("Task error: {}", e)))?;
        
        let response = response?;
        
        // Parse the robots.txt content
        self.parse_robots_txt(&response.body)
    }
    
    /// Parse robots.txt content
    fn parse_robots_txt(&self, content: &str) -> Result<RobotsRules> {
        let mut rules = RobotsRules::default();
        let mut current_user_agent = String::new();
        let mut applies_to_us = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Split directive and value
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }
            
            let directive = parts[0].trim().to_lowercase();
            let value = parts[1].trim();
            
            match directive.as_str() {
                "user-agent" => {
                    current_user_agent = value.to_lowercase();
                    applies_to_us = current_user_agent == "*" || 
                                   self.user_agent.to_lowercase().contains(&current_user_agent);
                }
                "disallow" if applies_to_us => {
                    if !value.is_empty() {
                        rules.disallowed_paths.push(value.to_string());
                    }
                }
                "allow" if applies_to_us => {
                    if !value.is_empty() {
                        rules.allowed_paths.push(value.to_string());
                    }
                }
                "crawl-delay" if applies_to_us => {
                    if let Ok(seconds) = value.parse::<u64>() {
                        rules.crawl_delay = Some(Duration::from_secs(seconds));
                    }
                }
                "sitemap" => {
                    rules.sitemap = Some(value.to_string());
                }
                _ => {}
            }
        }
        
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_robots_txt() {
        let checker = RobotsChecker::new("TestBot".to_string());
        let content = r#"
User-agent: *
Disallow: /private/
Disallow: /tmp/
Allow: /private/public.html
Crawl-delay: 1

User-agent: BadBot
Disallow: /

Sitemap: https://example.com/sitemap.xml
"#;
        
        let rules = checker.parse_robots_txt(content).unwrap();
        assert_eq!(rules.disallowed_paths.len(), 2);
        assert_eq!(rules.allowed_paths.len(), 1);
        assert_eq!(rules.crawl_delay, Some(Duration::from_secs(1)));
        assert_eq!(rules.sitemap, Some("https://example.com/sitemap.xml".to_string()));
    }
}