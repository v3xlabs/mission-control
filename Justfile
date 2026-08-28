default:
    just --list

dev:
    cd daemon && cargo run

web:
    cd web && pnpm dev

kiosk:
    cage -- just dev

# A nested niri with missiond inside it, on .tmp/lab. Mod+Shift+E quits it.
lab: web-dist
    ./dev/lab.sh

# Talk to the missiond running in the lab.
lab-api *ARGS:
    curl -sS -H 'content-type: application/json' http://127.0.0.1:3177/api{{ARGS}}

schema:
    cd web && pnpm api-schema

check:
    cd daemon && cargo clippy --all-targets -- -D warnings
    cd daemon && cargo test
    cd web && pnpm typecheck
    cd web && pnpm lint

# Build the web UI into daemon/web-dist, which the daemon embeds.
web-dist:
    cd web && pnpm build
    rm -rf daemon/web-dist
    cp -r web/dist daemon/web-dist

build: web-dist
    cd daemon && cargo build --release
