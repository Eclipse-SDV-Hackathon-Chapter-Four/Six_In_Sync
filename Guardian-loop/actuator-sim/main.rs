// Co-authored with AI assistance. Reviewed and approved by the project authors.

use async_trait::async_trait;
use std::sync::Arc;

use up_rust::{
    UListener,
    UMessage,
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

// ============================================================
// RPC METHOD
// ============================================================
//
// Guardian sends the request TO this URI.
//

const SET_WINDOW_POSITION: &str =
    "//guardian-demo/2001/1/0001";


// ============================================================
// GUARDIAN REPLY URI
// ============================================================
//
// Guardian uses this URI as the source/reply-to address
// when creating the RPC request.
//
// Therefore the actuator can use this as its source filter.
//

const GUARDIAN_REPLY: &str =
    "//guardian-demo/1000/1/0000";


// ============================================================
// ACTUATOR LISTENER
// ============================================================

struct ActuatorListener {
    transport: Arc<UPTransportZenoh>,
}


// ============================================================
// WINDOW COMMAND
// ============================================================

#[derive(Debug, serde::Deserialize)]
struct WindowCommand {
    window: String,
    percentage: u8,
}


// ============================================================
// RPC REQUEST HANDLER
// ============================================================

#[async_trait]
impl UListener for ActuatorListener {
    async fn on_receive(
        &self,
        message: UMessage,
    ) {
        println!();
        println!("==============================================");
        println!(" ACTUATOR ADAPTER");
        println!("==============================================");

        println!(
            "[ACTUATOR] RPC request received"
        );

        // ====================================================
        // READ PAYLOAD
        // ====================================================

        let payload = match message.payload {
            Some(payload) => payload,

            None => {
                eprintln!(
                    "[ACTUATOR] Request has no payload"
                );

                return;
            }
        };


        // ====================================================
        // CONVERT PAYLOAD TO UTF-8
        // ====================================================

        let text = match std::str::from_utf8(&payload) {
            Ok(value) => value,

            Err(error) => {
                eprintln!(
                    "[ACTUATOR] Invalid UTF-8 payload: {}",
                    error
                );

                return;
            }
        };

        println!(
            "[ACTUATOR] Payload: {}",
            text
        );


        // ====================================================
        // PARSE WINDOW COMMAND
        // ====================================================

        let command: WindowCommand =
            match serde_json::from_str(text) {

                Ok(value) => value,

                Err(error) => {
                    eprintln!(
                        "[ACTUATOR] Invalid command: {}",
                        error
                    );

                    return;
                }
            };


        println!(
            "[ACTUATOR] Window: {}",
            command.window
        );

        println!(
            "[ACTUATOR] Requested position: {}%",
            command.percentage
        );


        // ====================================================
        // VALIDATE WINDOW POSITION
        // ====================================================

        let percentage =
            command.percentage.min(100);


        // ====================================================
        // SIMULATED PHYSICAL ACTUATION
        // ====================================================

        println!();

        println!(
            "🚗 SIMULATED WINDOW ACTUATOR"
        );

        println!(
            "   Window: {}",
            command.window
        );

        println!(
            "   Target position: {}%",
            percentage
        );

        println!(
            "   Motor: RUNNING..."
        );


        // Simulate physical movement.

        tokio::time::sleep(
            std::time::Duration::from_millis(500)
        )
        .await;


        println!(
            "   Motor: STOPPED"
        );

        println!(
            "   Window position: {}%",
            percentage
        );

        println!();

        println!(
            "✓ ACTUATION COMPLETE"
        );

        println!(
            "=============================================="
        );

        println!();


        // ====================================================
        // BUILD RESPONSE
        // ====================================================

        let response =
            serde_json::json!({
                "success": true,
                "window": command.window,
                "percentage": percentage,
                "status": "COMPLETED"
            });


        let response_json =
            response.to_string();


        println!(
            "[ACTUATOR] Sending RPC response..."
        );


        // ====================================================
        // CREATE RESPONSE MESSAGE
        // ====================================================
        //
        // response_for_request() uses the original request's
        // attributes to construct the correct response.
        //

        let response_message =
            match UMessageBuilder::response_for_request(
                &message.attributes,
            )
            .build_with_payload(
                response_json,
                UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
            ) {

                Ok(message) => message,

                Err(error) => {
                    eprintln!(
                        "[ACTUATOR] Failed to build response: {}",
                        error
                    );

                    return;
                }
            };


        // ====================================================
        // SEND RESPONSE
        // ====================================================

        match self.transport.send(response_message).await {

            Ok(_) => {
                println!(
                    "[ACTUATOR] ✓ RPC response sent."
                );
            }

            Err(error) => {
                eprintln!(
                    "[ACTUATOR] Failed to send RPC response: {}",
                    error
                );
            }
        }

        println!();
    }
}


// ============================================================
// MAIN
// ============================================================

#[tokio::main]
async fn main()
    -> Result<(), Box<dyn std::error::Error>>
{
    println!("=================================");
    println!(" Simulated Actuation Adapter");
    println!("=================================");
    println!();


    // ========================================================
    // DISPLAY CONFIGURATION
    // ========================================================

    println!(
        "RPC method:"
    );

    println!(
        "  {}",
        SET_WINDOW_POSITION
    );

    println!();

    println!(
        "Source filter:"
    );

    println!(
        "  {}",
        GUARDIAN_REPLY
    );

    println!();


    // ========================================================
    // CREATE ZENOH / uPROTOCOL TRANSPORT
    // ========================================================

    let transport = Arc::new(
        UPTransportZenoh::builder(
            AUTHORITY
        )?
        .with_config(
            zenoh_config::Config::default()
        )
        .build()
        .await?,
    );


    println!(
        "[ACTUATOR] uProtocol / Zenoh transport started."
    );

    println!();


    // ========================================================
    // CREATE URI FILTERS
    // ========================================================

    // Source:
    //
    // Guardian's reply URI.
    //
    let source_filter =
        UUri::try_from(
            GUARDIAN_REPLY
        )?;


    // Sink:
    //
    // The RPC method that this actuator implements.
    //
    let sink_filter =
        UUri::try_from(
            SET_WINDOW_POSITION
        )?;


    // ========================================================
    // CREATE LISTENER
    // ========================================================

    let listener =
        Arc::new(
            ActuatorListener {
                transport: transport.clone(),
            }
        );


    // ========================================================
    // REGISTER RPC LISTENER
    // ========================================================
    //
    // IMPORTANT:
    //
    // register_listener(
    //     source_filter,
    //     sink_filter,
    //     listener
    // )
    //
    // Therefore:
    //
    // source = Guardian reply URI
    // sink   = SetWindowPosition RPC method
    //

    transport
        .register_listener(
            &source_filter,
            Some(&sink_filter),
            listener,
        )
        .await?;


    println!(
        "[ACTUATOR] ✓ RPC listener registered."
    );

    println!();

    println!(
        "  Source:"
    );

    println!(
        "    {}",
        GUARDIAN_REPLY
    );

    println!();

    println!(
        "  Sink:"
    );

    println!(
        "    {}",
        SET_WINDOW_POSITION
    );

    println!();


    // ========================================================
    // READY
    // ========================================================

    println!(
        "=============================================="
    );

    println!(
        " ACTUATOR READY"
    );

    println!(
        "=============================================="
    );

    println!(
        "Waiting for window RPC requests..."
    );

    println!();


    // ========================================================
    // KEEP PROCESS ALIVE
    // ========================================================

    tokio::signal::ctrl_c()
        .await?;


    println!();

    println!(
        "=============================================="
    );

    println!(
        " Actuation Adapter shutting down."
    );

    println!(
        "=============================================="
    );

    Ok(())
}