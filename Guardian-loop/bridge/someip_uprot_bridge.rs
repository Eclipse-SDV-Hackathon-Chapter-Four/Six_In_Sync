use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tracing::{debug, error, info, warn};
use up_rust::{UMessageBuilder, UPayloadFormat, UTransport, UUri};
use up_transport_zenoh::{zenoh_config, UPTransportZenoh};

const AUTHORITY: &str = "guardian-demo";
const TEMPERATURE_TOPIC: &str = "//guardian-demo/1002/1/8002";

// SOME/IP temperature event sent by the sensor.
const SOMEIP_HEADER_LEN: usize = 16;
const SOMEIP_MIN_PACKET: usize = SOMEIP_HEADER_LEN + 12;

const TEMP_SERVICE_ID: u16 = 0x1234;
const TEMP_EVENT_ID: u16 = 0x8001;
const SOMEIP_MSG_TYPE_NOTIFICATION: u8 = 0x02;

#[derive(Serialize)]
struct CabinTemperatureEvent {
    temperature_celsius: f32,
    timestamp_ms: u64,
    sensor_status: &'static str,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

/// Returns the temperature and timestamp from a valid SOME/IP notification.
fn parse_someip_temperature(buf: &[u8]) -> Option<(f32, u64)> {
    if buf.len() < SOMEIP_MIN_PACKET {
        debug!(
            "packet too short: {} < {} bytes",
            buf.len(),
            SOMEIP_MIN_PACKET
        );
        return None;
    }

    let service_id = u16::from_be_bytes([buf[0], buf[1]]);
    let event_id = u16::from_be_bytes([buf[2], buf[3]]);
    let message_type = buf[14];

    if service_id != TEMP_SERVICE_ID {
        debug!("unexpected service ID: 0x{:04X}", service_id);
        return None;
    }

    if event_id != TEMP_EVENT_ID {
        debug!("unexpected event ID: 0x{:04X}", event_id);
        return None;
    }

    if message_type != SOMEIP_MSG_TYPE_NOTIFICATION {
        debug!("unexpected message type: 0x{:02X}", message_type);
        return None;
    }

    // Temperature is a big-endian IEEE 754 float at byte offset 16.
    let temperature_bits = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    let temperature = f32::from_bits(temperature_bits);

    if !temperature.is_finite() || !(-40.0..=120.0).contains(&temperature) {
        warn!(
            "received out-of-range temperature: {}°C; discarding",
            temperature
        );
        return None;
    }

    // Timestamp is a big-endian uint64 at byte offset 20.
    let timestamp_ms = u64::from_be_bytes([
        buf[20], buf[21], buf[22], buf[23], buf[24], buf[25], buf[26], buf[27],
    ]);

    Some((temperature, timestamp_ms))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "someip_uprot_bridge=debug,info".to_string()),
        )
        .init();

    let listen_addr = std::env::var("SOMEIP_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0".to_string());
    let listen_port: u16 = std::env::var("SOMEIP_LISTEN_PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or(30511);
    let bind_addr = format!("{listen_addr}:{listen_port}");

    info!("=== SOME/IP to uProtocol bridge ===");
    info!("SOME/IP listener: {bind_addr}");
    info!("uProtocol temperature topic: {TEMPERATURE_TOPIC}");

    let transport = UPTransportZenoh::builder(AUTHORITY)?
        .with_config(zenoh_config::Config::default())
        .build()
        .await?;

    // Publish to the same URI that the separate Guardian subscribes to.
    let temperature_uri = UUri::try_from(TEMPERATURE_TOPIC)?;

    let socket = UdpSocket::bind(&bind_addr).await?;
    info!("Bridge ready; waiting for SOME/IP packets...");

    let mut buffer = vec![0u8; 1500];

    loop {
        let (length, source) = match socket.recv_from(&mut buffer).await {
            Ok(packet) => packet,
            Err(error) => {
                error!("UDP receive error: {error}");
                continue;
            }
        };

        debug!("received {length} bytes from {source}");

        let Some((temperature, sensor_timestamp_ms)) = parse_someip_temperature(&buffer[..length])
        else {
            debug!("ignored invalid or unexpected packet from {source}");
            continue;
        };

        let timestamp_ms = if sensor_timestamp_ms > 0 {
            sensor_timestamp_ms
        } else {
            now_ms()
        };

        let event = CabinTemperatureEvent {
            temperature_celsius: temperature,
            timestamp_ms,
            sensor_status: "OK",
        };

        info!(
            "SOME/IP from {source} -> uProtocol: {:.1}°C, timestamp={}ms",
            temperature, timestamp_ms
        );

        let json = serde_json::to_string(&event)?;
        let message = UMessageBuilder::publish(temperature_uri.clone())
            .build_with_payload(json, UPayloadFormat::UPAYLOAD_FORMAT_TEXT)?;

        if let Err(error) = transport.send(message).await {
            warn!("uProtocol publish failed: {error:?}");
        }
    }
}
