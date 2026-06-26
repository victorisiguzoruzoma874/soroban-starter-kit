# Storage Layout Documentation

This document details the storage keys, types, and TTL management policy for each contract in the Soroban Starter Kit. Each contract uses a mix of **instance** and **persistent** storage to optimize gas costs and TTL extension overhead.

## Storage Tier Policies

| Tier | Lifetime | Use Case | TTL Extension |
|------|----------|----------|----------------|
| **Instance** | Tied to contract instance | Immutable or rarely-changed config; contract state | Extended once per transaction |
| **Persistent** | Independent; outlives contract upgrades | Per-user balances, allowances, proposals | Extended per operation or batch |
| **Temporary** | Ledger sequence + TTL window | Time-limited access tokens (allowances) | Not extended; expires naturally |

---

## Token Contract

### Storage Keys

| Key | Tier | Type | TTL Policy | Description |
|-----|------|------|------------|-------------|
| `Admin` | Instance | `Address` | Extended on init | Current contract administrator |
| `PendingAdmin` | Instance | `Address` | Extended on proposal/acceptance | Pending admin for two-step transfer |
| `Balance(Address)` | Persistent | `i128` | Extended on every balance change | Token balance per address |
| `Allowance(AllowanceDataKey)` | Temporary | `AllowanceValue` | Expires naturally | Approve(spender) with expiration ledger |
| `Metadata(MetadataKey)` | Instance | `String` / `u32` | Extended on init | Token name, symbol, decimals |
| `TotalSupply` | Persistent | `i128` | Extended on mint/burn | Total circulating supply |
| `Paused` | Instance | `bool` | Extended on pause/unpause | Whether contract is paused |
| `MaxSupply` | Instance | `i128` | Extended on init | Hard cap on supply (if capped-supply feature enabled) |
| `PendingUpgrade` | Instance | `(BytesN<32>, u32)` | Extended on upgrade proposal | Pending WASM hash + ready ledger |
| `Version` | Instance | `u32` | Extended on init | Contract version number |
| `Frozen(Address)` | Persistent | `bool` | Extended per freeze/unfreeze | Whether an address is frozen |

---

## Escrow Contract

### Storage Keys

| Key | Tier | Type | TTL Policy | Description |
|-----|------|------|------------|-------------|
| `Buyer` | Instance | `Address` | Extended on init | Buyer address |
| `Seller` | Instance | `Address` | Extended on init | Seller address |
| `Arbiter` | Instance | `Address` | Extended on init | Single arbiter (deprecated; see `Arbiters`) |
| `Arbiters` | Instance | `Vec<Address>` | Extended on init | Multi-sig arbiter group for resolution |
| `TokenContract` | Instance | `Address` | Extended on init | Soroban token contract address |
| `Amount` | Instance | `i128` | Extended on init | Escrowed amount in token units |
| `Deadline` | Instance | `u32` | Extended on init | Refund-eligible ledger sequence |
| `State` | Instance | `EscrowState` | Extended on state change | Current escrow lifecycle state |
| `Paused` | Instance | `bool` | Extended on pause/unpause | Whether contract is paused |
| `Version` | Instance | `u32` | Extended on init | Contract version number |
| `PendingUpgrade` | Instance | `(BytesN<32>, u32)` | Extended on upgrade proposal | Pending WASM hash + ready ledger |
| `RequiredSignatures` | Instance | `u32` | Extended on init | Threshold for multi-sig resolution |
| `ArbiterVotes` | Instance | `Vec<Address>` | Extended per vote | Arbiters who have voted to release/refund |

**Escrow States:** `Created → Funded → Delivered → Completed`

---

## Vesting Contract

### Storage Keys

| Key | Tier | Type | TTL Policy | Description |
|-----|------|------|------------|-------------|
| `Admin` | Instance | `Address` | Extended on init | Contract administrator |
| `Beneficiary` | Instance | `Address` | Extended on init | Token recipient |
| `Token` | Instance | `Address` | Extended on init | Token contract address |
| `Amount` | Instance | `i128` | Extended on init | Total vesting amount |
| `CliffLedger` | Instance | `u32` | Extended on init | Ledger at which cliff vesting begins |
| `EndLedger` | Instance | `u32` | Extended on init | Ledger at which all tokens are fully vested |
| `Claimed` | Persistent | `i128` | Extended on claim | Tokens already withdrawn by beneficiary |
| `Revoked` | Persistent | `bool` | Extended on revoke | Whether vesting schedule has been revoked |

**Vesting Release:** Tokens accrue linearly from `CliffLedger` to `EndLedger`. Admin can revoke unvested tokens at any time; vested tokens remain claimable.

---

## Staking Contract

### Storage Keys

| Key | Tier | Type | TTL Policy | Description |
|-----|------|------|------------|-------------|
| `Admin` | Instance | `Address` | Extended on init | Contract administrator |
| `StakeToken` | Instance | `Address` | Extended on init | Token that users stake |
| `RewardToken` | Instance | `Address` | Extended on init | Token distributed as rewards (may equal StakeToken) |
| `TotalStaked` | Persistent | `i128` | Extended on every stake/unstake | Total staked across all stakers |
| `TotalRewards` | Persistent | `i128` | Extended on reward deposit/claim | Unclaimed reward pool |
| `RewardPerTokenStored` | Persistent | `i128` | Extended on reward snapshot | Global reward accumulator (scaled by 1e12) |
| `Stake(Address)` | Persistent | `i128` | Extended on stake/unstake | Per-staker staked amount |
| `RewardPerTokenPaid(Address)` | Persistent | `i128` | Extended on reward claim | Per-staker reward accumulator snapshot |
| `Rewards(Address)` | Persistent | `i128` | Extended on reward accrual/claim | Per-staker accrued rewards |

---

## Multisig Contract

### Storage Keys

| Key | Tier | Type | TTL Policy | Description |
|-----|------|------|------------|-------------|
| `Signers` | Instance | `Vec<Address>` | Extended on signer change | List of authorized signers |
| `Threshold` | Instance | `u32` | Extended on threshold change | Required number of signatures (N-of-M) |
| `NextTransactionId` | Instance | `u64` | Extended on proposal | Auto-incrementing transaction ID counter |
| `Transaction(u64)` | Persistent | `Transaction` | Extended on propose/sign/execute | Proposed transaction details and signatures |
| `Paused` | Instance | `bool` | Extended on pause/unpause | Whether contract is paused |
| `Version` | Instance | `u32` | Extended on init | Contract version number |

**Transaction Struct:** Contains proposer, target, function name, arguments, signatures, and execution status.

---

## DAO Contract

### Storage Keys

| Key | Tier | Type | TTL Policy | Description |
|-----|------|------|------------|-------------|
| `Admin` | Instance | `Address` | Extended on init | Contract administrator |
| `Token` | Instance | `Address` | Extended on init | Token used for voting power |
| `VotingPeriod` | Instance | `u32` | Extended on init | Proposal duration in ledgers |
| `Quorum` | Instance | `i128` | Extended on init | Minimum votes required for quorum |
| `ProposalCount` | Instance | `u32` | Extended on proposal creation | Total proposals created |
| `Initialized` | Instance | `bool` | Extended on init | Whether contract has been initialized |
| `Proposal(u32)` | Persistent | `Proposal` | Extended on proposal change | Proposal state, vote counts, deadline |
| `VoteKey { proposal_id, voter }` | Persistent | `i128` | Extended on vote | Per-voter vote amount (prevents double-voting) |

**Proposal States:** `Active → Executed` (if quorum + majority reached) or `Active → Cancelled` (admin action)

---

## TTL Extension Strategy

### Instance vs. Persistent

- **Instance storage** is extended once per transaction involving the contract. All instance keys share a single TTL bump.
- **Persistent storage** is extended individually per key on each read or write operation.

### Ledger Sequence and Renewal

On every operation that reads or writes a key:

1. The current ledger sequence is obtained via `env.ledger().sequence()`.
2. If TTL is below a threshold (typically 50 days), the entry is extended to `current_ledger + EXTENSION_LIFETIME`.
3. EXTENSION_LIFETIME is typically 31,536,000 ledgers (~1 year).

### Example: Token Transfer

```
User calls transfer(to, amount)
├─ Read Balance(from) → TTL bumped
├─ Write Balance(from) → TTL bumped
├─ Read Balance(to) → TTL bumped
├─ Write Balance(to) → TTL bumped
└─ Emit transferred event
```

---

## See Also

- [Architecture Decision Record 0001: Storage Tier Choices](adr/0001-storage-tier-choices.md) — rationale for instance vs. persistent tier selection
- [Integration Guide](integration-guide.md) — how to interpret storage in your integration
- [Soroban Storage Docs](https://soroban.stellar.org/docs/learn/storing-data)
