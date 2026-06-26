# ADR-0009: Batch Mint Design — Atomicity & Cap Enforcement

- **Status**: Accepted
- **Date**: 2026-06-26

## Context

The token contract supports administrative minting of tokens. As the system evolved, a need emerged to mint tokens to multiple recipients in a single operation, reducing transaction overhead and gas costs in bulk distribution scenarios (airdops, token sales, team allocations).

A key design decision must be made: should the batch mint operation be **atomic** (all-or-nothing), and how should the **supply cap** (if enabled) interact with batch operations?

## Decision

### Atomicity: Validate-All-Then-Mint Ordering

The `batch_mint` function follows a **validate-all-then-mint** pattern:

1. **Phase 1 (Validation)**: Iterate through all recipients and amounts. Validate each entry:
   - Recipient address is valid (not zero)
   - Amount is non-negative and non-zero
   - The cumulative sum does not overflow i128
2. **Phase 2 (Enforcement)**: If cap is enabled, check that `total_supply + cumulative_mint_amount ≤ supply_cap`
3. **Phase 3 (Mint)**: Apply all mints to recipient balances atomically

**Rationale**:
- If any validation fails, the entire batch is rejected without side effects. The contract state remains unchanged.
- This provides predictable behavior: either all recipients receive their tokens, or none do.
- Clients can safely estimate costs and balances without partial-mint surprises.

### Supply Cap Enforcement Across Batch

When `supply_cap` is enabled, the cap is enforced as a **batch-level atomic check**:

```rust
let total_minted = batch.iter().map(|(_, amt)| amt).sum::<i128>()?; // Overflow check
let new_total_supply = total_supply.checked_add(total_minted)?;

if new_total_supply > supply_cap {
    return Err(TokenError::InvalidAmount); // Cap exceeded
}
```

**Rationale**:
- The cap prevents supply from ever exceeding the configured limit.
- By checking the **cumulative total** before any mint, we guarantee atomicity: if the cap would be exceeded, the entire batch fails.
- Individual items in the batch are not validated separately against the cap; the cap is a batch invariant, not a per-item constraint.

### Why Partial Minting Is Not Supported

The starter kit does **not** support partial minting (minting as many recipients as possible until cap is hit, then failing the remainder). Reasons:

1. **Unpredictable state**: Partial success creates ambiguity. Callers cannot reliably determine which recipients were funded without querying balances afterward.
2. **Atomicity simplification**: All-or-nothing minting is easier to reason about and audit.
3. **Batch semantics**: Batch operations typically fail atomically (e.g., SQL transactions). Users expect this pattern.
4. **UX clarity**: A single rejection reason (cap exceeded, invalid entry) is clearer than "3 of 10 succeeded."

If partial minting is needed for your use case, mint to individual recipients in separate transactions and handle failures per-recipient.

## Implementation Notes

- `batch_mint` validates all amounts before any state mutation (fail-fast).
- The function emits a single `minted_batch` event (or multiple `minted` events per recipient, depending on implementation) for off-chain tracking.
- Storage TTL is extended once after all mints complete.
- Gas cost is proportional to batch size; no special optimization is done (callers should batch reasonably).

## Consequences

- Batch operations are **predictable and atomic**: easier for integrations to reason about.
- **Callers must pre-validate** their batch (totals, cap constraints) to avoid rejection.
- **No partial recovery**: If a batch fails, it fails entirely. This is correct for correctness but may require retry logic at the application layer.
- **Supply cap is air-tight**: Cannot be exceeded by batch or individual mints.

## Alternatives Considered

### Partial Minting (Rejected)
Mint as many recipients as possible until cap is hit. Problem: Unpredictable state, poor UX, harder to audit.

### No Batch Support (Rejected)
Require individual mint calls. Problem: Higher gas costs, worse UX for bulk operations.

### Separate Cap Check Per Item (Rejected)
Validate cap separately for each recipient. Problem: Inconsistent with atomic semantics, confusing when cap is hit mid-batch.
