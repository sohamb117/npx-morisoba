# morisoba (npm wrapper)

This npm package is a thin wrapper that downloads the appropriate prebuilt `morisoba` Rust binary from [GitHub Releases](https://github.com/sohamb117/npx-morisoba/releases) on first run, caches it under `$HOME/.cache/morisoba/v<version>/`, and execs it.

## USAGE

```bash
npx morisoba
```

or after install:

```bash
npm install -g morisoba
morisoba
```

## SUPPORTED PLATFORMS

- Linux x86_64
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

For other platforms, build from source: see the main [README](https://github.com/sohamb117/npx-morisoba#readme).

## CONFIGURATION

| Env var | Default | Description |
|---|---|---|
| `MORISOBA_REPO` | `sohamb117/npx-morisoba` | GitHub repo to fetch releases from. Override for forks. |

## WHAT IT DOES

1. Detects your `process.platform` + `process.arch`.
2. Looks up the cached binary at `$HOME/.cache/morisoba/v<version>/morisoba-<platform>-<arch>[.exe]`.
3. If absent, downloads from `https://github.com/$MORISOBA_REPO/releases/download/v<version>/morisoba-<platform>-<arch>[.exe]`.
4. On Unix, chmods +x.
5. Spawns it with `cwd` set to the npm package directory (so the bundled `assets/hero.png` is found).

## NETWORK / OFFLINE

The download happens once per version. After that, `npx morisoba` is fully offline. To prefetch the binary at install time, run `npx morisoba --version` once; subsequent runs use the cache.

## LICENSE

MIT OR Apache-2.0. Source: https://github.com/sohamb117/npx-morisoba
