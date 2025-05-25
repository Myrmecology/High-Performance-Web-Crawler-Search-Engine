//! Basic example of using the web crawler

use web_crawler::prelude::*;
use web_crawler::crawler::CrawlerBuilder;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create a new crawler with custom configuration
    let crawler = CrawlerBuilder::new()
        .max_depth(3)
        .max_pages(100)
        .max_concurrent_requests(10)
        .user_agent("MyBot/1.0")
        .build()?;
    
    // Add seed URLs
    let seed_urls = vec![
        "https://example.com",
        "https://rust-lang.org",
    ];
    
    for url in seed_urls {
        crawler.add_seed(Url::parse(url)?).await?;
    }
    
    // Set up progress callback
    let progress_callback = |stats| {
        println!("Progress: {} pages crawled, {} in queue", 
                 stats.pages_crawled, stats.queue_size);
    };
    
    // Start crawling
    println!("Starting crawl...");
    let results = crawler.crawl_with_callback(progress_callback).await?;
    
    // Print results
    println!("\nCrawl completed!");
    println!("Total pages crawled: {}", results.total_pages);
    println!("Successful: {}", results.successful);
    println!("Failed: {}", results.failed);
    println!("Total time: {:?}", results.duration);
    
    Ok(())
}