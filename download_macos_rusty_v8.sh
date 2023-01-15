#!/bin/bash

RUSTY_V8_MIRROR=$HOME/.cache/rusty_v8

# see https://github.com/denoland/rusty_v8/releases

for REL in v0.61.0; do
  mkdir -p $RUSTY_V8_MIRROR/$REL
  for FILE in \
    librusty_v8_debug_x86_64-apple-darwin.a \
    librusty_v8_release_x86_64-apple-darwin.a \
  ; do
    if [ ! -f $RUSTY_V8_MIRROR/$REL/$FILE ]; then
      wget -O $RUSTY_V8_MIRROR/$REL/$FILE \
        https://github.com/denoland/rusty_v8/releases/download/$REL/$FILE
    fi
  done
done
