use soroban_sdk::{contracttype, Address, Symbol, Val, Vec};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Signers,
    Threshold,
    NextTransactionId,
    Transaction(u64),
    Paused,
    Version,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    pub id: u64,
    pub proposer: Address,
    pub target: Address,
    pub function: Symbol,
    pub args: Vec<Val>,
    pub signatures: Vec<Address>,
    pub executed: bool,
}
