#!/usr/bin/env sh
set -eu

title="${1:-${PR_TITLE:-}}"

if [ -z "$title" ]; then
  echo "Missing PR title."
  exit 1
fi

types="feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert"
scopes="agents|ci|cli|config|deps|diagnostics|docs|graph|parser|release|repo|test"
pattern="^(${types})(\\((${scopes})\\))?(!)?: .+"

if ! printf '%s\n' "$title" | grep -Eq "$pattern"; then
  echo "Invalid PR title: ${title}"
  echo "Expected: <type>[optional scope][!]: <description>"
  echo "Example: feat(cli): add check command"
  echo "Types: ${types}"
  echo "Scopes: ${scopes}"
  exit 1
fi
