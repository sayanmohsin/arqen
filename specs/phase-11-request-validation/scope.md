# Scope

Validation happens at the extractor level (before handler runs).
Derive macros generate validation code from struct definitions.
Validation errors follow the error contract format (Phase 09).
Rejections are typed and include field-level details.
Built-in validators: required, min/max length, regex, email, URL, range.
Custom validators are supported via trait implementation.
