# Justfile for vsql_ranger_arranger
# Provides comprehensive tooling for development, testing, and release.
# Install just: cargo install just

set shell := ["bash", "-euc"]

# Default target: run the full local CI gate
default: ci

# Run the full local CI gate (fmt + clippy + tests + fuzz)
ci: fmt-check clippy test fuzz

# Format check
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# Clippy with all warnings denied
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all tests (unit + integration + benches)
test:
    cargo test --all-targets --all-features

# Run the stable fuzz harness
fuzz:
    cargo test --test fuzz_harness

# Run benchmarks (criterion)
bench:
    cargo bench

# Check for outdated dependencies (release gate)
outdated:
    cargo install cargo-outdated
    cargo outdated

# Security audit (release gate)
# Notes:
#   - --deny unmaintained hard-fails on unmaintained crates
#   - --ignore RUSTSEC-2024-0436 allows the transitive `paste` dep from the vendored SDK
#   - --ignore RUSTSEC-2026-0253 allows `lru` 0.16.4, which is pinned by the `mysql` crate's `lru = ^0.16.3` dependency and cannot be upgraded independently
audit:
    cargo install cargo-audit
    cargo audit --deny unmaintained --ignore RUSTSEC-2024-0436 --ignore RUSTSEC-2026-0253

# Run all release gates locally (does not package)
release-check: ci outdated audit

# Build the extension package (.veb) for release
# Requires VILLAGESQL_BUILD_DIR to be set
package:
    cargo vsql install

# Start a local VillageSQL dev server with this extension loaded.
# Requires VILLAGESQL_BUILD_DIR to point at a VillageSQL build tree
# (eg ~/build/villagesql).  The server binds 127.0.0.1:3307 and uses a temp datadir
# initialized on first run.  Run `just dev-server-stop` to stop it.
dev-server:
    @if [ -z "${VILLAGESQL_BUILD_DIR}" ]; then \
      echo "Error: VILLAGESQL_BUILD_DIR is not set"; \
      echo "Example: VILLAGESQL_BUILD_DIR=~/build/villagesql just dev-server"; \
      exit 1; \
    fi
    @if [ ! -d "${VILLAGESQL_BUILD_DIR}" ]; then \
      echo "Error: VILLAGESQL_BUILD_DIR does not exist: ${VILLAGESQL_BUILD_DIR}"; \
      exit 1; \
    fi
    @if [ ! -x "${VILLAGESQL_BUILD_DIR}/bin/mysqld" ]; then \
      echo "Error: mysqld not found in ${VILLAGESQL_BUILD_DIR}/bin/"; \
      exit 1; \
    fi
    @echo "==> Building and installing extension via cargo vsql..."
    cargo vsql install
    @echo "==> Initializing datadir if needed..."
    mkdir -p /tmp/villagesql-dev-datadir
    if [ ! -d "/tmp/villagesql-dev-datadir/mysql" ]; then \
      "${VILLAGESQL_BUILD_DIR}/bin/mysqld" --initialize-insecure --user=$(whoami) --datadir=/tmp/villagesql-dev-datadir; \
    fi
    @echo "==> Starting VillageSQL on 127.0.0.1:3307 ..."
    @echo "    Root password is empty.  Stop with: just dev-server-stop"
    nohup "${VILLAGESQL_BUILD_DIR}/bin/mysqld" \
      --datadir=/tmp/villagesql-dev-datadir \
      --port=3307 \
      --bind-address=127.0.0.1 \
      --socket=/tmp/villagesql-dev.sock \
      --pid-file=/tmp/villagesql-dev.pid \
      --skip-networking=0 \
      --log-error=/tmp/villagesql-dev.err \
      > /tmp/villagesql-dev.log 2>&1 &
    @echo "$$!" > /tmp/villagesql-dev.pid
    sleep 2
    @echo "==> Loading extension..."
    "${VILLAGESQL_BUILD_DIR}/bin/mysql" -h127.0.0.1 -P3307 -uroot -e "INSTALL EXTENSION vsql_ranger_arranger;" || true
    @echo "==> Server ready at 127.0.0.1:3307 (root / no password)"
    @echo "    Stop with: just dev-server-stop"

# Run the mysql client against the local dev server
dev-sql:
    "${VILLAGESQL_BUILD_DIR}/bin/mysql" -h127.0.0.1 -P3307 -uroot

# Stop the local VillageSQL dev server
dev-server-stop:
    @if [ -f /tmp/villagesql-dev.pid ]; then \
      kill "$(cat /tmp/villagesql-dev.pid)" 2>/dev/null || true; \
      rm -f /tmp/villagesql-dev.pid; \
      echo "Stopped VillageSQL dev server."; \
    else \
      echo "No pid file found at /tmp/villagesql-dev.pid"; \
    fi

# Clean build artifacts
clean:
    cargo clean

# Show help
help:
    @echo "Available targets:"
    @echo "  ci            - Run full local CI gate (fmt + clippy + test + fuzz)"
    @echo "  fmt-check     - Check code formatting"
    @echo "  fmt           - Format code"
    @echo "  clippy        - Run clippy with all warnings denied"
    @echo "  test          - Run all tests"
    @echo "  fuzz          - Run stable fuzz harness"
    @echo "  bench         - Run criterion benchmarks"
    @echo "  outdated      - Check for outdated dependencies"
    @echo "  audit         - Run security audit (deny unmaintained)"
    @echo "  release-check - Run full release quality gate locally"
    @echo "  package       - Build .veb extension package (needs VILLAGESQL_BUILD_DIR)"
    @echo "  dev-server    - Start a local VillageSQL dev server with this extension loaded"
    @echo "  clean         - Remove build artifacts"
    @echo "  help          - Show this help"
