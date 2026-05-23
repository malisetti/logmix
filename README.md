# logmix

Merge structured log streams by timestamp — JSON-lines, logfmt, and plain text, with per-source tagging.

## Install

```bash
cargo install logmix
```

## Quickstart

Merge two JSONL files into a single time-ordered stream (default `passthrough` output):

```bash
logmix app.jsonl api.jsonl
```

Mix JSONL and logfmt inputs; emit tab-separated source, timestamp, and line with `tagged` format:

```bash
logmix --format=tagged services.jsonl sidecar.logfmt
```

Ship merged logs into your existing toolchain:

```bash
logmix --format=jsonl /var/log/svc*.log | jq
```

## `--format`

| Value | Description |
|-------|-------------|
| `passthrough` | `[source] raw_line` (default) |
| `jsonl` | One JSON object per line: `source`, `ts` (number or `null`), `raw` |
| `tagged` | Tab-separated: `source`, `ts`, `raw` (empty `ts` when unknown) |

Pass `--format=<value>` on the command line. Unrecognized values fall back to `passthrough`.

## License

See [LICENSE](LICENSE).
