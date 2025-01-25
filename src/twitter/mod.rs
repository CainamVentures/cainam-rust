use anyhow::{Result, anyhow};
use reqwest::Client;
use reqwest::header::SET_COOKIE;

pub struct TwitterClient {
    client: Client,
    email: String,
    username: String,
    password: String,
    auth_token: Option<String>,
}

impl TwitterClient {
    pub fn new(email: String, username: String, password: String) -> Self {
        Self {
            client: Client::new(),
            email,
            username,
            password,
            auth_token: None,
        }
    }

    pub async fn login(&mut self) -> Result<()> {
        // Implement Twitter login
        let params = [
            ("session[username_or_email]", &self.username),
            ("session[password]", &self.password),
        ];

        let response = self.client
            .post("https://api.twitter.com/oauth2/token")
            .form(&params)
            .send()
            .await?;

        tracing::info!("Twitter login response status: {}", response.status());

        if response.status().is_success() {
            if let Some(cookie_header) = response.headers().get(SET_COOKIE) {
                let cookies = cookie_header.to_str()?.split(';').collect::<Vec<&str>>();
                for cookie in cookies {
                    if cookie.trim().starts_with("auth_token=") {
                        self.auth_token = Some(cookie.trim().replace("auth_token=", ""));
                        break;
                    }
                }
            }
            Ok(())
        } else {
            let error_message = response.text().await.unwrap_or_default();
            tracing::error!("Failed to login to Twitter: {}", error_message);
            Err(anyhow!("Failed to login to Twitter: {}", error_message))
        }
    }

    pub async fn post_tweet(&self, text: &str) -> Result<()> {
        // Ensure the user is logged in
        if self.auth_token.is_none() {
            return Err(anyhow!("Not authenticated"));
        }

        // Implement tweet posting
        let params = [
            ("status", text),
        ];

        let response = self.client
            .post("https://api.twitter.com/1.1/statuses/update.json")
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Failed to post tweet"))
        }
    }

    pub async fn delete_tweet(&self, tweet_id: &str) -> Result<()> {
        // TODO: Implement tweet deletion
        tracing::info!("Deleting tweet: {}", tweet_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_twitter_client() -> Result<()> {
        let mut client = TwitterClient::new(
            "test@example.com".to_string(),
            "test_user".to_string(),
            "test_pass".to_string(),
        );

        // Test login
        client.login().await?;

        // Test posting tweet
        client.post_tweet("Test tweet").await?;

        // Test deleting tweet
        client.delete_tweet("123456789").await?;

        Ok(())
    }
}