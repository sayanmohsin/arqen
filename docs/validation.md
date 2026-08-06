# Request validation

Validation is an explicit application contract. Implement `Validate` for a
request type and use `Validated<T>` at the HTTP boundary so handlers receive a
validated value or a structured `ValidationErrors` response.

The validation helpers cover required values, enum membership, patterns,
cross-field comparisons, and nested values. Validation is not deserialization:
parse the request first, then validate business invariants before calling a
service or enqueueing a job.

There is no derive macro in the current public release. Keep validation rules
close to the request contract and add tests for both failure shapes and
accepted boundary values.
