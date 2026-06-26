use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Price,
    UpdatedAt,
    StalenessThreshold,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriceData {
    pub price: i128,
    pub updated_at: u32,
    pub admin: Address,
    pub staleness_threshold: u32,
}
