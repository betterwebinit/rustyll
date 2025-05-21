.PHONY: build test clean run demo help

# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean
	rm -rf _site
	rm -rf demo
	rm -rf target

# Check the project
check:
	cargo check

# Run the development server
run:
	cargo run -- serve

# Create and run demo site
demo: clean
	cargo build
	target/debug/rustyll build --source demo_content --destination _site --verbose
	target/debug/rustyll serve

# Install dependencies
deps:
	cargo install cargo-watch

# Watch for changes and rebuild
watch:
	cargo watch -x run -- serve

# Run with safe mode
safe:
	cargo run -- serve --safe

# Run with drafts enabled
drafts:
	cargo run -- serve --drafts

# Run with verbose output
verbose:
	cargo run -- serve --verbose

# Show help
help:
	cargo run -- --help 