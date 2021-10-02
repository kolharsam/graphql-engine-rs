.PHONY: test-clean test migrate fmt set-default-db lint build build-docs clean

# Runs all the tests with output to STDOUT
test:
	cargo test -- --nocapture

# Runs all the tests no outputs to STDOUT
test-clean:
	cargo test

# Useful for setting up the schema in test DB
migrate:
	psql $(DATABASE_URL) -f migrate/schema.sql

dev:
	cargo run -- -c $(DATABASE_URL)

fmt:
	cargo fmt

lint:
	cargo clippy

set-default-db:
	export DATABASE_URL="postgres://postgres:postgrespassword@localhost:5432/postgres"

build:
	cargo build --release

build-docs:
	cargo doc --release

clean:
	cargo clean
