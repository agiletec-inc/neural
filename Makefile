# ================================
# Neural Translator Makefile
# ================================
# Tauri + React desktop app
# Docker-First development
# ================================

.DEFAULT_GOAL := help

# ========== Environment Settings ==========
export COMPOSE_DOCKER_CLI_BUILD := 1
export DOCKER_BUILDKIT := 1
export COMPOSE_IGNORE_ORPHANS := true

PROJECT ?= neural
export COMPOSE_PROJECT_NAME := $(PROJECT)

WORKSPACE_SVC := workspace

# Colors
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
RED := \033[0;31m
NC := \033[0m

# ========== Help ==========
.PHONY: help
help:
	@echo ""
	@echo "$(BLUE)Neural Translator - Available Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Project: $(PROJECT)$(NC)"
	@echo "$(YELLOW)Architecture: Tauri + React (Docker-First)$(NC)"
	@echo ""

# ========== Core Commands ==========

.PHONY: up
up: ## Start workspace container
	@echo "$(GREEN)Starting workspace...$(NC)"
	@docker compose up -d --remove-orphans
	@echo "$(GREEN)✅ Workspace started$(NC)"
	@echo "$(YELLOW)⚠️  Run 'make workspace' to enter shell$(NC)"

.PHONY: down
down: ## Stop all services
	@echo "$(YELLOW)Stopping services...$(NC)"
	@docker compose down --remove-orphans
	@echo "$(GREEN)✅ Stopped$(NC)"

.PHONY: restart
restart: down up ## Full restart

.PHONY: logs
logs: ## Show logs
	@docker compose logs -f

.PHONY: ps
ps: ## Show container status
	@docker compose ps

# ========== Development Commands ==========

.PHONY: workspace
workspace: ## Enter workspace shell
	@echo "$(BLUE)Entering workspace shell...$(NC)"
	@echo "$(YELLOW)Run 'pnpm install' first if dependencies not installed$(NC)"
	@docker compose exec $(WORKSPACE_SVC) sh

.PHONY: install
install: ## Install dependencies (inside Docker)
	@echo "$(GREEN)Installing dependencies in container...$(NC)"
	@docker compose exec $(WORKSPACE_SVC) pnpm install --frozen-lockfile
	@echo "$(GREEN)✅ Dependencies installed$(NC)"

.PHONY: dev
dev: ## Start Vite dev server (inside Docker)
	@echo "$(GREEN)Starting Vite dev server...$(NC)"
	@echo "$(YELLOW)Access at: http://localhost:1420$(NC)"
	@docker compose exec $(WORKSPACE_SVC) pnpm dev

.PHONY: build
build: ## Build frontend (inside Docker)
	@echo "$(GREEN)Building frontend...$(NC)"
	@docker compose exec $(WORKSPACE_SVC) pnpm build
	@echo "$(GREEN)✅ Build complete$(NC)"

.PHONY: tauri-dev
tauri-dev: ## Run Tauri desktop app (Mac host - requires GUI)
	@echo "$(RED)⚠️  TAURI DEV RUNS ON MAC HOST (GUI required)$(NC)"
	@echo "$(YELLOW)Prerequisites:$(NC)"
	@echo "  1. Dependencies installed: make install"
	@echo "  2. Rust toolchain on Mac"
	@echo "  3. Ollama running: ollama serve"
	@echo ""
	@echo "$(GREEN)Starting Tauri...$(NC)"
	@pnpm tauri dev

# ========== Ollama ==========

.PHONY: ollama-check
ollama-check: ## Check if Ollama is running
	@echo "$(BLUE)Checking Ollama...$(NC)"
	@if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Ollama is running$(NC)"; \
	else \
		echo "$(RED)❌ Ollama is NOT running$(NC)"; \
		echo "$(YELLOW)Start with: ollama serve$(NC)"; \
		exit 1; \
	fi

.PHONY: ollama-pull
ollama-pull: ## Pull qwen2.5:3b model (高速・多言語対応)
	@echo "$(GREEN)Pulling qwen2.5:3b model...$(NC)"
	@ollama pull qwen2.5:3b

# ========== Clean Commands ==========

.PHONY: clean
clean: ## Clean Mac host garbage - ALL build artifacts should be in Docker volumes
	@echo "$(YELLOW)🧹 Cleaning Mac host garbage (Docker-First violation artifacts)...$(NC)"
	@echo "$(YELLOW)   ⚠️  These files should NOT exist on Mac host in Docker-First dev$(NC)"
	@find . -name "node_modules" -type d -prune -exec rm -rf {} + 2>/dev/null || true
	@find . -name ".next" -type d -prune -exec rm -rf {} + 2>/dev/null || true
	@find . -name "dist" -type d -prune -exec rm -rf {} + 2>/dev/null || true
	@find . -name "build" -type d -prune -exec rm -rf {} + 2>/dev/null || true
	@find . -name ".turbo" -type d -prune -exec rm -rf {} + 2>/dev/null || true
	@find . -name ".cache" -type d -prune -exec rm -rf {} + 2>/dev/null || true
	@find . -name ".eslintcache" -type f -delete 2>/dev/null || true
	@find . -name "*.tsbuildinfo" -type f -delete 2>/dev/null || true
	@find . -name ".DS_Store" -type f -delete 2>/dev/null || true
	@find . -name "package-lock.json" -type f -delete 2>/dev/null || true
	@echo "$(GREEN)✅ Mac host cleaned$(NC)"
	@echo "$(GREEN)   If files were found, your Docker volume setup needs fixing!$(NC)"

.PHONY: clean-all
clean-all: down clean ## Stop containers + clean Mac + remove volumes
	@echo "$(RED)⚠️  Removing Docker volumes (will delete all dependencies)...$(NC)"
	@docker compose down -v
	@echo "$(GREEN)✅ Complete cleanup done$(NC)"

# ========== Config ==========

.PHONY: config
config: ## Show effective docker compose configuration
	@docker compose config
