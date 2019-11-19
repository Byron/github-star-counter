help:
	$(info -- Targets -------------------------------------------)
	$(info tests                                  | run all tests)
	$(info -- Less Useful ---------------------------------------)
	$(info fixtures              | re-generate fixtures for tests)

tests:
	cargo +nightly test --lib

fixtures:
	curl "https://api.github.com/users/Byron" > test/fixtures/github.com-byron.json
	curl "https://api.github.com/users/Byron/orgs" > test/fixtures/github.com-byron-orgs.json
	curl "https://api.github.com/users/Byron/repos?per_page=100&page=1" > test/fixtures/github.com-byron-repos-page-1.json
