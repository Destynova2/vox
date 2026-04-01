#!/bin/bash
export PKG_CONFIG_PATH="/var/home/linuxbrew/.linuxbrew/lib/pkgconfig:$PKG_CONFIG_PATH"
export LD_LIBRARY_PATH="/var/home/linuxbrew/.linuxbrew/lib:$LD_LIBRARY_PATH"
export LIBRARY_PATH="/var/home/linuxbrew/.linuxbrew/Cellar/gcc/15.2.0_1/lib/gcc/15:$LIBRARY_PATH"
cargo build --release "$@"
