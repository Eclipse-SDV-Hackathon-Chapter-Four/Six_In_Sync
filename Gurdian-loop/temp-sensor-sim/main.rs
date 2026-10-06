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

const TEMPERATURE_TOPIC: &str =
    "//guardian-demo/1002/1/8002";

#[derive(Debug, Serialize)]
struct CabinTemperatureEvent {
    temperature_celsius: f32,
    timestamp_ms: u64,
    sensor_status: String,
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
    println!(" Cabin Temperature Simulator");
    println!("=================================");

    let transport = UPTransportZenoh::builder(AUTHORITY)?
        .with_config(zenoh_config::Config::default())
        .build()
        .await?;

    let topic = UUri::try_from(TEMPERATURE_TOPIC)?;

    println!("Publishing to:");
    println!("  {}", TEMPERATURE_TOPIC);
    println!();

    // Deliberately create the Stage-1 scenario.
    let temperatures = [
        26.0,
        26.0,
        36.0,
        43.0,
        43.0,
    ];

    for temperature in temperatures {
        let event = CabinTemperatureEvent {
            temperature_celsius: temperature,
            timestamp_ms: timestamp_ms(),
            sensor_status: "OK".to_string(),
        };

        let json = serde_json::to_string(&event)?;

        let message = UMessageBuilder::publish(topic.clone())
            .build_with_payload(
                json,
                UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
            )?;

        transport.send(message).await?;

        println!(
            "[TEMP SENSOR] temperature={:.1}°C",
            temperature
        );

        sleep(Duration::from_secs(5)).await;
    }

    println!();
    println!("Temperature simulation finished.");

    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
