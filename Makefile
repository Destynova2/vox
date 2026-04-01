# Detect linuxbrew paths if available
BREW_PREFIX := $(shell brew --prefix 2>/dev/null)

ifdef BREW_PREFIX
  export PKG_CONFIG_PATH := $(BREW_PREFIX)/lib/pkgconfig:$(PKG_CONFIG_PATH)
  export LD_LIBRARY_PATH := $(BREW_PREFIX)/lib:$(LD_LIBRARY_PATH)
  # Find libstdc++.a from gcc
  GCC_LIB := $(shell find $(BREW_PREFIX)/Cellar/gcc -name "libstdc++.a" -print -quit 2>/dev/null)
  ifdef GCC_LIB
    export LIBRARY_PATH := $(dir $(GCC_LIB)):$(LIBRARY_PATH)
  endif
endif

.PHONY: build install clean

build:
	cargo build --release

install: build
	cp target/release/vox ~/.local/bin/vox
	@echo "installed: ~/.local/bin/vox"

clean:
	cargo clean
