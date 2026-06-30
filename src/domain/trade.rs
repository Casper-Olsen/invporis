use crate::domain::event::Event;

#[derive(Debug)]
pub struct Trade {
    pub event: Event,
    pub symbol: String,
}

impl std::fmt::Display for Trade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
