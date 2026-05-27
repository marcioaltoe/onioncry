#!/usr/bin/env sh
set -eu

types="feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert"
scopes="agents|ci|cli|config|deps|diagnostics|docs|graph|parser|release|repo|test"
pattern="^(${types})(\\((${scopes})\\))?(!)?: .+"

range="${1:-}"
if [ -n "$range" ]; then
  commits=$(git log --no-merges --format='%H%x09%s' "$range")
else
  commits=$(git log --no-merges --format='%H%x09%s')
fi

if [ -z "$commits" ]; then
  echo "No commits to check."
  exit 0
fi

failed=0
printf '%s\n' "$commits" | while IFS="$(printf '\t')" read -r hash subject; do
  if ! printf '%s\n' "$subject" | grep -Eq "$pattern"; then
    echo "Invalid commit message:"
    echo "  ${hash} ${subject}"
    echo "Expected: <type>[optional scope][!]: <description>"
    echo "Types: ${types}"
    echo "Scopes: ${scopes}"
    failed=1
  fi

  if [ "$failed" -ne 0 ]; then
    exit "$failed"
  fi
done
