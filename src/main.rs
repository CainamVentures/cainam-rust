use anyhow::Result;
use dotenv;
use std::io::{self, Write};
use tokio;
use crate::agent::{AgentConfig, TradingAgent};

mod agent;
mod trading;
mod twitter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    let config = AgentConfig {
        openai_api_key: std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY must be set"),
        birdeye_api_key: std::env::var("BIRDEYE_API_KEY")
            .expect("BIRDEYE_API_KEY must be set"),
        twitter_email: std::env::var("TWITTER_EMAIL")
            .expect("TWITTER_EMAIL must be set"),
        twitter_username: std::env::var("TWITTER_USERNAME")
            .expect("TWITTER_USERNAME must be set"),
        twitter_password: std::env::var("TWITTER_PASSWORD")
            .expect("TWITTER_PASSWORD must be set"),
    };

    // Initialize trading agent
    let agent = TradingAgent::new(config).await?;

    println!("Trading Agent initialized! Available commands:");
    println!("  analyze <symbol>           - Analyze market for a symbol");
    println!("  trade <symbol> <buy|sell> <amount>  - Execute a trade");
    println!("  exit                       - Exit the program");

    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush()?;
        input.clear();
        io::stdin().read_line(&mut input)?;

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "analyze" => {
                if parts.len() != 2 {
                    println!("Usage: analyze <symbol>");
                    continue;
                }
                agent.analyze_market(parts[1]).await?;
            }
            "trade" => {
                if parts.len() != 4 {
                    println!("Usage: trade <symbol> <buy|sell> <amount>");
                    continue;
                }
                let amount = parts[3].parse::<f64>()?;
                let success = agent.execute_trade(parts[1], parts[2], amount).await?;
                if success {
                    agent.post_trade_update(parts[1], parts[2], amount).await?;
                }
            }
            "exit" => break,
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }

    Ok(())
} 