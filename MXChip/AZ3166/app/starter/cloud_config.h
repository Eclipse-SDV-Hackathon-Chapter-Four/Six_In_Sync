/* 
 * Copyright (c) Microsoft
 * Copyright (c) 2024 Eclipse Foundation
 * 
 *  This program and the accompanying materials are made available 
 *  under the terms of the MIT license which is available at
 *  https://opensource.org/license/mit.
 * 
 * SPDX-License-Identifier: MIT
 *
 * AI assistance disclosure:
 * This file was modified with assistance from GitHub Copilot (GPT-5.6 Luna).
 * The changes were reviewed by the author.
 */

#ifndef _CLOUD_CONFIG_H
#define _CLOUD_CONFIG_H

typedef enum
{
    None         = 0,
    WEP          = 1,
    WPA_PSK_TKIP = 2,
    WPA2_PSK_AES = 3
} WiFi_Mode;

// ----------------------------------------------------------------------------
// WiFi connection config
// ----------------------------------------------------------------------------
#define HOSTNAME      "six-in-sync"  //Change to unique hostname.
#define WIFI_SSID     "Hackathon-Team-09"
#define WIFI_PASSWORD "SDVTeam-123456" 
#define WIFI_MODE     WPA2_PSK_AES

#define SOMEIP_BRIDGE_IP       IP_ADDRESS(192, 168, 1, 100)
#define SOMEIP_BRIDGE_PORT     30501
#define SOMEIP_SERVICE_ID      0x1234
#define SOMEIP_EVENT_ID        0x8001


// ----------------------------------------------------------------------------
// MQTT Config
// ----------------------------------------------------------------------------


#endif // _CLOUD_CONFIG_H
