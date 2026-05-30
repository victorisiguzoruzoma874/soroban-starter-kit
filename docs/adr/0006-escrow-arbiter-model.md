# ADR 0006: Escrow Arbiter Model

- Status: Accepted
- Date: 2026-05-29

## Context

The escrow contract in this repository uses an arbiter model where a single designated arbiter can resolve disputes and direct the transfer of funds between buyer and seller. This model simplifies dispute resolution and minimizes on-chain complexity, but concentrates authority in one actor.

## Decision

We accept the single-arbiter model for the escrow contract. The arbiter is a trusted off-chain entity with the authority to resolve escrow disputes and trigger fund settlement on-chain.

## Rationale / Single-arbiter benefits

- Simplicity: implementation and on-chain logic are straightforward, reducing surface area for bugs.
- Low gas and UX cost: fewer on-chain operations and simpler workflows for users.
- Fast dispute resolution: a single, identified actor can act quickly without coordination overhead.

## Trust assumptions

- The arbiter is honest, available, and will follow agreed rules for dispute resolution.
- Arbiter's private key(s) are securely managed and not compromised.
- Participants (buyer, seller) accept the arbiter's authority as part of the escrow terms.

## Known risks

- Centralization risk: a single arbiter can redirect funds or act maliciously.
- Key compromise: if the arbiter's signing key is stolen, attacker can resolve escrows illegitimately.
- Censorship/unavailability: the arbiter may become unreachable, delaying or preventing resolution.
- Governance and accountability: decisions depend on off-chain processes; on-chain auditability is limited to the resolution action but not the reasoning.

## Alternatives considered

- Multi-signature (multi-sig) arbiter group: require multiple independent arbiters to co-sign resolution transactions. Improves decentralization and resilience but increases coordination complexity and gas costs.
- On-chain voting/DAO: let token holders vote to resolve disputes. Offers decentralization but is slow and may be unsuitable for time-sensitive disputes.
- Algorithmic or rule-based auto-resolution: use pre-defined rules to automatically decide outcomes. Avoids trusted parties but may not cover complex disputes and risks unfair automated outcomes.
- Threshold cryptography / MPC: distribute signing power using threshold keys. Provides strong decentralization and single-transaction UX but increases operational complexity.

## Migration path to multi-sig

1. Design a multi-sig contract or adopt an existing, audited multisig wallet pattern compatible with Soroban.
2. Add an upgrade path in the escrow contract allowing the `arbiter` role to be replaced by a `resolution_contract` address (an indirection layer) via an on-chain governance or admin action.
3. Deploy the multi-sig/threshold resolution contract and configure it as the `resolution_contract` in escrow instances.
4. Revoke or rotate the single-arbiter key(s) and publish transition notices for users.
5. For existing escrows, provide a migration window during which disputes can still be appealed to the original arbiter if necessary; after the window, new resolutions must use the multi-sig.

## Mitigations

- Operational controls: use hardware security modules (HSMs) or multisig for arbiter key custody where possible.
- Auditing and transparency: publish arbiter policies, dispute logs, and rationale where feasible to build trust.
- Key rotation and monitoring: enforce regular key rotation, alerts on signing activity, and incident response plans.

## Consequences

- Short-term: easier implementation and lower cost for users and developers.
- Long-term: centralization risks require governance and operational safeguards; plan for migration to a more decentralized resolution mechanism if user needs evolve.

## References

- See related ADRs: [0003 Admin Model](0003-admin-model.md), [0004 Escrow State Machine Design](0004-escrow-state-machine.md)
