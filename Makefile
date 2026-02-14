PREFIX  ?= $(HOME)/.local
BINDIR  := $(PREFIX)/bin
BIN     := sidecar-on-dock
TARGET  := target/release/$(BIN)

.PHONY: all build dev test install uninstall clean lint format docs help

all: build

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "  build       Build release binary (default)"
	@echo "  dev         Build and run in debug mode"
	@echo "  install     Install binary to \$$(PREFIX)/bin [$(BINDIR)]"
	@echo "  uninstall   Remove binary from \$$(PREFIX)/bin"
	@echo "  clean       Remove build artifacts"
	@echo "  test        Run tests"
	@echo "  lint        Run clippy lints"
	@echo "  format      Run cargo fmt"
	@echo "  docs        Generate and open documentation"
	@echo "  help        Show this help"

build:
	cargo build --release

dev:
	cargo run

install: build
	install -d $(BINDIR)
	install -m 755 $(TARGET) $(BINDIR)/$(BIN)
	@echo "Installed $(BINDIR)/$(BIN)"

uninstall:
	rm -f $(BINDIR)/$(BIN)
	@echo "Removed $(BINDIR)/$(BIN)"

clean:
	cargo clean

test:
	cargo test

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt

docs:
	cargo doc --no-deps
	open target/doc/sidecar_on_dock/index.html
