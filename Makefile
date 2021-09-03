.PHONY: test-clean test migrate

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
