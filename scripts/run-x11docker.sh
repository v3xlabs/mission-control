#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

IMAGE_NAME="${MC_DEV_IMAGE:-v3x-mission-control-dev:x11}"
DISPLAY_SIZE="${MC_DISPLAY_SIZE:-1280x720}"
RUST_BIN="${MC_CARGO_BIN:-v3x-mission-control}"
EXTRA_OPTS="${MC_X11DOCKER_OPTS:-}"
BUILD_PROFILE="${MC_BUILD_PROFILE:-dev}"

if ! command -v docker >/dev/null 2>&1; then
    echo "docker is required but not installed or not on PATH" >&2
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is required but not installed or not on PATH" >&2
    exit 1
fi

if ! command -v x11docker >/dev/null 2>&1; then
    cat >&2 <<'EOF'
x11docker is required for this workflow.
See installation instructions at https://github.com/mviereck/x11docker#installation
EOF
    exit 1
fi

APP_WORKDIR="/workspace/app"

CARGO_CMD=(cargo build --bin "${RUST_BIN}")
TARGET_SUBDIR="debug"

case "${BUILD_PROFILE}" in
    release)
        CARGO_CMD+=("--release")
        TARGET_SUBDIR="release"
        ;;
    dev|"")
        TARGET_SUBDIR="debug"
        ;;
    *)
        CARGO_CMD+=("--profile" "${BUILD_PROFILE}")
        TARGET_SUBDIR="${BUILD_PROFILE}"
        ;;
esac

if [[ -n "${MC_CARGO_FEATURES:-}" ]]; then
    CARGO_CMD+=("--features" "${MC_CARGO_FEATURES}")
fi

if [[ "${MC_CARGO_NO_DEFAULT_FEATURES:-}" =~ ^(1|true|yes)$ ]]; then
    CARGO_CMD+=("--no-default-features")
fi

if [[ "${MC_CARGO_ALL_FEATURES:-}" =~ ^(1|true|yes)$ ]]; then
    CARGO_CMD+=("--all-features")
fi

if [[ -n "${MC_CARGO_FLAGS:-}" ]]; then
    # shellcheck disable=SC2206
    ADDITIONAL_FLAGS=(${MC_CARGO_FLAGS})
    CARGO_CMD+=("${ADDITIONAL_FLAGS[@]}")
fi

echo "[cargo] Building ${RUST_BIN} (${BUILD_PROFILE}) on host..."
(
    cd "${REPO_ROOT}/app"
    "${CARGO_CMD[@]}"
)

BINARY_RELATIVE="target/${TARGET_SUBDIR}/${RUST_BIN}"
BINARY_PATH="${REPO_ROOT}/app/${BINARY_RELATIVE}"

if [[ ! -x "${BINARY_PATH}" ]]; then
    echo "Built binary not found at ${BINARY_PATH}" >&2
    exit 1
fi

BIN_ARGS=()
if [[ -n "${MC_APP_ARGS:-}" ]]; then
    # shellcheck disable=SC2206
    BIN_ARGS=(${MC_APP_ARGS})
fi

RUN_COMMAND_ARGS=("./${BINARY_RELATIVE}")
RUN_COMMAND_ARGS+=("${BIN_ARGS[@]}")
printf -v RUN_COMMAND_STR '%q ' "${RUN_COMMAND_ARGS[@]}"
RUN_COMMAND_STR="${RUN_COMMAND_STR% }"

START_CMD="cd ${APP_WORKDIR} && exec ${RUN_COMMAND_STR}"

echo "[x11docker] Building dev image ${IMAGE_NAME}..."
docker build \
    --file "${REPO_ROOT}/docker/dev-x11.Dockerfile" \
    --tag "${IMAGE_NAME}" \
    "${REPO_ROOT}"

EXTRA_OPTS="${MC_X11DOCKER_OPTS:-${EXTRA_OPTS:-}}"
USE_XEPHYR=true

if [[ -z "${EXTRA_OPTS}" && -n "${WAYLAND_DISPLAY:-}" ]]; then
    EXTRA_OPTS="--weston-xwayland"
    USE_XEPHYR=false
fi

case " ${EXTRA_OPTS} " in
    *" --weston-xwayland "*|*" --wayland "*|*" --xpra "*|*" --nxagent "*)
        USE_XEPHYR=false
        ;;
    *" --xephyr "*)
        USE_XEPHYR=true
        ;;
esac

if "${USE_XEPHYR}"; then
    if ! command -v Xephyr >/dev/null 2>&1; then
        printf '%s\n' "Xephyr is required for the default --xephyr backend." \
                      "Install it (e.g. sudo apt install xserver-xephyr) or set MC_X11DOCKER_OPTS='--weston-xwayland' to use the Wayland backend." >&2
        exit 1
    fi
fi

echo "[x11docker] Launching ${RUST_BIN} inside container display (${DISPLAY_SIZE})..."
CMD=(x11docker)

CMD+=("-I")

if "${USE_XEPHYR}"; then
    CMD+=("--xephyr" "--size" "${DISPLAY_SIZE}" "--wm=none")
else
    CMD+=("--size" "${DISPLAY_SIZE}")
fi

CMD+=(
    "--clipboard"
    "--pulseaudio"
    "--home"
    "--env" "RUST_LOG=${RUST_LOG:-info}"
    "--env" "CHROMIUM_BINARY=/usr/bin/chromium"
    "--env" "CHROMIUM_EXTRA_ARGS=--no-sandbox,--disable-dev-shm-usage,--disable-gpu"
    "--env" "MC_CHROMIUM_PROFILE_DIR=/workspace/.dev-chromium-profile"
)

if "${USE_XEPHYR}"; then
    CMD+=("--env" "QT_QPA_PLATFORM=xcb")
fi

if [[ -n "${EXTRA_OPTS}" ]]; then
    # shellcheck disable=SC2206
    EXTRA_ARRAY=(${EXTRA_OPTS})
    CMD+=("${EXTRA_ARRAY[@]}")
fi

CMD+=("--workdir" "${APP_WORKDIR}")

DOCKER_OPTS=("--volume=${REPO_ROOT}:${APP_WORKDIR%/app}:rw" "--publish=3000:3000")

CMD+=("--")
CMD+=("${DOCKER_OPTS[@]}")
CMD+=("${IMAGE_NAME}" "bash" "-lc" "${START_CMD}")

printf '[x11docker] Command:'; printf ' %q' "${CMD[@]}"; printf '\n'
QT_QPA_PLATFORM=${QT_QPA_PLATFORM:-xcb} "${CMD[@]}"
