#!/usr/bin/env bash
# build_renode.sh — Cross-compile the temperature sensor for ARM Cortex-M4 / Renode
#
# Prerequisites
#   • arm-none-eabi-gcc in PATH
#   • cmake (≥ 3.20) and ninja in PATH
#   • Internet access (first run fetches Eclipse ThreadX + opensomeip)
#
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}/.."

echo "=== Building ThreadX temperature sensor for ARM Cortex-M4 (Renode) ==="

cmake --preset threadx-cortexm4-renode
cmake --build  --preset threadx-cortexm4-renode --parallel

ELF="build/cortexm4-renode/threadx_temp_sensor.elf"
if [[ -f "${ELF}" ]]; then
    SIZE=$(arm-none-eabi-size "${ELF}" 2>/dev/null || echo "(size unavailable)")
    echo ""
    echo "=== Build complete ==="
    echo "  ELF : ${ELF}"
    echo "  ${SIZE}"
    echo ""
    echo "Run in Renode:"
    echo "  ./scripts/run_renode.sh"
else
    echo "ERROR: ELF not found — check build output above."
    exit 1
fi
