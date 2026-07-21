/* 
 * Copyright (c) 2026 Eclipse ThreadX contributors
 * 
 *  This program and the accompanying materials are made available 
 *  under the terms of the MIT license which is available at
 *  https://opensource.org/license/mit.
 * 
 *  SPDX-License-Identifier: MIT
 * 
 *  Contributors:
 *     Ali Eissa - 2026 version.
 */

#include "sensor.h"
#include "stm32f7xx_hal.h"
#include <math.h>

/* Initialize mock sensor */
sensor_status_t mock_sensor_init(void)
{
    /* No hardware initialization needed for mock sensor */
    return SENSOR_OK;
}

/* Read mock sensor data */
sensor_status_t mock_sensor_read(float *temp, float *hum)
{
    /* Generate simulated values using a slow sine wave + small noise */
    uint32_t tick = HAL_GetTick();
    
    /* Simulated temperature: 22.0°C base + 3.0°C variation */
    float sim_temp = 22.0f + 3.0f * sinf((float)tick / 10000.0f);
    
    /* Simulated humidity: 55.0% base + 10.0% variation */
    float sim_hum = 55.0f + 10.0f * cosf((float)tick / 15000.0f);

    /* Add a tiny bit of noise (pseudo-random based on tick) */
    float noise = (float)(tick % 100) / 500.0f - 0.1f; // -0.1 to +0.1
    sim_temp += noise;
    sim_hum += noise * 5.0f; // humidity noise is larger

    /* Keep humidity within physical limits */
    if (sim_hum < 0.0f) sim_hum = 0.0f;
    if (sim_hum > 100.0f) sim_hum = 100.0f;

    *temp = sim_temp;
    *hum = sim_hum;
    return SENSOR_OK;
}
