# Request validation

Validation is an explicit application contract. Implement `Validate` for a
request type and use `Validated<T>` at the HTTP boundary so handlers receive a
validated value or a structured `ValidationErrors` response.

The validation helpers cover required values, enum membership, patterns,
length and numeric bounds, cross-field comparisons, and nested values.
Validation is not deserialization: parse the request first, then validate
business invariants before calling a service or enqueueing a job.

## Boundary rules

- Use `Validated<T>` at the HTTP boundary.
- Enforce request body limits, pagination limits, and batch-size limits before
  expensive work.
- Return field paths, stable validation codes, and safe messages; never echo
  passwords, tokens, or complete request bodies.
- Normalize paths consistently, such as `profile.email` and
  `items[0].title`.
- Keep cross-field and authorization-sensitive rules in manual `Validate`
  implementations.

Example:

```rust
#[derive(serde::Deserialize)]
struct CreateMovie {
    title: String,
    year: u16,
}

impl Validate for CreateMovie {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.title.trim().is_empty() {
            errors.push(FieldError::new("title", "required", "field is required"));
        } else if let Err(error) = validators::min_length("title", &self.title, 1) {
            errors.push(error);
        }
        if self.year < 1888 {
            errors.push(FieldError::new("year", "min_value", "year is too early"));
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

There is no derive macro in the current public release. Keep validation rules
close to the request contract and add tests for malformed input, missing and
boundary values, nested paths, cross-field failures, oversized requests, and
secret-bearing inputs.
