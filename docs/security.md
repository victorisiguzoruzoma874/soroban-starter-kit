# Security Best Practices

> See also: [SECURITY.md](../SECURITY.md) for vulnerability reporting.

This guide covers safe integration patterns, key management, and known attack
vectors for the Token and Escrow contracts in this repository.

---

## 1. Trust Model

### Token Contract

| Actor | Trusted for |
|-------|-------------|
| Admin | Minting new tokens, burning tokens, transferring admin role |
| Token holder | Approving allowances, transferring their own balance |
| Spender | Spending up to the approved allowance on behalf of a holder |

The admin key is the single most sensitive credential. Compromise of the admin
key allows unlimited minting and therefore complete devaluation of the token.

### Escrow Contract

| Actor | Trusted for |
|-------|-------------|
| Buyer | Funding the escrow, approving delivery, requesting a refund after deadline, cancelling before funding |
| Seller | Marking goods/services as delivered |
| Arbiter | Resolving disputes by releasing funds to either party |

No single actor can unilaterally move funds without the contract enforcing the
correct state machine transition.

---

## 2. Admin Key Management

- **Use a hardware wallet** (Ledger, Trezor) or a multi-sig account for the
  token admin key. A single hot-wallet key is not acceptable for production.
- **Key rotation** — use the two-step `propose_admin` / `accept_admin` flow to
  rotate the admin key. The pending admin must explicitly call `accept_admin`
  before the transfer takes effect, preventing accidental loss of admin access
  from a typo in the new address.
- **Never reuse keys** across environments. Use separate deployer accounts for
  local, testnet, and mainnet.
- **Revoke access immediately** when a team member leaves. Rotate the admin key
  via `propose_admin` before the old key is considered compromised.
- **Escrow arbiter** — the arbiter address is set at initialization and cannot
  be changed. Choose an arbiter that is independent of both buyer and seller.
  A multi-sig arbiter is strongly recommended for high-value escrows.

---

## 3. Never Log or Transmit Private Keys

- Do **not** store secret keys in `.env` files committed to version control.
  Use `.env.example` with placeholder values and add `.env` to `.gitignore`.
- Do **not** log secret keys in CI/CD pipelines. Store them as encrypted
  secrets (e.g., GitHub Actions secrets) and reference them by name only.
- Do **not** transmit secret keys over HTTP, WebSockets, or any unencrypted
  channel.
- Use Freighter, Albedo, or another browser wallet for frontend signing so the
  secret key never leaves the user's device.
- Audit your dependencies with `cargo audit` and `npm audit` regularly to catch
  packages that may exfiltrate secrets.

---

## 4. Replay Attack Prevention

Soroban transactions include the ledger sequence number of the source account,
which the network increments after every successful transaction. This makes
replaying a previously signed transaction impossible once the sequence number
has advanced.

**Integrator checklist:**

- Always fetch a fresh account sequence number with `server.getAccount()` before
  building a transaction. Cached sequence numbers cause `tx_bad_seq` errors and
  can, in edge cases, allow a stale transaction to be resubmitted.
- Set a tight `setTimeout` (30–60 seconds) on every transaction so that a
  delayed broadcast cannot be replayed after the window closes.
- For allowance-based flows (`approve` + `transfer_from`), set
  `expiration_ledger` to the minimum ledger at which the allowance is no longer
  needed. An open-ended allowance is a standing replay vector.

---

## 5. Front-Running Considerations for Escrow Deadlines

Escrow deadlines are expressed as ledger sequence numbers. Because ledger close
times vary (~5 s average), a deadline expressed in ledgers is approximate.

**Risks:**

- A seller can observe a pending `request_refund` transaction in the mempool and
  attempt to front-run it with `mark_delivered` before the refund is confirmed.
- A buyer can observe a pending `approve_delivery` and attempt to cancel or
  request a refund in the same ledger.

**Mitigations:**

- Add a generous buffer (at least 100–200 ledgers ≈ 8–16 minutes) beyond the
  expected delivery time when setting `deadline_ledger`.
- Use the arbiter flow for high-value escrows so that neither party can
  unilaterally resolve a disputed state.
- Monitor the mempool and use fee bumping to prioritize time-sensitive
  transactions.

---

## 6. Safe Upgrade Patterns

The `upgradeable` feature enables WASM replacement via `propose_upgrade` /
`execute_upgrade`. A timelock (`UPGRADE_DELAY_LEDGERS`) is enforced between
proposal and execution so that users have time to review and exit before a
potentially malicious upgrade takes effect.

**Best practices:**

- Announce upgrades publicly (Discord, Twitter, governance forum) before calling
  `propose_upgrade`. Give users at least as much notice as the timelock duration.
- Verify the new WASM hash on-chain after deployment:
  ```bash
  stellar contract info --id <CONTRACT_ID>
  ```
- Never upgrade directly on mainnet without a full testnet rehearsal.
- Consider a governance vote (DAO or multi-sig) as the gating mechanism for
  `propose_upgrade` rather than a single admin key.
- After a successful upgrade, emit an event and update your monitoring
  infrastructure with the new WASM hash.

---

## 7. Event Monitoring for Anomaly Detection

All state-changing operations emit Soroban events. Subscribe to these events
and alert on unexpected patterns:

| Event topic | Contract | Anomaly signal |
|-------------|----------|----------------|
| `mint` | Token | Large or unexpected mint |
| `burn` | Token | Burn exceeding expected volume |
| `admin_changed` | Token | Any admin change not initiated by your team |
| `upgrade_proposed` | Token / Escrow | Any upgrade proposal |
| `upgrade_executed` | Token / Escrow | Upgrade executed (verify WASM hash) |
| `dispute_raised` | Escrow | Spike in disputes |
| `funds_released` | Escrow | Release to unexpected address |

**Tooling:**

```ts
// Poll for events using the Stellar SDK
const events = await server.getEvents({
  startLedger: lastProcessedLedger,
  filters: [{ contractIds: [TOKEN_CONTRACT_ID], topics: [['mint']] }],
});
```

Set up alerts (PagerDuty, Slack webhook) for any `admin_changed` or
`upgrade_proposed` event that was not initiated by your own infrastructure.

---

## 8. Reentrancy

Soroban contracts execute in a single-threaded, deterministic VM. Cross-contract
calls are synchronous and the host does not allow re-entrant invocations of the
same contract instance within a single transaction. However, the
**Checks → Effects → Interactions** pattern is still followed as a best practice:
state is updated *before* any outbound token transfer.

---

## 9. Storage Expiry Risks

Soroban uses a rent model: storage entries expire if their TTL reaches zero.

- Both contracts bump instance TTL on every write
  (`BUMP_THRESHOLD ≈ 7 days`, `BUMP_AMOUNT ≈ 30 days`).
- If an escrow is ignored for more than ~30 days, instance storage may expire.
  The public `bump()` function can be called by anyone to extend the TTL.
- Integrators should monitor active escrows and call `bump()` proactively.

---

## 10. Known Limitations

| Limitation | Impact | Suggested Mitigation |
|------------|--------|----------------------|
| No arbiter replacement | Compromised arbiter cannot be removed | Deploy a new escrow; add governance layer |
| Single-token escrow | Only one token type per escrow | Deploy multiple escrows for multi-token deals |
| Deadline is ledger-sequence based | Ledger close times vary | Add a generous buffer when setting deadlines |

---

## 11. Threat Model Summary

| Threat | Likelihood | Impact | Mitigated by |
|--------|-----------|--------|--------------|
| Admin key compromise (token) | Low | Critical | Hardware wallet, multi-sig, two-step transfer |
| Arbiter collusion | Medium | High | Reputable arbiter, multi-sig |
| Storage expiry of live escrow | Low | High | `bump()` monitoring |
| Front-running escrow deadline | Low | Medium | Generous deadline buffer, arbiter flow |
| Integer overflow | Very Low | High | `checked_add` / `checked_sub` throughout |
| Reentrancy | Very Low | High | CEI pattern enforced |
| Malicious WASM upgrade | Low | Critical | Upgrade timelock, governance gating |

---

## 12. Reporting Vulnerabilities

Please do **not** open a public GitHub issue for security vulnerabilities.
See [SECURITY.md](../SECURITY.md) for the responsible disclosure process.
