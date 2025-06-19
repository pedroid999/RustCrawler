pub mod cli;
pub mod crawler;
pub mod robots;

pub use cli::CliArgs;
pub use crawler::{Crawler, CrawlerConfig, CrawlResult};
pub use robots::{RobotsManager, RobotsInfo};