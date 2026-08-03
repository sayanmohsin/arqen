# Tasks

- [ ] Define `Validate` trait for validation
- [ ] Create `Validated<T>` extractor that validates on extraction
- [ ] Implement derive macro `#[derive(Validate)]`
- [ ] Add built-in validators (required, min, max, regex, email, url)
- [ ] Create `ValidationRejection` type with field errors
- [ ] Implement `IntoResponse` for ValidationRejection (error contract)
- [ ] Add custom validator support via trait implementation
- [ ] Add validation tests (extractors, derive, rejections)
- [ ] Update examples to use validation
