.PHONY: build-cli build-analyzer build-web dev-web scan clean

build-analyzer:
	cd sysvista-analyzer && npm ci && npm run build

build-cli: build-analyzer
	cd sysvista-cli && cargo build --release

build-web:
	cd sysvista-web && npm run build

dev-web:
	cd sysvista-web && npm run dev

scan:
	test -n "$(TARGET)"
	cargo run --manifest-path sysvista-cli/Cargo.toml -- scan "$(TARGET)" --output "bundles/$(notdir $(abspath $(TARGET)))"
	cargo run --manifest-path sysvista-cli/Cargo.toml -- bundle --input "bundles/$(notdir $(abspath $(TARGET)))" --archive "bundles/$(notdir $(abspath $(TARGET))).zip" --source-root "$(abspath $(TARGET))"

clean:
	cd sysvista-cli && cargo clean
	cd sysvista-web && rm -rf dist node_modules
