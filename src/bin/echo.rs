use maelstromrs::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct Node {
    id: usize,
    #[allow(dead_code)]
    node_id: String,
    #[allow(dead_code)]
    node_ids: Vec<String>,
}

impl StateMachine<Payload> for Node {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.payload {
            Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        msg_id: Some(self.id),
                        in_reply_to: input.body.msg_id,
                        payload: Payload::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to init")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }
}

impl From<Init> for Node {
    fn from(init: Init) -> Self {
        Self {
            id: 1,
            node_id: init.node_id,
            node_ids: init.node_ids,
        }
    }
}

fn main() -> anyhow::Result<()> {
    event_loop::<Node, Payload>()
}
