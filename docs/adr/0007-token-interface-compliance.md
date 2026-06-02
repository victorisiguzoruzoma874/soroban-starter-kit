# ADR 0007: Token Interface Compliance

- Status: Accepted
- Date: 2026-05-29

## Context

The `contracts/token` implementation in this repo implements `soroban_sdk::token::TokenInterface` (see `impl token::TokenInterface for TokenContract` in `contracts/token/src/lib.rs`). Consumers and integrators need clear documentation of which standard this corresponds to, what compliance means in practice, and any repository-specific deviations or extensions.

## Decision

Document the contract as compliant with the Soroban SDK token interface (`soroban_sdk::token::TokenInterface`). This implementation aligns with SEP-41 token transfer semantics as referenced in the project changelog. At the time of this ADR, the Soroban token interface is exercised through the SDK trait rather than a single canonical SEP document; this ADR records the compliance checklist, deviations, and verification steps.

## Compliance checklist

- Implements the `TokenInterface` trait on the contract type.
- Public methods required by the interface and expected behaviors:
  - `name() -> String` — returns token name
  - `symbol() -> String` — returns token symbol
  - `decimals() -> u32` — returns decimals
  - `total_supply() -> i128` — reports current total supply
  - `balance(id: Address) -> i128` — returns 0 for unknown or zero balances
  - `allowance(from, spender) -> i128` — returns numeric allowance (0 when none or expired)
  - `approve(from, spender, amount, expiration_ledger)` — sets temporary allowance
  - `transfer(from, to, amount)` — requires `from` auth, updates balances and emits event
  - `transfer_from(spender, from, to, amount)` — requires `spender` auth, deducts allowance, updates balances
  - `burn(from, amount)` and `burn_from(spender, from, amount)` — supports authorized burns

## Known deviations and extensions

- `balance_of(id) -> Option<i128>`: this contract provides `balance_of` (returns `Some(balance)` or `None`) in addition to `balance()`; this is an extension for clients that need to distinguish unknown addresses from explicit zero balances.
- Allowance expirations: allowances are stored in temporary storage with an `expiration_ledger` field; this extends the basic approve/allowance semantics with expiry semantics.
- Admin-controlled operations: the contract exposes admin-only functions such as `mint`, `batch_mint`, `admin_burn`, `propose_admin`, `pause`/`unpause` (feature-gated), `freeze_account` (feature-gated), and upgrade-related functions (feature-gated). These are optional extensions and are not required by the `TokenInterface` trait.
- Event names and shapes: the contract emits custom event names defined in its `events` module; while functionally equivalent for many consumers, consumers should verify expected event payloads.

## How to verify compliance

1. Source inspection: confirm the presence of `impl token::TokenInterface for TokenContract` and that all trait methods are implemented (see `contracts/token/src/lib.rs`).
2. ABI / interface check: build the contract and inspect the generated contract interface (WASM exports or manifest) to ensure the expected functions are present.
3. Unit tests: review and run unit tests in `contracts/token/src/test.rs` to validate behaviors like transfers, allowances, and edge cases (note: this repo includes tests that exercise the interface).
4. Integration checks: exercise the built contract via the Soroban CLI or test harness to validate event emission, allowance expirations, and admin operations.
5. Review deviations: pay special attention to `balance_of` semantics, allowance expiry behavior, and admin-only extensions when integrating with wallets or marketplaces.

## Consequences

- Positive: clearly documenting compliance and deviations helps integrators and auditors understand expected behavior and reduces integration surprises.
- Negative: the added extensions (allowance expirations, admin ops) mean third-party integrators must be aware of differences from minimal token standards (e.g., ERC-20-like expectations).

## Migration / compatibility notes

- If a stricter or different token SEP emerges for Soroban, we can adapt by:
  1. Implementing any missing trait methods or changing semantics to match the canonical SEP.
  2. Providing adapter contracts or wrapper layers that translate between this contract's extensions and the canonical interface.

## References

- Source implementation: `contracts/token/src/lib.rs`
- Related ADRs: [0003 Admin Model](0003-admin-model.md)
