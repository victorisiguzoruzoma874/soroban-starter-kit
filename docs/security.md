# Security

## Authorization

| Contract | Function | Authorization |
| --- | --- | --- |
| Escrow | `initialize` | Anyone |
| Escrow | `initialize_with_arbiters` | Anyone |
| Escrow | `update_amount` | Buyer |
| Escrow | `fund` | Buyer |
| Escrow | `mark_delivered` | Seller |
| Escrow | `approve_delivery` | Buyer |
| Escrow | `release_partial` | Arbiter |
| Escrow | `request_refund` | Buyer |
| Escrow | `raise_dispute` | Buyer or Seller |
| Escrow | `resolve_dispute` | Arbiter |
| Escrow | `cancel` | Buyer |
| Escrow | `extend_deadline` | Buyer and Seller |
| Escrow | `bump` | Anyone |
| Escrow | `get_escrow_info` | Anyone |
| Escrow | `get_state` | Anyone |
| Escrow | `is_deadline_passed` | Anyone |
| Escrow | `get_remaining_ledgers` | Anyone |
| Escrow | `pause` | Admin |
| Escrow | `unpause` | Admin |
| Escrow | `propose_upgrade` | Admin |
| Escrow | `execute_upgrade` | Admin |
| Token | `initialize` | Admin |
| Token | `mint` | Admin |
| Token | `batch_mint` | Admin |
| Token | `admin_burn` | Admin |
| Token | `propose_admin` | Admin |
| Token | `accept_admin` | Pending Admin |
| Token | `cancel_admin_proposal` | Admin |
| Token | `set_admin` | Admin |
| Token | `admin` | Anyone |
| Token | `total_supply` | Anyone |
| Token | `balance_of` | Anyone |
| Token | `version` | Anyone |
| Token | `contract_version` | Anyone |
| Token | `allowance_expiry` | Anyone |
| Token | `pause` | Admin |
| Token | `unpause` | Admin |
| Token | `freeze_account` | Admin |
| Token | `unfreeze_account` | Admin |
| Token | `propose_upgrade` | Admin |
| Token | `execute_upgrade` | Admin |
| Token | `max_supply` | Anyone |
| Staking | `initialize` | Admin |
| Staking | `stake` | Staker |
| Staking | `unstake` | Staker |
| Staking | `claim_rewards` | Staker |
| Staking | `add_rewards` | Admin |
| Staking | `get_stake` | Anyone |
| Staking | `get_rewards` | Anyone |
| Staking | `get_total_staked` | Anyone |
| Staking | `get_total_rewards` | Anyone |
| Vesting | `initialize` | Admin |
| Vesting | `claim` | Beneficiary |
| Vesting | `revoke` | Admin |
| Vesting | `get_info` | Anyone |
| Vesting | `claimable` | Anyone |
| Multisig | `initialize` | Signers |
| Multisig | `add_signer` | Threshold of Signers |
| Multisig | `remove_signer` | Threshold of Signers |
| Multisig | `propose_transaction` | Signer |
| Multisig | `sign_transaction` | Signer |
| Multisig | `execute_transaction` | Anyone (but requires threshold of signatures) |
| Multisig | `get_signers` | Anyone |
| Multisig | `get_threshold` | Anyone |
| Multisig | `is_signer` | Anyone |
| Multisig | `get_transaction` | Anyone |
| Multisig | `signature_count` | Anyone |