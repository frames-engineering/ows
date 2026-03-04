# 06 - Agent Access Layer

> How AI agents, CLI tools, and applications access LWS wallets through MCP, REST, and library interfaces.

## Design Decision

**LWS exposes wallet operations through two access modes — an MCP server (for AI agents) and native language bindings (for programmatic use) — both backed by the same core Rust implementation. The MCP interface is the primary agent access path.**

### Why MCP as the Primary Agent Interface

The [Model Context Protocol](https://modelcontextprotocol.io/) (MCP), open-sourced by Anthropic in November 2024, has become the de facto standard for how AI agents invoke tools. Every major agent framework supports it:

| Framework | MCP Support |
|---|---|
| Claude (Anthropic) | Native |
| OpenAI Agents SDK | Native (March 2025) |
| Coinbase AgentKit | `getMcpTools()` helper |
| LangChain | MCP tool adapter |
| CrewAI | Via LangChain tools |
| OpenClaw | Native MCP integration |

By exposing LWS as an MCP server, any MCP-capable agent can access local wallets without custom integration code. The agent discovers available tools via the MCP `listTools` method and invokes them via `executeTool`.

## MCP Server Tools

The LWS MCP server exposes the following tools:

### `lws_list_wallets`

List all wallets in the vault (no sensitive data exposed).

```json
{
  "name": "lws_list_wallets",
  "description": "List all wallets in the local wallet vault",
  "inputSchema": {
    "type": "object",
    "properties": {
      "chainType": {
        "type": "string",
        "description": "Filter by chain type (evm, solana, etc.)",
        "enum": ["evm", "solana", "tron", "cosmos", "bitcoin"]
      }
    }
  }
}
```

**Returns:** Array of `WalletDescriptor` objects (id, name, accounts — never key material). If the caller is an API key, only wallets in the key's `walletIds` scope are returned.

### `lws_sign_transaction`

Sign a transaction (with policy enforcement).

```json
{
  "name": "lws_sign_transaction",
  "description": "Sign a transaction using a wallet. Enforces attached policies.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "walletId": { "type": "string" },
      "chainId": { "type": "string", "description": "CAIP-2 chain ID" },
      "transaction": {
        "type": "object",
        "description": "Chain-specific transaction object"
      }
    },
    "required": ["walletId", "chainId", "transaction"]
  }
}
```

### `lws_sign_and_send`

Sign and broadcast a transaction.

```json
{
  "name": "lws_sign_and_send",
  "description": "Sign and broadcast a transaction, waiting for confirmation",
  "inputSchema": {
    "type": "object",
    "properties": {
      "walletId": { "type": "string" },
      "chainId": { "type": "string" },
      "transaction": { "type": "object" },
      "confirmations": { "type": "number", "default": 1 }
    },
    "required": ["walletId", "chainId", "transaction"]
  }
}
```

### `lws_sign_message`

Sign an arbitrary message for authentication or attestation.

```json
{
  "name": "lws_sign_message",
  "description": "Sign a message with a wallet (for authentication, not transactions)",
  "inputSchema": {
    "type": "object",
    "properties": {
      "walletId": { "type": "string" },
      "chainId": { "type": "string" },
      "message": { "type": "string" },
      "typedData": { "type": "object", "description": "EIP-712 typed data (EVM only)" }
    },
    "required": ["walletId", "chainId", "message"]
  }
}
```

### `lws_get_policy`

View the policies attached to the caller's API key. Returns policy metadata (name, executable path, config) for each attached policy. If the caller is the owner (sudo), returns an empty array (no policies are evaluated for owners).

```json
{
  "name": "lws_get_policy",
  "description": "Get the policies attached to the caller's API key (name, executable, config). Returns empty for owner access.",
  "inputSchema": {
    "type": "object",
    "properties": {},
    "required": []
  }
}
```

## MCP Server Configuration

Agents configure the LWS MCP server in their MCP settings. The MCP server is scoped to an API key via the `LWS_API_KEY` environment variable. If no key is provided, the MCP server requires passphrase unlock and operates in owner (sudo) mode.

> **Security:** API keys MUST be passed via the `LWS_API_KEY` environment variable, NOT as command-line arguments. Command-line arguments are visible to all local users via `ps aux` and similar process-listing tools, which would expose the raw API key token.

```json
{
  "mcpServers": {
    "lws": {
      "command": "lws",
      "args": ["serve", "--mcp"],
      "env": {
        "LWS_API_KEY": "lws_key_a1b2c3d4e5f6...",
        "LWS_VAULT_PATH": "~/.lws"
      }
    }
  }
}
```

For owner (sudo) access — no API key, requires passphrase:

```json
{
  "mcpServers": {
    "lws": {
      "command": "lws",
      "args": ["serve", "--mcp"]
    }
  }
}
```

## Native Language Bindings

For non-MCP consumers (scripts, web apps, CLIs), LWS provides native bindings that call directly into the Rust `lws-lib` crate via FFI. No HTTP server or subprocess is required — the bindings are compiled native modules that run in-process.

### Node.js (NAPI)

```bash
npm install @lws/node
```

```typescript
import { createWallet, listWallets, signMessage, signTransaction, signAndSend } from "@lws/node";

// Create a wallet
const wallet = createWallet("agent-treasury", "evm", "my-passphrase");
// => { id, name, chain, address, derivation_path, created_at }

// List all wallets
const wallets = listWallets();

// Sign a message
const sig = signMessage("agent-treasury", "evm", "hello", "my-passphrase");
// => { signature, recoveryId? }

// Sign and broadcast a transaction
const result = signAndSend("agent-treasury", "evm", "<tx-hex>", "my-passphrase");
// => { txHash }
```

### Python (PyO3)

```bash
pip install lws
```

```python
from lws import create_wallet, list_wallets, sign_message, sign_transaction, sign_and_send

# Create a wallet
wallet = create_wallet("agent-treasury", "evm", "my-passphrase")
# => {"id", "name", "chain", "address", "derivation_path", "created_at"}

# List all wallets
wallets = list_wallets()

# Sign a message
sig = sign_message("agent-treasury", "evm", "hello", "my-passphrase")
# => {"signature", "recovery_id"}

# Sign and broadcast a transaction
result = sign_and_send("agent-treasury", "evm", "<tx-hex>", "my-passphrase")
# => {"tx_hash"}
```

### Available Functions

Both bindings expose the same 13 functions:

| Function | Description |
|---|---|
| `generate_mnemonic(words?)` | Generate a BIP-39 mnemonic (12 or 24 words) |
| `derive_address(mnemonic, chain, index?)` | Derive a chain-specific address from a mnemonic |
| `create_wallet(name, chain, passphrase, words?, vault_path?)` | Create a new wallet (generates mnemonic, encrypts, saves) |
| `import_wallet_mnemonic(name, chain, mnemonic, passphrase, index?, vault_path?)` | Import a wallet from a mnemonic |
| `import_wallet_private_key(name, chain, key_hex, passphrase, vault_path?)` | Import a wallet from a raw private key |
| `list_wallets(vault_path?)` | List all wallets in the vault |
| `get_wallet(name_or_id, vault_path?)` | Get a single wallet by name or ID |
| `delete_wallet(name_or_id, vault_path?)` | Delete a wallet |
| `export_wallet(name_or_id, passphrase, vault_path?)` | Export a wallet's secret (mnemonic or private key) |
| `rename_wallet(name_or_id, new_name, vault_path?)` | Rename a wallet |
| `sign_transaction(wallet, chain, tx_hex, passphrase, index?, vault_path?)` | Sign a transaction |
| `sign_message(wallet, chain, message, passphrase, encoding?, index?, vault_path?)` | Sign a message |
| `sign_and_send(wallet, chain, tx_hex, passphrase, index?, rpc_url?, vault_path?)` | Sign and broadcast a transaction |

All functions operate on the default vault (`~/.lws/`) unless a custom `vault_path` is provided. The passphrase is used to decrypt wallet key material for signing operations.

> **Note:** Because the bindings run in-process, key material is decrypted within the application's address space. For agent use cases where key isolation is critical, the MCP interface (which uses subprocess isolation) is recommended.

## Access Layer Comparison

| Mode | Key Isolation | Latency | Best For |
|---|---|---|---|
| MCP Server | Full (subprocess) | ~50ms overhead | AI agents (Claude, GPT, etc.) |
| Native Bindings | In-process (no isolation) | Minimal | Scripts, CLIs, embedded applications |

## Agent Interaction Example

Here's how an AI agent interacts with LWS through MCP using an API key. The MCP server was started with `LWS_API_KEY` set, scoping the agent to specific wallets and policies.

```
Agent: "I need to send 0.01 ETH to 0x4B08... on Base"

1. Agent calls lws_list_wallets to find available wallets
   → Returns only wallets in the API key's scope
   → [{ id: "3198bc9c-...", name: "agent-treasury", ... }]

2. Agent calls lws_sign_and_send to execute
   → API key verified: wallet is in key's scope
   → Policy engine evaluates the API key's attached policies
   → Signing enclave decrypts key, signs, wipes
   → Transaction broadcast to Base RPC
   → Returns: { transactionHash: "0xabc...", status: "confirmed" }
```

At no point does the agent see the private key. The API key determines which wallets the agent can access, and the policies attached to the key constrain what operations are permitted.

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [Coinbase AgentKit MCP Integration](https://github.com/coinbase/agentkit)
- [Base MCP Server](https://github.com/base/base-mcp)
- [Phala TEE MCP Wallet](https://phala.com/posts/developer-guide-securely-deploy-a-crypto-wallet-mcp-server-on-phala-cloud)
- [OpenClaw MCP Integration](https://ppaolo.substack.com/p/openclaw-system-architecture-overview)
- [Google Cloud: MCP with Web3](https://cloud.google.com/blog/products/identity-security/using-mcp-with-web3-how-to-secure-blockchain-interacting-agents)
- [Privy Server Wallet REST API](https://docs.privy.io/guide/server-wallets/create)
