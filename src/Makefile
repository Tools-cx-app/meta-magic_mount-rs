PROJECT = magic_mount
VERSION  ?= 0.1.0

# use zig cc
ZIG      ?= zig
CC       := $(ZIG) cc
STRIPPER := strip

# source files
SRCS     := utils.c magic_mount.c main.c

# output directory
OUTDIR   := bin

# common flags
CFLAGS_COMMON := -D_POSIX_C_SOURCE=200809L -std=c23 -Wall -Wextra -pedantic
CFLAGS_COMMON += -DVERSION=\"$(VERSION)\"

LDFLAGS_COMMON := -static

# build mode specific flags
CFLAGS_RELEASE := -O2 -DNDEBUG
CFLAGS_DEBUG   := -O0 -g -DDEBUG

# zig target triples
TARGET_AMD64 ?= x86_64-linux
TARGET_ARM64 ?= aarch64-linux
TARGET_ARMV7 ?= arm-linux

# output binary names
BIN_AMD64 := $(OUTDIR)/mm_amd64
BIN_ARM64 := $(OUTDIR)/mm_arm64
BIN_ARMV7 := $(OUTDIR)/mm_armv7

BINS := $(BIN_AMD64) $(BIN_ARM64) $(BIN_ARMV7)

.PHONY: all clean release debug amd64 arm64 armv7 dirs help strip-bins

# default target
all: release

help:
	@echo "Usage:"
	@echo "  make release [version=X.Y.Z]  - Build release version (optimized, stripped)"
	@echo "  make debug [version=X.Y.Z]    - Build debug version (with symbols)"
	@echo "  make clean                    - Clean build artifacts"
	@echo ""
	@echo "Individual targets:"
	@echo "  make amd64  - Build for x86_64"
	@echo "  make arm64  - Build for aarch64"
	@echo "  make armv7  - Build for armv7"

dirs:
	mkdir -p $(OUTDIR)

# Release build (default)
release: export CFLAGS := $(CFLAGS_COMMON) $(CFLAGS_RELEASE)
release: dirs $(BINS) strip-bins
	@echo "Release build completed: $(VERSION)"

# Debug build
debug: export CFLAGS := $(CFLAGS_COMMON) $(CFLAGS_DEBUG)
debug: dirs $(BINS)
	@echo "Debug build completed: $(VERSION)"

strip-bins:
	@echo "Stripping binaries..."
	@for bin in $(BINS); do \
		echo "  Stripping $$bin"; \
		$(STRIPPER) $$bin; \
	done

amd64: $(BIN_AMD64)
arm64: $(BIN_ARM64)
armv7: $(BIN_ARMV7)

$(BIN_AMD64): $(SRCS)
	$(CC) -target $(TARGET_AMD64) $(CFLAGS) $^ -o $@ $(LDFLAGS_COMMON)

$(BIN_ARM64): $(SRCS)
	$(CC) -target $(TARGET_ARM64) $(CFLAGS) $^ -o $@ $(LDFLAGS_COMMON)

$(BIN_ARMV7): $(SRCS)
	$(CC) -target $(TARGET_ARMV7) $(CFLAGS) $^ -o $@ $(LDFLAGS_COMMON)

clean:
	rm -rf $(OUTDIR)