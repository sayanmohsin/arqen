# Vision

Arqen should make it quick to create a small backend that remains readable as
it grows.

The intended workflow is straightforward:

1. generate a small project;
2. add modules for application features;
3. compose routes, tools, jobs, and health checks explicitly;
4. start with memory storage while developing;
5. move to native or HTTP Thingd when durable data is needed;
6. validate the schema, test the deployment, and operate the service with
   visible health and logs.

Arqen is designed for web and mobile backends, internal services, automation,
and applications that expose selected operations to agents. Rust is the
current implementation language. Other clients can use the HTTP API and
machine-readable manifests as those integrations mature.

The goal is a small amount of framework code around ordinary application code:
domain services stay yours, while configuration, storage adapters, jobs,
health, and capability discovery follow consistent patterns.
