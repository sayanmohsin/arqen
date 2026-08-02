# Interfaces

- `arqen new NAME [--template thingd-app]` creates a compiling project.
- `arqen dev` runs and clearly reports its reload behavior.
- `arqen start` runs one container-suitable server process.
- `arqen check` returns non-zero on failed checks; `doctor` gives actionable diagnostics.
- Container command: `arqen start --host 0.0.0.0 --port 3000`.
