#!/usr/bin/env bash
# run_renode.sh — Launch the STM32F407 ThreadX temperature sensor in Renode
#
# Prerequisites
#   • Renode installed (renode binary in PATH)
#   • CAP_NET_ADMIN (or run as root) so tap0 can be created
#   • The ARM ELF already built:
#       cmake --preset threadx-cortexm4-renode
#       cmake --build  --preset threadx-cortexm4-renode
#
# Usage
#   ./scripts/run_renode.sh [path/to/firmware.elf]
#
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

FIRMWARE="${1:-${REPO_ROOT}/build/cortexm4-renode/threadx_temp_sensor.elf}"
LOGFILE="/tmp/threadx_temp_sensor_uart.log"
RESC="${REPO_ROOT}/renode/stm32f4_temp_sensor.resc"

if [[ ! -f "${FIRMWARE}" ]]; then
    echo "ERROR: firmware not found at ${FIRMWARE}"
    echo "  Build it first:"
    echo "    cmake --preset threadx-cortexm4-renode"
    echo "    cmake --build  --preset threadx-cortexm4-renode"
    exit 1
fi

# ── TAP setup ────────────────────────────────────────────────────────────────
TAP_IF="tap0"
TAP_HOST_IP="192.168.100.1/24"

if ! ip link show "${TAP_IF}" &>/dev/null; then
    echo "[renode] Creating TAP interface ${TAP_IF}..."
    ip tuntap add dev "${TAP_IF}" mode tap
    ip addr add "${TAP_HOST_IP}" dev "${TAP_IF}"
    ip link set "${TAP_IF}" up
    echo "[renode] TAP ${TAP_IF} UP (${TAP_HOST_IP})"
else
    echo "[renode] TAP ${TAP_IF} already exists"
fi

# ── Launch Renode ─────────────────────────────────────────────────────────────
echo "[renode] Firmware : ${FIRMWARE}"
echo "[renode] UART log : ${LOGFILE}"
echo "[renode] Press Ctrl-C to stop."

renode --disable-xwt --plain \
    -e "\$firmware=@${FIRMWARE}; \$logfile=@${LOGFILE}; i @${RESC}; start"
