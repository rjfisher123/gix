# GIX Services

This directory contains runnable daemon services for the GIX system.

## Services

- **ajr-router**: Mixnet routing service that anonymizes job routing
- **gsee-runtime**: Secure enclave execution runtime for job execution
- **gcam-node**: Auction clearing engine and bridge services

## Architecture

Each service is a standalone binary that:
- Implements a specific GIX component
- Communicates with other services via defined protocols
- Uses shared libraries from `crates/`
- Can run independently or as part of a coordinated system

## Development

Services should be developed according to their specifications in `specs/`. They depend on crates but never define shared functionality that should be in a crate.



