# Test Plan

## Unit Tests
- Validation trait implementation
- Built-in validators (required, min, max, regex, email, url)
- Derive macro code generation
- Custom validator support

## Integration Tests
- Validated extractor validates on extraction
- Invalid requests return validation errors
- Validation errors follow error contract format

## Manual Verification
- Derive macros work with serde
- Validation errors include field details
