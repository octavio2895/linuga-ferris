#[derive(serde::Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn new(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(serde::Serialize)]
struct ApiRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: &'a [Message],
}

#[derive(serde::Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(serde::Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
}

pub async fn call_api(
    client: &reqwest::Client,
    api_key: &str,
    history: &[Message],
    system: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let request_body = ApiRequest {
        model: "claude-haiku-4-5-20251001",
        max_tokens: 1024,
        system,
        messages: history,
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request_body) // serializes your struct automatically
        .send()
        .await?
        .json::<ApiResponse>() // deserializes response into ApiResponse
        .await?;

    Ok(response
        .content
        .into_iter()
        .next()
        .ok_or("empty response")?
        .text)
}
