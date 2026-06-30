use clap::ValueEnum;

// TODO: Don't use 'ValueEnum' here. Have a separate type that can be used from the CLI. And map
// between them
#[derive(ValueEnum, Clone, Debug)]
pub enum Event {
    Buy,
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Buy => "buy",
        };
        write!(f, "{s}")
    }
}
