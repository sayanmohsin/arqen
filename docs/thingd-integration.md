# thingd integration

thingd is a first-class Arqen dependency. It supplies objects, events, search, links, and durable queues.

The first stable boundary is thingd's public HTTP API. Arqen should not import private thingd-cloud internals or require a Node.js SDK.

The adapter boundary should support:

- typed object repositories;
- batch writes;
- append-only events;
- queue push, claim, ack, nack, and dead-letter operations;
- full-text and vector search when enabled;
- links for relationships.

Planned implementations:

```text
ThingdBackend
  +-- MemoryThingdBackend
  +-- HttpThingdBackend
  +-- CloudThingdBackend (optional, future)
```

Switching implementations must not change application domain services.

## Adapter contract

See [adapter-contract.md](adapter-contract.md) for the full trait definition, data types, and implementation details.
