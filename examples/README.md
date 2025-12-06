# GIX Example Job Files

This directory contains sample job specifications for submitting to the GIX network.

## Available Examples

### job_sample.yaml
Basic example job with BF16 precision and moderate parameters.
```bash
gix submit examples/job_sample.yaml
```

### job_high_precision.yaml
High-precision job using BF16 with larger context window.
```bash
gix submit examples/job_high_precision.yaml
```

### job_low_precision.yaml
Optimized job using INT8 quantization for faster/cheaper execution.
```bash
gix submit examples/job_low_precision.yaml
```

## Job Specification Format

Jobs are specified in YAML with the following fields:

```yaml
# Model identifier
model: "model-name"

# Precision level: BF16, FP8, E5M2, or INT8
precision: "BF16"

# KV cache sequence length (required)
kv_cache_seq_len: 2048

# Token count (optional, defaults to 128)
token_count: 256

# Batch size (optional, defaults to 1)
batch_size: 1
```

## Precision Levels

- **BF16**: Brain Float 16 - Highest quality, slower
- **FP8**: Float 8 - Good balance of quality and speed
- **E5M2**: 8-bit with 5-bit exponent - Specialized format
- **INT8**: 8-bit integer - Fastest, lowest quality

## Usage

1. Create or edit a job YAML file
2. Generate a wallet if you haven't: `gix keygen`
3. Submit the job: `gix submit path/to/job.yaml`
4. Check status: `gix status`


