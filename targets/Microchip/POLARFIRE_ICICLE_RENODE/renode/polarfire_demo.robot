*** Settings ***
Suite Setup     Setup
Suite Teardown  Teardown
Test Setup      Reset Emulation
Test Teardown   Test Teardown
Resource        ${RENODEKEYWORDS}

*** Test Cases ***
Should Boot ThreadX And Trigger Telemetry Alarms
    Execute Command          include @${CURDIR}/polarfire_demo.resc
    Create Terminal Tester   sysbus.mmuart1

    Wait For Line On Uart    Microchip PolarFire SoC Icicle Kit (Renode Target)    timeout=10
    Wait For Line On Uart    [SELF-TEST] All startup verification tests PASSED!    timeout=10
    Wait For Line On Uart    [Monitor] Ticks: 0                                    timeout=10
    Wait For Line On Uart    [Monitor] Ticks: 100                                  timeout=10
    Wait For Line On Uart    OVERTEMP ALARM TRIGGERED                              timeout=15

Should Deliver MMUART1 RX Interrupt Through The PLIC
    Execute Command          include @${CURDIR}/polarfire_demo.resc
    Create Terminal Tester   sysbus.mmuart1

    Wait For Line On Uart    [Monitor] Ticks: 0                                    timeout=10
    Write Char On Uart       X
    Wait For Line On Uart    PLIC IRQ 91 handled                                   timeout=10
