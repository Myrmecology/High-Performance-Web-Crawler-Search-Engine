use web_crawler::crawler::{Fetcher, Parser};
use url::Url;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();
    
    println!("Web Crawler v0.1.0");
    println!("==================");
    
    // Create a fetcher and parser
    let fetcher = Fetcher::new(
        "RustCrawler/0.1.0".to_string(),
        30, // timeout seconds
        10 * 1024 * 1024, // 10MB max size
    );
    let parser = Parser::new();
    
    // Test URL
    let test_url = Url::parse("https://example.com")?;
    
    println!("\nFetching {}...", test_url);
    
    match fetcher.fetch(&test_url) {
        Ok(response) => {
            println!("✓ Success!");
            println!("  Status: {}", response.status_code);
            println!("  Content-Type: {:?}", response.content_type);
            println!("  Body length: {} bytes", response.body.len());
            
            // Parse the HTML
            println!("\nParsing HTML...");
            match parser.parse(&response.body, &response.url) {
                Ok(parsed) => {
                    println!("✓ Parsed successfully!");
                    println!("  Title: {:?}", parsed.title);
                    println!("  Links found: {}", parsed.links.len());
                    println!("  Text length: {} chars", parsed.text_content.len());
                    
                    if !parsed.links.is_empty() {
                        println!("\n  First few links:");
                        for (i, link) in parsed.links.iter().take(5).enumerate() {
                            println!("    {}. {}", i + 1, link);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Parse error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Fetch error: {}", e);
        }
    }
    
    Ok(())
}