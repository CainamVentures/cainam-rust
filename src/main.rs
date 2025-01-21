use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tracing_subscriber::EnvFilter;
use crate::agent::TradingAgent;
use rig::providers::openai::{Client as OpenAIClient, EmbeddingModel};
use std::io::{self, Write};
use tokio;
use crate::agent::AgentConfig;

mod agent;
mod trading;
mod twitter;
mod vector_store;

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

    // Initialize OpenAI client and embedding model
    let openai_client = OpenAIClient::new(&config.openai_api_key);
    let embedding_model = openai_client.embedding_model("text-embedding-3-small");

    // Initialize trading agent with OpenAI embedding model
    let agent = agent::TradingAgent::<EmbeddingModel>::new(config, embedding_model).await?;

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