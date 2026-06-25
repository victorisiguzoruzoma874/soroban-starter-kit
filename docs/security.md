# Security

## Arbiter Time-Lock

The arbiter time-lock mechanism in the escrow contract is designed to ensure that funds are not released to the seller until the buyer has had a chance to inspect the goods or services. The time-lock is implemented as a `deadline` ledger sequence number, after which the buyer can request a refund.

### Bypassing the Time-Lock

There are no known vulnerabilities that would allow a malicious actor to bypass the time-lock. The `request_refund` function strictly enforces that the current ledger sequence number is greater than the `deadline` before allowing a refund.

### Deadline Extension

The contract includes an `extend_deadline` function that allows the buyer and seller to mutually agree to extend the deadline. This is a feature of the contract and not a vulnerability. It requires the authentication of both the buyer and the seller, so it cannot be triggered unilaterally.

### Multi-Sig Vote Accumulation

The contract supports multi-sig arbiters. In this scenario, a dispute can only be resolved when the required number of arbiters have voted. This mechanism is independent of the time-lock and does not provide a way to bypass it.

### State Machine Bypass

The contract's state machine is designed to prevent invalid state transitions. For example, a refund can only be requested when the contract is in the `Funded` or `Delivered` state. The state machine is enforced by the `require_state` function, which is called by all state-changing functions. There are no known ways to bypass the state machine.