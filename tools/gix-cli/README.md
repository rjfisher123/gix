# GIX CLI

Command-line interface for the Global Intelligence Exchange (GIX) network.

## Overview

The GIX CLI provides a user-friendly interface for:
- **Wallet Management:** Generate and manage Dilithium3 post-quantum keypairs
- **Job Submission:** Submit inference jobs to the GIX network
- **Network Status:** Query auction statistics and network health
- **Secure Operations:** All submissions cryptographically signed

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/gix-network/gix
cd gix

# Build the CLI
cargo build --release -p gix-cli

# Binary will be at: target/release/gix
```

### Quick Start (Development)

```bash
# Run without installing
cargo run -p gix-cli -- <command>
```

## Quick Start

### 1. Generate a Wallet

```bash
gix keygen
```

This creates `~/.gix/wallet.json` with a new Dilithium3 keypair.

### 2. Start a GCAM Node

In another terminal:

```bash
cargo run --bin gcam-node
```

### 3. Submit a Job

```bash
gix submit examples/job_sample.yaml
```

### 4. Check Status

```bash
gix status
```

## Commands

### `gix keygen`

Generate a new post-quantum cryptographic wallet.

```bash
gix keygen                    # Default location: ~/.gix/wallet.json
gix keygen -o my-wallet.json  # Custom location
```

**Output:**
- Generates Dilithium3 keypair (NIST Level 3 PQC)
- Saves with secure permissions (600 on Unix)
- Displays public key

### `gix submit <job.yaml>`

Submit a job to the GIX auction.

```bash
gix submit job.yaml                           # Basic
gix submit job.yaml -w wallet.json            # Custom wallet
gix submit job.yaml -n http://node:50052      # Remote node
gix submit job.yaml -p 200                    # High priority
```

**Options:**
- `-w, --wallet <path>` - Wallet file (default: `~/.gix/wallet.json`)
- `-n, --node <url>` - GCAM node (default: `http://127.0.0.1:50052`)
- `-p, --priority <0-255>` - Priority (default: 128)

**Process:**
1. Loads job specification from YAML
2. Loads keypair from wallet
3. Creates GXF envelope with metadata
4. Signs envelope with Dilithium3
5. Submits to GCAM via gRPC
6. Displays auction results (SLP match, price, route)

### `gix status`

Query network statistics.

```bash
gix status                         # Local node
gix status -n http://node:50052    # Remote node
```

**Shows:**
- Total auctions processed
- Total matches found
- Trading volume (μGIX)
- Breakdown by precision level
- Breakdown by routing lane

### `gix wallet`

Display wallet information.

```bash
gix wallet                    # Default wallet
gix wallet -f wallet.json     # Custom wallet
```

**Shows:**
- Public key (hex encoded)
- Key sizes (public: 1952 bytes, secret: 4000 bytes)
- Algorithm details

## Job File Format

Jobs are specified in YAML:

```yaml
# Model identifier
model: "llama-3.1-8b"

# Precision level: BF16, FP8, E5M2, or INT8
precision: "BF16"

# KV cache sequence length (required)
kv_cache_seq_len: 2048

# Optional parameters
token_count: 256      # Default: 128
batch_size: 1         # Default: 1
```

### Example Jobs

See the `examples/` directory:
- `job_sample.yaml` - Basic BF16 job
- `job_high_precision.yaml` - Large context, high precision
- `job_low_precision.yaml` - INT8 quantized, fast

## Security

### Wallet Security

Wallets contain your private keys. Keep them secure:

✅ **Best Practices:**
- Store in `~/.gix/` (default)
- Use file permissions 600 (owner read/write only)
- Backup to secure offline storage
- Use separate wallets for dev/prod

❌ **Never:**
- Share wallet files
- Store in public directories
- Commit to version control
- Reuse across machines

### Cryptographic Signatures

All job submissions are signed with Dilithium3:
- **Post-quantum secure:** Resistant to quantum computers
- **NIST Level 3:** Strong security (roughly 192-bit classical security)
- **Deterministic:** Same message always produces same signature
- **Verifiable:** GCAM nodes can verify submitter identity

## Architecture

```
┌─────────────┐
│  GIX CLI    │
│   (User)    │
└──────┬──────┘
       │
       │ gRPC (tonic)
       │
       ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ GCAM Node   │────▶│ AJR Router  │────▶│ GSEE Runtime│
│ (Auction)   │     │ (Routing)   │     │ (Execution) │
└─────────────┘     └─────────────┘     └─────────────┘
```

## Dependencies

### Runtime
- `clap` - CLI argument parsing
- `tokio` - Async runtime
- `tonic` - gRPC client
- `serde_yaml` - Job file parsing
- `colored` - Terminal formatting

### GIX Crates
- `gix-common` - Common types
- `gix-crypto` - Dilithium3 signatures
- `gix-gxf` - GXF envelope format
- `gix-proto` - gRPC protocol definitions

## Development

### Building

```bash
cargo build -p gix-cli
```

### Testing

```bash
# Unit tests
cargo test -p gix-cli

# Integration test (requires running GCAM node)
./test_cli.sh
```

### Adding Commands

1. Add variant to `Commands` enum in `src/main.rs`
2. Implement handler function
3. Add match arm in `main()`
4. Update documentation

## Troubleshooting

### "Wallet not found"

```bash
gix keygen  # Generate new wallet
```

### "Failed to connect to GCAM node"

```bash
# Ensure GCAM is running:
cargo run --bin gcam-node

# Or specify remote node:
gix submit job.yaml --node http://remote:50052
```

### "Invalid YAML"

Check your job file:
- Valid YAML syntax
- Required fields present (`model`, `precision`, `kv_cache_seq_len`)
- Valid precision level (BF16, FP8, E5M2, INT8)

### "Permission denied" (Unix)

```bash
chmod 600 ~/.gix/wallet.json
```

## Roadmap

Future enhancements:
- [ ] Multi-wallet management
- [ ] Job history tracking
- [ ] Automatic node discovery
- [ ] Configuration file support
- [ ] Interactive mode
- [ ] Batch job submission
- [ ] Result retrieval
- [ ] Cost estimation before submission

## Contributing

See the main [GIX repository](https://github.com/gix-network/gix) for contribution guidelines.

## License

Apache-2.0 OR MIT

## Support

- **Documentation:** See `docs/` directory
- **Examples:** See `examples/` directory
- **Issues:** GitHub issue tracker
- **Discussions:** GitHub discussions

---

**Version:** 0.1.0  
**Status:** Production Ready  
**Last Updated:** December 6, 2025


