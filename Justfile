default:
    just --list

dev:
    cd daemon && cargo run

web:
    cd web && pnpm dev

kiosk:
    cage -- just dev

schema:
    cd web && pnpm api-schema

check:
    cd daemon && cargo clippy --all-targets -- -D warnings
    cd daemon && cargo test
    cd web && pnpm typecheck
    cd web && pnpm lint

build:
    # The daemon embeds daemon/web-dist, so the copy belongs to building rather than to each
    # release workflow separately.
    cd web && pnpm build
    rm -rf daemon/web-dist
    cp -r web/dist daemon/web-dist
    cd daemon && cargo build --release
