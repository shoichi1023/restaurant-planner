use std::env;

use anyhow::anyhow;
use google_cloud_auth::project::{create_token_source, Config};
use once_cell::sync::OnceCell;
use reqwest::{Body, Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Debug, Clone)]
pub enum Role {
    USER,
    MODEL,
}

#[derive(Serialize, Debug, Clone)]
pub struct ChatContent {
    pub role: Role,
    pub parts: Vec<Value>,
}

pub async fn get_gcp_token() -> anyhow::Result<String> {
    let gcp_token_config = Config::default();
    let gcp_token_source = create_token_source(gcp_token_config).await?;
    let token = gcp_token_source.token().await?;
    Ok(token.access_token)
}

static DEFAULT_CLIENT: OnceCell<Client> = OnceCell::new();
static LLM_ENDPOINT: OnceCell<String> = OnceCell::new();

pub async fn request_llm(
    contents: Vec<ChatContent>,
    functions: Vec<Value>,
) -> anyhow::Result<ChatContent> {
    let token = get_gcp_token().await?;
    let client = DEFAULT_CLIENT
        .get_or_init(|| {
            ClientBuilder::new()
                .build()
                .map_err(|e| anyhow!("can't create reqwest::Client. err: {}", e))
                .unwrap()
        })
        .clone();
    let endpoint = LLM_ENDPOINT.get_or_init(|| {
        let project_id = env::var("PROJECT_ID").unwrap_or_else(|_| panic!("PROJECT_ID must be set!"));
        let region = env::var("REGION").unwrap_or_else(|_| panic!("REGION must be set!"));
        format!("https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/gemini-pro:generateContent", region, project_id, region)
    });
    let mut body_json = json!({
        "contents": contents
    });
    if !functions.is_empty() {
        body_json["tools"] = json!([{"functionDeclarations": functions}]);
    }
    let res = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::from(body_json.to_string()))
        .send()
        .await?;
    match res.status() {
        reqwest::StatusCode::OK => {
            let llm_res: Value = res.json().await?;
            #[cfg(feature = "debug")]
            println!("llm_res: {:?}", llm_res);
            Ok(ChatContent {
                role: Role::MODEL,
                parts: llm_res["candidates"][0]["content"]["parts"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .clone(),
            })
        }
        _ => Err(anyhow!("request failed. because: {:?}", res.text().await?)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_gcp_token() {
        let token = get_gcp_token().await.unwrap();
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_request_llm() {
        dotenv::dotenv().ok();
        let content = ChatContent {
            role: Role::USER,
            parts: vec![json!({
                "text": "Hello".to_string(),
            })],
        };
        let res = request_llm(vec![content], vec![]).await.unwrap();
        println!("res: {:?}", res);
    }

    #[test]
    fn test_decode() {
        let content = ChatContent {
            role: Role::USER,
            parts: vec![json!({
                "text": "Hello".to_string(),
            })],
        };
        let json = serde_json::to_string(&content).unwrap();

        assert_eq!(json, r#"{"role":"USER","parts":[{"text":"Hello"}]}"#);
    }
}
