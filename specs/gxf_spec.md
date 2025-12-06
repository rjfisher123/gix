# GXF (GIX Exchange Format) ABI v3 Specification

## Status

🚧 Specification in development

## Overview

GXF is the standardized format for job execution envelopes in the GIX system. Version 3 represents the current ABI.

## Structure

- **Version**: Schema version identifier
- **Job ID**: Unique identifier for the job
- **Payload**: Encrypted job data
- **Metadata**: Timestamps, expiration, additional fields

## Serialization

- Format: TBD (likely CBOR or MessagePack)
- Validation: All envelopes must pass schema validation
- Versioning: Backward compatibility rules TBD

## TODO

- Complete schema definition
- Serialization format specification
- Validation rules
- Versioning strategy




