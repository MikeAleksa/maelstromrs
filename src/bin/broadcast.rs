use maelstromrs::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Topology {
    #[serde(flatten)]
    nodes: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast { message: i32 },
    BroadcastOk {},
    Read {},
    ReadOk { messages: Vec<i32> },
    Topology { topology: Topology },
    TopologyOk {},
}

#[allow(dead_code)]
struct BroadcastNode {
    id: usize,
    node_id: String,
    node_ids: Vec<String>,
    memory: Vec<i32>,
}

impl Id for BroadcastNode {
    fn get_msg_id(&self) -> usize {
        self.id
    }

    fn increment_msg_id(&mut self) {
        self.id += 1;
    }
}

impl From<Init> for BroadcastNode {
    fn from(init: Init) -> Self {
        Self {
            id: 1,
            node_id: init.node_id,
            node_ids: init.node_ids,
            memory: Vec::new(),
        }
    }
}

impl Node<Payload> for BroadcastNode {  
    fn handle(&mut self, input: Payload) -> Option<Payload> {
        let payload: Option<Payload> = match input {
            Payload::Broadcast { message } => {
                self.memory.push(message);
                Some(Payload::BroadcastOk {})
            }
            Payload::Read { .. } => Some(Payload::ReadOk {
                messages: self.memory.clone(),
            }),
            Payload::Topology { .. } => Some(Payload::TopologyOk {}),
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
    event_loop::<BroadcastNode, Payload>()
}
