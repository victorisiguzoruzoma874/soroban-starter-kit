# ADR-0005: Feature Flag Design

- **Status**: Accepted
- **Date**: 2026-06-01

## Context

Soroban contracts are immutable once deployed. Upgrading logic requires either a full redeployment or the use of the `update_current_contract_wasm` host function, both of which carry operational risk. Several scenarios motivate the ability to toggle behaviour at runtime without redeployment:

- **Phased mainnet rollouts** — enable a new capability for a subset of users or after a time-lock, without deploying a new binary.
- **Safety circuit breakers** — disable a function (e.g. withdrawals, minting) instantly in response to an exploit or oracle failure.
- **Isolated protocol testing** — activate experimental logic on testnet while keeping mainnet behaviour unchanged, using the same compiled binary.
- **Deprecation paths** — mark a feature as disabled before removing it in a future upgrade, giving integrators time to adapt.

Without a deliberate flag strategy, developers either hard-code behaviour (inflexible) or scatter ad-hoc boolean checks across the codebase (unmaintainable).

## Decision

### Flag storage

Feature flags are stored as individual `bool` values in **instance storage** under a dedicated key enum:

```rust
#[derive(Clone)]
#[contracttype]
pub enum FeatureKey {
    Flag(Symbol), // e.g. Flag(symbol_short!("mint"))
}
```

Instance storage is chosen because:

- Flags are global contract state, always read together with other instance data.
- A single TTL bump covers all flags — no per-key TTL management.
- Flags are small (one bool each); the cost of instance storage is negligible.

### Admin-only writes

Only the contract admin may set or clear a flag. Flag writes call `admin.require_auth()` via the existing `require_admin` helper (see ADR-0003):

```rust
pub fn set_flag(env: &Env, name: Symbol, enabled: bool) -> Result<(), AdminError> {
    let admin = require_admin(env)?;
    admin.require_auth();
    env.storage().instance().set(&FeatureKey::Flag(name), &enabled);
    Ok(())
}
```

### Guard macro at call sites

A single helper centralises the flag check and returns a typed error on failure:

```rust
pub fn require_feature(env: &Env, name: Symbol) -> Result<(), FeatureError> {
    let enabled: bool = env.storage().instance()
        .get(&FeatureKey::Flag(name))
        .unwrap_or(false); // absent == disabled
    if enabled { Ok(()) } else { Err(FeatureError::Disabled) }
}
```

Absent keys default to `false` (disabled). New flags are therefore off-by-default, which is the safe choice for circuit breakers and experimental features.

### Choosing what to flag

Flag a feature when **all** of the following apply:

1. The feature can be meaningfully disabled without leaving the contract in an inconsistent state.
2. The admin needs the ability to toggle it post-deployment (rollout, emergency, or deprecation).
3. The conditional path does not introduce unbounded storage growth or loops.

Do **not** flag core invariants (e.g. auth checks, state-machine transitions) — those must always be enforced.

### Upgrade and deprecation path

| Stage | Action |
|-------|--------|
| **Experimental** | Flag defaults to `false`; admin enables on testnet only. |
| **Rollout** | Admin enables flag on mainnet; monitor for issues. |
| **Stable** | Remove the `require_feature` guard; delete the flag key from storage via a migration call. |
| **Deprecated** | Set flag to `false`; document removal timeline; remove in next upgrade. |

When promoting a feature to stable, the flag key must be explicitly deleted to avoid orphan storage:

```rust
env.storage().instance().remove(&FeatureKey::Flag(symbol_short!("my_feature")));
```

## Consequences

- **Storage overhead**: one `bool` per flag in instance storage. Cost is negligible; instance storage is already bumped on every write.
- **Gas overhead**: one additional storage read per guarded call site. Acceptable for infrequently-toggled paths; avoid placing guards inside tight loops.
- **Absent == disabled**: simplifies circuit-breaker semantics but means new deployments must explicitly enable features intended to be on by default.
- **Admin dependency**: flag writes require the admin key to be live. Contracts that burn their admin (see ADR-0003) cannot toggle flags post-deployment — this is intentional for fully immutable contracts.
- **Orphan storage risk**: flags not removed before the next upgrade persist indefinitely. The deprecation path above mitigates this; code review should verify flag cleanup in upgrade PRs.
