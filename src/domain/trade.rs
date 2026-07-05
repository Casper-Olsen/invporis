use crate::cli;

#[derive(Clone, Debug)]
pub enum Event {
    Buy,
}

impl Event {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "buy",
        }
    }
}

impl From<cli::command::Event> for Event {
    fn from(item: cli::command::Event) -> Self {
        match item {
            cli::command::Event::Buy => Self::Buy,
        }
    }
}

#[derive(Debug)]
pub struct Trade {
    pub event: Event,
    pub symbol: String,
}
