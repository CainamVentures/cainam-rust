use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tracing_subscriber::EnvFilter;
use crate::agent::TradingAgent;
use rig::providers::openai::Client as OpenAIClient;
use std::io::{self, Write};
use tokio;

mod agent;
mod trading;
mod twitter;
mod vector_store;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Get environment variables
    let openai_key = env::var("OPENAI_API_KEY")?;
    let twitter_email = env::var("TWITTER_EMAIL")?;
    let twitter_username = env::var("TWITTER_USERNAME")?;
    let twitter_password = env::var("TWITTER_PASSWORD")?;

    // Initialize OpenAI client and embedding model
    let openai_client = OpenAIClient::new(&openai_key);
    let embedding_model = openai_client.embedding_model("text-embedding-3-small");

    // Create trading agent
    let agent = TradingAgent::new(
        &openai_key,
        twitter_email,
        twitter_username,
        twitter_password,
        embedding_model,
    ).await?;

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
            "analyze" if parts.len() == 2 => {
                let symbol = parts[1];
                println!("Analyzing market for {}", symbol);
                if let Err(e) = agent.analyze_market(symbol).await {
                    eprintln!("Error analyzing market: {}", e);
                }
            }
            "trade" if parts.len() == 4 => {
                let symbol = parts[1];
                let action = parts[2];
                let amount: f64 = match parts[3].parse() {
                    Ok(val) => val,
                    Err(_) => {
                        eprintln!("Invalid amount");
                        continue;
                    }
                };

                println!("Executing {} trade for {} {}", action, amount, symbol);
                match agent.execute_trade(symbol, action, amount).await {
                    Ok(true) => {
                        println!("Trade executed successfully");
                        if let Err(e) = agent.post_trade_update(symbol, action, amount).await {
                            eprintln!("Error posting trade update: {}", e);
                        }
                    }
                    Ok(false) => println!("Trade rejected"),
                    Err(e) => eprintln!("Error executing trade: {}", e),
                }
            }
            "exit" => break,
            _ => {
                println!("Invalid command. Available commands:");
                println!("  analyze <symbol>");
                println!("  trade <symbol> <buy|sell> <amount>");
                println!("  exit");
            }
        }
    }

    Ok(())
} 