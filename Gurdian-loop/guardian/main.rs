use async_trait::async_trait;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use up_rust::{UListener, UMessage, UMessageBuilder, UPayloadFormat, UTransport, UUri};

use up_transport_zenoh::{zenoh_config, UPTransportZenoh};

const AUTHORITY: &str = "guardian-demo";

// ============================================================
// SENSOR TOPICS
// ============================================================
//
// IMPORTANT:
// uProtocol resource IDs are written in hexadecimal in the URI.
//
// 8001 -> child presence topic
// 8002 -> cabin temperature topic
//

const CHILD_TOPIC: &str = "//guardian-demo/1001/1/8001";

const TEMPERATURE_TOPIC: &str = "//guardian-demo/1002/1/8002";

// ============================================================
// ACTUATION RPC
// ============================================================
//
// Guardian sends an RPC request to this method.
//

const SET_WINDOW_POSITION: &str = "//guardian-demo/2001/1/0001";

// Guardian's reply address.
//
// The actuator sends its RPC response back here.
//
const GUARDIAN_REPLY: &str = "//guardian-demo/1000/1/0000";

// ============================================================
// GUARDIAN STATE
// ============================================================

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

// ============================================================
// CHILD SENSOR EVENT
// ============================================================

#[derive(Debug, Deserialize)]
struct ChildPresenceEvent {
    present: bool,
    confidence: f32,
    zone: String,
    timestamp_ms: u64,
}

// ============================================================
// TEMPERATURE SENSOR EVENT
// ============================================================

#[derive(Debug, Deserialize)]
struct CabinTemperatureEvent {
    temperature_celsius: f32,
    timestamp_ms: u64,
    sensor_status: String,
}

// ============================================================
// GUARDIAN INTERNAL STATE
// ============================================================

struct GuardianData {
    child_present: bool,
    temperature: f32,
    state: GuardianState,

    // Prevents sending the same mitigation command
    // repeatedly while the vehicle remains in CRITICAL.
    mitigation_requested: bool,
}

impl GuardianData {
    fn new() -> Self {
        Self {
            child_present: false,
            temperature: 0.0,
            state: GuardianState::Clear,
            mitigation_requested: false,
        }
    }

    // --------------------------------------------------------
    // Calculate Guardian state
    // --------------------------------------------------------
    //
    // CLEAR
    //      no child detected
    //
    // MONITORING
    //      child present + temperature < 35°C
    //
    // WARNING
    //      child present + temperature >= 35°C
    //
    // CRITICAL
    //      child present + temperature >= 40°C
    //
    // Returns true if we have JUST entered CRITICAL and
    // therefore need to request mitigation.
    //

    fn update_state(&mut self) -> bool {
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

        // ----------------------------------------------------
        // Print state transition
        // ----------------------------------------------------

        if old_state != self.state {
            println!();
            println!("==============================================");
            println!(" GUARDIAN STATE CHANGE");
            println!("==============================================");

            println!(" {} -> {}", old_state.as_str(), self.state.as_str());

            println!("==============================================");
            println!();
        }

        // ----------------------------------------------------
        // Print current state
        // ----------------------------------------------------

        println!(
            "Child: {} | Temperature: {:.1}°C → {}",
            self.child_present,
            self.temperature,
            self.state.as_str()
        );

        // ----------------------------------------------------
        // Entering CRITICAL
        // ----------------------------------------------------

        if self.state == GuardianState::Critical
            && old_state != GuardianState::Critical
            && !self.mitigation_requested
        {
            self.mitigation_requested = true;

            return true;
        }

        // ----------------------------------------------------
        // Hazard cleared
        //
        // Allow another mitigation request if the vehicle
        // becomes safe and later becomes critical again.
        // ----------------------------------------------------

        if self.state == GuardianState::Clear {
            self.mitigation_requested = false;
        }

        false
    }
}

// ============================================================
// SENSOR / GUARDIAN LISTENER
// ============================================================

struct GuardianListener {
    data: Arc<Mutex<GuardianData>>,

    // Identifies which sensor this listener handles.
    topic: &'static str,

    // Needed to send the actuation RPC when CRITICAL occurs.
    transport: Arc<UPTransportZenoh>,
}

// ============================================================
// SENSOR MESSAGE HANDLER
// ============================================================

#[async_trait]
impl UListener for GuardianListener {
    async fn on_receive(&self, message: UMessage) {
        println!();
        println!("----------------------------------------------");
        println!(" Guardian received uProtocol message");
        println!("----------------------------------------------");

        // ----------------------------------------------------
        // Get payload
        // ----------------------------------------------------

        let payload = match message.payload {
            Some(payload) => payload,

            None => {
                eprintln!("[GUARDIAN] Received message without payload");

                return;
            }
        };

        // ----------------------------------------------------
        // Convert Bytes -> UTF-8 string
        // ----------------------------------------------------

        let text = match std::str::from_utf8(&payload) {
            Ok(value) => value,

            Err(error) => {
                eprintln!("[GUARDIAN] Invalid UTF-8 payload: {}", error);

                return;
            }
        };

        println!("[GUARDIAN] Raw payload: {}", text);

        // ----------------------------------------------------
        // Lock Guardian state
        // ----------------------------------------------------

        let mut data = self.data.lock().unwrap();

        // ----------------------------------------------------
        // CHILD SENSOR
        // ----------------------------------------------------

        if self.topic == CHILD_TOPIC {
            let event: ChildPresenceEvent = match serde_json::from_str(text) {
                Ok(value) => value,

                Err(error) => {
                    eprintln!("[GUARDIAN] Invalid child event: {}", error);

                    return;
                }
            };

            println!("[GUARDIAN] Child event:");

            println!("  present    = {}", event.present);

            println!("  confidence = {}", event.confidence);

            println!("  zone       = {}", event.zone);

            println!("  timestamp  = {}", event.timestamp_ms);

            data.child_present = event.present;
        }

        // ----------------------------------------------------
        // TEMPERATURE SENSOR
        // ----------------------------------------------------

        if self.topic == TEMPERATURE_TOPIC {
            let event: CabinTemperatureEvent = match serde_json::from_str(text) {
                Ok(value) => value,

                Err(error) => {
                    eprintln!("[GUARDIAN] Invalid temperature event: {}", error);

                    return;
                }
            };

            println!("[GUARDIAN] Temperature event:");

            println!("  temperature = {:.1}°C", event.temperature_celsius);

            println!("  status      = {}", event.sensor_status);

            println!("  timestamp   = {}", event.timestamp_ms);

            data.temperature = event.temperature_celsius;
        }

        // ----------------------------------------------------
        // UPDATE GUARDIAN STATE
        // ----------------------------------------------------

        let should_mitigate = data.update_state();

        // We no longer need the Mutex lock.
        drop(data);

        // ----------------------------------------------------
        // CRITICAL -> REQUEST ACTUATION
        // ----------------------------------------------------

        if should_mitigate {
            println!();
            println!("==============================================");
            println!(" GUARDIAN MITIGATION");
            println!("==============================================");

            println!("[GUARDIAN] CRITICAL condition detected.");

            println!("[GUARDIAN] Requesting window opening...");

            let transport = self.transport.clone();

            tokio::spawn(async move {
                if let Err(error) = request_window_open(transport).await {
                    eprintln!("[GUARDIAN] Mitigation failed: {}", error);
                }
            });
        }
    }
}

// ============================================================
// ACTUATION RESPONSE LISTENER
// ============================================================
//
// Receives the response from actuator-sim after the window
// command has been processed.
//

struct ActuationResponseListener;

#[async_trait]
impl UListener for ActuationResponseListener {
    async fn on_receive(&self, message: UMessage) {
        println!();
        println!("==============================================");
        println!(" ACTUATION RESPONSE");
        println!("==============================================");

        // ----------------------------------------------------
        // Get response payload
        // ----------------------------------------------------

        let payload = match message.payload {
            Some(payload) => payload,

            None => {
                eprintln!("[GUARDIAN] Actuation response has no payload");

                return;
            }
        };

        // ----------------------------------------------------
        // Decode response
        // ----------------------------------------------------

        match std::str::from_utf8(&payload) {
            Ok(text) => {
                println!("[GUARDIAN] Actuator response:");

                println!("  {}", text);
            }

            Err(error) => {
                eprintln!("[GUARDIAN] Invalid actuator response: {}", error);

                return;
            }
        }

        println!();
        println!("✓ Guardian mitigation request completed.");

        println!("==============================================");

        println!();
    }
}

// ============================================================
// SEND WINDOW ACTUATION RPC
// ============================================================
//
// Sends:
//
// {
//     "window": "rear-left",
//     "percentage": 25
// }
//
// to:
//
// //guardian-demo/2001/1/9001
//

async fn request_window_open(
    transport: Arc<UPTransportZenoh>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("[GUARDIAN] RPC method: {}", SET_WINDOW_POSITION);

    // --------------------------------------------------------
    // Convert method URI
    // --------------------------------------------------------

    let method = UUri::try_from(SET_WINDOW_POSITION)?;

    // --------------------------------------------------------
    // Convert reply URI
    // --------------------------------------------------------

    let reply_to = UUri::try_from(GUARDIAN_REPLY)?;

    // --------------------------------------------------------
    // Build command
    // --------------------------------------------------------

    let command = serde_json::json!({
        "window": "rear-left",
        "percentage": 25
    });

    let command_json = command.to_string();

    println!("[GUARDIAN] Command: {}", command_json);

    // --------------------------------------------------------
    // Build uProtocol RPC request
    // --------------------------------------------------------

    let message = UMessageBuilder::request(method, reply_to, 5000)
        .build_with_payload(command_json, UPayloadFormat::UPAYLOAD_FORMAT_TEXT)?;

    // --------------------------------------------------------
    // Send request
    // --------------------------------------------------------

    transport.send(message).await?;

    println!("[GUARDIAN] ✓ Window RPC request sent.");

    println!("==============================================");

    println!();

    Ok(())
}

// ============================================================
// MAIN
// ============================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================");
    println!(" Guardian Loop - Stage 2");
    println!("=================================");
    println!();

    // ========================================================
    // CREATE uProtocol / ZENOH TRANSPORT
    // ========================================================

    let transport = Arc::new(
        UPTransportZenoh::builder(AUTHORITY)?
            .with_config(zenoh_config::Config::default())
            .build()
            .await?,
    );

    println!("[GUARDIAN] uProtocol / Zenoh transport started.");

    println!();

    // ========================================================
    // CREATE SENSOR URIs
    // ========================================================

    let child_topic = UUri::try_from(CHILD_TOPIC)?;

    let temperature_topic = UUri::try_from(TEMPERATURE_TOPIC)?;

    // ========================================================
    // CREATE ACTUATION URIs
    // ========================================================

    let actuator_method = UUri::try_from(SET_WINDOW_POSITION)?;

    let guardian_reply = UUri::try_from(GUARDIAN_REPLY)?;

    // ========================================================
    // SHARED GUARDIAN STATE
    // ========================================================

    let data = Arc::new(Mutex::new(GuardianData::new()));

    // ========================================================
    // CHILD SENSOR LISTENER
    // ========================================================

    let child_listener = Arc::new(GuardianListener {
        data: data.clone(),
        topic: CHILD_TOPIC,
        transport: transport.clone(),
    });

    // ========================================================
    // TEMPERATURE SENSOR LISTENER
    // ========================================================

    let temperature_listener = Arc::new(GuardianListener {
        data: data.clone(),
        topic: TEMPERATURE_TOPIC,
        transport: transport.clone(),
    });

    // ========================================================
    // REGISTER CHILD SENSOR
    // ========================================================

    transport
        .register_listener(&child_topic, None, child_listener)
        .await?;

    println!("[GUARDIAN] ✓ Child sensor subscribed.");

    println!("             {}", CHILD_TOPIC);

    // ========================================================
    // REGISTER TEMPERATURE SENSOR
    // ========================================================

    transport
        .register_listener(&temperature_topic, None, temperature_listener)
        .await?;

    println!("[GUARDIAN] ✓ Temperature sensor subscribed.");

    println!("             {}", TEMPERATURE_TOPIC);

    // ========================================================
    // REGISTER ACTUATION RESPONSE LISTENER
    // ========================================================

    let response_listener = Arc::new(ActuationResponseListener);

    transport
        .register_listener(&actuator_method, Some(&guardian_reply), response_listener)
        .await?;

    println!("[GUARDIAN] ✓ Actuation response listener registered.");

    println!("             Reply URI: {}", GUARDIAN_REPLY);

    // ========================================================
    // READY
    // ========================================================

    println!();
    println!("==============================================");
    println!(" GUARDIAN READY");
    println!("==============================================");

    println!("Child topic:");

    println!("  {}", CHILD_TOPIC);

    println!();

    println!("Temperature topic:");

    println!("  {}", TEMPERATURE_TOPIC);

    println!();

    println!("Actuation RPC:");

    println!("  {}", SET_WINDOW_POSITION);

    println!();

    println!("Waiting for sensor events...");

    println!();

    // ========================================================
    // KEEP GUARDIAN RUNNING
    // ========================================================

    tokio::signal::ctrl_c().await?;

    println!();

    println!("==============================================");

    println!(" Guardian shutting down.");

    println!("==============================================");

    Ok(())
}
