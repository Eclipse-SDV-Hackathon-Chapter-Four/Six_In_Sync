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
#ifndef TEMPERATURE_PUBLISHER_H
#define TEMPERATURE_PUBLISHER_H

#include <stdint.h>

#include "nx_api.h"

UINT temperature_publisher_init(void);
UINT temperature_publisher_send(float temperature_celsius);
void temperature_publisher_deinit(void);

#endif
