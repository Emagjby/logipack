SHELL := /bin/bash

.PHONY: help test drytest fmt clippy dev-api dev-web build-web

help:
	@echo "Targets:"
	@echo "  test      - cargo test (serialized output with summary)" @echo "  fmt       - cargo fmt --all" @echo "  clippy    - cargo clippy --workspace --all-targets --all-features"
	@echo "  drytest   - cargo test (serialized, no tty)"
	@echo "  dev-api   - run hub API"
	@echo "  dev-web   - run SvelteKit dev server"
	@echo "  build-web - build SvelteKit"

test:
	@set -o pipefail; \
	TEE_TARGET=$$(tty -s && echo /dev/tty || echo /dev/null); \
	RUST_SUMMARY=$$(mktemp); \
	cargo test -- --test-threads=1 2>&1 \
	| tee $$TEE_TARGET \
	| rg '^test result:' \
	| awk -v out=$$RUST_SUMMARY 'BEGIN{ \
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
		printf("%d %d %d %d %d\n", p+0, f+0, ig+0, m+0, fo+0) > out; \
		exit(f>0); \
	}'; \
	BUN_OUTPUT=$$(mktemp); \
	BUN_SUMMARY=$$(mktemp); \
	cd logipack-hub/hub-web && bun test 2>&1 | tee $$BUN_OUTPUT; \
	BUN_STATUS=$${PIPESTATUS[0]}; \
	awk -v out=$$BUN_SUMMARY 'BEGIN{ \
		esc=sprintf("%c",27); reset=esc"[0m"; bold=esc"[1m"; \
		c_label=esc"[38;2;90;200;255m"; \
		c_pass =esc"[38;2;80;220;140m"; \
		c_fail =esc"[38;2;255;90;90m"; \
	} \
	/^[[:space:]]*[0-9]+[[:space:]]+pass$$/ { p=$$1 } \
	/^[[:space:]]*[0-9]+[[:space:]]+fail$$/ { f=$$1 } \
	END{ \
		printf("%s%s%sBUN:%s ", bold, c_label, bold, reset); \
		printf("%s%s%d passed;%s ", bold, c_pass, p+0, reset); \
		printf("%s%s%d failed;%s\n", bold, c_fail, f+0, reset); \
		printf("%d %d\n", p+0, f+0) > out; \
	}' $$BUN_OUTPUT; \
	awk 'BEGIN{ \
		esc=sprintf("%c",27); reset=esc"[0m"; bold=esc"[1m"; \
		c_label=esc"[38;2;90;200;255m"; \
		c_pass =esc"[38;2;80;220;140m"; \
		c_fail =esc"[38;2;255;90;90m"; \
	} \
	NR==FNR { rp=$$1; rf=$$2; next } \
	{ bp=$$1; bf=$$2 } \
	END{ \
		printf("%s%s%sCOMBINED:%s ", bold, c_label, bold, reset); \
		printf("%s%s%d passed;%s ", bold, c_pass, rp+bp, reset); \
		printf("%s%s%d failed;%s\n", bold, c_fail, rf+bf, reset); \
	}' $$RUST_SUMMARY $$BUN_SUMMARY; \
	rm -f $$BUN_OUTPUT; \
	rm -f $$RUST_SUMMARY $$BUN_SUMMARY; \
	exit $$BUN_STATUS

drytest:
	@set -o pipefail; \
	OUTPUT=$$(mktemp); \
	RUST_SUMMARY=$$(mktemp); \
	cargo test -- --test-threads=1 2>&1 \
	| tee $$OUTPUT; \
	STATUS=$${PIPESTATUS[0]}; \
	rg '^test result:' $$OUTPUT || true \
	| awk -v out=$$RUST_SUMMARY 'BEGIN{ \
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
		printf("%d %d %d %d %d\n", p+0, f+0, ig+0, m+0, fo+0) > out; \
			exit(f>0); \
		}'; \
	rm -f $$OUTPUT; \
	if [ $$STATUS -ne 0 ]; then exit $$STATUS; fi; \
	BUN_OUTPUT=$$(mktemp); \
	BUN_SUMMARY=$$(mktemp); \
	cd logipack-hub/hub-web && bun test 2>&1 | tee $$BUN_OUTPUT; \
	BUN_STATUS=$${PIPESTATUS[0]}; \
	awk -v out=$$BUN_SUMMARY 'BEGIN{ \
		esc=sprintf("%c",27); reset=esc"[0m"; bold=esc"[1m"; \
		c_label=esc"[38;2;90;200;255m"; \
		c_pass =esc"[38;2;80;220;140m"; \
		c_fail =esc"[38;2;255;90;90m"; \
	} \
	/^[[:space:]]*[0-9]+[[:space:]]+pass$$/ { p=$$1 } \
	/^[[:space:]]*[0-9]+[[:space:]]+fail$$/ { f=$$1 } \
	END{ \
		printf("%s%s%sBUN:%s ", bold, c_label, bold, reset); \
		printf("%s%s%d passed;%s ", bold, c_pass, p+0, reset); \
		printf("%s%s%d failed;%s\n", bold, c_fail, f+0, reset); \
		printf("%d %d\n", p+0, f+0) > out; \
	}' $$BUN_OUTPUT; \
	awk 'BEGIN{ \
		esc=sprintf("%c",27); reset=esc"[0m"; bold=esc"[1m"; \
		c_label=esc"[38;2;90;200;255m"; \
		c_pass =esc"[38;2;80;220;140m"; \
		c_fail =esc"[38;2;255;90;90m"; \
	} \
	NR==FNR { rp=$$1; rf=$$2; next } \
	{ bp=$$1; bf=$$2 } \
	END{ \
		printf("%s%s%sCOMBINED:%s ", bold, c_label, bold, reset); \
		printf("%s%s%d passed;%s ", bold, c_pass, rp+bp, reset); \
		printf("%s%s%d failed;%s\n", bold, c_fail, rf+bf, reset); \
	}' $$RUST_SUMMARY $$BUN_SUMMARY; \
	rm -f $$BUN_OUTPUT; \
	rm -f $$RUST_SUMMARY $$BUN_SUMMARY; \
	exit $$BUN_STATUS

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
