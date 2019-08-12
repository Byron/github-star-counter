help:
	$(info -- Targets -------------------------------------------)
	$(info fixtures              | re-generate fixtures for tests)
	$(info tests                 | run all tests)

tests:
	cargo test --lib
fixtures:
	curl "https://api.github.com/users/Byron" > test/fixtures/github.com-byron.json
	curl "https://api.github.com/users/Byron/repos?per_page=100&page=1" > test/fixtures/github.com-byron-repos-page-1.json
