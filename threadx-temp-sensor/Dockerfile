# syntax=docker/dockerfile:1.7
# ─────────────────────────────────────────────────────────────────────────────
# Stage 1 — Build
#   Fetches Eclipse ThreadX (Linux port) and OpenSOME/IP via CMake FetchContent,
#   then compiles the ThreadX temperature sensor.
# ─────────────────────────────────────────────────────────────────────────────
FROM docker.io/library/debian:bookworm AS build

RUN apt-get update && apt-get install -y --no-install-recommends \
        cmake ninja-build \
        g++ gcc \
        git ca-certificates \
        python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

COPY CMakeLists.txt CMakePresets.json ./
COPY src/ src/

# Configure using the Linux-ThreadX preset.
# FetchContent will clone ThreadX and OpenSOME/IP during configure.
RUN --mount=type=cache,target=/root/.cache/cmake \
    cmake --preset linux-threadx \
          -DFETCHCONTENT_UPDATES_DISCONNECTED=ON

# Build
RUN --mount=type=cache,target=/root/.cache/cmake \
    cmake --build --preset linux-threadx --parallel

# ─────────────────────────────────────────────────────────────────────────────
# Stage 2 — Runtime image
# ─────────────────────────────────────────────────────────────────────────────
FROM docker.io/library/debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        libstdc++6 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build \
    /workspace/build/linux-threadx/threadx_temp_sensor \
    /app/threadx_temp_sensor

RUN chmod +x /app/threadx_temp_sensor

# Target bridge hostname (Docker service name) and port.
# Override with -e BRIDGE_IP=<ip> BRIDGE_PORT=<port> if needed.
ENV BRIDGE_IP=someip-uprot-bridge
ENV BRIDGE_PORT=30501

ENTRYPOINT ["/app/threadx_temp_sensor"]
