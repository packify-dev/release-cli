# release-cli

## Release!

```bash
release-cli release -t major
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