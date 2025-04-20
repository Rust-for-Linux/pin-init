#!/usr/bin/env bash

set -e

krepo="$1"
urepo=$(git rev-parse --show-toplevel)
patches=$(mktemp -d)
commit="$2"

if [ ! -d "$krepo" ]; then
    echo "expected the kernel directory as \$1, but got \"$krepo\" which either doesn't exist or is not a directory"
    exit 1
fi

pushd "$krepo" > /dev/null

if ! git cat-file -e "$commit^{commit}"; then
    echo "commit $commit does not seem to exist."
    exit 1
fi

echo "Copying all commits in \"$krepo\" into \"$urepo\" from commit $commit onwards:"
git log --oneline "$commit..HEAD"
read -p "Does this look good to you? [Y/n] " ans
case "$ans" in
    Y)
        ;;
    y)
        ;;
    *)
        exit 1
        ;;
esac

popd > /dev/null # $krepo

krepo=$(realpath "$krepo")
urepo=$(realpath "$urepo")
patches=$(realpath "$patches")

pushd "$krepo" > /dev/null

git format-patch --quiet --output-directory "$patches" "$commit"

pushd "$patches" > /dev/null

sed -i 's/^\(Subject: \[PATCH .*\] \)rust: pin-init: /\1/' *

popd > /dev/null # $patches

popd > /dev/null # $krepo

pushd "$urepo" > /dev/null

head=$(git rev-parse HEAD)

git am                                              \
    --signoff                                       \
    --reject                                        \
    --interactive                                   \
    -p3                                             \
    --empty=drop                                    \
    $patches/*

# need the `--exec 'true'` in order for the `--no-keep-empty` option to actually do stuff
git rebase --no-keep-empty --quiet --exec 'true' "$head"

popd > /dev/null # $urepo

rm $patches/*
