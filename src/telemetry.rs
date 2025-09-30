pub mod cave_telem {
    tonic::include_proto!("cave_telem"); 
}

use tokio::time::{timeout, Duration};
use cave_telem::{cave_telemetry_client::CaveTelemetryClient, Telemetry};
use log::debug;
use tonic::transport::{Channel, ClientTlsConfig};

pub async fn send_execution_data(e: ExecutionData, local: bool) -> Result<(), Box<dyn std::error::Error>> {
    debug!("=== DÉBUT DE LA TÉLÉMÉTRIE ===");
    debug!("Initialisation du client gRPC pour la télémétrie");
    debug!("Données à envoyer: {:?}", e);
    let mut client;

    if local {
        debug!("=== CONNEXION EN LOCAL ===");
        debug!("Initialisation du client gRPC");
        client = CaveTelemetryClient::connect("http://127.0.0.1:50051").await?;
        debug!("Client gRPC connecté à http://127.0.0.1:50051");
    }
    else {
        debug!("=== CONNEXION A DISTANCE ===");
        debug!("Configuration TLS pour le domaine: code-insight.simvia-app.fr");
        let tls = ClientTlsConfig::new()
            .domain_name("code-insight.simvia-app.fr");

        debug!("Tentative de connexion à: https://code-insight.simvia-app.fr:8443");
        let endpoint = Channel::from_static("https://code-insight.simvia-app.fr:8443")
            .tls_config(tls)
            .expect("Configuration TLS failed");
        debug!("Configuration du canal TLS réussie");

        debug!("Établissement de la connexion...");
        let connect_timeout = Duration::from_millis(1000);
        let channel  = match timeout(connect_timeout, endpoint.connect()).await {
            Ok(Ok(ch)) => {
                debug!("Connexion TCP/TLS établie avec succès");
                ch
            }
            Ok(Err(e)) => {
                debug!("Échec de la connexion TCP/TLS: {}", e);
                debug!("Type d'erreur: {:?}", e);
                return Err(e.into());
            }
            Err(e) => {
                debug!("Timeout atteint lors de la connexion TCP/TLS");
                return Err(e.into());
            }
        };

        debug!("Création du client gRPC...");
        client = CaveTelemetryClient::new(channel);
        debug!("Client gRPC créé avec succès et connecté avec TLS à code-insight.simvia-app.fr:8443");
    }

    debug!("Construction de la requête Telemetry:");
    debug!("  - user_id: {}", e.user_id);
    debug!("  - time_execution: {} ms", e.time_execution);
    debug!("  - valid_result: {}", e.valid_result);
    debug!("  - timezone: {}", e.timezone);
    debug!("  - version: {}", e.version);
    debug!("  - id_docker: {}", e.id_docker);

    let request = tonic::Request::new(Telemetry {
        user_id: e.user_id.clone(),
        time_execution: e.time_execution as i64,
        valid_result: e.valid_result,
        timezone: e.timezone.clone(),
        version: e.version.clone(),
        id_docker: e.id_docker.clone(),
    });

    debug!("Envoi de la requête telemetry via gRPC...");
    match client.send_telemetry(request).await {
        Ok(response) => {
            debug!("✅ Requête telemetry envoyée avec succès!");
            debug!("Réponse du serveur: {:?}", response);
            debug!("=== FIN DE LA TÉLÉMÉTRIE (SUCCÈS) ===");
        }
        Err(e) => {
            debug!("❌ Échec de l'envoi de la requête telemetry");
            debug!("Erreur détaillée: {}", e);
            debug!("Type d'erreur gRPC: {:?}", e);
            debug!("Code d'erreur: {:?}", e.code());
            debug!("Message d'erreur: {}", e.message());
            debug!("=== FIN DE LA TÉLÉMÉTRIE (ÉCHEC) ===");
            return Err(e.into());
        }
    }

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

