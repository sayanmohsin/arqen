# Deployment

Deployment guidance should cover a generic Docker host first, then common container platforms and Kubernetes.

Every guide must address:

- release builds;
- environment variables;
- secret management;
- health and readiness checks;
- graceful shutdown;
- worker scaling;
- thingd connectivity and credentials;
- structured log collection;
- durable storage and backup ownership.

Cloud integration is optional. A hosted thingd-cloud adapter must use a documented public customer API, not control-plane databases or private modules.
