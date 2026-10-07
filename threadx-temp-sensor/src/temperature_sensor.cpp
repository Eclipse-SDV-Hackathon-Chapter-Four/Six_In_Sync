/**
 * TemperatureSensor implementation.
 *
 * Two build paths share most of this file:
 *
 *  Linux / ThreadX-Linux-port (Docker service)
 *    • Standard POSIX sockets (sys/socket.h, netdb.h)
 *    • std::chrono for timestamps
 *    • Defined automatically when __linux__ is set
 *
 *  ARM / Renode (STM32F407 + ThreadX + lwIP)
 *    • lwIP BSD-socket layer (LWIP_COMPAT_SOCKETS=1)
 *    • tx_time_get() for timestamps
 *    • Selected by defining THREADX_ARM_TARGET=1
 *
 * Both paths use the same ThreadX thread-sleep API (tx_thread_sleep).
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#include "temperature_sensor.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <tx_api.h>

/* ── Platform socket includes ───────────────────────────────────────────── */
#ifdef THREADX_ARM_TARGET
    /* lwIP with BSD-socket compatibility layer */
#   include <lwip/sockets.h>
#   include <lwip/netdb.h>
    /* For ARM we expect a numeric IP via BRIDGE_IP env / compile flag */
#else
    /* Linux POSIX sockets */
#   include <sys/socket.h>
#   include <netinet/in.h>
#   include <arpa/inet.h>
#   include <netdb.h>
#   include <unistd.h>
#   include <fcntl.h>
#   include <chrono>
#endif

/* ── Timestamp helper ───────────────────────────────────────────────────── */
#ifdef THREADX_ARM_TARGET
uint64_t get_timestamp_ms()
{
    /* Convert ThreadX ticks to milliseconds */
    return static_cast<uint64_t>(tx_time_get()) * 1000ULL
           / static_cast<uint64_t>(TX_TIMER_TICKS_PER_SECOND);
}
#else
uint64_t get_timestamp_ms()
{
    using namespace std::chrono;
    return static_cast<uint64_t>(
        duration_cast<milliseconds>(
            system_clock::now().time_since_epoch())
        .count());
}
#endif

/* ── TemperatureSensor ──────────────────────────────────────────────────── */

TemperatureSensor::TemperatureSensor()
{
    const char* host = getenv("BRIDGE_IP");
    const char* port = getenv("BRIDGE_PORT");

    strncpy(bridge_host_,
            host ? host : "someip-uprot-bridge",
            sizeof(bridge_host_) - 1);
    bridge_host_[sizeof(bridge_host_) - 1] = '\0';

    if (port) {
        bridge_port_ = static_cast<uint16_t>(atoi(port));
    }
}

TemperatureSensor::~TemperatureSensor()
{
    if (tx_sock_ >= 0) {
#ifdef THREADX_ARM_TARGET
        lwip_close(tx_sock_);
#else
        close(tx_sock_);
#endif
        tx_sock_ = -1;
    }
    if (rx_sock_ >= 0) {
#ifdef THREADX_ARM_TARGET
        lwip_close(rx_sock_);
#else
        close(rx_sock_);
#endif
        rx_sock_ = -1;
    }
}

bool TemperatureSensor::initialize()
{
    /* TX socket: send temperature notifications to the bridge */
    tx_sock_ = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (tx_sock_ < 0) {
        perror("[temp-sensor] tx_socket");
        return false;
    }

    /* RX socket: receive window state notifications on local port 30502
     * (window bridge will send SOME/IP window events here)                 */
    rx_sock_ = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if (rx_sock_ < 0) {
        perror("[temp-sensor] rx_socket");
        return false;
    }

    /* Bind to 0.0.0.0:30502 to receive window state */
    struct sockaddr_in local_addr;
    memset(&local_addr, 0, sizeof(local_addr));
    local_addr.sin_family      = AF_INET;
    local_addr.sin_addr.s_addr = htonl(INADDR_ANY);
    local_addr.sin_port        = htons(30502);

    if (bind(rx_sock_, (struct sockaddr*)&local_addr, sizeof(local_addr)) < 0) {
        perror("[temp-sensor] bind");
        return false;
    }

    /* Set non-blocking on rx_sock so we can poll without blocking the sender */
#ifndef THREADX_ARM_TARGET
    int flags = fcntl(rx_sock_, F_GETFL, 0);
    fcntl(rx_sock_, F_SETFL, flags | O_NONBLOCK);
#endif

    printf("[temp-sensor] publishing to %s:%u\n", bridge_host_, bridge_port_);
    printf("[temp-sensor] listening for window state on 0.0.0.0:30502\n");
    running_ = true;
    return true;
}

void TemperatureSensor::stop()
{
    running_ = false;
}

/**
 * Craft and send a SOME/IP v1 Notification packet over UDP.
 *
 * Wire layout (28 bytes):
 *   [0..1]  Service ID  0x1234  (big-endian)
 *   [2..3]  Event ID    0x8001  (big-endian)
 *   [4..7]  Length      0x0014  (= 20 = 8 header tail + 12 payload)
 *   [8..9]  Client ID   0x0001
 *   [10..11] Session ID (incrementing)
 *   [12]    Protocol Version = 0x01
 *   [13]    Interface Version = 0x01
 *   [14]    Message Type = 0x02 (NOTIFICATION)
 *   [15]    Return Code  = 0x00 (E_OK)
 *   [16..19] Temperature °C  (IEEE 754 big-endian float)
 *   [20..27] Timestamp ms    (uint64 big-endian)
 */
void TemperatureSensor::send_temperature(float celsius, uint64_t timestamp_ms)
{
    uint8_t buf[TEMP_SOMEIP_PACKET_SIZE] = {};

    /* Service ID */
    buf[0] = 0x12; buf[1] = 0x34;
    /* Event ID */
    buf[2] = 0x80; buf[3] = 0x01;
    /* Length = 20 (everything after the Length field itself) */
    buf[4] = 0x00; buf[5] = 0x00; buf[6] = 0x00; buf[7] = 0x14;
    /* Client ID */
    buf[8]  = static_cast<uint8_t>(TEMP_SOMEIP_CLIENT_ID >> 8);
    buf[9]  = static_cast<uint8_t>(TEMP_SOMEIP_CLIENT_ID);
    /* Session ID */
    buf[10] = static_cast<uint8_t>(session_id_ >> 8);
    buf[11] = static_cast<uint8_t>(session_id_);
    ++session_id_;
    /* Protocol / Interface version */
    buf[12] = 0x01;
    buf[13] = TEMP_SOMEIP_IFACE_VER;
    /* Message Type: NOTIFICATION */
    buf[14] = 0x02;
    /* Return Code: E_OK */
    buf[15] = 0x00;

    /* Temperature payload — big-endian float */
    uint32_t temp_bits;
    memcpy(&temp_bits, &celsius, sizeof(float));
    buf[16] = static_cast<uint8_t>(temp_bits >> 24);
    buf[17] = static_cast<uint8_t>(temp_bits >> 16);
    buf[18] = static_cast<uint8_t>(temp_bits >>  8);
    buf[19] = static_cast<uint8_t>(temp_bits);

    /* Timestamp payload — big-endian uint64 */
    buf[20] = static_cast<uint8_t>(timestamp_ms >> 56);
    buf[21] = static_cast<uint8_t>(timestamp_ms >> 48);
    buf[22] = static_cast<uint8_t>(timestamp_ms >> 40);
    buf[23] = static_cast<uint8_t>(timestamp_ms >> 32);
    buf[24] = static_cast<uint8_t>(timestamp_ms >> 24);
    buf[25] = static_cast<uint8_t>(timestamp_ms >> 16);
    buf[26] = static_cast<uint8_t>(timestamp_ms >>  8);
    buf[27] = static_cast<uint8_t>(timestamp_ms);

    /* Resolve destination (DNS works on Linux; use numeric IP on ARM) */
    struct addrinfo hints{}, *res = nullptr;
    hints.ai_family   = AF_INET;
    hints.ai_socktype = SOCK_DGRAM;
    char port_str[8];
    snprintf(port_str, sizeof(port_str), "%u", bridge_port_);

    if (getaddrinfo(bridge_host_, port_str, &hints, &res) != 0 || !res) {
        fprintf(stderr, "[temp-sensor] getaddrinfo failed for %s\n",
                bridge_host_);
        return;
    }

    ssize_t sent = sendto(tx_sock_, buf, sizeof(buf), 0,
                          res->ai_addr,
                          static_cast<socklen_t>(res->ai_addrlen));
    freeaddrinfo(res);

    if (sent < 0) {
        perror("[temp-sensor] sendto");
    }
}

/**
 * Thermal model: apply cooling/heating based on window state.
 * Mirrors the temperature_sim logic.
 */
float TemperatureSensor::apply_thermal_model(float current_temp, uint8_t window_pct)
{
    static constexpr float HEAT_GAIN_PER_STEP         = 0.18f;
    static constexpr float COOLING_PER_STEP_FULL_WINDOW = 1.20f;
    static constexpr float AMBIENT_CELSIUS            = 24.0f;
    static constexpr float MAX_CELSIUS                = 48.0f;

    float window_normalized = (window_pct > 100 ? 100 : window_pct) / 100.0f;
    float cooling = window_normalized * COOLING_PER_STEP_FULL_WINDOW;
    float delta = HEAT_GAIN_PER_STEP - cooling;
    float next_temp = current_temp + delta;

    /* Clamp to plausible range */
    return next_temp < AMBIENT_CELSIUS ? AMBIENT_CELSIUS
         : next_temp > MAX_CELSIUS     ? MAX_CELSIUS
         : next_temp;
}

/**
 * Parse SOME/IP window state notification packet.
 * Wire format:
 *   [0..1]  Service ID  0x5678
 *   [2..3]  Event ID    0x8002
 *   [4..7]  Length      0x0001 (1 byte payload)
 *   [8..15] Header tail
 *   [16]    Window percentage (0-100)
 */
void TemperatureSensor::handle_window_state(uint8_t window_percentage)
{
    if (window_percentage <= 100) {
        window_percentage_.store(window_percentage);
        printf("[temp-sensor] window state: %u%%\n", window_percentage);
    }
}

void TemperatureSensor::run()
{
    /* Initial temperature ramp stages (like temperature_sim) */
    struct Stage { float celsius; int ticks; };
    const Stage stages[] = {
        { 26.0f, 2  * TX_TIMER_TICKS_PER_SECOND },
        { 36.0f, 5  * TX_TIMER_TICKS_PER_SECOND },
        { 43.0f, 10 * TX_TIMER_TICKS_PER_SECOND },
    };

    for (const auto& s : stages) {
        if (!running_) return;
        send_temperature(s.celsius, get_timestamp_ms());
        printf("[temp-sensor] %.1f C\n", s.celsius);
        tx_thread_sleep(static_cast<ULONG>(s.ticks));
    }

    /* Steady-state with thermal feedback loop */
    float temperature = 43.0f;
    uint8_t buf[WINDOW_SOMEIP_PACKET_SIZE];

    while (running_) {
        /* Try to receive window state updates (non-blocking) */
        struct sockaddr_in from;
        socklen_t from_len = sizeof(from);
        ssize_t recv_len = recvfrom(rx_sock_, buf, sizeof(buf), 0,
                                     (struct sockaddr*)&from, &from_len);

        if (recv_len > 0) {
            /* Parse SOME/IP window state packet if it looks valid */
            if (recv_len >= WINDOW_SOMEIP_PACKET_SIZE) {
                uint16_t svc_id = (buf[0] << 8) | buf[1];
                uint16_t evt_id = (buf[2] << 8) | buf[3];

                if (svc_id == WINDOW_SOMEIP_SERVICE_ID &&
                    evt_id == WINDOW_SOMEIP_EVENT_ID) {
                    uint8_t window_pct = buf[16];
                    handle_window_state(window_pct);
                }
            }
        }

        /* Apply thermal model based on current window state */
        uint8_t win_pct = window_percentage_.load();
        temperature = apply_thermal_model(temperature, win_pct);

        /* Publish adjusted temperature */
        send_temperature(temperature, get_timestamp_ms());
        printf("[temp-sensor] %.1f C (window: %u%%)\n", temperature, win_pct);

        /* Sleep before next cycle */
        tx_thread_sleep(1 * TX_TIMER_TICKS_PER_SECOND);
    }

    printf("[temp-sensor] stopped\n");
}
