#!/bin/bash

set -e

if [ -z "$(git status --porcelain)" ]; then
  echo "Process..."
else
  echo "Working directory is not clean."
  exit 1
fi

TAG=$(git describe --tags --always --abbrev=7)
RELEASE_DIR="release"
NEWPATH="${RELEASE_DIR}/${TAG}"

mkdir -p ${NEWPATH}

cargo build --release &&\
cp ./target/release/fork_notes "${NEWPATH}/fork_notes" &&\
cp ./fork_notes.desktop "${NEWPATH}/fork_notes.desktop" &&\
cp ./install.sh "${NEWPATH}/install.sh" &&\
(
  cd "${RELEASE_DIR}"
  zip -r "../${TAG}.zip" "${TAG}"
)

rm -rf "${NEWPATH}"

echo "Done: ${TAG}.zip created."
