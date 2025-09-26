pub mod cave_telem {
    tonic::include_proto!("cave_telem"); 
}

use cave_telem::{cave_telemetry_client::CaveTelemetryClient, Telemetry};
use log::debug;

pub async fn send_execution_data(e: ExecutionData,) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Initialisation du client gRPC pour la télémétrie");
    let mut client = CaveTelemetryClient::connect("http://0.0.0.0:50051").await?;
    debug!("Client gRPC connecté à http://0.0.0.0:50051");

    debug!("Construction de la requête Telemetry: user_id={}", e.user_id);

    let request = tonic::Request::new(Telemetry {
        user_id: e.user_id,
        time_execution: e.time_execution as i64,
        valid_result: e.valid_result,
        timezone: e.timezone,
        version: e.version,
        id_docker: e.id_docker,
    });

    debug!("Envoi de la requête telemetry via gRPC...");
    let _ = client.send_telemetry(request).await?;
    debug!("Requête telemetry envoyée avec succès");

    Ok(())
}





#[derive(Debug)]
pub struct ExecutionData {
    pub user_id: String,
    pub time_execution: u128,
    pub valid_result: bool,
    pub timezone: String,
    pub version: String,
    pub id_docker: String,
}

impl Default for ExecutionData {
    fn default() -> Self {
        Self { 
            user_id: String::new(), 
            time_execution: 0,
            valid_result: false,
            timezone: String::new(),
            version: String::new(),
            id_docker: String::new(),
        }
    }
}

