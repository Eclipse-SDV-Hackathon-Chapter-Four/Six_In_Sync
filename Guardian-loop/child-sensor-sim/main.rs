use async_trait::async_trait;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

use up_rust::{
    UMessageBuilder,
    UPayloadFormat,
    UTransport,
    UUri,
};

use up_transport_zenoh::{
    zenoh_config,
    UPTransportZenoh,
};

const AUTHORITY: &str = "guardian-demo";

const CHILD_TOPIC: &str =
    "//guardian-demo/1001/1/8001";


#[derive(Debug, Serialize)]
struct ChildPresenceEvent {
    present: bool,
    confidence: f32,
    zone: String,
    timestamp_ms: u64,
}

fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================");
    println!(" Child Presence Sensor Simulator");
    println!("=================================");

    let transport = UPTransportZenoh::builder(AUTHORITY)?
        .with_config(zenoh_config::Config::default())
        .build()
        .await?;

    let topic = UUri::try_from(CHILD_TOPIC)?;

    println!("Publishing to:");
    println!("  {}", CHILD_TOPIC);
    println!();

    // Start with no child.
    let scenarios = [
        (false, 0.99),
        (true, 0.98),
        (true, 0.98),
        (true, 0.98),
        (false, 0.99),
    ];

    for (present, confidence) in scenarios {
        let event = ChildPresenceEvent {
            present,
            confidence,
            zone: "rear-seat".to_string(),
            timestamp_ms: timestamp_ms(),
        };

        let json = serde_json::to_string(&event)?;

        let message = UMessageBuilder::publish(topic.clone())
            .build_with_payload(
                json,
                UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
            )?;

        transport.send(message).await?;

        println!(
            "[CHILD SENSOR] child_present={}",
            present
        );

        sleep(Duration::from_secs(5)).await;
    }

    println!();
    println!("Child sensor simulation finished.");

    // Keep process alive so you can observe the system.
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
