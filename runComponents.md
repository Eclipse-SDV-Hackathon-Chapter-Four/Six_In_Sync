# Run components

The Guardian loop contains, besides other parts, the following main components:

- Guardian: the main decision loop
- Bridge: receives temperature values via UDP and forwards them on uProtocol
- Child sensor sim: simulates the child sensor
- Temp sensor sim: the old temperature sensor simulation, now replaced by the bridge with the outer ThreadX temperature sensor or the real sensor
- Outer ThreadX sensor: simulates the packages coming from the real sensor using UDP

## Commands to run each component

### Guardian
```bash
cd ~/hackathon/Six_In_Sync/Guardian-loop
cargo run --bin guardian
```

### Bridge
```bash
cd ~/hackathon/Six_In_Sync/Guardian-loop
SOMEIP_LISTEN_PORT=30511 cargo run --bin bridge
```

### Child sensor sim
```bash
cd ~/hackathon/Six_In_Sync/Guardian-loop
cargo run --bin child-sensor-sim
```

### Temp sensor sim
```bash
cd ~/hackathon/Six_In_Sync/Guardian-loop
cargo run --bin temp-sensor-sim
```

### Outer ThreadX sensor
```bash
cd ~/hackathon/Six_In_Sync/threadx-temp-sensor
cmake --preset linux-threadx -DCMAKE_POLICY_VERSION_MINIMUM=3.5
cmake --build --preset linux-threadx --parallel
BRIDGE_IP=127.0.0.1 BRIDGE_PORT=30511 ./build/linux-threadx/threadx_temp_sensor
```
