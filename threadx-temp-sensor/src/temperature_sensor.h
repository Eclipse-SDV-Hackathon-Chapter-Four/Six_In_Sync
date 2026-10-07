/**
 * ThreadX Temperature Sensor — SOME/IP publisher + window state subscriber
 *
 * Publishes cabin temperature over SOME/IP (Service 0x1234, Event 0x8001).
 * Also subscribes to window state over SOME/IP (Service 0x5678, Event 0x8002)
 * to implement a thermal feedback loop: as the window opens, temperature drops.
 *
 * SOME/IP constants:
 *
 *   Temperature (published)
 *     Service ID  : 0x1234
 *     Event ID    : 0x8001
 *     Payload     : 4-byte big-endian IEEE 754 float (°C)
 *                 + 8-byte big-endian uint64 (milliseconds since epoch)
 *
 *   Window state (subscribed)
 *     Service ID  : 0x5678
 *     Event ID    : 0x8002
 *     Payload     : 1-byte window percentage (0-100)
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#pragma once

#include <cstdint>
#include <atomic>

/* ── SOME/IP constants ──────────────────────────────────────────────────── */
static constexpr uint16_t TEMP_SOMEIP_SERVICE_ID = 0x1234u;
static constexpr uint16_t TEMP_SOMEIP_EVENT_ID   = 0x8001u;
static constexpr uint16_t TEMP_SOMEIP_CLIENT_ID  = 0x0001u;
static constexpr uint8_t  TEMP_SOMEIP_IFACE_VER  = 0x01u;
static constexpr uint16_t TEMP_SOMEIP_PORT        = 30501u;

static constexpr uint16_t WINDOW_SOMEIP_SERVICE_ID = 0x5678u;
static constexpr uint16_t WINDOW_SOMEIP_EVENT_ID   = 0x8002u;

/* Packet sizes */
static constexpr std::size_t TEMP_SOMEIP_PACKET_SIZE   = 28u;  /* 16 header + 4 float + 8 ts */
static constexpr std::size_t WINDOW_SOMEIP_PACKET_SIZE = 17u;  /* 16 header + 1 byte         */

/* ── TemperatureSensor ──────────────────────────────────────────────────── */

class TemperatureSensor {
public:
    TemperatureSensor();
    ~TemperatureSensor();

    /** Open UDP sockets for both publishing and subscribing. */
    bool initialize();

    /** Publishing + receiving loop — blocks until stop() is called. */
    void run();

    /** Signal the loop to exit. */
    void stop();

private:
    void send_temperature(float celsius, uint64_t timestamp_ms);
    void handle_window_state(uint8_t window_percentage);
    float apply_thermal_model(float current_temp, uint8_t window_pct);

    int                  tx_sock_{-1};
    int                  rx_sock_{-1};
    std::atomic<bool>    running_{false};
    std::atomic<uint8_t> window_percentage_{0u};
    char                 bridge_host_[128];
    uint16_t             bridge_port_{TEMP_SOMEIP_PORT};
    uint16_t             session_id_{1u};
};

/** Platform-specific millisecond timestamp. */
uint64_t get_timestamp_ms();
