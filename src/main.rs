use std::env;

use clipboard::{ClipboardContext, ClipboardProvider};
use dotenv;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIChoice {
    text: String,
    index: i32,
    logprobs: Option<Logprobs>,
}

#[derive(Serialize, Deserialize)]
struct Logprobs {
    tokens: Vec<String>,
    token_logprobs: Vec<f32>,
}

async fn send_request(english: &String, api_key: &String) -> String {
    let prompt = format!("Convert the following natural language into a one-line zsh command:\n{}\n", english);

    let url = "https://api.openai.com/v1/completions";
    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap());

    let body = serde_json::json!({
        "model": "text-davinci-003",
        "prompt": prompt,
        "temperature": 0.0,
        "max_tokens": 100,
        "top_p": 1,
        "frequency_penalty": 0.2,
        "presence_penalty": 0.0
    });

    let res = client.post(url)
        .headers(headers)
        .body(body.to_string())
        .send()
        .await
        .unwrap();

    let res_json = res
        .json::<OpenAIResponse>()
        .await
        .unwrap();

    let choice = &res_json.choices[0];
    let command = choice.text.trim();

    command.to_string()
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let mut env_path = home::home_dir().unwrap();
    env_path.push(".zaa");
    dotenv::from_path(env_path.as_path())
        .expect("Failed to load ~/.zaa file. Create a .zaa file with OPENAI_API_KEY=<your key> in your home directory.");
    let api_key = dotenv::var("OPENAI_API_KEY").unwrap();

    let prompt = args[1..].join(" ");
    let response = send_request(&prompt, &api_key).await;
    println!("Command copied to clipboard: {}", response);

    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(response.to_owned()).unwrap();
}
