use anyhow::Result;
use reqwest::Client;
use serde_json::json;

pub struct TelegramClient {
    client: Client,
    token: Option<String>,
    chat_id: Option<String>,
}

impl TelegramClient {
    pub fn new(token: Option<String>, chat_id: Option<String>) -> Self {
        Self {
            client: Client::new(),
            token,
            chat_id,
        }
    }
    
    pub fn set_chat_id(&mut self, chat_id: String) {
        self.chat_id = Some(chat_id);
    }
    
    pub async fn send_message(&self, text: &str) -> Result<()> {
        let token = match &self.token {
            Some(t) => t,
            None => return Ok(()), // Silently skip if no token
        };
        
        let chat_id = match &self.chat_id {
            Some(id) => id.clone(),
            None => return Ok(()), // Silently skip if no chat ID
        };
        
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        
        let body = json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "HTML",
        });
        
        let response = self.client.post(&url).json(&body).send().await?;
        
        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Telegram API error: {}", error));
        }
        
        Ok(())
    }
    
    pub async fn get_updates(&self) -> Result<Vec<serde_json::Value>> {
        let token = self.token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No token configured"))?;
        
        let url = format!("https://api.telegram.org/bot{}/getUpdates", token);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Telegram API error: {}", error));
        }
        
        let json: serde_json::Value = response.json().await?;
        let updates = json["result"].as_array()
            .cloned()
            .unwrap_or_default();
        
        Ok(updates)
    }
    
    pub async fn get_chat_id_from_updates(&self) -> Result<Option<String>> {
        let updates = self.get_updates().await?;
        
        for update in updates.iter().rev() {
            if let Some(message) = update.get("message") {
                if let Some(chat) = message.get("chat") {
                    if let Some(id) = chat.get("id") {
                        return Ok(Some(id.to_string()));
                    }
                }
            }
        }
        
        Ok(None)
    }
}
