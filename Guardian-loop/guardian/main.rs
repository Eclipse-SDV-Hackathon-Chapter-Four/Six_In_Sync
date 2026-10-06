use async_trait::async_trait;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use up_rust::{
    UListener,
    UMessage,
    UTransport,
    UUri,
};

use up_transport_zenoh::{
    zenoh_config,
    UPTransportZenoh,
};

const AUTHORITY: &str = "guardian-demo";

// uProtocol topic resource IDs.
// Topic range starts at 0x8000.
// 0x8001 = 32769
// 0x8002 = 32770
const CHILD_TOPIC: &str =
    "//guardian-demo/1001/1/8001";

const TEMPERATURE_TOPIC: &str =
    "//guardian-demo/1002/1/8002";


#[derive(Debug, Clone, Copy, PartialEq)]
enum GuardianState {
    Clear,
    Monitoring,
    Warning,
    Critical,
}

impl GuardianState {
    fn as_str(&self) -> &'static str {
        match self {
            GuardianState::Clear => "CLEAR",
            GuardianState::Monitoring => "MONITORING",
            GuardianState::Warning => "WARNING",
            GuardianState::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChildPresenceEvent {
    present: bool,
    confidence: f32,
    zone: String,
    timestamp_ms: u64,
}

#[derive(Debug, Deserialize)]
struct CabinTemperatureEvent {
    temperature_celsius: f32,
    timestamp_ms: u64,
    sensor_status: String,
}

struct GuardianData {
    child_present: bool,
    temperature: f32,
    state: GuardianState,
}

impl GuardianData {
    fn new() -> Self {
        Self {
            child_present: false,
            temperature: 0.0,
            state: GuardianState::Clear,
        }
    }

    fn update_state(&mut self) {
        let old_state = self.state;

        self.state = if !self.child_present {
            GuardianState::Clear
        } else if self.temperature >= 40.0 {
            GuardianState::Critical
        } else if self.temperature >= 35.0 {
            GuardianState::Warning
        } else {
            GuardianState::Monitoring
        };

        if old_state != self.state {
            println!();
            println!("==============================================");
            println!(" GUARDIAN STATE CHANGE");
            println!(
                " {} -> {}",
                old_state.as_str(),
                self.state.as_str()
            );
            println!("==============================================");
            println!();
        }

        println!(
            "Child: {} | Temperature: {:.1}°C → {}",
            self.child_present,
            self.temperature,
            self.state.as_str()
        );
    }
}

struct GuardianListener {
    data: Arc<Mutex<GuardianData>>,
    topic: &'static str,
}

#[async_trait]
impl UListener for GuardianListener {
    async fn on_receive(&self, message: UMessage) {
        let payload = match message.payload {
            Some(payload) => payload,
            None => {
                eprintln!(
                    "[GUARDIAN] Received message without payload"
                );
                return;
            }
        };

        // In up-rust 0.9, payload is already Bytes.
        let text = match std::str::from_utf8(&payload) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "[GUARDIAN] Invalid UTF-8 payload: {}",
                    error
                );
                return;
            }
        };

        let mut data = self.data.lock().unwrap();

        // ----------------------------------------
        // CHILD SENSOR MESSAGE
        // ----------------------------------------
        if self.topic == CHILD_TOPIC {
            let event: ChildPresenceEvent =
                match serde_json::from_str(text) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!(
                            "[GUARDIAN] Invalid child event: {}",
                            error
                        );
                        return;
                    }
                };

            println!(
                "[GUARDIAN] Child event: present={} confidence={} zone={} timestamp={}",
                event.present,
                event.confidence,
                event.zone,
                event.timestamp_ms
            );

            data.child_present = event.present;
        }

        // ----------------------------------------
        // TEMPERATURE SENSOR MESSAGE
        // ----------------------------------------
        if self.topic == TEMPERATURE_TOPIC {
            let event: CabinTemperatureEvent =
                match serde_json::from_str(text) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!(
                            "[GUARDIAN] Invalid temperature event: {}",
                            error
                        );
                        return;
                    }
                };

            println!(
                "[GUARDIAN] Temperature event: {:.1}°C status={} timestamp={}",
                event.temperature_celsius,
                event.sensor_status,
                event.timestamp_ms
            );

            data.temperature = event.temperature_celsius;
        }

        // Recalculate Guardian state.
        data.update_state();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================");
    println!(" Guardian Loop");
    println!("=================================");
    println!();

    // ----------------------------------------
    // CREATE uProtocol / Zenoh TRANSPORT
    // ----------------------------------------

    let transport = Arc::new(
        UPTransportZenoh::builder(AUTHORITY)?
            .with_config(zenoh_config::Config::default())
            .build()
            .await?,
    );

    // ----------------------------------------
    // CREATE uProtocol URIs
    // ----------------------------------------

    let child_topic = UUri::try_from(CHILD_TOPIC)?;

    let temperature_topic =
        UUri::try_from(TEMPERATURE_TOPIC)?;

    // ----------------------------------------
    // SHARED GUARDIAN STATE
    // ----------------------------------------

    let data = Arc::new(
        Mutex::new(
            GuardianData::new()
        )
    );

    // ----------------------------------------
    // CHILD SENSOR LISTENER
    // ----------------------------------------

    let child_listener = Arc::new(
        GuardianListener {
            data: data.clone(),
            topic: CHILD_TOPIC,
        }
    );

    // ----------------------------------------
    // TEMPERATURE SENSOR LISTENER
    // ----------------------------------------

    let temperature_listener = Arc::new(
        GuardianListener {
            data: data.clone(),
            topic: TEMPERATURE_TOPIC,
        }
    );

    // ----------------------------------------
    // REGISTER CHILD SENSOR LISTENER
    // ----------------------------------------

    transport
        .register_listener(
            &child_topic,
            None,
            child_listener,
        )
        .await?;

    // ----------------------------------------
    // REGISTER TEMPERATURE LISTENER
    // ----------------------------------------

    transport
        .register_listener(
            &temperature_topic,
            None,
            temperature_listener,
        )
        .await?;

    // ----------------------------------------
    // READY
    // ----------------------------------------

    println!("Guardian subscribed to:");

    println!("  Child:");
    println!("    {}", CHILD_TOPIC);

    println!("  Temperature:");
    println!("    {}", TEMPERATURE_TOPIC);

    println!();
    println!("Waiting for sensor events...");
    println!();

    // ----------------------------------------
    // KEEP GUARDIAN RUNNING
    // ----------------------------------------

    tokio::signal::ctrl_c().await?;

    println!();
    println!("Guardian shutting down.");

    Ok(())
}
