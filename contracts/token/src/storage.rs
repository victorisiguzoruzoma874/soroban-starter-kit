use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    /// Persistent storage – token balance (`i128`) for a given [`Address`].
    Balance(Address),
    Allowance(AllowanceDataKey),
    Metadata(MetadataKey),
    TotalSupply,
    /// Instance storage – whether the contract is paused (`bool`).
    Paused,
    /// Instance storage – maximum tokens that may ever be minted (`i128`).
    MaxSupply,
    /// Instance storage – pending admin address for two-step admin transfer.
    PendingAdmin,
    /// Instance storage – pending WASM upgrade: `(BytesN<32>, u32)` = (hash, ready_after_ledger).
    PendingUpgrade,
    /// Instance storage – pending admin address for two-step admin transfer.
    /// Instance storage – address of the pending new admin awaiting acceptance.
    PendingAdmin,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum MetadataKey {
    Name,
    Symbol,
    Decimals,
}
