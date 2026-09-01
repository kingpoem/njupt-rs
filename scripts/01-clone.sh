#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="${ROOT}/references"

REPOS=(
  "https://github.com/gaoliang/NJUPT-API"
  "https://github.com/mangofanfan/NJUPT-Suan-API"
)

mkdir -p "${DEST}"

for url in "${REPOS[@]}"; do
  name="$(basename "${url}" .git)"
  target="${DEST}/${name}"

  if [[ -d "${target}/.git" ]]; then
    echo "skip (already exists): ${name}"
    continue
  fi

  echo "cloning: ${name}"
  git clone --depth=1 --single-branch "${url}" "${target}"
done

echo "done."
