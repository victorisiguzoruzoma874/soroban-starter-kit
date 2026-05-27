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
    /// `true` once the buyer has approved delivery (`bool`).
    BuyerApproved,
    /// `true` once the seller has marked goods/services as delivered (`bool`).
    SellerDelivered,
    /// Whether the contract is paused (`bool`).
    Paused,
    /// Contract version number (`u32`).
    Version,
    /// Pending WASM upgrade: `(BytesN<32>, u32)` = (hash, ready_after_ledger).
    PendingUpgrade,
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

/// Snapshot of all escrow fields returned by
/// [`EscrowContract::get_escrow_info`](crate::EscrowContract::get_escrow_info).
#[contracttype]
#[derive(Clone)]
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
