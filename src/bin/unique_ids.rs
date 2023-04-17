use maelstromrs::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Generate {},
    GenerateOk { id: String },
}

#[allow(dead_code)]
struct UniqueIdNode {
    id: usize,
    node_id: String,
    node_ids: Vec<String>,
}

impl Id for UniqueIdNode {
    fn get_msg_id(&self) -> usize {
        self.id
    }

    fn increment_msg_id(&mut self) {
        self.id += 1;
    }
}

impl From<Init> for UniqueIdNode {
    fn from(init: Init) -> Self {
        Self {
            id: 1,
            node_id: init.node_id,
            node_ids: init.node_ids,
        }
    }
}

impl Node<Payload> for UniqueIdNode {
    fn handle(&mut self, input: Payload) -> Option<Payload> {
        let payload: Option<Payload> = match input {
            Payload::Generate { .. } => Some(Payload::GenerateOk {
                id: self.id.to_string() + self.node_id.as_str(),
            }),
            _ => None,
        };
        if let Some(payload) = payload {
            Some(payload)
        } else {
            None
        }
    }
}

fn main() -> anyhow::Result<()> {
    event_loop::<UniqueIdNode, Payload>()
}
