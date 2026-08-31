#  Copyright (c) 2026 Eclipse ThreadX contributors
# 
#  This program and the accompanying materials are made available 
#  under the terms of the MIT license which is available at
#  https://opensource.org/licenses/MIT.
# 
#  SPDX-License-Identifier: MIT

# ThreadX configuration
set(TX_USER_FILE
    "${CMAKE_CURRENT_LIST_DIR}/threadx/tx_user.h"
    CACHE STRING
    "Enable TX user configuration"
)

# Core RTOS libraries
add_subdirectory(${SHARED_LIB_DIR}/threadx threadx)

# TODO: Include vendor hardware libraries / HAL here if applicable
# Example: add_subdirectory(vendor_hal)

# Add BSP library
add_subdirectory(bsp)
