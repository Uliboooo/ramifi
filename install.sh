#!/bin/bash

DESKFILE="./fork_notes.desktop"
BIN="./fork_notes"

if [[ -f "$DESKFILE" && -f "$BIN" ]]; then
  echo "ok"
else
  echo "please run on unziped folder"
fi

