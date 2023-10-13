use schemars::{schema_for, JsonSchema};
use serde::Deserialize;

use crate::{model::Agent, openai::{RequestMessage, ChatCompletionBody, Function, recieve_function_call_args, OpenAIClientError}};

#[derive(JsonSchema,Deserialize, Debug,Clone)]
pub struct FunctionArgs {
    pub message: String,
}

pub async fn chat(agent: &Agent, system_prompt: &str) -> Result<std::option::Option<FunctionArgs>, OpenAIClientError> {
    let mut messages = vec![
        RequestMessage {
            role: "system".to_string(),
            content: system_prompt.to_string(),
        },
        RequestMessage {
            role: "system".to_string(),
            content: agent.prompt.clone(),
        },
    ];
    messages.append(&mut agent.events.iter().map(|event| RequestMessage {
        role: "user".to_string(),
        content: event.clone(),
    }).collect::<Vec<_>>());
    let body = ChatCompletionBody {
        model: "gpt-4-0613".to_string(),
        messages,
        temperature: 0.7,
        max_tokens: 4000,
        function_call: "auto".to_string(),
        functions: vec![
            Function {
                name: "chat".to_string(),
                description: "他プレイヤーに対して発言します。".to_string(),
                parameters: schema_for!(FunctionArgs),
            }
        ]
    };
    recieve_function_call_args::<FunctionArgs>(body).await
}