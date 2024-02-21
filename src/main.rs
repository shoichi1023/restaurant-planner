use async_recursion::async_recursion;
use restaurant_planner::{
    gemini_client::{request_llm, ChatContent, Role},
    restaurant_planner::init_restaurant_planner,
    serp_client::{search_location, search_location_schema},
};
use serde_json::{json, Value};
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    println!("\n\n\n===========================================================\n\n");
    chat_event_loop().await?;
    Ok(())
}

async fn chat_event_loop() -> anyhow::Result<()> {
    let mut chat_contents = init_restaurant_planner();
    loop {
        let chat_input = get_chat_input();
        if chat_input.is_empty() {
            continue;
        }
        let chat_content = ChatContent {
            role: Role::USER,
            parts: vec![json!({ "text": chat_input })],
        };
        chat_contents.push(chat_content.clone());
        let res =
            recursive_request_llm(chat_contents.clone(), vec![search_location_schema()]).await?;
        chat_contents.push(res.clone());
        print_ai_message(
            &res.parts[0]
                .get("text")
                .unwrap_or(&Value::Null)
                .as_str()
                .unwrap_or("")
                .to_string(),
        );
    }
}

#[async_recursion]
async fn recursive_request_llm(
    mut chat_contents: Vec<ChatContent>,
    functions: Vec<Value>,
) -> anyhow::Result<ChatContent> {
    let res = request_llm(chat_contents.clone(), functions.clone()).await?;
    if res.parts[0].get("text").is_some() {
        Ok(res)
    } else {
        chat_contents.push(res.clone());
        let part = res.parts[0]["functionCall"].clone();
        let function_name = part["name"].as_str().unwrap_or("").to_string();
        let result: Value = match &*function_name {
            "search_location" => {
                search_location(part["args"]["q"].as_str().unwrap_or("").to_string() + " 周辺")
                    .await?
            }
            _ => Value::Null,
        };
        chat_contents.push(ChatContent {
            role: Role::MODEL,
            parts: vec![json!({
                "functionResponse": {
                    "name": function_name,
                    "response": {
                      "name": function_name,
                      "content": result
                    }
                  }
            })],
        });
        recursive_request_llm(chat_contents, functions.clone()).await
    }
}

fn print_ai_message(message: &str) {
    println!("[ AI ] \n{}", message);
}

fn get_chat_input() -> String {
    let mut s = String::new();
    println!("[ You ]");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}
