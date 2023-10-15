#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
    pub prompt: String,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]

pub enum Event {
    Reaction {
        thinking: String,
        positivity: usize,
    },
    Speak{
        message: String,   
    },
    ListenOtherSpeak{
        player_name: String,
        message: String,
    },
}

impl Agent {
    pub fn new(name: String, prompt: String) -> Self {
        Self {
            name,
            prompt,
            events: Vec::new(),
        }
    }
}
