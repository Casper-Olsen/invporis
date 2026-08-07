use std::hash::Hash;

#[derive(Debug)]
pub struct Security {
    pub isin: String,
    pub name: Option<String>,
    pub currency: String,
}

impl Security {
    pub fn key(&self) -> (String, String) {
        (self.isin.clone(), self.currency.clone())
    }
}

impl PartialEq for Security {
    fn eq(&self, other: &Self) -> bool {
        self.isin == other.isin && self.currency == other.currency
    }
}

impl Hash for Security {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.isin.hash(state);
        self.currency.hash(state);
    }
}

impl Eq for Security {}
