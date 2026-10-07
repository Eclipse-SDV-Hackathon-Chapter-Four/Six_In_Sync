/**
 * ThreadX Temperature Sensor — application entry point.
 *
 * Works on both build paths:
 *
 *  Linux / ThreadX-Linux-port (Docker)
 *    int main() → tx_kernel_enter() → tx_application_define()
 *
 *  ARM / Renode (STM32F407)
 *    BSP Reset_Handler → tx_kernel_enter() → tx_application_define()
 *    (main() is not called; THREADX_ARM_TARGET must be defined)
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#include "temperature_sensor.h"
#include <tx_api.h>
#include <cstdio>
#include <cstdlib>

/* ── ThreadX thread objects ─────────────────────────────────────────────── */
static TX_THREAD sensor_thread;
static UCHAR     sensor_stack[4096];

/* Static storage for the sensor so we don't need to cast a 64-bit pointer
 * to the 32-bit ULONG that tx_thread_create accepts on Linux x86-64.       */
static TemperatureSensor* g_sensor = nullptr;

static void sensor_thread_entry(ULONG /*arg*/)
{
    if (g_sensor) g_sensor->run();
}

/* tx_application_define — called by ThreadX after tx_kernel_enter()        */
extern "C" void tx_application_define(void* /*first_unused_memory*/)
{
    static TemperatureSensor sensor;
    g_sensor = &sensor;

    if (!sensor.initialize()) {
        printf("[app] FATAL: temperature sensor init failed\n");
        exit(1);
    }

    UINT status = tx_thread_create(
        &sensor_thread,
        const_cast<char*>("temp-sensor"),
        sensor_thread_entry,
        /*entry_input=*/0,          /* not used; g_sensor carries the pointer */
        sensor_stack,
        sizeof(sensor_stack),
        /*priority=*/16,
        /*preempt_threshold=*/16,
        TX_NO_TIME_SLICE,
        TX_AUTO_START
    );

    if (status != TX_SUCCESS) {
        printf("[app] tx_thread_create failed: %u\n", (unsigned)status);
        exit(1);
    }
}

/* ── Linux-port entry point ─────────────────────────────────────────────── */
#ifndef THREADX_ARM_TARGET

#include <csignal>

static void signal_handler(int /*sig*/)
{
    if (g_sensor) g_sensor->stop();
    tx_thread_terminate(&sensor_thread);
}

int main()
{
    std::signal(SIGINT,  signal_handler);
    std::signal(SIGTERM, signal_handler);

    printf("=== ThreadX Temperature Sensor (SOME/IP publisher) ===\n");
    printf("    Service ID 0x1234 / Event ID 0x8001\n\n");

    /* tx_kernel_enter never returns on ARM; on the Linux port it returns
     * when all threads have terminated.                                    */
    tx_kernel_enter();
    return 0;
}

#endif /* !THREADX_ARM_TARGET */
