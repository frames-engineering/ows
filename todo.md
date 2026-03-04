# LWS: Docs vs Implementation Audit

## Legend

- **Match** = implemented as documented
- **Partial** = partially implemented
- **Missing** = documented but not implemented
- **Divergent** = implemented differently than documented
- **Extra** = implemented but not in docs

---

## 01 — Storage Format

| Spec Item | Status | Details |
|-----------|--------|---------|
| Vault dir `~/.lws/` | **Match** | Default in `Config::default()` |
| `wallets/` subdir | **Match** | Created with 0o700 perms |
| `keys/` subdir | **Missing** | No API key file management implemented |
| `policies/` subdir | **Missing** | No policy file management implemented |
| `plugins/` subdir | **Missing** | No plugin loading implemented |
| `logs/audit.jsonl` | **Partial** | Only `wallet_created` events logged; no sign/policy/delete events |
| `config.json` | **Match** | `Config::load_or_default()` works |
| Dir perms 700 / file perms 600 | **Match** | Enforced in `vault.rs` |
| Wallet file: `lws_version` | **Match** | Set to `1` |
| Wallet file: `id` (UUID v4) | **Match** | `WalletId::new()` uses `uuid::Uuid::new_v4()` |
| Wallet file: `name` | **Match** | |
| Wallet file: `created_at` | **Match** | ISO 8601 |
| Wallet file: `chain_type` | **Match** | Enum serializes to lowercase |
| Wallet file: `accounts` array | **Partial** | Always exactly 1 account; multi-account not supported |
| Wallet file: `crypto` envelope | **Match** | AES-256-GCM + scrypt |
| Wallet file: `key_type` | **Match** | `Mnemonic` or `PrivateKey` |
| Wallet file: `metadata` | **Partial** | Field exists, always `{}` |
| AES-256-GCM cipher | **Match** | 12-byte nonce, auth tag |
| AES-128-CTR compat | **Missing** | Only GCM supported |
| Scrypt params (n=262144, r=8, p=1) | **Match** | Hardcoded in `crypto.rs` |
| PBKDF2 KDF option | **Missing** | Only scrypt supported |
| Passphrase min 12 chars | **Match** | Enforced in `ops.rs` |
| API key file format | **Missing** | `ApiKey` type exists in core but no CRUD/storage |
| Audit log format | **Partial** | Only creation; missing `policy_result`, most fields |
| Passphrase from keychain | **Missing** | Only env var or interactive prompt |
| Passphrase from file descriptor | **Missing** | Not implemented |
| `LWS_PASSPHRASE` env var | **Match** | Read in CLI `vault.rs` |
| Keystore v3 import | **Missing** | No `--format keystore` in CLI |

---

## 02 — Chain-Agnostic Addressing (CAIP)

| Spec Item | Status | Details |
|-----------|--------|---------|
| CAIP-2 `ChainId` parsing | **Match** | `namespace:reference` with validation |
| CAIP-10 `AccountId` parsing | **Match** | `namespace:reference:address` |
| EVM namespace `eip155` | **Match** | |
| Solana namespace `solana` | **Match** | |
| Cosmos namespace `cosmos` | **Match** | |
| Bitcoin namespace `bip122` | **Match** | |
| Tron namespace `tron` | **Match** | |
| Polkadot namespace | **Missing** | Not in `ChainType` enum |
| Shorthand aliases (`base` → `eip155:8453`) | **Missing** | No alias resolution |
| CAIP-27 method invocation | **Missing** | No JSON-RPC routing layer |
| Asset identification (CAIP-19) | **Missing** | No asset ID type |

---

## 03 — Signing Interface

| Spec Item | Status | Details |
|-----------|--------|---------|
| `sign()` operation | **Match** | `sign_transaction()` in `ops.rs` |
| `signAndSend()` operation | **Match** | `sign_and_send()` in `ops.rs` |
| `signMessage()` operation | **Match** | `sign_message()` in `ops.rs` |
| `SignRequest.walletId` | **Match** | |
| `SignRequest.chainId` (CAIP-2) | **Divergent** | CLI takes chain *type* (`evm`), not CAIP-2 chain ID |
| `SignRequest.transaction` as object | **Divergent** | Takes raw hex bytes, not structured chain-specific JSON object |
| `SignResult.signature` | **Match** | Hex string |
| `SignResult.signedTransaction` | **Divergent** | Not returned; only `signature` + `recovery_id` |
| `SignAndSendResult.transactionHash` | **Match** | `SendResult.tx_hash` |
| `SignAndSendResult.blockNumber` | **Missing** | Not returned |
| `SignAndSendResult.status` | **Missing** | No confirmation waiting |
| `maxRetries` parameter | **Missing** | Not implemented |
| `confirmations` parameter | **Missing** | No confirmation waiting |
| `signMessage` with `typedData` (EIP-712) | **Partial** | EIP-712 hashing implemented in signer, but CLI `--typed-data` flag not fully integrated |
| Message `encoding` options | **Partial** | `utf8` and `hex` only; spec also mentions `Uint8Array` |
| EVM tx format (structured JSON) | **Missing** | Takes raw hex, doesn't build from structured fields |
| Solana tx format (instructions array) | **Missing** | Takes raw hex |
| Cosmos tx format (messages array) | **Missing** | Takes raw hex |
| Auto-fill nonce/gasLimit/blockhash | **Missing** | No RPC queries for tx building |
| Error: `WALLET_NOT_FOUND` | **Match** | |
| Error: `CHAIN_NOT_SUPPORTED` | **Match** | |
| Error: `POLICY_DENIED` | **Partial** | Error variant exists, no evaluation |
| Error: `INSUFFICIENT_FUNDS` | **Partial** | Error variant exists, never produced |
| Error: `INVALID_PASSPHRASE` | **Match** | |
| Error: `VAULT_LOCKED` | **Partial** | Error variant exists, no lock mechanism |
| Error: `BROADCAST_FAILED` | **Match** | |
| Error: `TIMEOUT` | **Partial** | Error variant exists, never produced |
| Per-wallet mutex (concurrency) | **Missing** | No concurrency control |

---

## 04 — Policy Engine

| Spec Item | Status | Details |
|-----------|--------|---------|
| Two-tier access (owner vs agent) | **Missing** | All access is owner-level |
| Policy file format | **Missing** | `Policy` type in core has `id`, `name`, `executable`, `timeout_ms` but no `version`, `config`, `action` fields |
| `PolicyContext` struct | **Partial** | Type exists with `wallet_id`, `chain`, `transaction`, `api_key_id` but no `wallet` object, no `timestamp` |
| `PolicyResult` struct | **Divergent** | Has `action: PolicyAction` (Allow/Deny) instead of `allow: bool` |
| Policy executable protocol (stdin/stdout) | **Missing** | No execution logic |
| 5-second timeout | **Missing** | `timeout_ms` field exists but unused |
| Deny-by-default on failure | **Missing** | |
| AND semantics for multiple policies | **Missing** | |
| `lws policy create` command | **Missing** | |
| `lws key create` command | **Missing** | |
| `warn` vs `deny` actions | **Missing** | |
| Policy attachment to API keys | **Missing** | |

---

## 05 — Key Isolation

| Spec Item | Status | Details |
|-----------|--------|---------|
| Signing enclave (subprocess) | **Missing** | Signing happens in-process |
| Unix domain socket (`enclave.sock`) | **Missing** | No IPC |
| JSON-RPC enclave protocol | **Missing** | |
| `SecretBytes` with zeroize-on-drop | **Match** | Implemented in `zeroizing.rs` |
| `mlock()` for memory pages | **Match** | In `SecretBytes` and `process_hardening.rs` |
| Disable core dumps | **Match** | `prctl`/`setrlimit` |
| Disable ptrace | **Match** | `PT_DENY_ATTACH` on macOS |
| Signal handlers (SIGTERM/SIGINT/SIGHUP) | **Match** | Clears key cache |
| Key cache (TTL ≤30s, max 32) | **Match** | 5s TTL, 32 entries |
| Passphrase via file descriptor | **Missing** | |
| Clear `LWS_PASSPHRASE` after read | **Partial** | `clear_env_var()` exists in process_hardening, unclear if called |

---

## 06 — Agent Access Layer

| Spec Item | Status | Details |
|-----------|--------|---------|
| Library SDK | **Partial** | `lws-lib` exists but interface differs from spec |
| Node.js bindings | **Match** | `bindings/node/` via NAPI |
| Python bindings | **Match** | `bindings/python/` via PyO3 |

---

## 07 — Multi-Chain Support

| Spec Item | Status | Details |
|-----------|--------|---------|
| `ChainPlugin` trait | **Divergent** | `ChainSigner` trait with different methods (`sign`, `sign_message`, `sign_transaction`, `derive_address`) vs spec's `buildTransaction`, `broadcast`, etc. |
| EVM signer (secp256k1, coin 60) | **Match** | |
| Solana signer (ed25519, coin 501) | **Match** | |
| Bitcoin signer (secp256k1, coin 0) | **Match** | |
| Cosmos signer (secp256k1, coin 118) | **Match** | |
| Tron signer (secp256k1, coin 195) | **Match** | |
| EVM derivation `m/44'/60'/0'/0/{i}` | **Match** | |
| Solana derivation `m/44'/501'/{i}'/0'` | **Match** | |
| Bitcoin derivation `m/84'/0'/0'/0/{i}` | **Match** | |
| Cosmos derivation `m/44'/118'/0'/0/{i}` | **Match** | |
| Tron derivation `m/44'/195'/0'/0/{i}` | **Match** | |
| EVM address (EIP-55 checksum) | **Match** | |
| Solana address (Base58 Ed25519) | **Match** | |
| Bitcoin address (Bech32 segwit) | **Match** | |
| Cosmos address (Bech32) | **Match** | |
| Tron address (Base58Check, T-prefix) | **Match** | |
| Plugin discovery (npm packages) | **Missing** | No plugin system |
| External plugin registration | **Missing** | |
| RPC config in `config.json` | **Match** | Default RPCs + override |
| Per-request `rpcUrl` override | **Match** | `--rpc-url` flag |
| HD derivation (BIP-39/32/44) | **Match** | |
| EIP-712 typed data | **Match** | Full implementation in `eip712.rs` |
| ADR-036 (Cosmos off-chain signing) | **Match** | `sign_message_adr036()` |
| Solana commitment mapping | **Missing** | No confirmation waiting |
| Transaction building from structured JSON | **Missing** | Signing only accepts raw bytes |

---

## 08 — Wallet Lifecycle

| Spec Item | Status | Details |
|-----------|--------|---------|
| `lws wallet create` | **Match** | Works with `--name`, `--chain`, `--words` |
| `lws wallet import` (mnemonic) | **Match** | Via `--mnemonic` flag |
| `lws wallet import` (private key) | **Match** | Via `--private-key` flag |
| `lws wallet import` (keystore v3) | **Missing** | No `--format keystore` |
| `lws wallet import` (WIF) | **Missing** | No `--format wif` |
| `lws wallet import` (Solana keypair) | **Missing** | No `--format solana-keypair` |
| `lws wallet export` | **Match** | Returns mnemonic or private key |
| `lws wallet export --format keystore` | **Missing** | Only raw export |
| `lws wallet export --format raw --account` | **Missing** | No per-account export |
| `lws wallet delete` | **Match** | With `--confirm` flag |
| Secure overwrite before unlink | **Missing** | Simple file deletion |
| Remove wallet from API keys on delete | **Missing** | No API key management |
| `lws wallet list` | **Match** | |
| `lws wallet rename` | **Match** | Extra — not in spec |
| `lws wallet info` | **Match** | Extra — not in spec |
| `lws backup` | **Missing** | `BackupConfig` type exists, no implementation |
| `lws restore` | **Missing** | |
| Automated backup | **Missing** | |
| `lws wallet recover` (gap limit scan) | **Missing** | |
| `lws wallet rotate` (asset transfer) | **Missing** | |
| Wallet discovery (glob patterns) | **Missing** | |
| `lws mnemonic generate` | **Match** | Extra — not explicitly in spec |
| `lws mnemonic derive` | **Match** | Extra — not explicitly in spec |
| `lws config show` | **Match** | Extra — not explicitly in spec |
| `lws update` | **Match** | Extra — not in spec |
| `lws uninstall` | **Match** | Extra — not in spec |
| `--show-mnemonic` on create | **Match** | Extra — spec says mnemonic never leaves enclave |

---

## Key Divergences

### 1. No signing enclave / subprocess isolation

The biggest architectural gap. The spec calls for a separate child process communicating over a Unix domain socket. Currently, signing happens in the same process as the CLI. The security model documented in docs 05 and 06 isn't enforced.

### 2. No policy engine

Types exist but zero evaluation logic. No executable protocol, no API key scoping, no deny-by-default enforcement.

### 3. Raw bytes vs structured transactions

The spec describes structured transaction objects (JSON with `to`, `value`, `data` for EVM; `instructions` for Solana). The implementation takes pre-serialized hex bytes, pushing transaction construction entirely onto the caller.

### 4. No transaction building or simulation

Spec describes auto-filling nonce, gas, blockhash via RPC queries. Implementation doesn't query RPCs for building — only for broadcasting.

### 5. Single-account wallets only

Spec supports multiple accounts per wallet (multi-chain from one mnemonic). Implementation always derives exactly one account at index 0 (or specified index).

### 6. CLI takes chain type, not CAIP-2 ID

`--chain evm` instead of `--chain eip155:8453`. The CAIP types exist but aren't used in the CLI/lib interface.

---

## Implementation Quality Notes

### Strong areas

- Cryptographic implementation is solid (scrypt, AES-256-GCM, BIP-39/32/44, EIP-712)
- All 5 chain signers pass test vectors
- `SecretBytes` with mlock + zeroize is well-implemented
- Process hardening (core dumps, ptrace, signal handlers) is thorough
- Error types are comprehensive and map cleanly to spec error codes

### Priority TODO (ordered)

1. **Policy engine** — core security feature for agents
2. **API key management** — prerequisite for policies
3. **Signing enclave subprocess** — key isolation guarantee
4. **Structured transaction building** — usability for callers
5. **Multi-account wallet support** — spec compliance
6. **Backup/restore** — operational safety
7. **Keystore v3 / WIF import formats** — ecosystem compatibility
