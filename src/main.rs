use std::collections::HashMap;

use dialoguer::Confirm;
use model::Agent;
use rand::seq::SliceRandom;
use thinking::thinking;
use tokio::task;

mod chat;
mod model;
mod openai;
mod thinking;

const MANIFESTS: [(&str, &str); 4] = [
    ("山田", "./prompts/agent-a.md"),
    ("田中", "./prompts/agent-b.md"),
    ("鈴木", "./prompts/agent-c.md"),
    ("佐藤", "./prompts/agent-d.md"),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut agents = MANIFESTS
        .iter()
        .map(|(name, prompt)| {
            Agent::new(name.to_string(), std::fs::read_to_string(prompt).unwrap())
        })
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
                    task::spawn(async move {
                        (
                            agent.name.clone(),
                            thinking(&agent, system_prompt.as_str()).await,
                        )
                    })
                })
                .collect();

            let thinkings: HashMap<_, _> = futures::future::join_all(tasks)
                .await
                .into_iter()
                .filter_map(|result| {
                    let (name, arguments) = result.ok()?;
                    let arguments = arguments.ok()??;
                    Some((name, arguments))
                })
                .collect();
            for agent in agents.iter_mut() {
                if let Some(thinking) = thinkings.get(&agent.name) {
                    agent
                        .events
                        .push(format!("考え中:（{}）", thinking.thinking));
                    println!("{}:({})", agent.name, thinking.thinking)
                }
            }
            async {
                let most_possible = thinkings
                    .iter()
                    .max_by_key(|(_, args)| args.positivity)
                    .map(|e| e.0.clone())?;
                let result = chat::chat(
                    &agents
                        .iter()
                        .find(|agent| agent.name == most_possible)?
                        .clone(),
                    &system_prompt,
                )
                .await
                .ok()??;
                latest_speaker = Some(most_possible.clone());
                println!("{}:「{}」", most_possible.clone(), result.message);
                for agent in agents.iter_mut() {
                    agent.events.push(format!(
                        "{}が発言しました。:「{}」",
                        most_possible.clone(),
                        result.message
                    ));
                }
                Some(())
            }
            .await;
        } else {
            println!("プログラムを終了します．");
            break 'main;
        }
    }
    Ok(())
}
