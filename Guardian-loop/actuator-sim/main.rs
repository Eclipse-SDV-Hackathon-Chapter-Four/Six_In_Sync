// use async_trait::async_trait;
// use std::sync::Arc;

// use up_rust::{UListener, UMessage, UMessageBuilder, UPayloadFormat, UTransport, UUri};

// use up_transport_zenoh::{zenoh_config, UPTransportZenoh};

// const AUTHORITY: &str = "guardian-demo";

// ============================================================
// SOVD
// ============================================================
// use std::time::Duration;

use async_trait::async_trait;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

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

use common::{
    fault::{LifecyclePhase, LifecycleStage},
    types::MetadataVec,
};

use fault_lib::{
    FaultApi,
    catalog::FaultCatalogBuilder,
    reporter::{
        Reporter,
        ReporterApi,
        ReporterConfig,
    },
    utils::to_static_short_string,
};

const AUTHORITY: &str = "guardian-demo";
// Temporary name for smoke test
//const WINDOW_ACTUATION_FAILED: &str = "CabinTempSensorStuck";

#[derive(serde::Serialize)]
struct AuthorizationRequest {
    client_id: String,
    client_secret: String,
}

#[derive(serde::Deserialize)]
struct AuthorizationResponse {
    access_token: String,
}

#[derive(serde::Serialize)]
struct LockRequest {
    lock_expiration: u64,
}

#[derive(Debug, serde::Deserialize)]
struct LockResponse {
    id: String,
}

#[derive(Clone)]
struct SovdClient {
    http: reqwest::Client,
    base_url: String,
    component_id: String,
    service_id: String,
    bearer_token: String,
    lock_id: Arc<Mutex<Option<String>>>,
}

#[derive(Debug)]
struct SovdActuationResult {
    status: String,
    details: String,
}

// use std::{
//     path::PathBuf,
//     sync::{Arc, Mutex},
//     time::Duration,
// };

// use common::{fault::*, types::MetadataVec};
// use fault_lib::{
//     FaultApi,
//     catalog::FaultCatalogBuilder,
//     reporter::{Reporter, ReporterApi, ReporterConfig},
//     utils::to_static_short_string,
// };

#[derive(Debug)]
enum SovdClientError {
    InvalidWindow(String),
    InvalidAuthorizationResponse(String),

    Http(reqwest::Error),

    Rejected {
        status: reqwest::StatusCode,
        body: String,
    },
}


struct ActuationFaultReporter {
    reporter: Mutex<Reporter>,
    sovd_path: String,
}

impl ActuationFaultReporter {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = ReporterConfig {
            source: common::ids::SourceId {
                entity: to_static_short_string(
                    "actuation-adapter",
                )?,
                ecu: Some(
                    common::types::ShortString::from_bytes(
                        b"ECU-A",
                    )?,
                ),
                domain: Some(
                    to_static_short_string("Body")?,
                ),
                sw_component: Some(
                    to_static_short_string(
                        "WindowActuationAdapter",
                    )?,
                ),
                instance: Some(
                    to_static_short_string("0")?,
                ),
            },
            lifecycle_phase: LifecyclePhase::Running,
            default_env_data: MetadataVec::new(),
        };

        let catalog = FaultApi::get_fault_catalog();

        let fault_id = catalog
            .descriptors()
            .next()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "catalog '{}' contains no faults",
                        catalog.id
                    ),
                )
            })?
            .id
            .clone();

        println!(
            "[ACTUATOR] Using fault ID: {:?}",
            fault_id
        );

        let sovd_path = catalog.id.to_string();

        let reporter = Reporter::new(&fault_id, config)?;

        Ok(Self {
            reporter: Mutex::new(reporter),
            sovd_path,
        })
    }

    fn publish_stage(
        &self,
        stage: LifecycleStage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reporter = self
            .reporter
            .lock()
            .map_err(|_| {
                std::io::Error::other(
                    "actuation fault reporter mutex was poisoned",
                )
            })?;

        let record = reporter.create_record(stage);

        reporter.publish(&self.sovd_path, record)?;

        Ok(())
    }

    fn passed(&self) {
        match self.publish_stage(LifecycleStage::Passed) {
            Ok(()) => {
                println!(
                    "[ACTUATOR] Published fault -> Passed"
                );
            }
            Err(error) => {
                eprintln!(
                    "[ACTUATOR] Could not publish Passed: {}",
                    error
                );
            }
        }
    }

    fn failed(&self) {
        match self.publish_stage(LifecycleStage::Failed) {
            Ok(()) => {
                println!(
                    "[ACTUATOR] Published fault -> Failed"
                );
            }
            Err(error) => {
                eprintln!(
                    "[ACTUATOR] Could not publish Failed: {}",
                    error
                );
            }
        }
    }
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

            Self::InvalidAuthorizationResponse(error) => {
                write!(
                    formatter,
                    "invalid CDA authorization response: {}",
                    error
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
    async fn new(
        base_url: impl Into<String>,
        component_id: impl Into<String>,
        service_id: impl Into<String>,
        client_id: &str,
        client_secret: &str,
    ) -> Result<Self, SovdClientError> {
        let base_url =
            base_url
                .into()
                .trim_end_matches('/')
                .to_string();

        let http =
            reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(2))
                .timeout(Duration::from_secs(5))
                .build()?;

        let authorization_url =
            format!("{}/authorize", base_url);

        let authorization_body =
            AuthorizationRequest {
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
            };

        println!(
            "[ACTUATOR] Requesting CDA access token from {}",
            authorization_url
        );

        let response =
            http
                .post(&authorization_url)
                .json(&authorization_body)
                .send()
                .await?;

        let status = response.status();

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

        let authorization_response:
            AuthorizationResponse =
                serde_json::from_str(&response_body)
                    .map_err(|error| {
                        SovdClientError::InvalidAuthorizationResponse(
                            error.to_string()
                        )
                    })?;

        println!(
            "[ACTUATOR] Token: {}",
            authorization_response.access_token
        );

        if authorization_response.access_token.is_empty() {
            return Err(
                SovdClientError::InvalidAuthorizationResponse(
                    "access_token was empty".to_string()
                )
            );
        }

        println!(
            "[ACTUATOR] CDA access token obtained."
        );

        Ok(Self {
            http,
            base_url,
            component_id: component_id.into(),
            service_id: service_id.into(),
            bearer_token: authorization_response.access_token,
            lock_id: Arc::new(Mutex::new(None)),
        })
    }

    //
    async fn acquire_lock(
        &self,
    ) -> Result<String, SovdClientError> {

        {
            let guard =
                self.lock_id.lock().unwrap();

            if let Some(lock_id) = guard.clone() {
                return Ok(lock_id);
            }
        }

        let url = format!(
            "{}/components/{}/locks",
            self.base_url,
            self.component_id,
        );

        println!(
            "[ACTUATOR] Acquiring SOVD lock..."
        );

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.bearer_token)
            .json(&LockRequest {
                lock_expiration: 100000,
            })
            .send()
            .await?;

        let status = response.status();

        let response_body =
            response.text().await?;

        println!(
            "[ACTUATOR] Lock response body: {}",
            response_body
        );

        if !status.is_success() {
            return Err(
                SovdClientError::Rejected {
                    status,
                    body: response_body,
                }
            );
        }

        let lock_response: LockResponse =
            serde_json::from_str(&response_body)
                .map_err(|e| {
                    SovdClientError::InvalidAuthorizationResponse(
                        e.to_string()
                    )
                })?;

        println!(
            "[ACTUATOR] Lock acquired: {}",
            lock_response.id
        );

        {
            let mut guard =
                self.lock_id.lock().unwrap();

            *guard =
                Some(lock_response.id.clone());
        }

        Ok(lock_response.id)
    }

    //
    async fn set_window_position(
        &self,
        window: &str,
        percentage: u8,
    ) -> Result<SovdActuationResult, SovdClientError> {
        match window {
            "rear-left" | "rear-right" | "front-left" | "front-right" => {}

            other => {
                return Err(SovdClientError::InvalidWindow(other.to_string()));
            }
        }

        let lock_id =
            self.acquire_lock().await?;

        let url = format!(
            "{}/components/{}/data/{}",
            self.base_url, self.component_id, self.service_id,
        );

        /*
         * This body is still dependent on the schema
         * of the selected MDD service.
         *
         * Verify it using:
         *
         * GET /vehicle/v15/components/az3166/data/{service}/docs
         */
        let body = serde_json::json!({
            "data": {
                "value": percentage
            }
        });

        println!("[ACTUATOR] SOVD request:");
        println!("  PUT {}", url);
        println!("  Body: {}", body);

        let response = self
            .http
            .put(&url)
            .bearer_auth(&self.bearer_token)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        if !status.is_success() {
            return Err(SovdClientError::Rejected {
                status,
                body: response_body,
            });
        }

        Ok(SovdActuationResult {
            status: "COMPLETED".to_string(),

            details: if response_body.is_empty() {
                format!(
                    "CDA accepted service '{}' for component '{}'",
                    self.service_id, self.component_id,
                )
            } else {
                response_body
            },
        })
    }
}

// ============================================================
// RPC METHOD
// ============================================================
//
// Guardian sends the request TO this URI.
//

const SET_WINDOW_POSITION: &str = "//guardian-demo/2001/1/0001";

// ============================================================
// GUARDIAN REPLY URI
// ============================================================
//
// Guardian uses this URI as the source/reply-to address
// when creating the RPC request.
//
// Therefore the actuator can use this as its source filter.
//

const GUARDIAN_REPLY: &str = "//guardian-demo/1000/1/0000";

// ============================================================
// ACTUATOR LISTENER
// ============================================================

struct ActuatorListener {
    transport: Arc<UPTransportZenoh>,
    sovd_client: Arc<SovdClient>,
    actuation_fault: Arc<ActuationFaultReporter>,
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

    let response_message = match UMessageBuilder::response_for_request(&request.attributes)
        .build_with_payload(response_json, UPayloadFormat::UPAYLOAD_FORMAT_JSON)
    {
        Ok(message) => message,

        Err(error) => {
            eprintln!("[ACTUATOR] Failed to build RPC response: {}", error);

            return;
        }
    };

    match transport.send(response_message).await {
        Ok(_) => {
            println!("[ACTUATOR] RPC response sent.");
        }

        Err(error) => {
            eprintln!("[ACTUATOR] Failed to send RPC response: {}", error);
        }
    }
}

#[async_trait]
impl UListener for ActuatorListener {
    async fn on_receive(&self, message: UMessage) {
        println!();
        println!("==============================================");
        println!(" ACTUATOR ADAPTER");
        println!("==============================================");

        println!("[ACTUATOR] RPC request received");

        // ====================================================
        // READ PAYLOAD
        // ====================================================

        let payload = match message.payload.as_ref() {
            Some(payload) => payload,

            None => {
                eprintln!("[ACTUATOR] Request has no payload");

                return;
            }
        };

        // ====================================================
        // CONVERT PAYLOAD TO UTF-8
        // ====================================================

        let text = match std::str::from_utf8(&payload) {
            Ok(value) => value,

            Err(error) => {
                eprintln!("[ACTUATOR] Invalid UTF-8 payload: {}", error);

                return;
            }
        };

        println!("[ACTUATOR] Payload: {}", text);

        // ====================================================
        // PARSE WINDOW COMMAND
        // ====================================================

        let command: WindowCommand = match serde_json::from_str(text) {
            Ok(value) => value,

            Err(error) => {
                eprintln!("[ACTUATOR] Invalid command: {}", error);

                return;
            }
        };

        println!("[ACTUATOR] Window: {}", command.window);

        println!("[ACTUATOR] Requested position: {}%", command.percentage);

        // ====================================================
        // VALIDATE WINDOW POSITION
        // ====================================================

        if command.percentage > 100 {
            let response = serde_json::json!({
                "success": false,
                "window": command.window,
                "percentage": command.percentage,
                "status": "INVALID_ARGUMENT",
                "details":
                    "percentage must be between 0 and 100"
            });

            send_rpc_response(&self.transport, &message, response).await;

            return;
        }

        let percentage = command.percentage;

        // ====================================================
        // SIMULATED PHYSICAL ACTUATION
        // ====================================================

        println!();

        println!("🚗 SEND REQUEST TO SOVD");

        println!("   Window: {}", command.window);

        println!("   Target position: {}%", percentage);

        println!();
        println!("==============================================");
        println!(" SOVD WINDOW ACTUATION");
        println!("==============================================");

        let result = self
            .sovd_client
            .set_window_position(&command.window, percentage)
            .await;

        let response = match result {
            Ok(result) => {
                println!("[ACTUATOR] SOVD actuation completed.");
                println!("[ACTUATOR] Details: {}", result.details);

                // The monitored operation succeeded, so report Passed.
                self.actuation_fault.passed();

                serde_json::json!({
                    "success": true,
                    "window": command.window,
                    "percentage": percentage,
                    "status": result.status,
                    "details": result.details
                })
            }

            Err(error) => {
                eprintln!("[ACTUATOR] SOVD actuation failed: {}", error);

                // The monitored operation failed, so report Failed.
                self.actuation_fault.failed();

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

        println!("[ACTUATOR] Sending RPC response...");

        send_rpc_response(&self.transport, &message, response).await;

        println!();
    }
}

// ============================================================
// MAIN
// ============================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================");
    println!(" Simulated Actuation Adapter");
    println!("=================================");
    println!();

    // ========================================================
    // DISPLAY CONFIGURATION
    // ========================================================

    println!("RPC method:");

    println!("  {}", SET_WINDOW_POSITION);

    println!();

    println!("Source filter:");

    println!("  {}", GUARDIAN_REPLY);

    println!();

    // ========================================================
    // CREATE ZENOH / uPROTOCOL TRANSPORT
    // ========================================================

    let transport = Arc::new(
        UPTransportZenoh::builder(AUTHORITY)?
            .with_config(zenoh_config::Config::default())
            .build()
            .await?,
    );

    println!("[ACTUATOR] uProtocol / Zenoh transport started.");

    println!();

    // ========================================================
    // CREATE SOVD CLIENT
    // ========================================================

    let fault_catalog_file = std::env::var("FAULT_CATALOG_FILE")
    .unwrap_or_else(|_| {
        "./diagnostics/catalog/window_actuation_fault_catalog.json".to_string()
    });

    println!(
        "[ACTUATOR] Fault catalog: {}",
        fault_catalog_file
    );

    let fault_catalog = FaultCatalogBuilder::new()
        .json_file(PathBuf::from(&fault_catalog_file))?
        .build();

    // This value must remain alive until main exits.
    let _fault_api = FaultApi::try_new(fault_catalog)?;

    let actuation_fault = Arc::new(
        ActuationFaultReporter::new()?
    );

    println!(
        "[ACTUATOR] Fault reporter initialized"
    );

    let cda_base_url =
        std::env::var("CDA_BASE_URL")
            .unwrap_or_else(|_| {
                "http://127.0.0.1:20002/vehicle/v15".to_string()
            });

    let cda_component_id =
        std::env::var("CDA_COMPONENT_ID")
            .unwrap_or_else(|_| {
                "az3166".to_string()
            });

    let cda_service_id =
        std::env::var("CDA_SERVICE_ID")
            .unwrap_or_else(|_| {
                "window-position".to_string()
            });

    let cda_client_id =
        std::env::var("CDA_CLIENT_ID")
            .unwrap_or_else(|_| {
                "test".to_string()
            });

    let cda_client_secret =
        std::env::var("CDA_CLIENT_SECRET")
            .unwrap_or_else(|_| {
                "test".to_string()
            });

    println!(
        "[ACTUATOR] CDA base URL: {}",
        cda_base_url
    );

    println!(
        "[ACTUATOR] CDA component: {}",
        cda_component_id
    );

    println!(
        "[ACTUATOR] CDA service: {}",
        cda_service_id
    );

    println!(
        "[ACTUATOR] CDA client ID: {}",
        cda_client_id
    );

    println!(
        "[ACTUATOR] CDA authentication: client credentials configured"
    );

    let sovd_client =
        Arc::new(
            SovdClient::new(
                cda_base_url,
                cda_component_id,
                cda_service_id,
                &cda_client_id,
                &cda_client_secret,
            )
            .await?
        );

    // ========================================================
    // CREATE URI FILTERS
    // ========================================================

    // Source:
    //
    // Guardian's reply URI.
    //
    let source_filter = UUri::try_from(GUARDIAN_REPLY)?;

    // Sink:
    //
    // The RPC method that this actuator implements.
    //
    let sink_filter = UUri::try_from(SET_WINDOW_POSITION)?;

    // ========================================================
    // CREATE LISTENER
    // ========================================================

    let listener = Arc::new(ActuatorListener {
    transport: transport.clone(),
    sovd_client: sovd_client.clone(),
    actuation_fault: actuation_fault.clone(),
    });

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
        .register_listener(&source_filter, Some(&sink_filter), listener)
        .await?;

    println!("[ACTUATOR] ✓ RPC listener registered.");

    println!();

    println!("  Source:");

    println!("    {}", GUARDIAN_REPLY);

    println!();

    println!("  Sink:");

    println!("    {}", SET_WINDOW_POSITION);

    println!();

    // ========================================================
    // READY
    // ========================================================

    println!("==============================================");

    println!(" ACTUATOR READY");

    println!("==============================================");

    println!("Waiting for window RPC requests...");

    println!();

    // ========================================================
    // KEEP PROCESS ALIVE
    // ========================================================

    tokio::signal::ctrl_c().await?;

    println!();

    println!("==============================================");

    println!(" Actuation Adapter shutting down.");

    println!("==============================================");

    Ok(())
}
