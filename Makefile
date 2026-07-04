ifneq (,$(wildcard .env))
include .env
export
endif

RTK := $(shell command -v rtk >/dev/null 2>&1 && echo rtk)
CARGO := $(RTK) cargo

.DEFAULT_GOAL := help

.PHONY: help bootstrap verify rust-verify fmt fmt-check check lint test build install doc \
  check-conventions check-pr-title publish-dry-run update clean skills-link skills-update

help: ## Show this help
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make <target>\n"} \
		/^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } \
		/^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)


##@ Bootstrap

bootstrap: ## Prepare the workspace and fetch Rust dependencies when scaffolded
	@if [ -f Cargo.toml ]; then \
		$(CARGO) fetch; \
	else \
		echo "No Cargo.toml yet; nothing to bootstrap."; \
	fi


##@ Quality & Testing

verify: ## Run all repository checks
	$(MAKE) check-conventions
	$(MAKE) rust-verify

rust-verify: ## Run Rust fmt-check, check, clippy, and tests when scaffolded
	@if [ -f Cargo.toml ]; then \
		$(MAKE) fmt-check && \
		$(MAKE) check && \
		$(MAKE) lint && \
		$(MAKE) test; \
	else \
		echo "No Cargo.toml yet; skipping Rust checks."; \
	fi

fmt: ## Format Rust code
	@if [ -f Cargo.toml ]; then \
		$(CARGO) fmt --all; \
	else \
		echo "No Cargo.toml yet; skipping cargo fmt."; \
	fi

fmt-check: ## Check Rust formatting
	@if [ -f Cargo.toml ]; then \
		$(CARGO) fmt --all -- --check; \
	else \
		echo "No Cargo.toml yet; skipping cargo fmt --check."; \
	fi

check: ## Run cargo check
	@if [ -f Cargo.toml ]; then \
		$(CARGO) check --all-targets --all-features; \
	else \
		echo "No Cargo.toml yet; skipping cargo check."; \
	fi

lint: ## Run clippy with warnings denied
	@if [ -f Cargo.toml ]; then \
		$(CARGO) clippy --all-targets --all-features -- -D warnings; \
	else \
		echo "No Cargo.toml yet; skipping cargo clippy."; \
	fi

test: ## Run Rust tests
	@if [ -f Cargo.toml ]; then \
		$(CARGO) test --all-features; \
	else \
		echo "No Cargo.toml yet; skipping cargo test."; \
	fi

build: ## Build the Rust project
	@if [ -f Cargo.toml ]; then \
		$(CARGO) build --all-targets --all-features; \
	else \
		echo "No Cargo.toml yet; skipping cargo build."; \
	fi

install: build ## Build and install onioncry into Cargo bin for local testing
	@if [ -f Cargo.toml ]; then \
		$(CARGO) install --path . --force --locked; \
	else \
		echo "No Cargo.toml yet; skipping cargo install."; \
	fi

doc: ## Build Rust docs without dependencies
	@if [ -f Cargo.toml ]; then \
		$(CARGO) doc --no-deps --all-features; \
	else \
		echo "No Cargo.toml yet; skipping cargo doc."; \
	fi


##@ Git & CI

check-conventions: ## Validate Conventional Commit history
	./scripts/check-conventional-commits.sh

check-pr-title: ## Validate a PR title: make check-pr-title TITLE="feat(cli): add check command"
	@if [ -z "$(TITLE)" ]; then \
		echo 'Usage: make check-pr-title TITLE="feat(cli): add check command"'; \
		exit 1; \
	fi
	./scripts/check-conventional-title.sh "$(TITLE)"

publish-dry-run: ## Verify the crate package can be published
	@if [ -f Cargo.toml ]; then \
		$(CARGO) publish --dry-run --locked --allow-dirty; \
	else \
		echo "No Cargo.toml yet; skipping cargo publish --dry-run."; \
	fi


##@ Dependencies & Cleanup

update: ## Update Cargo.lock when the Rust crate exists
	@if [ -f Cargo.toml ]; then \
		$(CARGO) update; \
	else \
		echo "No Cargo.toml yet; skipping cargo update."; \
	fi

clean: ## Remove Rust build artifacts when scaffolded
	@if [ -f Cargo.toml ]; then \
		$(CARGO) clean; \
	else \
		echo "No Cargo.toml yet; skipping cargo clean."; \
	fi


##@ Agent Skills
skills-update: ## Install missing skills and update existing ones to latest (reads skills-lock.json)
	@bunx skills experimental_install
	@bunx skills update -p -y
	@$(MAKE) fmt
	
skills-link: ## Recreate .claude/skills symlinks from .agents/skills
	@mkdir -p .claude/skills
	@rm -f .claude/skills/*
	@for skill in .agents/skills/*/; do \
		name=$$(basename "$$skill"); \
		ln -s "../../.agents/skills/$$name" ".claude/skills/$$name"; \
	done
	@echo "Linked $$(ls .claude/skills | wc -l | tr -d ' ') skills from .agents/skills to .claude/skills"
