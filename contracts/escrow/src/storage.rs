use soroban_sdk::{contracttype, Address};

/// Top-level storage keys used by [`EscrowContract`](crate::EscrowContract).
///
/// All keys are stored in instance storage so they share a single TTL bump.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// The buyer's [`Address`].
    Buyer,
    /// The seller's [`Address`].
    Seller,
    /// The arbiter's [`Address`] (used for dispute resolution).
    Arbiter,
    /// The Soroban token contract [`Address`] used for fund transfers.
    TokenContract,
    /// Escrowed token amount as `i128`.
    Amount,
    /// Ledger sequence number after which a refund may be requested (`u32`).
    Deadline,
    /// Current [`EscrowState`] of the escrow lifecycle.
    State,
    /// Whether the contract is paused (`bool`).
    Paused,
    /// Contract version number (`u32`).
    Version,
    /// Pending WASM upgrade: `(BytesN<32>, u32)` = (hash, `ready_after_ledger`).
    PendingUpgrade,
    /// Multiple arbiters for multi-sig support (`Vec<Address>`).
    Arbiters,
    /// Number of required signatures for multi-sig resolution (`u32`).
    RequiredSignatures,
    /// Arbiter votes for dispute resolution (`Vec<Address>`).
    ArbiterVotes,
}

/// Lifecycle states of an escrow.
///
/// Transitions follow a strict order:
/// `Created → Funded → Delivered → Completed`
/// with side exits to `Refunded` or `Cancelled`.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EscrowState {
    /// Escrow has been initialized but not yet funded.
    Created = 0,
    /// Buyer has transferred tokens to the contract.
    Funded = 1,
    /// Seller has marked the obligation as delivered.
    Delivered = 2,
    /// Escrow is under arbiter review.
    Disputed = 3,
    /// Funds have been released to the seller.
    Completed = 4,
    /// Funds have been returned to the buyer.
    Refunded = 5,
    /// Escrow was cancelled before funding.
    Cancelled = 6,
}

impl core::fmt::Display for EscrowState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            EscrowState::Created => "created",
            EscrowState::Funded => "funded",
            EscrowState::Delivered => "delivered",
            EscrowState::Disputed => "disputed",
            EscrowState::Completed => "completed",
            EscrowState::Refunded => "refunded",
            EscrowState::Cancelled => "cancelled",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::EscrowState;

    #[test]
    fn test_escrow_state_display() {
        assert_eq!(EscrowState::Created.to_string(), "created");
        assert_eq!(EscrowState::Funded.to_string(), "funded");
        assert_eq!(EscrowState::Delivered.to_string(), "delivered");
        assert_eq!(EscrowState::Disputed.to_string(), "disputed");
        assert_eq!(EscrowState::Completed.to_string(), "completed");
        assert_eq!(EscrowState::Refunded.to_string(), "refunded");
        assert_eq!(EscrowState::Cancelled.to_string(), "cancelled");
    }
}

#[cfg(test)]
mod discriminant_tests {
    use super::*;

    // In Soroban, #[contracttype] enums use the variant NAME as the XDR storage discriminant.
    // NEVER rename, reorder, or remove variants — doing so will corrupt on-chain storage for
    // any live deployment. To add a new key, append it at the END of the enum definition.
    //
    // This exhaustive match is the primary guard: it causes a COMPILE ERROR if a variant is
    // renamed or removed, and a non-exhaustive warning if one is added without updating here.
    fn escrow_data_key_index(key: &DataKey) -> u32 {
        match key {
            DataKey::Buyer => 0,
            DataKey::Seller => 1,
            DataKey::Arbiter => 2,
            DataKey::TokenContract => 3,
            DataKey::Amount => 4,
            DataKey::Deadline => 5,
            DataKey::State => 6,
            DataKey::Paused => 7,
            DataKey::Version => 8,
            DataKey::PendingUpgrade => 9,
            DataKey::Arbiters => 10,
            DataKey::RequiredSignatures => 11,
            DataKey::ArbiterVotes => 12,
        }
    }

    #[test]
    fn data_key_discriminants_are_stable() {
        assert_eq!(escrow_data_key_index(&DataKey::Buyer), 0);
        assert_eq!(escrow_data_key_index(&DataKey::Seller), 1);
        assert_eq!(escrow_data_key_index(&DataKey::Arbiter), 2);
        assert_eq!(escrow_data_key_index(&DataKey::TokenContract), 3);
        assert_eq!(escrow_data_key_index(&DataKey::Amount), 4);
        assert_eq!(escrow_data_key_index(&DataKey::Deadline), 5);
        assert_eq!(escrow_data_key_index(&DataKey::State), 6);
        assert_eq!(escrow_data_key_index(&DataKey::Paused), 7);
        assert_eq!(escrow_data_key_index(&DataKey::Version), 8);
        assert_eq!(escrow_data_key_index(&DataKey::PendingUpgrade), 9);
        assert_eq!(escrow_data_key_index(&DataKey::Arbiters), 10);
        assert_eq!(escrow_data_key_index(&DataKey::RequiredSignatures), 11);
        assert_eq!(escrow_data_key_index(&DataKey::ArbiterVotes), 12);
    }
}

/// Snapshot of all escrow fields returned by
/// [`EscrowContract::get_escrow_info`](crate::EscrowContract::get_escrow_info).
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct EscrowInfo {
    /// Buyer address.
    pub buyer: Address,
    /// Seller address.
    pub seller: Address,
    /// Arbiter address.
    pub arbiter: Address,
    /// Token contract address.
    pub token_contract: Address,
    /// Current escrowed amount.
    pub amount: i128,
    /// Deadline ledger sequence number.
    pub deadline: u32,
    /// Current lifecycle state.
    pub state: EscrowState,
}
