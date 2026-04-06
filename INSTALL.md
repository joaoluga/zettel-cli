# Installation & Distribution

## Install locally (from source)

```bash
cargo install --path .
```

Compiles a release build and places the binary in `~/.cargo/bin/zettel-cli`, which is on `$PATH` with a standard Rust setup.

```bash
# Uninstall
cargo uninstall zettel-cli
```

---

## Distribution options

### Tier 1 — Anyone with Rust installed

**From the GitHub repository:**
```bash
cargo install --git https://github.com/<you>/zettel-cli
```

**From [crates.io](https://crates.io) (after `cargo publish`):**
```bash
cargo install zettel-cli
```

Before publishing, add the required metadata to `Cargo.toml`:

```toml
[package]
name        = "zettel-cli"
version     = "0.1.0"
edition     = "2024"
description = "A Zettelkasten/Obsidian note manager for the terminal"
license     = "MIT"
repository  = "https://github.com/<you>/zettel-cli"
homepage    = "https://github.com/<you>/zettel-cli"
keywords    = ["notes", "zettelkasten", "obsidian", "cli", "markdown"]
categories  = ["command-line-utilities"]
```

Then publish:
```bash
cargo publish
```

---

### Tier 2 — Pre-compiled binaries (no Rust required)

Use **[cargo-dist](https://opensource.axo.dev/cargo-dist/)** to cross-compile and publish binaries automatically on every release tag.

```bash
cargo install cargo-dist
cargo dist init   # adds config to Cargo.toml and creates .github/workflows/release.yml
cargo dist build  # test locally
```

On every `git tag v0.x.y` push, the generated GitHub Actions workflow:
- Cross-compiles for Linux (x86\_64, aarch64), macOS (x86\_64, Apple Silicon), and Windows
- Creates a GitHub Release with the binaries attached
- Generates a one-line install script for users:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/<you>/zettel-cli/releases/latest/download/zettel-cli-installer.sh | sh
```

**Install without compiling using [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall)** (downloads the binary from GitHub Releases):
```bash
cargo binstall zettel-cli
```

---

### Tier 3 — Package managers

| Manager | Audience | Notes |
|---|---|---|
| **Homebrew** | macOS + Linux | Create a formula in `homebrew-core` or your own tap. `cargo-dist` can generate and PR the formula automatically. |
| **AUR** | Arch Linux | Submit a `PKGBUILD` to the AUR — `zettel-cli` to build from source, `zettel-cli-bin` for the pre-compiled binary. |
| **Nix** | NixOS / nix users | Add a derivation to nixpkgs or ship a `flake.nix` in the repository. |
| **Scoop** | Windows | Create a manifest in a Scoop bucket. |

---

## Shell completions

After installing, generate completions for your shell:

```bash
# Bash
zettel-cli completions bash >> ~/.bash_completion

# Zsh
zettel-cli completions zsh > "${fpath[1]}/_zettel-cli"

# Fish
zettel-cli completions fish > ~/.config/fish/completions/zettel-cli.fish
```

---

## Recommended release path

| Step | Action |
|---|---|
| **Now** | `cargo install --path .` for local use |
| **First release** | Add metadata to `Cargo.toml`, run `cargo publish`, run `cargo dist init`, push a `v0.1.0` tag |
| **Later** | Submit to Homebrew tap and/or AUR once there is a user base |
