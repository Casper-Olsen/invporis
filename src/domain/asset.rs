#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Asset {
    Equity(Equity),
    Crypto(Crypto),
}

/// FIGI data fetched from `OpenFigi` for the equity.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct FigiData {
    pub ticker: String,
    pub exchange_code: String,
    pub figi: String,
}

/// Equity details associated with a trade or set of trades.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct EquityDetails {
    pub isin: String,
    pub currency: String,
    pub market_identifier_code: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Equity {
    pub figi_data: FigiData,
    pub details: EquityDetails,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Crypto {
    pub symbol: String,
    pub currency: String,
}
