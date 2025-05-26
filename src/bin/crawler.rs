use clap::Parser as ClapParser;
use web_crawler::prelude::*;
use url::Url;
use tracing::Level;

#[derive(ClapParser, Debug)]
#[clap(author, version, about = "High-performance web crawler")]
struct Args {
    /// Starting URL to crawl
    #[clap(value_parser)]
    url: String,
    
    /// Maximum number of pages to crawl
    #[clap(short, long, default_value = "100")]
    max_pages: usize,
    
    /// Maximum crawl depth
    #[clap(short = 'd', long, default_value = "3")]
    max_depth: usize,
    
    /// Number of concurrent workers
    #[clap(short = 'c', long, default_value = "5")]
    concurrent: usize,
    
    /// Delay between requests to same domain (milliseconds)
    #[clap(long, default_value = "1000")]
    delay: u64,
    
    /// Enable debug logging
    #[clap(short = 'v', long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize tracing
    let level = if args.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .init();
    
    println!("🕷️  Web Crawler v0.1.0");
    println!("====================");
    
    // Parse the starting URL
    let start_url = Url::parse(&args.url)
        .map_err(|e| Error::UrlParseError(e))?;
    
    println!("\n📋 Configuration:");
    println!("  Starting URL: {}", start_url);
    println!("  Max pages: {}", args.max_pages);
    println!("  Max depth: {}", args.max_depth);
    println!("  Concurrent workers: {}", args.concurrent);
    println!("  Delay: {}ms", args.delay);
    
    // Create crawler
    let crawler = CrawlerBuilder::new()
        .max_pages(args.max_pages)
        .max_depth(args.max_depth)
        .max_concurrent(args.concurrent)
        .delay_ms(args.delay)
        .user_agent("RustCrawler/0.1.0 (https://github.com/yourusername/crawler)".to_string())
        .build();
    
    // Add seed URL
    crawler.add_seed(start_url).await?;
    
    println!("\n🚀 Starting crawl...\n");
    
    // Start crawling
    let start_time = std::time::Instant::now();
    
    // Run the crawler
    let result = crawler.crawl().await;
    
    match result {
        Ok(stats) => {
            let duration = start_time.elapsed();
            
            println!("\n✅ Crawl completed!");
            println!("\n📈 Final Statistics:");
            println!("  Total pages crawled: {}", stats.pages_crawled);
            println!("  Failed pages: {}", stats.pages_failed);
            println!("  Total links found: {}", stats.total_links_found);
            println!("  Duration: {:.2?}", duration);
            
            if stats.pages_crawled > 0 {
                let pages_per_second = stats.pages_crawled as f64 / duration.as_secs_f64();
                println!("  Speed: {:.2} pages/second", pages_per_second);
            }
        }
        Err(e) => {
            eprintln!("\n❌ Crawl failed: {}", e);
        }
    }
    
    Ok(())
}