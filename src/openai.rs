use schemars::schema::RootSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize)]
pub struct RequestMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatCompletionBody {
    pub model: String,
    pub messages: Vec<RequestMessage>,
    pub temperature: f64,
    pub max_tokens: u32,
    pub function_call: String,
    pub functions: Vec<Function>,
}

#[derive(Serialize)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: RootSchema,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MessageResponse {
    pub role: String,
    pub content: Option<String>,
    pub function_call: Option<FunctionCall>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Choice {
    pub index: i32,
    pub message: MessageResponse,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug)]
pub struct OpenAIClientError;

async fn send_request(body: ChatCompletionBody) -> Result<Response, OpenAIClientError> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header(
            "Authorization",
            "Bearer ".to_string() + &std::env::var("OPENAI_API_KEY").unwrap(),
        )
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            println!("Error: {:?}", e);
            OpenAIClientError
        })?;
    if !response.status().is_success() {
        println!("Error: {:?}", response);
        return Err(OpenAIClientError);
    }
    let response = response.json::<Response>().await.map_err(|e| {
        println!("Error: {:?}", e);
        OpenAIClientError
    })?;
    Ok(response)
}

pub async fn recieve_function_call_args<T: DeserializeOwned>(
    body: ChatCompletionBody,
) -> Result<Option<T>, OpenAIClientError> {
    let response = send_request(body).await?;
    let function_call = response.choices[0].message.function_call.clone();
    if let Some(function_call) = function_call {
        let args = serde_json::from_str(&function_call.arguments).map_err(|e| {
            println!("Error: {:?}", e);
            OpenAIClientError
        })?;
        Ok(Some(args))
    } else {
        Ok(None)
    }
}
