use maelstromrs::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

#[allow(dead_code)]
struct EchoNode {
    id: usize,
    node_id: String,
    node_ids: Vec<String>,
}

impl Id for EchoNode {
    fn get_node_id(&self) -> String {
        self.node_id.clone()
    }

    fn get_msg_id(&self) -> usize {
        self.id
    }

    fn increment_msg_id(&mut self) {
        self.id += 1;
    }
}

impl From<Init> for EchoNode {
    fn from(init: Init) -> Self {
        Self {
            id: 1,
            node_id: init.node_id,
            node_ids: init.node_ids,
        }
    }
}

impl Node<Payload> for EchoNode {
    fn handle(&mut self, input: Payload) -> Option<Payload> {
        let payload: Option<Payload> = match input {
            Payload::Echo { echo } => Some(Payload::EchoOk { echo }),
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
    event_loop::<EchoNode, Payload>()
}
