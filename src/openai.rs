use schemars::schema::RootSchema;
use serde::{Serialize, Deserialize};

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
    pub parameters: RootSchema
}

#[derive(Deserialize, Debug,Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Deserialize, Debug,Clone)]
pub struct MessageResponse {
    pub role: String,
    pub content: Option<String>,
    pub function_call: Option<FunctionCall>,
}

#[derive(Deserialize, Debug,Clone)]
pub struct Choice {
    pub index: i32,
    pub message: MessageResponse,
}

#[derive(Deserialize, Debug,Clone)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}