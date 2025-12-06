# GIX CLI Quick Reference

**Version:** 0.1.0  
**Binary:** `gix`

---

## Installation

```bash
# Build from source
cargo build --release -p gix-cli

# Run directly
cargo run -p gix-cli -- <command>
```

---

## Commands

### `gix keygen`
Generate a new Dilithium3 keypair.

```bash
gix keygen                      # Save to ~/.gix/wallet.json
gix keygen -o custom.json       # Save to custom location
```

**Output:** Creates wallet file with secure permissions (600)

---

### `gix submit`
Submit a job to the GIX network.

```bash
gix submit <job.yaml>                    # Basic submission
gix submit job.yaml -w wallet.json       # Custom wallet
gix submit job.yaml -n http://node:50052 # Custom node
gix submit job.yaml -p 200               # High priority
```

**Options:**
- `-w, --wallet <path>` - Wallet file (default: `~/.gix/wallet.json`)
- `-n, --node <url>` - GCAM node URL (default: `http://127.0.0.1:50052`)
- `-p, --priority <0-255>` - Job priority (default: 128)

**Requirements:**
- Wallet must exist (run `gix keygen` first)
- GCAM node must be running
- Valid YAML job file

---

### `gix status`
Query auction statistics.

```bash
gix status                          # Query local node
gix status -n http://node:50052     # Query remote node
```

**Shows:**
- Total auctions processed
- Total matches found
- Total volume (in μGIX)
- Matches by precision level
- Matches by lane

---

### `gix wallet`
Display wallet information.

```bash
gix wallet                      # Show default wallet
gix wallet -f custom.json       # Show custom wallet
```

**Shows:**
- Public key (hex)
- Key sizes
- Algorithm (Dilithium3)

---

## Job File Format (YAML)

```yaml
# Model identifier
model: "llama-3.1-8b"

# Precision: BF16, FP8, E5M2, or INT8
precision: "BF16"

# KV cache sequence length (required)
kv_cache_seq_len: 2048

# Optional fields
token_count: 256      # Default: 128
batch_size: 1         # Default: 1
```

---

## Examples

### Complete Workflow

```bash
# 1. Generate wallet (first time only)
gix keygen

# 2. Start GCAM node (in another terminal)
cargo run --bin gcam-node

# 3. Submit a job
gix submit examples/job_sample.yaml

# 4. Check status
gix status
```

### Multiple Submissions

```bash
# Submit different job types
gix submit examples/job_sample.yaml
gix submit examples/job_high_precision.yaml
gix submit examples/job_low_precision.yaml

# Check accumulated stats
gix status
```

### Remote Node

```bash
# Submit to production node
gix submit job.yaml --node https://gix.example.com:50052

# Check production stats
gix status --node https://gix.example.com:50052
```

---

## Precision Levels

| Level | Description | Use Case |
|-------|-------------|----------|
| **BF16** | Brain Float 16 | Highest quality, training |
| **FP8** | Float 8 | Balanced quality/speed |
| **E5M2** | 8-bit (5 exp, 2 mantissa) | Specialized models |
| **INT8** | 8-bit integer | Fastest, inference only |

---

## Wallet Security

### Best Practices

✅ **DO:**
- Keep wallet in `~/.gix/` (default)
- Use `chmod 600` on wallet file
- Backup wallet to secure location
- Use different wallets for dev/prod

❌ **DON'T:**
- Share wallet files
- Store in public directories
- Commit to version control
- Use same wallet on multiple machines

### Wallet File Permissions

```bash
# Check permissions
ls -l ~/.gix/wallet.json

# Should show: -rw------- (600)

# Fix if needed
chmod 600 ~/.gix/wallet.json
```

---

## Troubleshooting

### Wallet Not Found

```
Error: Wallet file not found: ~/.gix/wallet.json

Run 'gix keygen' to create a new wallet.
```

**Solution:** Run `gix keygen`

### Connection Failed

```
Error: Failed to connect to GCAM node
```

**Solutions:**
- Ensure GCAM node is running: `cargo run --bin gcam-node`
- Check node address is correct
- Verify network connectivity
- Check firewall rules

### Invalid YAML

```
Error: Failed to parse job YAML
```

**Solutions:**
- Check YAML syntax
- Ensure all required fields present
- Verify precision level is valid (BF16/FP8/E5M2/INT8)
- Check field types match schema

### Permission Warnings

```
⚠️  Warning: Wallet file has insecure permissions!
   Recommended: chmod 600 /path/to/wallet.json
```

**Solution:** Run `chmod 600 /path/to/wallet.json`

---

## Environment Variables

Currently, the CLI does not use environment variables, but this may be added in future versions:

```bash
# Future:
export GIX_WALLET=~/.gix/wallet.json
export GIX_NODE=http://127.0.0.1:50052
export GIX_PRIORITY=128
```

---

## Exit Codes

- `0` - Success
- `1` - Error (generic)
- `2` - Invalid arguments

---

## Help

```bash
# General help
gix --help

# Command-specific help
gix keygen --help
gix submit --help
gix status --help
gix wallet --help

# Version
gix --version
```

---

## See Also

- **Full Documentation:** `GIX_CLI_COMPLETE.md`
- **Example Jobs:** `examples/README.md`
- **Network Protocol:** `specs/integrated/network_protocol_v0.2.0.md`
- **GCAM Service:** `services/gcam-node/README.md`

---

**Last Updated:** December 6, 2025  
**Version:** 0.1.0

