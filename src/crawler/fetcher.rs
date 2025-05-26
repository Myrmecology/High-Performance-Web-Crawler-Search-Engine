use crate::common::error::{Error, Result};
use std::io::Read;
use std::time::Duration;
use url::Url;

/// Response from fetching a URL
#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub url: Url,
    pub status_code: u16,
    pub content_type: Option<String>,
    pub body: String,
    pub headers: Vec<(String, String)>,
}

/// HTTP Fetcher for downloading web pages
pub struct Fetcher {
    client: ureq::Agent,
    max_size: usize,
}

impl Fetcher {
    /// Create a new fetcher with configuration
    pub fn new(user_agent: String, timeout_seconds: u64, max_size: usize) -> Self {
        let client = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(timeout_seconds))
            .user_agent(&user_agent)
            .build();
        
        Self {
            client,
            max_size,
        }
    }
    
    /// Fetch a URL and return the response
    pub fn fetch(&self, url: &Url) -> Result<FetchResponse> {
        // Only fetch HTTP(S) URLs
        match url.scheme() {
            "http" | "https" => {},
            scheme => return Err(Error::InvalidResponse(
                format!("Unsupported URL scheme: {}", scheme)
            )),
        }
        
        // Make the request
        let response = self.client
            .get(url.as_str())
            .call()
            .map_err(|e| Error::HttpError(e.to_string()))?;
        
        let status_code = response.status();
        
        // Check if successful
        if !(200..300).contains(&status_code) {
            return Err(Error::HttpError(
                format!("HTTP {} for {}", status_code, url)
            ));
        }
        
        // Get content type
        let content_type = response.header("content-type")
            .map(|s| s.to_string());
        
        // Check if HTML
        if let Some(ct) = &content_type {
            if !ct.contains("text/html") && !ct.contains("text/plain") {
                return Err(Error::InvalidResponse(
                    format!("Non-HTML content type: {}", ct)
                ));
            }
        }
        
        // Get headers
        let headers: Vec<(String, String)> = response
            .headers_names()
            .into_iter()
            .filter_map(|name| {
                response.header(&name)
                    .map(|value| (name.to_string(), value.to_string()))
            })
            .collect();
        
        // Read body with size limit
        let mut body = String::new();
        response
            .into_reader()
            .take(self.max_size as u64)
            .read_to_string(&mut body)
            .map_err(|e| Error::HttpError(format!("Failed to read body: {}", e)))?;
        
        Ok(FetchResponse {
            url: url.clone(),
            status_code,
            content_type,
            body,
            headers,
        })
    }
    
    /// Check if a URL should be fetched based on scheme and extension
    pub fn should_fetch(url: &Url) -> bool {
        // Only HTTP(S)
        if !matches!(url.scheme(), "http" | "https") {
            return false;
        }
        
        // Skip common non-HTML extensions
        if let Some(path) = url.path_segments() {
            if let Some(last) = path.last() {
                let skip_extensions = [
                    ".jpg", ".jpeg", ".png", ".gif", ".webp", ".svg",
                    ".pdf", ".doc", ".docx", ".xls", ".xlsx",
                    ".zip", ".rar", ".tar", ".gz",
                    ".mp3", ".mp4", ".avi", ".mov",
                    ".css", ".js", ".json", ".xml",
                ];
                
                for ext in &skip_extensions {
                    if last.to_lowercase().ends_with(ext) {
                        return false;
                    }
                }
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_should_fetch() {
        assert!(Fetcher::should_fetch(&Url::parse("https://example.com").unwrap()));
        assert!(Fetcher::should_fetch(&Url::parse("http://example.com/page.html").unwrap()));
        assert!(!Fetcher::should_fetch(&Url::parse("https://example.com/image.jpg").unwrap()));
        assert!(!Fetcher::should_fetch(&Url::parse("ftp://example.com").unwrap()));
    }
}