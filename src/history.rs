use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub expression: String,
    pub result: String,
    pub approximate: bool,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct History {
    pub entries: Vec<Entry>,
}
impl History {
    pub const LIMIT: usize = 500;
    pub fn push(&mut self, entry: Entry) {
        if self
            .entries
            .last()
            .is_some_and(|e| e.expression == entry.expression && e.result == entry.result)
        {
            return;
        }
        self.entries.push(entry);
        if self.entries.len() > Self::LIMIT {
            self.entries.drain(..self.entries.len() - Self::LIMIT);
        }
    }
    pub fn sanitize(&mut self) {
        self.entries
            .retain(|e| e.expression.len() <= 2048 && e.result.len() <= 128);
        if self.entries.len() > Self::LIMIT {
            self.entries.drain(..self.entries.len() - Self::LIMIT);
        }
    }
}
