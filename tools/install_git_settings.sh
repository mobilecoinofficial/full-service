#!/bin/bash

set -e

GIT_BASE=$(git rev-parse --show-toplevel)

pushd "${GIT_BASE}/.git/hooks" >/dev/null

echo -n "Installing pre-commit hook..."
ln -sf "../../hooks/pre-commit" .
echo " Done."

echo -n "Installing pre-push hook..."
ln -sf "../../hooks/pre-push" .
echo " Done."

popd >/dev/null

echo -n "Installing 'theirs' merge driver..."
git config --local merge.theirs.driver "mv %B %A"
echo " Done."
