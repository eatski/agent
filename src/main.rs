use dialoguer::Confirm;
use model::Agent;
use rand::seq::SliceRandom;
use tokio::task;

mod openai;
mod chat;
mod model;

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
                        (agent.name.clone(), chat::chat(&agent, system_prompt).await)
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
