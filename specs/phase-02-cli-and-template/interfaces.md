# Interfaces

Commands:

- `arqen new NAME`
- `arqen generate module NAME`
- `arqen generate tool NAME`
- `arqen generate job NAME`
- `arqen dev`
- `arqen start`
- `arqen check`
- `arqen doctor`

The default bind address is `127.0.0.1:8888`. `arqen new` writes the
module-based starter project directly and does not accept a `--template` flag.

## Compatibility rule

Keep interfaces small, typed, documented, and replaceable. Do not expose
private implementation details through public application contracts.
