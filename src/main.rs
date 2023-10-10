use dialoguer::Confirm;
use openai::{ChatCompletionBody, RequestMessage, Function, Response};
use rand::seq::SliceRandom;
use serde::Deserialize;
use tokio::task;
use schemars::{schema_for, JsonSchema};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut agents = vec![
        Agent::new("A".to_string(), "./prompts/agent-a.md".to_string()),
        Agent::new("B".to_string(), "./prompts/agent-b.md".to_string()),
        Agent::new("C".to_string(), "./prompts/agent-c.md".to_string()),
    ];

    'main: loop {
        if Confirm::new()
            .with_prompt("プログラムを続けますか?")
            .default(true)
            .interact()?
        {
            println!("プログラムを続けます．");

            let system_prompt = std::fs::read_to_string("./prompts/system.md").unwrap();

            // Prepare requests in parallel for each agent
            let tasks: Vec<_> = agents
                .clone()
                .into_iter()
                .map(|agent| task::spawn(handle_agent(agent,system_prompt.clone())))
                .collect();

            let mut results: Vec<_> = futures::future::join_all(tasks).await.into_iter().filter_map(|result| {
                let (name,response) = result.ok()?.ok()?;
                let choice = response.choices[0].clone();
                let arguments = choice.message.function_call.clone().map(|e| e.arguments)?;
                let arguments = serde_json::from_str::<FunctionArgs>(&arguments).ok()?;
                let thinking = choice.message.content;
                Some((name,arguments,thinking))
            })
            .collect();

            for (name,_,content) in results.iter() {
                println!("{}:({})", name, content);
            }

            let mut rng = rand::thread_rng();
            results.shuffle(&mut rng);
            let most_possible_chat = results.iter().max_by_key(|(_,args,_): &&(String, FunctionArgs, String)| args.positivity);

            if let Some(most_possible_chat) = most_possible_chat {
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

#[derive(Debug)]
struct OpenAIClientError;
async fn handle_agent(
    agent: Agent,
    system_prompt: String,
) -> Result<(String, Response), OpenAIClientError> {
    let prompt = std::fs::read_to_string(&agent.prompt).unwrap();
    let mut messages = vec![
        RequestMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        RequestMessage {
            role: "user".to_string(),
            content: prompt,
        },
    ];

    messages.extend(agent.events.iter().map(|e| RequestMessage {
        role: "user".to_string(),
        content: e.to_string(),
    }));

    let body = ChatCompletionBody {
        model: "gpt-4-0613".to_string(),
        messages,
        temperature: 0.3,
        max_tokens: 4000,
        function_call: "auto".to_string(),
        functions: vec![
            Function {
                name: "chat".to_string(),
                description: "他プレイヤーに対して発言します。messageは発言の内容、positivity(0~100)は発言する際の積極性（高いほど積極的）を意味します。積極性が低いと発言そのものが無視される可能性があります。".to_string(),
                parameters: schema_for!(FunctionArgs),
            }
        ]
    };
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .json(&body)
        .header(
            "Authorization",
            format!("Bearer {}", std::env::var("OPENAI_TOKEN").unwrap()),
        )
        .send()
        .await.map_err(|_| OpenAIClientError)?;

    let response: Response = res.json().await.map_err(|_| OpenAIClientError)?;
    Ok((agent.name, response))
}
