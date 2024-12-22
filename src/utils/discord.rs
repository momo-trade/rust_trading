use lazy_static::lazy_static;
use reqwest::Client;
use serde_json::json;
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

lazy_static! {
    static ref NOTIFIER: Arc<Mutex<Option<DiscordNotifier>>> = Arc::new(Mutex::new(None));
}

pub struct DiscordNotifier {
    webhook_url: String,
}

impl DiscordNotifier {
    pub fn new(webhook_url: &str) -> Self {
        Self {
            webhook_url: webhook_url.to_string(),
        }
    }

    pub async fn init_global(webhook_url: &str) {
        let notifier = DiscordNotifier::new(webhook_url);
        let mut global_notifier = NOTIFIER.lock().await;
        *global_notifier = Some(notifier);
    }

    pub async fn send_message(&self, content: &str) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let payload = json!({ "content": content });

        let response = client.post(&self.webhook_url).json(&payload).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "Failed to send message to Discord. Status: {}",
                response.status()
            )
            .into())
        }
    }
}

/// Send a Discord notification
pub async fn notify(content: &str) {
    let global_notifier = NOTIFIER.lock().await;
    if let Some(notifier) = &*global_notifier {
        if let Err(e) = notifier.send_message(content).await {
            eprintln!("Failed to send Discord notification: {}", e);
        }
    } else {
        eprintln!("DiscordNotifier is not initialized.");
    }
}
