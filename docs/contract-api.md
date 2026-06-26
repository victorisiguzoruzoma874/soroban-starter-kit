# Contract API Reference

Complete public API documentation for all Soroban starter kit contracts.

## Token Contract

**Location:** `contracts/token/src/lib.rs`

| Function | Parameters | Returns | Errors |
|----------|-----------|---------|--------|
| `initialize` | `env: Env, admin: Address, name: String, symbol: String, decimals: u32, max_supply: Option<i128>` | `Result<(), TokenError>` | `AlreadyInitialized`, `InvalidAmount` |
| `mint` | `env: Env, to: Address, amount: i128` | `Result<(), TokenError>` | `Unauthorized`, `Overflow`, `InvalidAmount` |
| `burn` | `env: Env, from: Address, amount: i128` | `Result<(), TokenError>` | `InsufficientBalance`, `Unauthorized`, `InvalidAmount` |
| `transfer` | `env: Env, from: Address, to: Address, amount: i128` | `Result<(), TokenError>` | `InsufficientBalance`, `InvalidAmount` |
| `transfer_from` | `env: Env, spender: Address, from: Address, to: Address, amount: i128` | `Result<(), TokenError>` | `InsufficientAllowance`, `InsufficientBalance`, `InvalidAmount` |
| `approve` | `env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32` | `Result<(), TokenError>` | `InvalidAmount` |
| `allowance` | `env: Env, from: Address, spender: Address` | `i128` | None |
| `balance` | `env: Env, id: Address` | `i128` | None |
| `total_supply` | `env: Env` | `i128` | None |
| `name` | `env: Env` | `String` | None |
| `symbol` | `env: Env` | `String` | None |
| `decimals` | `env: Env` | `u32` | None |

**Errors:**
- `InsufficientBalance` (1) — Caller's balance too low
- `InsufficientAllowance` (2) — Allowance too low for transfer_from
- `Unauthorized` (3) — Caller not admin
- `AlreadyInitialized` (4) — initialize called twice
- `NotInitialized` (5) — Operation before initialize
- `InvalidAmount` (6) — Amount zero, negative, or exceeds cap
- `Overflow` (7) — Arithmetic overflow

---

## Escrow Contract

**Location:** `contracts/escrow/src/lib.rs`

### Core Operations

| Function | Parameters | Returns | Errors |
|----------|-----------|---------|--------|
| `initialize` | `env: Env, buyer: Address, seller: Address, arbiter: Address, token_contract: Address, amount: i128, deadline_ledger: u32` | `Result<(), EscrowError>` | `AlreadyInitialized`, `InvalidAmount`, `InvalidParties` |
| `initialize_with_arbiters` | `env: Env, buyer: Address, seller: Address, arbiters: Vec<Address>, token_contract: Address, amount: i128, deadline_ledger: u32, required_signatures: u32` | `Result<(), EscrowError>` | Same + validation |
| `fund` | `env: Env` | `Result<(), EscrowError>` | `InvalidState`, `InsufficientFunds` |
| `mark_delivered` | `env: Env` | `Result<(), EscrowError>` | `NotAuthorized`, `InvalidState` |
| `approve_delivery` | `env: Env` | `Result<(), EscrowError>` | `NotAuthorized`, `InvalidState` |
| `release_partial` | `env: Env, amount: i128` | `Result<(), EscrowError>` | `NotAuthorized`, `InvalidState`, `InvalidAmount` |
| `request_refund` | `env: Env` | `Result<(), EscrowError>` | `NotAuthorized`, `InvalidState` |
| `raise_dispute` | `env: Env, caller: Address` | `Result<(), EscrowError>` | `NotAuthorized`, `InvalidState` |
| `resolve_dispute` | `env: Env, release_to_seller: bool` | `Result<(), EscrowError>` | `NotAuthorized`, `InvalidState` |
| `cancel` | `env: Env` | `Result<(), EscrowError>` | `NotAuthorized`, `InvalidState` |
| `extend_deadline` | `env: Env, new_deadline: u32` | `Result<(), EscrowError>` | `NotAuthorized`, `InvalidState` |

### Query Functions

| Function | Parameters | Returns | Errors |
|----------|-----------|---------|--------|
| `get_escrow_info` | `env: Env` | `Result<EscrowInfo, EscrowError>` | `NotInitialized` |
| `get_state` | `env: Env` | `Option<EscrowState>` | None |
| `is_deadline_passed` | `env: Env` | `bool` | None |
| `get_remaining_ledgers` | `env: Env` | `i64` | None |

**Errors:**
- `NotAuthorized` (1) — Caller not permitted
- `InvalidState` (2) — Escrow not in required state
- `DeadlinePassed` (3) — Deadline already elapsed
- `DeadlineNotReached` (4) — Deadline not yet passed
- `AlreadyInitialized` (5) — initialize called twice
- `NotInitialized` (6) — Operation before initialize
- `InsufficientFunds` (7) — Buyer balance too low
- `InvalidAmount` (8) — Amount zero or invalid
- `InvalidParties` (9) — Invalid addresses or conflicts

---

## Staking Contract

**Location:** `contracts/staking/src/lib.rs`

| Function | Parameters | Returns | Errors |
|----------|-----------|---------|--------|
| `initialize` | `env: Env, admin: Address, stake_token: Address, reward_token: Address` | `Result<(), StakingError>` | `AlreadyInitialized` |
| `stake` | `env: Env, staker: Address, amount: i128` | `Result<(), StakingError>` | `NotInitialized`, `InvalidAmount` |
| `unstake` | `env: Env, staker: Address, amount: i128` | `Result<(), StakingError>` | `NotInitialized`, `InvalidAmount`, `InsufficientStake`, `NoStake` |
| `add_rewards` | `env: Env, amount: i128` | `Result<(), StakingError>` | `Unauthorized`, `NotInitialized`, `InvalidAmount` |
| `claim_rewards` | `env: Env, staker: Address` | `Result<(), StakingError>` | `NotInitialized`, `NoRewards` |
| `total_staked` | `env: Env` | `i128` | None |
| `total_rewards` | `env: Env` | `i128` | None |
| `user_stake` | `env: Env, staker: Address` | `i128` | None |
| `user_rewards` | `env: Env, staker: Address` | `i128` | None |

**Errors:**
- `AlreadyInitialized` (1) — initialize called twice
- `NotInitialized` (2) — Operation before initialize
- `Unauthorized` (3) — Caller not admin
- `InvalidAmount` (4) — Amount zero or negative
- `NoStake` (5) — No stake to unstake/claim
- `InsufficientStake` (6) — Unstake amount exceeds stake
- `NoRewards` (7) — No rewards available

---

## Vesting Contract

**Location:** `contracts/vesting/src/lib.rs`

| Function | Parameters | Returns | Errors |
|----------|-----------|---------|--------|
| `initialize` | `env: Env, admin: Address, beneficiary: Address, token: Address, amount: i128, cliff_ledger: u32, end_ledger: u32` | `Result<(), VestingError>` | `AlreadyInitialized`, `InvalidAmount`, `InvalidSchedule` |
| `claim` | `env: Env` | `Result<(), VestingError>` | `NotInitialized`, `NothingToClaim` |
| `revoke` | `env: Env` | `Result<(), VestingError>` | `NotInitialized`, `Unauthorized`, `AlreadyRevoked` |
| `get_vested_amount` | `env: Env` | `i128` | None |
| `get_claimed_amount` | `env: Env` | `i128` | None |
| `get_unvested_amount` | `env: Env` | `i128` | None |
| `is_revoked` | `env: Env` | `bool` | None |

**Errors:**
- `AlreadyInitialized` (1) — initialize called twice
- `NotInitialized` (2) — Operation before initialize
- `Unauthorized` (3) — Caller not admin
- `InvalidAmount` (4) — Amount zero or negative
- `InvalidSchedule` (5) — cliff_ledger >= end_ledger or end_ledger in past
- `NothingToClaim` (6) — No tokens vested since last claim
- `AlreadyRevoked` (7) — revoke called on revoked schedule

---

## Multisig Contract

**Location:** `contracts/multisig/src/lib.rs`

### Core Operations

| Function | Parameters | Returns | Errors |
|----------|-----------|---------|--------|
| `initialize` | `env: Env, signers: Vec<Address>, threshold: u32` | `Result<(), MultisigError>` | `AlreadyInitialized`, `InvalidThreshold`, `InvalidSigners` |
| `add_signer` | `env: Env, approvals: Vec<Address>, signer: Address, new_threshold: u32` | `Result<(), MultisigError>` | `NotInitialized`, `NotSigner`, `InsufficientApprovals`, `InvalidThreshold` |
| `remove_signer` | `env: Env, approvals: Vec<Address>, signer: Address, new_threshold: u32` | `Result<(), MultisigError>` | `NotInitialized`, `NotSigner`, `InsufficientApprovals`, `InvalidThreshold` |
| `propose` | `env: Env, target: Address, func: Symbol, args: Vec<Val>` | `Result<u64, MultisigError>` | `NotInitialized` |
| `approve` | `env: Env, tx_id: u64` | `Result<(), MultisigError>` | `TransactionNotFound`, `AlreadyExecuted`, `AlreadySigned`, `NotSigner` |
| `execute` | `env: Env, tx_id: u64` | `Result<Val, MultisigError>` | `TransactionNotFound`, `AlreadyExecuted`, `ThresholdNotMet` |
| `get_signers` | `env: Env` | `Vec<Address>` | None |
| `get_threshold` | `env: Env` | `u32` | None |

**Errors:**
- `AlreadyInitialized` (1) — initialize called twice
- `NotInitialized` (2) — Operation before initialize
- `InvalidThreshold` (3) — Threshold zero or > signer count
- `InvalidSigners` (4) — Signers empty, duplicate, or invalid
- `NotSigner` (5) — Caller/approver not in signer set
- `TransactionNotFound` (6) — TX ID does not exist
- `AlreadyExecuted` (7) — Transaction already executed
- `AlreadySigned` (8) — Signer already approved
- `ThresholdNotMet` (9) — Not enough signatures
- `InsufficientApprovals` (10) — Signer change lacks threshold approvals

---

## Cross-Contract Patterns

### Token Interface
All contracts that transfer tokens implement the standard Soroban token interface:
- `transfer(from, to, amount)`
- `transfer_from(spender, from, to, amount)`
- `approve(from, spender, amount, expiration_ledger)`
- `balance(id)`
- `allowance(from, spender)`

### Admin Pattern
Contracts with admin controls require:
1. Admin address set at initialization
2. `require_auth()` on admin operations
3. TTL bumping on state changes

### Error Handling
All contracts follow consistent error patterns:
- `AlreadyInitialized` / `NotInitialized` gate operations
- `Unauthorized` for permission failures
- `InvalidAmount` for validation failures
- Specific errors for state machine violations
