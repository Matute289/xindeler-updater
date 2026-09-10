# Xindeler Updater

A cross-platform launcher/updater for Xindeler.

## Features

- [x] Update/Download and start the game.
- [x] Fancy UI with batteries included.
- [x] Updates itself on windows.

## Download

**NOTE:** Xindeler Updater cannot be considered stable yet.

For *binary* packages, use the [GitHub releases](https://github.com/Matute289/xindeler-updater/releases).

For *source* packages **do not** use the `master` branch — it still carries upstream Airshipper's
in-progress work. Use `main`/`development` instead.

#### Compile from source

```bash
git clone git@github.com:Matute289/xindeler-updater.git
cd xindeler-updater
cargo run --release
```

Make sure to have [rustup](https://rustup.rs/) installed to compile Rust code, and `git lfs` for assets.

### Xindeler-Updater-Server

**NOTE:** the server component is inherited from upstream Airshipper's GitLab-CI-webhook design and
does not transfer as-is to a GitHub-Actions-based release pipeline — see `docs/design` for the port
plan before relying on it.

#### Compile from source

```bash
cargo run --release --bin xindeler-updater-server
```

On first execution, a template configuration file will be created at `config/config.template.ron` and
the server will exit.

Rename this to `config.ron` and edit as appropriate before running again.
