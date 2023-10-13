use schemars::{JsonSchema, schema_for};
use serde::Deserialize;

use crate::{model::Agent, openai::{RequestMessage, ChatCompletionBody, Function, recieve_function_call_args, OpenAIClientError}};

#[derive(JsonSchema,Deserialize, Debug,Clone)]
pub struct FunctionArgs {
    pub positivity: usize,
    pub thinking: String,
}

pub async fn thinking(
    agent: &Agent,
    system_promot: &str,
) -> Result<std::option::Option<FunctionArgs>, OpenAIClientError> {
    let mut messages = vec![
        RequestMessage {
            role: "system".to_string(),
            content: agent.prompt.clone(),
        },
        RequestMessage {
            role: "system".to_string(),
            content: system_promot.to_string(),
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
                name: "thinking".to_string(),
                description: "thinkingは何を考えているか、positivity(1~5)は発言する際の積極性（高いほど積極的）を意味します。positivityは自分に対して質問等が来た場合は最大にしてください。".to_string(),
                parameters: schema_for!(FunctionArgs),
            }
        ]
    };
    recieve_function_call_args::<FunctionArgs>(body).await
}