use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Admin address (instance).
    Admin,
    /// Payment token address (instance).
    Token,
    /// Merkle root as a 32-byte value stored as Bytes (instance).
    MerkleRoot,
    /// Whether a given address has already claimed (persistent).
    Claimed(Address),
}
