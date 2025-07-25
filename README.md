# 🕷️ High-Performance Web Crawler & Search Engine

A blazing-fast, concurrent web crawler built with Rust that respects robots.txt, implements intelligent rate limiting, and can crawl thousands of pages efficiently.

For a DEMO of this project: https://www.youtube.com/watch?v=d9wnkTk6er0

## 🎯 Project Overview

This web crawler is designed to:
- **Crawl websites concurrently** using multiple async workers
- **Respect robots.txt rules** to be a good web citizen
- **Extract and follow links** intelligently with depth control
- **Handle errors gracefully** with automatic retries
- **Implement rate limiting** to avoid overwhelming servers
- **Provide real-time statistics** about crawling progress

## ✨ Key Features

### Core Crawling
- **Concurrent Crawling**: Multiple workers crawl different pages simultaneously
- **Async/Await**: Built on Tokio for maximum performance
- **URL Frontier**: Smart queue management with deduplication
- **Depth Control**: Limit how deep the crawler goes from seed URLs
- **Domain Rate Limiting**: Configurable delays between requests to same domain

### Robots.txt Compliance
- **Automatic Fetching**: Downloads and parses robots.txt for each domain
- **Rule Caching**: Caches robots.txt rules for 1 hour to reduce requests
- **Path Matching**: Correctly interprets Allow/Disallow directives
- **Crawl-Delay**: Respects crawl-delay directives when specified

### Smart Filtering
- **Content Type Detection**: Only crawls HTML pages
- **URL Filtering**: Skips images, videos, PDFs, and other non-HTML content
- **Duplicate Detection**: Never crawls the same URL twice

### Error Handling
- **Graceful Failures**: Continues crawling even if some pages fail
- **Retry Logic**: Configurable retry attempts for failed requests
- **Timeout Protection**: Prevents hanging on slow servers

## 🚀 Installation & Setup

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git

### Clone and Build
```bash
# Clone the repository
git clone https://github.com/yourusername/High-Performance-Web-Crawler-Search-Engine.git
cd High-Performance-Web-Crawler-Search-Engine

# Build in release mode for best performance
cargo build --release
```

## 📖 How to Use

### Basic Usage
```bash
# Crawl a website with default settings (100 pages max, depth 3)
cargo run --bin crawler -- https://example.com

# Or use the release build (faster)
./target/release/crawler https://example.com
```

### Command Line Options
```bash
# See all available options
cargo run --bin crawler -- --help

# Options:
#   URL                  Starting URL to crawl
#   -m, --max-pages      Maximum number of pages to crawl (default: 100)
#   -d, --max-depth      Maximum depth from starting URL (default: 3)
#   -c, --concurrent     Number of concurrent workers (default: 5)
#   --delay              Delay between requests in milliseconds (default: 1000)
#   -v, --verbose        Enable debug logging
```

### Example Commands

#### Small Crawl (Testing)
```bash
# Crawl 10 pages from example.com
cargo run --bin crawler -- https://example.com --max-pages 10
```

#### Medium Crawl (Typical Use)
```bash
# Crawl 500 pages with 10 workers
cargo run --bin crawler -- https://news.ycombinator.com \
  --max-pages 500 \
  --concurrent 10 \
  --max-depth 3
```

#### Large Crawl (Performance Test)
```bash
# Crawl 1000 pages as fast as allowed
cargo run --release --bin crawler -- https://en.wikipedia.org \
  --max-pages 1000 \
  --concurrent 20 \
  --delay 500 \
  --max-depth 4
```

#### Respectful Crawl (Slow but Polite)
```bash
# Crawl slowly with 2-second delays
cargo run --bin crawler -- https://small-blog.com \
  --max-pages 50 \
  --concurrent 2 \
  --delay 2000
```

## 📊 Understanding the Output

### During Crawling
```
🕷️  Web Crawler v0.1.0
====================

📋 Configuration:
  Starting URL: https://example.com/
  Max pages: 100
  Max depth: 3
  Concurrent workers: 5
  Delay: 1000ms

🚀 Starting crawl...

[INFO] Worker 0 started
[INFO] Worker 1 started
[INFO] Fetching robots.txt from https://example.com/robots.txt
[INFO] Worker 0 crawling: https://example.com/ (depth: 0)
[INFO] Crawled: https://example.com/ - Example Domain
```

### Final Statistics
```
✅ Crawl completed!

📈 Final Statistics:
  Total pages crawled: 100      # Successfully downloaded pages
  Failed pages: 2               # Pages that failed (404, timeout, etc.)
  Total links found: 1,523      # Total number of links discovered
  Duration: 95.42s              # Total time taken
  Speed: 1.05 pages/second      # Average crawling speed
```

## 🏗️ Architecture

### Component Overview
```
┌─────────────────┐
│   CLI Binary    │ ← User interacts here
└────────┬────────┘
         │
┌────────▼────────┐
│     Crawler     │ ← Coordinates everything
├─────────────────┤
│ - URL Frontier  │ ← Manages queue of URLs
│ - Workers Pool  │ ← Concurrent crawling tasks
│ - Rate Limiter  │ ← Prevents overwhelming servers
│ - Stats Tracker │ ← Collects metrics
└────────┬────────┘
         │
┌────────▼────────┐
│   Components    │
├─────────────────┤
│ - HTTP Fetcher  │ ← Downloads web pages
│ - HTML Parser   │ ← Extracts links and content
│ - Robots Check  │ ← Validates against robots.txt
└─────────────────┘
```

### How It Works

1. **Initialization**
   - Parse command-line arguments
   - Create crawler with specified configuration
   - Initialize worker pool

2. **URL Frontier Management**
   - Start with seed URL at depth 0
   - Discovered links added at depth + 1
   - Deduplication ensures no URL crawled twice
   - Priority to shallower depths

3. **Worker Process**
   ```
   For each worker:
   1. Get URL from frontier
   2. Check robots.txt rules
   3. Apply rate limiting for domain
   4. Fetch the page
   5. Parse HTML and extract links
   6. Add new URLs to frontier
   7. Update statistics
   8. Repeat until done
   ```

4. **Robots.txt Handling**
   - Fetched once per domain
   - Cached for 1 hour
   - Blocks disallowed paths
   - Respects crawl-delay

## 🎯 Performance Characteristics

### Speed Factors
- **Network Latency**: Primary bottleneck
- **Concurrent Workers**: More workers = faster crawling
- **Rate Limiting**: Necessary politeness slows crawling
- **Page Size**: Large pages take longer
- **Parse Complexity**: More links = more processing

### Typical Performance
- **Small sites**: 2-5 pages/second
- **Large sites**: 1-2 pages/second (with rate limiting)
- **Memory usage**: ~50-100MB for 10,000 URLs in frontier
- **CPU usage**: Minimal (mostly waiting for network)

### Optimization Tips
1. **Increase workers** for faster crawling (be respectful!)
2. **Reduce delay** if site can handle it
3. **Use release build** for 2-3x performance boost
4. **Adjust max depth** to avoid crawling too deep

## 🔧 Configuration Details

### Default Settings
- **User Agent**: "RustCrawler/0.1.0"
- **Timeout**: 30 seconds per request
- **Max Page Size**: 10MB
- **Default Delay**: 1000ms between requests to same domain
- **Retries**: 3 attempts for failed requests

### Respecting Websites
The crawler is designed to be respectful:
- Always checks robots.txt
- Implements rate limiting
- Identifies itself clearly
- Handles errors gracefully
- Doesn't crawl non-HTML content

## 🐛 Troubleshooting

### Common Issues

**"Too many open files" error**
- Reduce concurrent workers
- Increase system ulimit: `ulimit -n 4096`

**Slow crawling speed**
- Check if robots.txt has crawl-delay
- Increase concurrent workers
- Reduce delay between requests

**High failure rate**
- Check your internet connection
- Some sites block crawlers
- Increase timeout for slow sites

**Memory usage growing**
- Normal for large crawls
- Frontier stores all discovered URLs
- Consider reducing max pages

## 🎬 Demo Script

For your demonstration video, here's a suggested flow:

### 1. Show Project Structure
```bash
# Show the Rust project
ls -la src/crawler/

# Show the main components
cat src/crawler/mod.rs
```

### 2. Simple Crawl Demo
```bash
# Start with a simple example
cargo run --bin crawler -- https://example.com --max-pages 10
```

### 3. Show Robots.txt Compliance
```bash
# Crawl a site with robots.txt rules
cargo run --bin crawler -- https://github.com --max-pages 20 --verbose
# Point out the robots.txt fetching and blocked URLs
```

### 4. Performance Demo
```bash
# Show high-performance crawling
cargo run --release --bin crawler -- https://en.wikipedia.org \
  --max-pages 100 \
  --concurrent 10 \
  --delay 500
```

### 5. Compare Different Settings
```bash
# Slow, respectful crawl
cargo run --bin crawler -- https://news.ycombinator.com \
  --max-pages 50 \
  --concurrent 2 \
  --delay 2000

# Fast crawl (if appropriate)
cargo run --bin crawler -- https://example.com \
  --max-pages 50 \
  --concurrent 20 \
  --delay 100
```

## 🚧 Current Limitations

- **No JavaScript rendering** (can't crawl SPAs)
- **No data persistence** (doesn't save crawled content yet)
- **Basic retry logic** (could be smarter)
- **No distributed crawling** (single machine only)

## 🔮 Future Enhancements

### Coming Soon
- **Data Storage**: Save crawled content for searching
- **Search Engine**: Full-text search of crawled content
- **Web Dashboard**: Real-time monitoring interface
- **REST API**: Control crawler programmatically

### Planned Features
- **Sitemap.xml support**: Discover URLs more efficiently
- **Headless browser**: Crawl JavaScript-heavy sites
- **Distributed mode**: Crawl across multiple machines
- **Custom extractors**: Extract specific data while crawling

## 📝 License

MIT License 

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

Built with ❤️ and Rust for high-performance web crawling

Happy coding 



