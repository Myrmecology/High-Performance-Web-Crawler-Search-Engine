use crate::common::error::{Error, Result};
use scraper::{Html, Selector};
use url::Url;
use std::collections::HashSet;

/// Extracted data from an HTML page
#[derive(Debug, Clone)]
pub struct ParsedPage {
    pub title: Option<String>,
    pub links: Vec<Url>,
    pub text_content: String,
}

/// HTML Parser for extracting links and content
pub struct Parser {
    link_selector: Selector,
    title_selector: Selector,
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            link_selector: Selector::parse("a[href]").unwrap(),
            title_selector: Selector::parse("title").unwrap(),
        }
    }
    
    /// Parse HTML and extract links and content
    pub fn parse(&self, html: &str, base_url: &Url) -> Result<ParsedPage> {
        let document = Html::parse_document(html);
        
        // Extract title
        let title = document
            .select(&self.title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string());
        
        // Extract all links
        let mut links = Vec::new();
        let mut seen_links = HashSet::new();
        
        for element in document.select(&self.link_selector) {
            if let Some(href) = element.value().attr("href") {
                // Skip empty hrefs and anchors
                if href.is_empty() || href.starts_with('#') {
                    continue;
                }
                
                // Try to resolve the URL
                match self.resolve_url(href, base_url) {
                    Ok(url) => {
                        let url_str = url.as_str();
                        if !seen_links.contains(url_str) {
                            seen_links.insert(url_str.to_string());
                            links.push(url);
                        }
                    }
                    Err(_) => {
                        // Skip invalid URLs
                        continue;
                    }
                }
            }
        }
        
        // Extract text content (for future search functionality)
        let text_content = self.extract_text(&document);
        
        Ok(ParsedPage {
            title,
            links,
            text_content,
        })
    }
    
    /// Resolve a potentially relative URL against a base URL
    fn resolve_url(&self, href: &str, base_url: &Url) -> Result<Url> {
        // First try to parse as absolute URL
        if let Ok(url) = Url::parse(href) {
            return Ok(url);
        }
        
        // Otherwise, join with base URL
        base_url.join(href)
            .map_err(|e| Error::UrlParseError(e))
    }
    
    /// Extract visible text content from the document
    fn extract_text(&self, document: &Html) -> String {
        let mut text = String::new();
        
        // Simple text extraction - just get all text nodes
        for node in document.root_element().descendants() {
            if let Some(text_node) = node.value().as_text() {
                let text_str = text_node.trim();
                if !text_str.is_empty() {
                    text.push_str(text_str);
                    text.push(' ');
                }
            }
        }
        
        text.trim().to_string()
    }
    
    /// Filter links to only include crawlable URLs
    pub fn filter_links(&self, links: Vec<Url>) -> Vec<Url> {
        links.into_iter()
            .filter(|url| {
                // Only HTTP(S) URLs
                matches!(url.scheme(), "http" | "https")
            })
            .filter(|url| {
                // Skip common non-HTML extensions
                if let Some(path) = url.path_segments() {
                    if let Some(last) = path.last() {
                        let skip_extensions = [
                            ".jpg", ".jpeg", ".png", ".gif", ".webp",
                            ".pdf", ".zip", ".mp3", ".mp4", ".css", ".js"
                        ];
                        
                        let lower = last.to_lowercase();
                        !skip_extensions.iter().any(|ext| lower.ends_with(ext))
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .collect()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}