# Tooling

Arqen provides recommended tooling configuration for projects that use the
framework. These are not required, but they help maintain code quality and
consistency.

## Rust tooling

Arqen projects use standard Rust tooling. Run these from your project root:

```bash
cargo fmt --all -- --check    # format check
cargo clippy --all-targets --all-features -- -D warnings  # lint
cargo test --all-features     # tests
cargo doc --no-deps           # docs
```

To auto-fix formatting:

```bash
cargo fmt --all
```

## Prettier (Markdown, YAML, JSON)

Prettier formats your documentation, configuration files, and API schemas.
Install it in your project:

```bash
pnpm add -D prettier
```

Create a `.prettierrc.json` in your project root:

```json
{
  "proseWrap": "preserve",
  "printWidth": 100,
  "trailingComma": "all"
}
```

Create a `.prettierignore` to skip build artifacts:

```
target/
node_modules/
pnpm-lock.yaml
```

Add scripts to your `package.json`:

```json
{
  "scripts": {
    "format": "prettier --write .",
    "format:check": "prettier --check ."
  }
}
```

Run:

```bash
pnpm format          # auto-fix
pnpm format:check    # verify only
```

## Markdownlint (Markdown linting)

Markdownlint catches common Markdown issues. Install it:

```bash
pnpm add -D markdownlint-cli2
```

Create a `.markdownlint.json` in your project root:

```json
{
  "MD013": false,
  "MD033": false,
  "MD041": false
}
```

Add a lint script to `package.json`:

```json
{
  "scripts": {
    "lint": "markdownlint-cli2 \"**/*.md\" \"!node_modules/**\" \"!target/**\""
  }
}
```

Run:

```bash
pnpm lint
```

## Complete setup

A minimal `package.json` for an Arqen project with both tools:

```json
{
  "name": "my-arqen-app",
  "private": true,
  "scripts": {
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "lint": "markdownlint-cli2 \"**/*.md\" \"!node_modules/**\" \"!target/**\"",
    "check": "cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features"
  },
  "devDependencies": {
    "prettier": "^3.6.2",
    "markdownlint-cli2": "^0.17.2"
  }
}
```

## CI integration

Add these checks to your CI pipeline:

```yaml
- name: Format check
  run: cargo fmt --all -- --check

- name: Lint
  run: cargo clippy --workspace --all-targets --all-features -- -D warnings

- name: Test
  run: cargo test --workspace --all-features

- name: Docs
  run: cargo doc --workspace --no-deps

- name: Prettier
  working-directory: .
  run: npx prettier --check "**/*.{md,json,yml,yaml}"

- name: Markdownlint
  run: npx markdownlint-cli2 "**/*.md" "!node_modules/**" "!target/**"
```
