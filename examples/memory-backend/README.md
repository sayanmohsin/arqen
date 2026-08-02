# Memory Backend Example

Demonstrates using the MemoryThingdBackend for in-memory data storage.

## Getting started

```bash
cargo run
```

## Features demonstrated

- Object CRUD operations
- Event append and read
- Job push, claim, and complete
- Link creation and retrieval
- Search functionality
- Reset and seed operations

## Usage

This example shows how to use the MemoryThingdBackend for development and testing. The backend provides:

- In-memory object storage
- Event sourcing
- Job queues with lease management
- Relationship links
- Full-text search

All data is stored in memory and will be lost when the process exits.