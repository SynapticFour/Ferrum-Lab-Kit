# Ferrum Lab Kit — Synaptic Four unified local lifecycle

COMPOSE_OUT ?= docker-compose.yml

.PHONY: help up up-with-infra down destroy install-cli

help:
	@echo "Ferrum Lab Kit — local lifecycle (Synaptic Four GA4GH stack)"
	@echo ""
	@echo "  make up              Start field-edge stack (first run: full install)"
	@echo "  make up-with-infra   Start Ferrum + ga4gh-infra co-deploy"
	@echo "  make down            Stop stack; keep volumes"
	@echo "  make destroy         Stop stack; remove volumes"
	@echo ""
	@echo "  make install-cli     Build lab-kit binary only (./install.sh)"
	@echo ""
	@echo "Scripts: scripts/stack-up.sh, scripts/stack-down.sh"

up:
	@chmod +x scripts/stack-up.sh scripts/stack-down.sh install-edge.sh 2>/dev/null || true
	./scripts/stack-up.sh

up-with-infra:
	@chmod +x scripts/stack-up.sh scripts/stack-down.sh install-edge.sh 2>/dev/null || true
	./scripts/stack-up.sh --with-infra

down:
	@chmod +x scripts/stack-down.sh 2>/dev/null || true
	./scripts/stack-down.sh

destroy:
	@chmod +x scripts/stack-down.sh 2>/dev/null || true
	./scripts/stack-down.sh --volumes

install-cli:
	./install.sh
