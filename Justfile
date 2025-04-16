#!/bin/env -S just --justfile

# run the application in demo mode
serve:
    [ -f "./db_password.txt" ] || echo "{{ choose('10', HEX) }}" > "./db_password.txt"
    docker compose up --build

# run static checking on the application
check: check-backend check-spelling check-formatting check-links

# check spelling throughout the application
check-spelling:
    codespell

# check hyperlink validity throughout the application
check-links:
    lychee --exclude "localhost.*" .

# check code formatting throughout the application
check-formatting:
    just --unstable --fmt --check   # justfile formatting
    cd backend && cargo fmt -- --check   # backend formatting
    taplo format --check   # TOML formatting

# run static checking on the backend
check-backend:
    cd backend && cargo check && cargo clippy

# run tests on the application
test: test-backend

# run tests on the backend
test-backend:
    cd backend && cargo test

# run git pre-commit checklist
run-pre-commit-hook: check test

# install pre-commit git hook
add-hooks:
    #!/usr/bin/env bash
    set -euo pipefail

    echo just run-pre-commit-hook > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
