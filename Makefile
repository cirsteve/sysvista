.PHONY: build-cli build-web dev-web scan clean

build-cli:
	cd sysvista-cli && cargo build --release

build-web:
	cd sysvista-web && npm run build

dev-web:
	cd sysvista-web && npm run dev

scan:
	cd sysvista-cli && cargo run -- scan $(TARGET) -o ../sysvista-web/public/sample-output.json

clean:
	cd sysvista-cli && cargo clean
	cd sysvista-web && rm -rf dist node_modules
