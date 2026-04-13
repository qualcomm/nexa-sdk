# llama.cpp for nexa SDK

This repo is the intenral llama.cpp verison for Nexa SDK. The original repo is https://github.com/ggml-org/llama.cpp

Upon first setup, you want to add the original repo as a remote by running the following command

```
git remote add upstream https://github.com/ggml-org/llama.cpp.git
```

or verify you already have the repo as a remote

```
git remote -v
```

Then when you need to sync with the upstream llama.cpp, run

```
git pull upstream master --no-rebase
```

(Note: that we don't use rebase when merging with upstream so that we can keep separated git history)

Then you can complete the regular git merge process - resolve conflicts, submit PR, merge into main, and update the submodule commit in other repos to use the latest.

When merging into main, also use merge pull request. Do not use `Squash and merge` or `Rebase and merge`.
