use rig_birdeye::{
    TokenSearchAction, WalletSearchAction,
    TokenSortBy, SortType, BirdeyeError
};
use rig_core::{Agent, AgentBuilder};
use std::env;

fn setup() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    // Initialize logging
    tracing_subscriber::fmt::init();
}

async fn setup_agent() -> Agent {
    // Load API key from environment
    let api_key = env::var("BIRDEYE_API_KEY")
        .expect("BIRDEYE_API_KEY must be set");

    // Create agent with Birdeye plugin
    AgentBuilder::new()
        .with_secret("BIRDEYE_API_KEY", api_key)
        .build()
        .await
        .expect("Failed to create agent")
}

#[tokio::test]
async fn test_token_search() {
    setup();
    let agent = setup_agent().await;

    let action = TokenSearchAction {
        keyword: "SOL".to_string(),
        sort_by: Some(TokenSortBy::Volume24h),
        sort_type: Some(SortType::Desc),
        limit: Some(10),
    };

    let result = agent.execute(action).await;
    assert!(result.is_ok(), "Token search failed: {:?}", result.err());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty(), "No tokens found");
    
    // Validate first token
    let token = &tokens[0];
    assert!(!token.address.is_empty(), "Token address is empty");
    assert!(!token.symbol.is_empty(), "Token symbol is empty");
    assert!(token.price > 0.0, "Token price should be positive");
}

#[tokio::test]
async fn test_wallet_search() {
    setup();
    let agent = setup_agent().await;

    // Use a known Solana wallet address for testing
    let action = WalletSearchAction {
        wallet: "DYw8jCTfwHNRJhhmFcbXvVDTqWMEVFBX6ZKUmG5CNSKK".to_string(),
    };

    let result = agent.execute(action).await;
    assert!(result.is_ok(), "Wallet search failed: {:?}", result.err());

    let portfolio = result.unwrap();
    assert_eq!(portfolio.wallet, action.wallet, "Wallet address mismatch");
    assert!(portfolio.total_usd >= 0.0, "Total USD should be non-negative");
}

#[tokio::test]
async fn test_invalid_api_key() {
    setup();
    let agent = AgentBuilder::new()
        .with_secret("BIRDEYE_API_KEY", "invalid_key")
        .build()
        .await
        .expect("Failed to create agent");

    let action = TokenSearchAction {
        keyword: "SOL".to_string(),
        sort_by: None,
        sort_type: None,
        limit: None,
    };

    let result = agent.execute(action).await;
    assert!(result.is_err(), "Expected error with invalid API key");
    
    match result.unwrap_err() {
        BirdeyeError::InvalidApiKey => (),
        err => panic!("Expected InvalidApiKey error, got: {:?}", err),
    }
} 