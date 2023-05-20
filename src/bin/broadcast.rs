use maelstromrs::*;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    ReadOk { messages: HashSet<i32> },
    Topology { topology: Topology },
    TopologyOk {},
}

#[allow(dead_code)]
struct BroadcastNode {
    id: usize,
    node_id: String,
    node_ids: Vec<String>,
    memory: HashSet<i32>,
}

impl Id for BroadcastNode {
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

impl From<Init> for BroadcastNode {
    fn from(init: Init) -> Self {
        Self {
            id: 1,
            node_id: init.node_id,
            node_ids: init.node_ids,
            memory: HashSet::new(),
        }
    }
}

impl Node<Payload> for BroadcastNode {
    fn handle(&mut self, input: Payload) -> Option<Payload> {
        let payload: Option<Payload> = match input {
            Payload::Broadcast { message } => {
                // check if message is already in memory
                if self.memory.contains(&message) {
                    return Some(Payload::BroadcastOk {});
                }

                // add message to memory
                self.memory.insert(message);

                // broadcast to all nodes
                let node_ids = self.node_ids.clone();
                let self_id = self.node_id.clone();
                for node_id in node_ids {
                    if node_id != self_id {
                        self.send(&node_id, Payload::Broadcast { message });
                    }
                }
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
