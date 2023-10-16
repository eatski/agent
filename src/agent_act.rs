use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    model::{Agent, Event},
    openai::{
        recieve_function_call_args, ChatCompletionBody, Function, OpenAIClientError, RequestMessage, FunctionCallName, FunctionCall,
    },
};

fn aggressiveness_examples() -> Vec<String> {
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
        description = "Aggressiveness of whether to speak up or not (the higher the higher, the more aggressive). Maximize If you are being asked a question, being addressed directly, or have the responsibility to speak. Minimize if you want to wait for someone else to speak.",
        example = "aggressiveness_examples"
    )]
    pub aggressiveness: usize,
    #[schemars(
        description = "What you think in 日本語. Your perceived situation and how the conversation will unfold."
    )]
    pub thinking: String,
}

#[derive(JsonSchema, Deserialize, Serialize,Debug, Clone)]
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
    common_prompts: &Vec<String>,
) -> Result<std::option::Option<F>, OpenAIClientError> {
    let mut messages = common_prompts
        .iter()
        .map(|prompt| RequestMessage::System {
            content: prompt.to_string(),
        })
        .collect::<Vec<_>>();
    messages.push(RequestMessage::System {
        content: format!("あなたの名前は「{}」です。", agent.name)
    });
    messages.push(RequestMessage::System {
        content: agent.prompt.clone(),
    });
    messages.append(
        &mut agent
            .events
            .iter()
            .map(|event| match event {
                Event::Reaction { thinking, aggressiveness} => RequestMessage::Function {
                    name: ReactionFunctionArgs::get_name(),
                    function_call: FunctionCall {
                        name: ReactionFunctionArgs::get_name(),
                        arguments: serde_json::to_string(&ReactionFunctionArgs {
                            aggressiveness: aggressiveness.clone(),
                            thinking: thinking.clone(),
                        })
                        .unwrap(),
                    },
                    content: serde_json::Value::Null
                },
                Event::Speak { message } => RequestMessage::Function {
                    name: ChatFunctionArgs::get_name(),
                    function_call: FunctionCall {
                        name: ChatFunctionArgs::get_name(),
                        arguments: serde_json::to_string(&ChatFunctionArgs {
                            message: message.clone(),
                        })
                        .unwrap(),
                    },
                    content: serde_json::Value::Null
                },
                Event::ListenOtherSpeak {
                    player_name,
                    message,
                } => RequestMessage::System {
                    content: format!("{}: {}", player_name, message),
                },
                
            })
            .collect::<Vec<_>>(),
    );
    let body = ChatCompletionBody {
        model: "gpt-4-0613".to_string(),
        messages,
        temperature: 0.7,
        max_tokens: 500,
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
