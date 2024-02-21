use std::{collections::HashMap, env};

use anyhow::anyhow;
use serde_json::{json, Value};
use serpapi_search_rust::serp_api_search::SerpApiSearch;

pub fn search_location_schema() -> Value {
    json!({
        "name": "search_location",
        "description": "検索内容に応じた飲食店のリストを返します。",
        "parameters": {
            "type": "object",
            "properties": {
                "q": {
                    "type": "string"
                }
            }
        }
    })
}

pub async fn search_location(q: String) -> anyhow::Result<Value> {
    let api_key = env::var("SERP_API_KEY").expect("SERP_API_KEY is not set");

    let mut params = HashMap::<String, String>::new();
    params.insert("engine".to_string(), "google_local".to_string());
    params.insert("GL".to_string(), "jp".to_string());
    params.insert("hl".to_string(), "ja".to_string());
    params.insert("q".to_string(), q);

    let search = SerpApiSearch::google(params, api_key);

    let results = search.json().await.map_err(|e| {
        println!("error: {:?}", e);
        anyhow!("serp error")
    })?;
    let local_results = results["local_results"].clone();
    #[cfg(feature = "debug")]
    println!("--- search result ---");
    #[cfg(feature = "debug")]
    println!(
        " - results: {}",
        local_results.as_array().unwrap_or(&vec![]).len()
    );

    Ok(local_results)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_search_location() {
        dotenv::dotenv().ok();
        let res = search_location("渋谷 中華料理".to_string()).await;
        println!("res: {:?}", res);
    }
}
