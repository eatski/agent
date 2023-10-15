#[derive(Debug, Clone)]
pub struct Agent {
    pub name: String,
    pub prompt: String,
    pub events: Vec<String>,
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
