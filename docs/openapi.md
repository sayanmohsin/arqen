# OpenAPI

`OpenApiGenerator` builds an OpenAPI 3.0.3 document from application-registered
operations. Add `GET`, `POST`, `PUT`, `PATCH`, or `DELETE` operations, schemas,
tags, and bearer or API-key security schemes, then expose the resulting JSON
and `swagger_ui_html()` from your router.

Arqen does not infer every application route automatically. Register the
public contract beside the route and test that the generated document matches
the deployed behavior.

Treat the document as an interface for people, SDKs, automation, and agents.
Do not publish internal credentials, implementation-only routes, or schemas
that the server does not actually enforce.
