RTK := $(shell command -v rtk >/dev/null 2>&1 && echo rtk)
CARGO := $(RTK) cargo

.PHONY: verify check-conventions rust-verify

verify: check-conventions rust-verify

check-conventions:
	./scripts/check-conventional-commits.sh

rust-verify:
	@if [ -f Cargo.toml ]; then \
		$(CARGO) fmt --all -- --check && \
		$(CARGO) clippy --all-targets --all-features -- -D warnings && \
		$(CARGO) test --all-features; \
	else \
		echo "No Cargo.toml yet; skipping Rust checks."; \
	fi
