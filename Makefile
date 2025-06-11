#!/usr/bin/make -f

########################################################
# Utilities
########################################################
# Force Make to be silent
ifndef VERBOSE
.SILENT:
endif

# Get the OS name
UNAME := $(shell uname)

# Setting SHELL to bash allows bash commands to be executed by recipes.
# Options are set to exit when a recipe line exits non-zero or a piped command fails.
SHELL = /usr/bin/env bash -o pipefail
.SHELLFLAGS = -ec

# Binaries directory
LOCALBIN ?= $(shell pwd)/target
$(LOCALBIN):
	mkdir -p $(LOCALBIN)

DOCKERENV=DOCKER_BUILDKIT=1
COMPOSE := $(shell echo "-f $(pwd)/docker-compose.yml)")

default: help
.PHONY: help
help:
	@grep -hE '^[ a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-25s\033[0m %s\n", $$1, $$2}'

########################################################
# Build Executables
########################################################

########################################################
# Infrastructure
########################################################
infrastructure-up: ## Boot up infrastructure
	$(DOCKERENV) docker compose $(COMPOSE) up -d --remove-orphans --wait
.PHONY: infrastructure-up

infrastructure-down: ## Stop up infrastructure
	$(DOCKERENV) docker compose $(COMPOSE) down
.PHONY: infrastructure-down
	
infrastructure-down-volumes: ## Stop up infrastructure
	$(DOCKERENV) docker compose $(COMPOSE) down -v
.PHONY: infrastructure-down-volumes
	
########################################################
# Database 
########################################################
migrate: sqlx-cli ## Run database migrations
	sqlx migrate run
.PHONY: migrate

########################################################
# Tests
########################################################
test-svc-up: ## Start up a service for testing
	python3 test/app.py
.PHONY: test-svc-up
	
########################################################
# Install dependencies
########################################################
sqlx-cli: ## Checks if github CLI is installed
ifeq ($(CI),0) ## sIf not in a CI environment (CI=true)
	if command -v sqlx >/dev/null 2>&1; then \
	  echo "ℹ️  SQLX found at: $$(command -v sqlx)"; \
	else \
	  cargo install sqlx-cli
	  exit 1; \
	fi
endif
.PHONY: sqlx-cli

# make-executable ensures a file is executable (+x).
# $1 - file path to check and make executable
define make-executable
@[ -f "$(1)" ] && { \
  if [ ! -x "$(1)" ]; then \
    chmod +x "$(1)" ;\
  fi ;\
} || { \
  echo "File $(1) does not exist" ;\
  exit 1 ;\
}
endef