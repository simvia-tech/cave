use log::debug;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct TelemetryPayload {
    user_id: String,
    time_execution: i64,
    valid_result: bool,
    timezone: String,
    version: String,
    id_docker: String,
    r#type: i32,
}

pub async fn send_execution_data(e: ExecutionData, local: bool) -> Result<(), Box<dyn std::error::Error>> {
    debug!("=== DÉBUT DE LA TÉLÉMÉTRIE ===");
    debug!("Initialisation du client HTTP pour la télémétrie");
    debug!("Données à envoyer: {:?}", e);

    let endpoint = if local {
        debug!("=== CONNEXION EN LOCAL ===");
        "http://localhost:8080/"
    } else {
        debug!("=== CONNEXION A DISTANCE ===");
        "https://7a98391a395292bd9f0f.lambda.simvia-app.fr"
    };

    debug!("Endpoint: {}", endpoint);

    let payload = TelemetryPayload {
        user_id: e.user_id.clone(),
        time_execution: e.time_execution as i64,
        valid_result: e.valid_result,
        timezone: e.timezone.clone(),
        version: e.version.clone(),
        id_docker: e.id_docker.clone(),
        r#type: 0, // 0 for cave, 1 for vs-code-aster
    };

    debug!("Construction de la requête Telemetry:");
    debug!("  - user_id: {}", payload.user_id);
    debug!("  - time_execution: {} ms", payload.time_execution);
    debug!("  - valid_result: {}", payload.valid_result);
    debug!("  - timezone: {}", payload.timezone);
    debug!("  - version: {}", payload.version);
    debug!("  - id_docker: {}", payload.id_docker);
    debug!("  - type: {}", payload.r#type);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(1000))
        .build()?;

    debug!("Envoi de la requête telemetry via HTTP POST...");
    match client.post(endpoint).json(&payload).send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                debug!("✅ Requête telemetry envoyée avec succès!");
                debug!("Status: {}", status);
                if let Ok(body) = response.text().await {
                    debug!("Réponse du serveur: {}", body);
                }
                debug!("=== FIN DE LA TÉLÉMÉTRIE (SUCCÈS) ===");
            } else {
                debug!("❌ Échec de l'envoi de la requête telemetry");
                debug!("Status: {}", status);
                if let Ok(body) = response.text().await {
                    debug!("Erreur détaillée: {}", body);
                }
                debug!("=== FIN DE LA TÉLÉMÉTRIE (ÉCHEC) ===");
                return Err(format!("HTTP error: {}", status).into());
            }
        }
        Err(e) => {
            debug!("❌ Échec de l'envoi de la requête telemetry");
            debug!("Erreur détaillée: {}", e);
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

