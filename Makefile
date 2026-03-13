# Monel development tasks

.PHONY: book book-serve book-install

# Install doc tooling (one-time setup)
book-install:
	cargo xtask book-install

# Build the spec book
book:
	cargo xtask book-build

# Serve with live reload at http://localhost:3000
book-serve:
	cargo xtask book-serve
