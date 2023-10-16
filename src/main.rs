use std::{collections::HashMap, path::PathBuf, fs::read_to_string};

use dialoguer::{Confirm, Input};
use model::Agent;
use rand::seq::SliceRandom;
use tokio::task;
use agent_act::agent_act;

use crate::{agent_act::{ReactionFunctionArgs, ChatFunctionArgs}, model::Event};

mod model;
mod openai;
mod agent_act;
#[derive(serde::Deserialize)]
struct ManifestJson {
    common: String,
    agents: Vec<ManifestJsonAgent>,
}

#[derive(serde::Deserialize)]
struct ManifestJsonAgent {
    name: String,
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let manifest_path: String =  Input::new()
        .with_prompt("manifest.jsonのあるディレクトリを指定してください。")
        .interact()?;
    let dir_path = PathBuf::from(manifest_path);
    let manifest = read_to_string(dir_path.join("manifest.json"))?;
    let manifest: ManifestJson = serde_json::from_str(&manifest)?;

    let mut latest_speaker: Option<String> = None;
    let common_prompt = read_to_string(dir_path.join(manifest.common))?;
   
    let mut agents = manifest
    .agents
    .iter()
    .map(|agent| {
        Agent::new(agent.name.clone(),  read_to_string(dir_path.join(agent.prompt.as_str())).unwrap())
    })
    .collect::<Vec<_>>();

    let participants = agents.iter().map(|agent| agent.name.clone()).collect::<Vec<_>>().join(",");
    let participants_promot = format!("参加者は {} の{}人です。", participants, manifest.agents.len());
    let common_prompts = vec![common_prompt, participants_promot];
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
                    let common_prompts = common_prompts.clone();
                    task::spawn(async move {
                        (
                            agent.name.clone(),
                            agent_act::<ReactionFunctionArgs>(&agent, &common_prompts).await,
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
                if let Some(reaction) = thinkings.get(&agent.name) {
                    agent
                        .events
                        .push(Event::Reaction { 
                            thinking: reaction.thinking.clone(),
                            aggressiveness: reaction.aggressiveness,
                        });
                    println!("{}:({})", agent.name, reaction.thinking)
                }
            }
            async {
                let most_possible = thinkings
                    .iter()
                    .max_by_key(|(_, args)| args.aggressiveness)
                    .map(|e| e.0.clone())?;
                let result = agent_act::<ChatFunctionArgs>(
                    &agents
                        .iter()
                        .find(|agent| agent.name == most_possible)?
                        .clone(),
                    &common_prompts,
                )
                .await
                .ok()??;
                latest_speaker = Some(most_possible.clone());
                println!("{}:「{}」", most_possible.clone(), result.message);
                for agent in agents.iter_mut() {
                    if agent.name == most_possible {
                        agent.events.push(Event::Speak {
                            message: result.message.clone(),
                        });
                    } else {
                        agent.events.push(Event::ListenOtherSpeak {
                            player_name: most_possible.clone(),
                            message: result.message.clone(),
                        });
                    }
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
