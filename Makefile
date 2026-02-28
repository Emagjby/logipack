SHELL := /bin/bash

.PHONY: help test fmt clippy dev-api dev-web build-web

help:
	@echo "Targets:"
	@echo "  test      - cargo test (serialized output with summary)" @echo "  fmt       - cargo fmt --all" @echo "  clippy    - cargo clippy --workspace --all-targets --all-features"
	@echo "  dev-api   - run hub API"
	@echo "  dev-web   - run SvelteKit dev server"
	@echo "  build-web - build SvelteKit"

test:
	@set -o pipefail; \
	cargo test -- --test-threads=1 2>&1 \
	| tee /dev/tty \
	| rg '^test result:' \
	| awk 'BEGIN{ \
		esc=sprintf("%c",27); reset=esc"[0m"; bold=esc"[1m"; \
		c_total=esc"[38;2;90;200;255m"; \
		c_pass =esc"[38;2;80;220;140m"; \
		c_fail =esc"[38;2;255;90;90m"; \
		c_ign  =esc"[38;2;255;170;60m"; \
		c_meas =esc"[38;2;170;140;255m"; \
		c_filt =esc"[38;2;255;120;200m"; \
	} \
	{ \
		for(i=1;i<=NF;i++){ \
			if($$(i+1)=="passed;")   p+=$$i; \
			if($$(i+1)=="failed;")   f+=$$i; \
			if($$(i+1)=="ignored;")  ig+=$$i; \
			if($$(i+1)=="measured;") m+=$$i; \
			if($$(i+1)=="filtered")  fo+=$$i; \
		} \
	} \
	END{ \
		printf("\n\n"); \
		printf("%s%s%sTOTAL:%s ", bold, c_total, bold, reset); \
		printf("%s%s%d passed;%s ",  bold, c_pass, p+0,  reset); \
		printf("%s%s%d failed;%s ",  bold, c_fail, f+0,  reset); \
		printf("%s%s%d ignored;%s ", bold, c_ign,  ig+0, reset); \
		printf("%s%s%d measured;%s ",bold, c_meas, m+0,  reset); \
		printf("%s%s%d filtered out;%s\n", bold, c_filt, fo+0, reset); \
		printf("\n"); \
		exit(f>0); \
	}'

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features

dev-api:
	cd logipack-hub/hub-api && cargo run

dev-web:
	cd logipack-hub/hub-web && bun run dev --host

build-web:
	cd logipack-hub/hub-web && bun run build 
