use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use crate::{model::Agent, openai::{RequestMessage, ChatCompletionBody, Function, recieve_function_call_args, OpenAIClientError}};

fn positivity_examples() -> Vec<String> {
    vec![
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "4".to_string(),
        "5".to_string(),
    ]
}

#[derive(JsonSchema,Deserialize, Debug,Clone,Serialize)]
pub struct FunctionArgs {
    #[schemars(
        description = "Aggressiveness of whether to speak up or not (the higher the higher, the more aggressive). Maximize if you receive a question or reference to yourself, minimize if you want to wait for someone else to speak.",
        example = "positivity_examples"
    )]
    pub positivity: usize,
    #[schemars(description = "What you think in 日本語. Your perceived situation and how the conversation will unfold.")]
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
                description: "Output what you are thinking.".to_string(),
                parameters: schema_for!(FunctionArgs),
            }
        ]
    };
    recieve_function_call_args::<FunctionArgs>(body).await
}