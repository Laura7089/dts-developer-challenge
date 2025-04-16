#!/bin/env -S just --justfile

# run the application in demo mode
serve:
    [ -f "./db_password.txt" ] || echo "{{ choose('10', HEX) }}" > "./db_password.txt"
    docker compose up --build
