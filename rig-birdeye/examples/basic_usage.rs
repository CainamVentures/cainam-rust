use rig_birdeye::{TokenSearchAction, TokenSortBy, SortType};
use rig_core::AgentBuilder;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load API key from environment
    let api_key = env::var("BIRDEYE_API_KEY")
        .expect("BIRDEYE_API_KEY must be set");

    // Create agent with Birdeye plugin
    let agent = AgentBuilder::new()
        .with_secret("BIRDEYE_API_KEY", api_key)
        .build()
        .await?;

    // Search for top SOL tokens by volume
    let action = TokenSearchAction {
        keyword: "SOL".to_string(),
        sort_by: Some(TokenSortBy::Volume24h),
        sort_type: Some(SortType::Desc),
        limit: Some(5),
    };

    println!("\nSearching for top SOL tokens by volume...");
    let tokens = agent.execute(action).await?;

    println!("\nTop 5 SOL tokens by 24h volume:");
    println!("{:<20} {:<10} {:<15} {:<15}", "Name", "Symbol", "Price ($)", "24h Volume ($)");
    println!("{}", "-".repeat(60));

    for token in tokens {
        println!(
            "{:<20} {:<10} {:<15.2} {:<15.2}",
            token.name,
            token.symbol,
            token.price,
            token.volume_24h
        );
    }

    Ok(())
} 