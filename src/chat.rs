use schemars::{schema_for, JsonSchema};
use serde::Deserialize;

use crate::{
    model::Agent,
    openai::{
        recieve_function_call_args, ChatCompletionBody, Function, OpenAIClientError, RequestMessage,
    },
};

#[derive(JsonSchema, Deserialize, Debug, Clone)]
pub struct FunctionArgs {
    #[schemars(description = "What you want to say in 日本語.")]
    pub message: String,
}

pub async fn chat(
    agent: &Agent,
    system_prompt: &str,
) -> Result<std::option::Option<FunctionArgs>, OpenAIClientError> {
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
    messages.append(
        &mut agent
            .events
            .iter()
            .map(|event| RequestMessage {
                role: "user".to_string(),
                content: event.clone(),
            })
            .collect::<Vec<_>>(),
    );
    let body = ChatCompletionBody {
        model: "gpt-4-0613".to_string(),
        messages,
        temperature: 0.7,
        max_tokens: 4000,
        function_call: "auto".to_string(),
        functions: vec![Function {
            name: "chat".to_string(),
            description: "Speaks out against other players.".to_string(),
            parameters: schema_for!(FunctionArgs),
        }],
    };
    recieve_function_call_args::<FunctionArgs>(body).await
}
