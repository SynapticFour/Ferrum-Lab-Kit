# Ferrum Lab Kit — Synaptic Four unified local lifecycle

COMPOSE_OUT ?= docker-compose.yml

.PHONY: help up up-with-infra up-with-solum up-with-infra-solum down destroy install-cli pi-kit

help:
	@echo "Ferrum Lab Kit — local lifecycle (Synaptic Four GA4GH stack)"
	@echo ""
	@echo "  make up                    Start field-edge stack (first run: full install)"
	@echo "  make up-with-infra         Start Ferrum + ga4gh-infra co-deploy"
	@echo "  make up-with-solum         Start Ferrum + Solum sidecar companion"
	@echo "  make up-with-infra-solum   Start Ferrum + ga4gh-infra + Solum"
	@echo "  make down                  Stop stack; keep volumes"
	@echo "  make destroy               Stop stack; remove volumes"
	@echo ""
	@echo "  make install-cli           Build lab-kit binary only (./install.sh)"
	@echo "  make pi-kit                Generate portable Raspberry Pi kit → ./pi-kit"
	@echo ""
	@echo "Scripts: scripts/stack-up.sh, scripts/stack-down.sh"
	@echo "Env:     copy .env.example → .env"
	@echo "Pi docs: docs/RASPBERRY-PI.md"

up:
	@chmod +x scripts/stack-up.sh scripts/stack-down.sh install-edge.sh 2>/dev/null || true
	./scripts/stack-up.sh

up-with-infra:
	@chmod +x scripts/stack-up.sh scripts/stack-down.sh install-edge.sh 2>/dev/null || true
	./scripts/stack-up.sh --with-infra

up-with-solum:
	@chmod +x scripts/stack-up.sh scripts/stack-down.sh install-edge.sh 2>/dev/null || true
	./scripts/stack-up.sh --with-solum

up-with-infra-solum:
	@chmod +x scripts/stack-up.sh scripts/stack-down.sh install-edge.sh 2>/dev/null || true
	./scripts/stack-up.sh --with-infra --with-solum

down:
	@chmod +x scripts/stack-down.sh 2>/dev/null || true
	./scripts/stack-down.sh

destroy:
	@chmod +x scripts/stack-down.sh 2>/dev/null || true
	./scripts/stack-down.sh --volumes

install-cli:
	./install.sh

# Portable kit for USB / scp to a Raspberry Pi (see docs/RASPBERRY-PI.md).
# Optional: PI_PROFILE=field-edge+solum PI_RAM_GB=8 make pi-kit
PI_PROFILE ?= field-edge
PI_RAM_GB ?= 4
PI_OUT ?= pi-kit
pi-kit: install-cli
	./target/release/lab-kit generate raspberry-pi \
		--profile $(PI_PROFILE) \
		--ram-gb $(PI_RAM_GB) \
		--fragments deploy/docker-compose \
		--output $(PI_OUT)
	@echo "Kit ready: $(PI_OUT)/  →  copy to Pi and run ./install-on-pi.sh"
