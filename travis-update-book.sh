#!/bin/bash

# this script based off of this gist https://gist.github.com/domenic/ec8b0fc8ab45f39403dd

set -e # Exit with nonzero exit code if anything fails

SOURCE_BRANCH="develop"
TARGET_BRANCH="gh-pages"

function doCompile {
  . docker/build-mdbook-image && . docker/build-mdbook
}

# Pull requests and commits to other branches shouldn't build
if [ "$TRAVIS_PULL_REQUEST" != "false" -o "$TRAVIS_BRANCH" != "$SOURCE_BRANCH" ]; then
    exit 0
fi

# Save some useful information
REPO=`git config remote.origin.url`
SSH_REPO=${REPO/https:\/\/github.com\//git@github.com:}
SHA=`git rev-parse --verify HEAD`

# Clone the existing gh-pages for this repo into doc/holochain_101/working
git clone -b $TARGET_BRANCH $REPO doc/holochain_101/working

# Clean out existing contents
cd doc/holochain_101/working
git ls-files | xargs rm -rf
cd ../../..

# Run our compile script
doCompile

# Move all our built files into the working directory
mv doc/holochain_101/book/* doc/holochain_101/working

# Move a copy of our Github Pages config file back into the directory
cp _config.yml doc/holochain_101/working/_config.yml

# Set things up with git for committing new changes to the mdbook
cd doc/holochain_101/working
git config user.name "Travis CI"
git config user.email "$COMMIT_AUTHOR_EMAIL"

# If there are no changes to the compiled out (e.g. this is a README update) then just bail.
if git diff --quiet; then
    echo "No changes to the output on this push; exiting."
    exit 0
fi

# Commit the "changes", i.e. the new version.
# The delta will show diffs between new and old versions.
git add -A .
git commit -m "Deploy to GitHub Pages: ${SHA}"

# Get the deploy key by using Travis's stored variables to decrypt deploy_key.enc
ENCRYPTED_KEY_VAR="encrypted_${ENCRYPTION_LABEL}_key"
ENCRYPTED_IV_VAR="encrypted_${ENCRYPTION_LABEL}_iv"
ENCRYPTED_KEY=${!ENCRYPTED_KEY_VAR}
ENCRYPTED_IV=${!ENCRYPTED_IV_VAR}
openssl aes-256-cbc -K $ENCRYPTED_KEY -iv $ENCRYPTED_IV -in ../../../build_docs_key.enc -out ../../../build_docs_key -d
chmod 600 ../../../build_docs_key
eval `ssh-agent -s`
ssh-add build_docs_key

# Now that we're all set up, we can push.
git push $SSH_REPO $TARGET_BRANCH
