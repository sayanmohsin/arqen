---
name: fix-github-issues
description: |
  Use when the user says to fix GitHub issues one by one. Covers the full
  workflow: list open issues, pick one, investigate, implement, test, commit,
  and close.
---

# Fix GitHub Issues (One by One)

## Workflow

### 1. List all open issues

```bash
gh issue list --repo sayanmohsin/arqen --state open --json number,title,createdAt,labels --limit 30
```

If no issues exist, report to the user.

### 2. Pick the next issue

Sort by `createdAt` ascending (oldest first). Start with the oldest open issue
to work through the backlog systematically.

### 3. Investigate

Read the issue body fully:

```bash
gh issue view <number> --json title,body,labels,comments
```

Explore the codebase to understand the root cause. Look at relevant source
files, tests, and docs. Form a plan before writing code.

### 4. Evaluate fit

Before writing any code, assess whether the issue aligns with the product
vision. Arqen is a **reusable Rust backend framework for agent-ready applications**
— it prioritizes simplicity, composability, and a small surface area. Ask:

- **Does this solve a real user problem?** Skip issues that are speculative
  or solutions in search of a problem.
- **Is this in scope?** Arqen is a backend framework for reusable patterns,
  CLI, agent tools, jobs, and adapters. Ensure the issue fits this scope.
- **Does it preserve simplicity?** Reject requests that add heavyweight
  deps, complex protocols, or niche features with narrow appeal.
- **Is the approach sound?** If the proposed implementation conflicts with
  existing architecture, push back or propose an alternative in the issue comments.

If the issue is a **bad fit**, close it with a comment explaining why:

```bash
gh issue close <number> -c "Closing — <reason>. Out of scope for arqen because..."
```

If it's a **good fit but needs clarification**, comment and leave open:

```bash
gh issue comment <number> -m "Could you clarify <question>? This would help determine the right approach."
```

Only proceed to implementation if the issue is a clear yes.

### 5. Implement

Apply the fix. Follow the repo conventions:
- Rust (edition 2024, cargo fmt, clippy -D warnings)
- Conventional commits (fix:, feat:, refactor:, chore:)

### 6. Verify

```bash
cargo test -p arqen --all-features
cargo clippy -p arqen --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

### 7. Commit and push

```bash
git add -A
git commit -m "type(scope): short description

Closes #<number>"
git pull --rebase
git push
```

The `Closes #<number>` in the commit message auto-closes the issue on push.

### 8. Verify issue is closed

```bash
gh issue view <number> --json number,state,title
```

Proceed to the next open issue.

## Relevant commands

| Action | Command |
|--------|---------|
| List open issues | `gh issue list --repo sayanmohsin/arqen --state open --json number,title,createdAt` |
| View issue | `gh issue view <number> --json title,body,labels,comments` |
| Close issue manually | `gh issue close <number> -c "reason"` |
| Git commit to auto-close | Include `Closes #<number>` in commit message |
