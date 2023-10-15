use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    model::Agent,
    openai::{
        recieve_function_call_args, ChatCompletionBody, Function, OpenAIClientError, RequestMessage, FunctionCallName,
    },
};

fn positivity_examples() -> Vec<String> {
    vec![
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
        "4".to_string(),
        "5".to_string(),
    ]
}

pub trait FunctionArgs: JsonSchema {
    fn get_name() -> String;
    fn get_description() -> String;
    fn get_function() -> Function {
        Function {
            name: Self::get_name(),
            description: Self::get_description(),
            parameters: schema_for!(Self),
        }
    }
}


impl FunctionArgs for ReactionFunctionArgs {
    fn get_name() -> String {
        "thinking".to_string()
    }
    fn get_description() -> String {
        "Output what you are thinking.".to_string()
    }
}

#[derive(JsonSchema, Deserialize, Debug, Clone, Serialize)]
pub struct ReactionFunctionArgs {
    #[schemars(
        description = "Aggressiveness of whether to speak up or not (the higher the higher, the more aggressive). Maximize if you receive a question or reference to yourself, minimize if you want to wait for someone else to speak.",
        example = "positivity_examples"
    )]
    pub positivity: usize,
    #[schemars(
        description = "What you think in 日本語. Your perceived situation and how the conversation will unfold."
    )]
    pub thinking: String,
}


#[derive(JsonSchema, Deserialize, Debug, Clone)]
pub struct ChatFunctionArgs {
    #[schemars(description = "What you want to say in 日本語.")]
    pub message: String,
}

impl FunctionArgs for ChatFunctionArgs {
    fn get_name() -> String {
        "chat".to_string()
    }
    fn get_description() -> String {
        "Speaks out against other players.".to_string()
    }
}

pub async fn agent_act<F: FunctionArgs + JsonSchema + DeserializeOwned>(
    agent: &Agent,
    system_promot: &str,
) -> Result<std::option::Option<F>, OpenAIClientError> {
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
        function_call: FunctionCallName {
            name: F::get_name()
        },
        functions: vec![
            ReactionFunctionArgs::get_function(),
            ChatFunctionArgs::get_function()
        ],
    };
    recieve_function_call_args::<F>(body).await
}
