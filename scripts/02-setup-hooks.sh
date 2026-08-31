#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

chmod +x .githooks/post-commit .githooks/commit-msg
git config core.hooksPath .githooks

echo "hooksPath → .githooks"
if ! command -v git-cliff >/dev/null 2>&1; then
  echo "警告: 未安装 git-cliff。CHANGELOG 同步需要: cargo install git-cliff"
else
  echo "git-cliff: $(git-cliff --version)"
fi
echo "done."