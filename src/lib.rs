use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{BufRead, StdoutLock, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<P> {
    pub src: String,
    pub dest: String,
    pub body: Body<P>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<P> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: P,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Id {
    fn get_msg_id(&self) -> usize;
    fn increment_msg_id(&mut self);
}

pub trait Node<P>: Id {
    fn handle(&mut self, input: P) -> Option<P>;

    fn reply(&mut self, input: Message<P>, output: &mut StdoutLock) -> anyhow::Result<()>
    where
        P: Serialize,
    {
        let reply_payload = self.handle(input.body.payload);
        if let Some(reply_payload) = reply_payload {
            let reply = Message {
                src: input.dest,
                dest: input.src,
                body: Body {
                    msg_id: Some(self.get_msg_id()),
                    in_reply_to: input.body.msg_id,
                    payload: reply_payload,
                },
            };
            serde_json::to_writer(&mut *output, &reply).context("serialize response to init")?;
            output.write_all(b"\n").context("write trailing newline")?;
            self.increment_msg_id();
        }
        Ok(())
    }

    fn from_init(input: Message<InitPayload>, output: &mut StdoutLock) -> anyhow::Result<Self>
    where
        Self: Sized + From<Init>,
    {
        // if the first message is not an Init message, return an error
        let init = match input.body.payload {
            InitPayload::Init(init) => init,
            _ => anyhow::bail!("First message from Maelstrom was not an Init message"),
        };
        // send an InitOk message back to Maelstrom
        let reply = Message {
            src: input.dest,
            dest: input.src,
            body: Body {
                msg_id: Some(0),
                in_reply_to: input.body.msg_id,
                payload: InitPayload::InitOk,
            },
        };
        serde_json::to_writer(&mut *output, &reply).context("serialize response to init")?;
        // write a trailing newline
        output.write_all(b"\n").context("write trailing newline")?;
        let state = Self::from(init);
        Ok(state)
    }
}

pub fn event_loop<S, P>() -> anyhow::Result<()>
where
    S: Node<P> + From<Init> + Id,
    P: DeserializeOwned + Serialize,
{
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();

    // read the first line from stdin and deserialize it as an Init message
    let init = stdin
        .next()
        .context("Maelstrom input from STDIN could not be deserialized")?
        .context("Maelstrom input from STDIN could not be deserialized")?;

    let init: Message<InitPayload> = serde_json::from_str(&init)
        .context("Maelstrom input from STDIN could not be deserialized")?;

    // initialize the state machine
    let mut state = S::from_init(init, &mut stdout).context("Node initialization failed")?;

    // loop over the remaining lines from stdin and deserialize them as Messages
    for input in stdin {
        let input = input.context("Maelstrom input from STDIN could not be deserialized")?;
        let input: Message<P> = serde_json::from_str(&input)
            .context("Maelstrom input from STDIN could not be deserialized")?;
        state
            .reply(input, &mut stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}
