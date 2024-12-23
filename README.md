# release-cli

## Install

```bash
sudo curl -L \
    https://ghr.packify.dev/packify-dev/release-cli/latest/download/linux/amd64 \
    -o /usr/local/bin/release-cli
```

### Binary downloads

[![Linux (amd64)](https://img.shields.io/badge/linux-amd64-fcc624?logo=linux&style=for-the-badge&logoColor=fcc624)](https://ghr.packify.dev/packify-dev/release-cli/latest/download/linux/amd64)
[![Windows (amd64)](https://img.shields.io/badge/windows-amd64-800000?logo=wine&style=for-the-badge&logoColor=800000)](https://ghr.packify.dev/packify-dev/release-cli/latest/download/windows/amd64)

## Release!

```bash
release-cli release -t <major|minor|patch|alpha|beta|rc>
```

## Build release binaries

```bash
release-cli build -t v0.1.0
```

## Setup repository

You will need at least the following branches in your repository:

```bash
git branch alpha
git checkout alpha
git push origin alpha

git branch beta
git checkout beta
git push origin beta

git branch rc
git checkout rc
git push origin rc

git checkout main
```

It's recommended to use a development branch, but it's not required.

Add the following to your `release.toml`:

```toml
type = "rust"

[build]
platforms = [
    { target = "x86_64-unknown-linux-gnu", platform = "linux", arch = "amd64" },
    { target = "x86_64-pc-windows-gnu", platform = "windows", arch = "amd64" },
]
```