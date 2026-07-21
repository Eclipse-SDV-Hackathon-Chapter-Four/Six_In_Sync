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

#ifndef SENSOR_H
#define SENSOR_H

#include <stdint.h>

/* Sensor reading message structure sent through ThreadX queue */
typedef struct {
    float temperature;
    float humidity;
    uint32_t timestamp;
} sensor_msg_t;

/* Sensor status enum */
typedef enum {
    SENSOR_OK = 0,
    SENSOR_ERROR = 1
} sensor_status_t;

/* Public API */
sensor_status_t sensor_init(void);
sensor_status_t sensor_read(float *temp, float *hum);

/* Mock Sensor API (used internally by sensor_driver when physical sensor is missing) */
sensor_status_t mock_sensor_init(void);
sensor_status_t mock_sensor_read(float *temp, float *hum);

#endif /* SENSOR_H */
