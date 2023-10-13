use schemars::{schema_for, JsonSchema};
use serde::Deserialize;

use crate::{model::Agent, openai::{RequestMessage, ChatCompletionBody, Function, recieve_function_call_args, OpenAIClientError}};

#[derive(JsonSchema,Deserialize, Debug,Clone)]
pub struct FunctionArgs {
    pub message: String,
    pub positivity: usize,
    pub thinking: String,
}

pub async fn chat(agent: &Agent, system_prompt: String) -> Result<std::option::Option<FunctionArgs>, OpenAIClientError> {
    let mut messages = vec![
        RequestMessage {
            role: "system".to_string(),
            content: system_prompt,
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
                description: "他プレイヤーに対して発言します。thinkingは何を考えているか、messageは発言の内容、positivity(1~5)は発言する際の積極性（高いほど積極的）を意味します。積極性が低いと発言そのものが無視される可能性があります。".to_string(),
                parameters: schema_for!(FunctionArgs),
            }
        ]
    };
    recieve_function_call_args::<FunctionArgs>(body).await
}