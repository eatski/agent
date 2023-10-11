use dialoguer::Confirm;
use openai::{ChatCompletionBody, RequestMessage, Function};
use rand::seq::SliceRandom;
use serde::Deserialize;
use tokio::task;
use schemars::{schema_for, JsonSchema};

use crate::openai::recieve_function_call_args;

mod openai;

#[derive(Debug,Clone)]
struct Agent {
    name: String,
    prompt: String,
    events: Vec<String>,
}

impl Agent {
    fn new(name: String, prompt: String) -> Self {
        Self { 
            name, 
            prompt,
            events: Vec::new(),
        }
    }
}

#[derive(JsonSchema,Deserialize, Debug,Clone)]
pub struct FunctionArgs {
    pub message: String,
    pub positivity: usize,
    pub thinking: String,
}

const MANIFESTS: [(&str, &str); 4] = [
    ("山田", "./prompts/agent-a.md"),
    ("田中", "./prompts/agent-b.md"),
    ("鈴木", "./prompts/agent-c.md"),
    ("佐藤", "./prompts/agent-d.md")
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut agents = MANIFESTS
        .iter()
        .map(|(name, prompt)| Agent::new(name.to_string(), std::fs::read_to_string(prompt).unwrap()))
        .collect::<Vec<_>>();

    let mut latest_speaker: Option<String> = None;
    let system_prompt = std::fs::read_to_string("./prompts/system.md")?;

    'main: loop {
        if Confirm::new()
            .with_prompt("プログラムを続けますか?")
            .default(true)
            .interact()?
        {
            println!("プログラムを続けます．");

            let mut cloned_agents = agents.clone();
            let mut rng = rand::thread_rng();
            cloned_agents.shuffle(&mut rng);

            // Prepare requests in parallel for each agent
            let tasks: Vec<_> = cloned_agents
                .into_iter()
                .filter(|agent| {
                    if let Some(latest_speaker) = latest_speaker.clone() {
                        agent.name != latest_speaker
                    } else {
                        true
                    }
                })
                .map(|agent| {
                    let system_prompt = system_prompt.clone();
                    task::spawn( async move {
                        let mut messages = vec![
                            RequestMessage {
                                role: "system".to_string(),
                                content: system_prompt,
                            },
                            RequestMessage {
                                role: "system".to_string(),
                                content: agent.prompt,
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
                        (agent.name, recieve_function_call_args::<FunctionArgs>(body).await)
                    })
                })
                .collect();

            let results: Vec<_> = futures::future::join_all(tasks).await.into_iter().filter_map(|result| {
                let (name,arguments) = result.ok()?;
                let arguments = arguments.ok()??;
                Some((name,arguments))
            })
            .collect();
            let most_possible_chat = results.iter().max_by_key(|(_,args)| args.positivity);

            latest_speaker = most_possible_chat.clone().map(|e| e.0.clone());

            if let Some(most_possible_chat) = most_possible_chat {
                println!("{}:({})", most_possible_chat.0, most_possible_chat.1.thinking);
                println!("{}:「{}」", most_possible_chat.0, most_possible_chat.1.message);
                for agent in agents.iter_mut() {
                    agent.events.push(format!("{}が発言しました。:「{}」", most_possible_chat.0, most_possible_chat.1.message));
                }
            }
        } else {
            println!("プログラムを終了します．");
            break 'main;
        }
    }
    Ok(())
}
