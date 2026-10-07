/*
 * Copyright (c) Microsoft
 * Copyright (c) 2024 Eclipse Foundation
 *
 * This program and the accompanying materials are made available
 * under the terms of the MIT license.
 *
 * SPDX-License-Identifier: MIT
 *
 * AI assistance disclosure:
 * This file was modified with assistance from GitHub Copilot (GPT-5.6 Luna).
 * The changes were reviewed by the author.
 */
#include "temperature_publisher.h"

#include "nanoprintf.h"
#include <stdio.h>
#include <string.h>

#include "cloud_config.h"
#include "sntp_client.h"
#include "tx_api.h"
#include "wwd_networking.h"

#define SOMEIP_PACKET_SIZE 28U
#define SOMEIP_CLIENT_ID   0x0001U
#define SOMEIP_INTERFACE_VERSION 0x01U
#define SOMEIP_MESSAGE_TYPE_NOTIFICATION 0x02U
#define SOMEIP_RETURN_CODE_OK 0x00U

static NX_UDP_SOCKET temperature_socket;
static UINT publisher_initialized;
static UINT session_id = 1U;

static void format_u64(uint64_t value, char* output, size_t output_size)
{
    char reversed[20];
    size_t length = 0U;
    size_t index;

    if (output_size == 0U)
    {
        return;
    }

    if (value == 0U)
    {
        output[0] = '0';
        output[1] = '\0';
        return;
    }

    while (value > 0U && length < sizeof(reversed))
    {
        reversed[length++] = (char)('0' + (value % 10U));
        value /= 10U;
    }

    if (length + 1U > output_size)
    {
        output[0] = '\0';
        return;
    }

    for (index = 0U; index < length; ++index)
    {
        output[index] = reversed[length - index - 1U];
    }
    output[length] = '\0';
}

static void write_u16_be(uint8_t* buffer, UINT value)
{
    buffer[0] = (uint8_t)(value >> 8);
    buffer[1] = (uint8_t)value;
}

static void write_u32_be(uint8_t* buffer, uint32_t value)
{
    buffer[0] = (uint8_t)(value >> 24);
    buffer[1] = (uint8_t)(value >> 16);
    buffer[2] = (uint8_t)(value >> 8);
    buffer[3] = (uint8_t)value;
}

static void write_u64_be(uint8_t* buffer, uint64_t value)
{
    buffer[0] = (uint8_t)(value >> 56);
    buffer[1] = (uint8_t)(value >> 48);
    buffer[2] = (uint8_t)(value >> 40);
    buffer[3] = (uint8_t)(value >> 32);
    buffer[4] = (uint8_t)(value >> 24);
    buffer[5] = (uint8_t)(value >> 16);
    buffer[6] = (uint8_t)(value >> 8);
    buffer[7] = (uint8_t)value;
}

UINT temperature_publisher_init(void)
{
    UINT status;

    if (publisher_initialized)
    {
        return NX_SUCCESS;
    }

    status = nx_udp_socket_create(
        &nx_ip,
        &temperature_socket,
        "Temperature SOMEIP Publisher",
        NX_IP_NORMAL,
        NX_FRAGMENT_OKAY,
        NX_IP_TIME_TO_LIVE,
        5);
    if (status != NX_SUCCESS)
    {
        printf("ERROR: temperature UDP socket create (0x%08x)\r\n", status);
        return status;
    }

    status = nx_udp_socket_bind(&temperature_socket, 30502, TX_NO_WAIT);
    if (status != NX_SUCCESS)
    {
        printf("ERROR: temperature UDP socket bind (0x%08x)\r\n", status);
        nx_udp_socket_delete(&temperature_socket);
        return status;
    }

    publisher_initialized = 1U;
    printf("Temperature SOME/IP publisher initialized\r\n");
    return NX_SUCCESS;
}

UINT temperature_publisher_send(float temperature_celsius)
{
    uint8_t payload[SOMEIP_PACKET_SIZE] = {0};
    uint32_t temperature_bits;
    uint64_t timestamp_ms;
    NX_PACKET* packet;
    UINT status;

    if (!publisher_initialized)
    {
        return NX_NOT_SUCCESSFUL;
    }

    write_u16_be(&payload[0], SOMEIP_SERVICE_ID);
    write_u16_be(&payload[2], SOMEIP_EVENT_ID);
    write_u32_be(&payload[4], 20U);
    write_u16_be(&payload[8], SOMEIP_CLIENT_ID);
    write_u16_be(&payload[10], session_id++);
    payload[12] = 0x01U;
    payload[13] = SOMEIP_INTERFACE_VERSION;
    payload[14] = SOMEIP_MESSAGE_TYPE_NOTIFICATION;
    payload[15] = SOMEIP_RETURN_CODE_OK;

    memcpy(&temperature_bits, &temperature_celsius, sizeof(temperature_bits));
    write_u32_be(&payload[16], temperature_bits);

    timestamp_ms = (uint64_t)sntp_time_get() * 1000U;
    write_u64_be(&payload[20], timestamp_ms);

    status = nx_packet_allocate(&nx_pool[0], &packet, NX_UDP_PACKET, TX_NO_WAIT);
    if (status != NX_SUCCESS)
    {
        printf("ERROR: temperature packet allocate (0x%08x)\r\n", status);
        return status;
    }

    status = nx_packet_data_append(packet, payload, SOMEIP_PACKET_SIZE, &nx_pool[0], TX_NO_WAIT);
    if (status != NX_SUCCESS)
    {
        nx_packet_release(packet);
        printf("ERROR: temperature packet append (0x%08x)\r\n", status);
        return status;
    }

    status = nx_udp_socket_send(
        &temperature_socket,
        packet,
        SOMEIP_BRIDGE_IP,
        SOMEIP_BRIDGE_PORT);
    if (status != NX_SUCCESS)
    {
        nx_packet_release(packet);
        printf("ERROR: temperature UDP send (0x%08x)\r\n", status);
    }
    else
    {
        char message[96];
        char timestamp_text[21];

        format_u64(timestamp_ms, timestamp_text, sizeof(timestamp_text));
        npf_snprintf(message, sizeof(message),
            "SOME/IP sent: Temperature %.2f C, Timestamp %s ms\r\n",
            (double)temperature_celsius,
            timestamp_text);
        printf("%s", message);
    }

    return status;
}

void temperature_publisher_deinit(void)
{
    if (publisher_initialized)
    {
        nx_udp_socket_unbind(&temperature_socket);
        nx_udp_socket_delete(&temperature_socket);
        publisher_initialized = 0U;
    }
}
