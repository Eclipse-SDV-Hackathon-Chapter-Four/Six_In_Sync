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
// SOVD
// ============================================================
use std::time::Duration;

#[derive(Clone)]
struct SovdClient {
    http: reqwest::Client,
    base_url: String,
}

#[derive(Debug)]
struct SovdActuationResult {
    status: String,
    details: String,
}

#[derive(Debug)]
enum SovdClientError {
    InvalidWindow(String),
    Http(reqwest::Error),
    Rejected {
        status: reqwest::StatusCode,
        body: String,
    },
}

impl std::fmt::Display for SovdClientError {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::InvalidWindow(window) => {
                write!(
                    formatter,
                    "unsupported window '{}'",
                    window
                )
            }

            Self::Http(error) => {
                write!(
                    formatter,
                    "SOVD HTTP request failed: {}",
                    error
                )
            }

            Self::Rejected { status, body } => {
                write!(
                    formatter,
                    "CDA rejected request with {}: {}",
                    status,
                    body
                )
            }
        }
    }
}

impl std::error::Error for SovdClientError {}

impl From<reqwest::Error> for SovdClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl SovdClient {
    fn new(
        base_url: impl Into<String>,
    ) -> Result<Self, reqwest::Error> {
        let http =
            reqwest::Client::builder()
                .connect_timeout(
                    Duration::from_secs(2)
                )
                .timeout(
                    Duration::from_secs(5)
                )
                .build()?;

        Ok(Self {
            http,
            base_url:
                base_url
                    .into()
                    .trim_end_matches('/')
                    .to_string(),
        })
    }

    async fn set_window_position(
        &self,
        window: &str,
        percentage: u8,
    ) -> Result<
        SovdActuationResult,
        SovdClientError,
    > {
        let component_id =
            match window {
                "rear-left" => {
                    "rear-left-door"
                }

                "rear-right" => {
                    "rear-right-door"
                }

                "front-left" => {
                    "front-left-door"
                }

                "front-right" => {
                    "front-right-door"
                }

                other => {
                    return Err(
                        SovdClientError::InvalidWindow(
                            other.to_string()
                        )
                    );
                }
            };

        /*
         * PLACEHOLDER:
         *
         * Replace "window-position" and possibly
         * the complete path after inspecting the
         * CDA Swagger API and your MDD.
         */
        let url = format!(
            "{}/components/{}/data/window-position",
            self.base_url,
            component_id,
        );

        let body =
            serde_json::json!({
                "value": percentage
            });

        println!(
            "[ACTUATOR] SOVD request:"
        );

        println!(
            "  PUT {}",
            url
        );

        println!(
            "  Body: {}",
            body
        );

        let response =
            self.http
                .put(&url)
                .json(&body)
                .send()
                .await?;

        let status =
            response.status();

        let response_body =
            response
                .text()
                .await?;

        if !status.is_success() {
            return Err(
                SovdClientError::Rejected {
                    status,
                    body: response_body,
                }
            );
        }

        Ok(
            SovdActuationResult {
                status:
                    "COMPLETED".to_string(),

                details:
                    if response_body.is_empty() {
                        "CDA accepted window position"
                            .to_string()
                    } else {
                        response_body
                    },
            }
        )
    }
}

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
    sovd_client: Arc<SovdClient>,
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

async fn send_rpc_response(
    transport: &UPTransportZenoh,
    request: &UMessage,
    response: serde_json::Value,
) {
    let response_json = response.to_string();

    let response_message =
        match UMessageBuilder::response_for_request(
            &request.attributes,
        )
        .build_with_payload(
            response_json,
            UPayloadFormat::UPAYLOAD_FORMAT_JSON,
        ) {
            Ok(message) => message,

            Err(error) => {
                eprintln!(
                    "[ACTUATOR] Failed to build RPC response: {}",
                    error
                );

                return;
            }
        };

    match transport.send(response_message).await {
        Ok(_) => {
            println!(
                "[ACTUATOR] RPC response sent."
            );
        }

        Err(error) => {
            eprintln!(
                "[ACTUATOR] Failed to send RPC response: {}",
                error
            );
        }
    }
}

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

        let payload =
            match message.payload.as_ref() {
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

        if command.percentage > 100 {
            let response =
                serde_json::json!({
                    "success": false,
                    "window": command.window,
                    "percentage": command.percentage,
                    "status": "INVALID_ARGUMENT",
                    "details":
                        "percentage must be between 0 and 100"
                });

            send_rpc_response(
                &self.transport,
                &message,
                response,
            )
            .await;

            return;
        }

        let percentage =
            command.percentage;

        // ====================================================
        // SIMULATED PHYSICAL ACTUATION
        // ====================================================

        println!();

        println!(
            "🚗 SEND REQUEST TO SOVD"
        );

        println!(
            "   Window: {}",
            command.window
        );

        println!(
            "   Target position: {}%",
            percentage
        );

        println!();
        println!(
            "=============================================="
        );
        println!(
            " SOVD WINDOW ACTUATION"
        );
        println!(
            "=============================================="
        );

        let result =
            self.sovd_client
                .set_window_position(
                    &command.window,
                    percentage,
                )
                .await;

        let response =
            match result {
                Ok(result) => {
                    println!(
                        "[ACTUATOR] SOVD actuation completed."
                    );

                    println!(
                        "[ACTUATOR] Details: {}",
                        result.details
                    );

                    serde_json::json!({
                        "success": true,
                        "window": command.window,
                        "percentage": percentage,
                        "status": result.status,
                        "details": result.details
                    })
                }

                Err(error) => {
                    eprintln!(
                        "[ACTUATOR] SOVD actuation failed: {}",
                        error
                    );

                    serde_json::json!({
                        "success": false,
                        "window": command.window,
                        "percentage": percentage,
                        "status": "FAILED",
                        "details": error.to_string()
                    })
                }
            };
        //

        // ====================================================
        // BUILD RESPONSE
        // ====================================================

        // let response =
        //     serde_json::json!({
        //         "success": true,
        //         "window": command.window,
        //         "percentage": percentage,
        //         "status": "COMPLETED"
        //     });


        // let response_json =
        //     response.to_string();


        // println!(
        //     "[ACTUATOR] Sending RPC response..."
        // );


        // ====================================================
        // CREATE RESPONSE MESSAGE
        // ====================================================
        //
        // response_for_request() uses the original request's
        // attributes to construct the correct response.
        //

        // let response_message =
        //     match UMessageBuilder::response_for_request(
        //         &message.attributes,
            // )
            // .build_with_payload(
            //     response_json,
            //     UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
            // ) {

            //     Ok(message) => message,

            //     Err(error) => {
            //         eprintln!(
            //             "[ACTUATOR] Failed to build response: {}",
            //             error
            //         );

            //         return;
            //     }
            // };


        // ====================================================
        // SEND RESPONSE
        // ====================================================

        // match self.transport.send(response_message).await {

        //     Ok(_) => {
        //         println!(
        //             "[ACTUATOR] ✓ RPC response sent."
        //         );
        //     }

        //     Err(error) => {
        //         eprintln!(
        //             "[ACTUATOR] Failed to send RPC response: {}",
        //             error
        //         );
        //     }
        // }

        println!(
            "[ACTUATOR] Sending RPC response..."
        );

        send_rpc_response(
            &self.transport,
            &message,
            response,
        )
        .await;

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
    // CREATE SOVD CLIENT
    // ========================================================
    let cda_base_url =
    std::env::var("CDA_BASE_URL")
        .unwrap_or_else(|_| {
            "http://127.0.0.1:20002/vehicle/v15"
                .to_string()
        });

    println!(
        "[ACTUATOR] CDA base URL: {}",
        cda_base_url
    );

    let sovd_client =
        Arc::new(
            SovdClient::new(
                cda_base_url
            )?
        );

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
            sovd_client:
                sovd_client.clone(),
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