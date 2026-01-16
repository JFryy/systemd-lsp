.PHONY: build test test-coverage test-section-coverage test-directive-coverage clean

build: test
	cargo build --release

test:
	@cargo test --verbose

test-section-coverage:
	@cargo test --test lsp_tests all_sections_have_documentation -- --ignored --no-capture

test-directive-coverage:
	@cargo test --test lsp_tests all_directives_have_documentation -- --ignored --no-capture

test-coverage:
	@echo "=== Checking Section Documentation Coverage ==="
	@cargo test --test lsp_tests all_sections_have_documentation -- --ignored --no-capture || true
	@echo ""
	@echo "=== Checking Directive Documentation Coverage ==="
	@cargo test --test lsp_tests all_directives_have_documentation -- --ignored --no-capture || true

clean:
	cargo clean
