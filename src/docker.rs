//! Docker and version management for the `cave` CLI.
//!
//! This module handles interacting with Docker images, checking for local
//! and remote versions of code_aster, pulling images, running
//! images, and managing registry authentication.

use crate::manage::CaveError;
use std::process::{Command, Stdio};
use serde::Deserialize;
use std::io::ErrorKind;
use chrono::{Local, Offset};
use crate::config::{read_user_id};
use crate::telemetry::{send_execution_data, ExecutionData};
use log::debug;
use std::env;

// TODO : uncomment to have registry option
// use regex::Regex;
// use crate::config::Registry;


/// Returns a list of locally code_aster Docker image tags.
///
/// # Errors
/// Returns [`CaveError::NoDocker`] if Docker is not installed,
/// [`CaveError::DockerError`] if the `docker images` command fails.
///
/// # Example
/// ```
/// let versions = local_versions().expect("Failed to get local versions");
/// println!("Local versions: {:?}", versions);
/// ```
pub fn local_versions() -> Result<Vec<String>, CaveError> {
    let output = Command::new("docker")
        .arg("images")
        .arg("--filter")
        .arg("reference=simvia/code_aster")
        .arg("--format")
        .arg("{{.Tag}}")
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CaveError::NoDocker
            } else {
                CaveError::IoError(e)
            }
        })?;

    if !output.status.success() {
        return Err(CaveError::DockerError(
            "Failed to run `docker images`.".into(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let versions: Vec<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(versions)
}



/// Checks if a specific version exists locally.
///
/// # Example
/// ```
/// let exists = exists_locally("22.0").unwrap_or(false);
/// println!("Version exists locally? {}", exists);
/// ```
pub fn exists_locally(version: &str) -> Result<bool, CaveError> {
    let versions = local_versions()?;
    Ok(versions.contains(&version.to_string()))
}


#[derive(Debug, Deserialize)]
struct TagImage {
    last_pushed: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Tag {
    name: String,
    images: Vec<TagImage>,
}


#[derive(Debug, Deserialize)]
struct TagsResponse {
    results: Vec<Tag>,
    next: Option<String>,
}

/// Returns a list of remote `simvia/code_aster` Docker image tags.
/// 
/// If there is a registry in the user's config, we return additionnaly those in the registry
///
/// # Errors
/// Returns [`CaveError::HttpError`] if the request fails or cannot be parsed.
///
/// # Example
/// ```
/// let versions = remote_versions().expect("Failed to fetch remote versions");
/// for (tag, date) in versions {
///     println!("{} pushed on {}", tag, date);
/// }
/// ```
pub fn remote_versions() -> Result<Vec<(String, String)>, CaveError> {
    let mut versions = Vec::new();
    let mut url = "https://hub.docker.com/v2/repositories/simvia/code_aster/tags?page_size=100".to_string();

    loop {
        let resp = reqwest::blocking::get(&url)
            .map_err(|e| CaveError::HttpError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CaveError::HttpError(format!(
                "Failed to fetch Docker tags: {}",
                resp.status()
            )));
        }

        let tags_response: TagsResponse =
            resp.json().map_err(|e| CaveError::HttpError(e.to_string()))?;

        for tag in tags_response.results {
            let last_pushed = tag
                .images
                .get(0)
                .and_then(|img| img.last_pushed.clone())
                .unwrap_or_else(|| "unknown".to_string());

            versions.push((tag.name, last_pushed));
        }

        if let Some(next_url) = tags_response.next {
            url = next_url;
        } else {
            break;
        }
    }

    Ok(versions)
}

/// Checks if a specific version exists on the Simvia Docker hub or in the private registry.
/// 
/// # TO DO :
/// If there is a registry in the user's config, we look firstly in the private registry
///
/// # Example
/// ```
/// let exists = exists_remotely("22.0").unwrap_or(false);
/// println!("Version exists remotely? {}", exists);
/// ```
pub fn exists_remotely(version: &str) -> Result<bool, CaveError> {
    let versions = remote_versions()?;
    Ok(versions.iter().any(|(tag, _date)| tag == version))
}


/// Pulls a specific version of `simvia/code_aster` from the Simvia Docker Hub or in the private registry.
///
/// # TO DO :
/// If there is a registry in the user's config, we pull firstly in the private registry
/// 
/// # Errors
/// Returns [`CaveError::DockerError`] if the pull fails.
///
/// # Example
/// ```
/// pull_version("22.0").expect("Failed to pull version");
/// ```
pub fn pull_version(version: &str) -> Result<(), CaveError> {
    let image = format!("simvia/code_aster:{}", version);

    let output = Command::new("docker")
        .arg("pull")
        .arg(&image)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CaveError::NoDocker
            } else {
                CaveError::IoError(e)
            }
        })?;


    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CaveError::DockerError(format!(
            "Failed to pull version: {}\n{}",
            version, stderr
        )));
    }
    Ok(())
}


/// Runs code_aster with Docker with the given version, optional export file, and arguments.
/// Envoie ensuite les données d'exécution via gRPC si l'exécution n'est pas interactive.
///
/// # Example
/// ```
/// docker_aster("22.0", &Some("output.msh".to_string()), &vec!["command".to_string()])
///     .expect("Failed to run Code_Aster in Docker");
/// ```
pub fn docker_aster(version: &str, export_file: &Option<String>, args: &Vec<String>) -> Result<(), CaveError> {
    let start = std::time::Instant::now();

    let current_dir = std::env::current_dir().map_err(CaveError::IoError)?;
    let volume_arg = format!("{}:/home/user/data", current_dir.display());
    let image = format!("simvia/code_aster:{}", version);
    let export = export_file.clone().unwrap_or_default();
    let docker_command = format!("run_aster {} {}", args.join(" "), export);

    let mut child = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-it")
        .arg("-v")
        .arg(&volume_arg)
        .arg("-w")
        .arg("/home/user/data")
        .arg(&image)
        .arg("/bin/bash")
        .arg("-i")
        .arg("-c")
        .arg(&docker_command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                CaveError::NoDocker
            } else {
                CaveError::IoError(e)
            }
        })?;

    let status = child.wait().map_err(CaveError::IoError)?;
    let interactive = args.contains(&"-i".to_string());

    if !interactive {
        debug!("Début de la telemetry");
        debug!("Début de la collecte des données du run");

        let mut execution_data = ExecutionData::default();
        execution_data.user_id = read_user_id()?;
        debug!("user_id récupéré: {}", execution_data.user_id);

        execution_data.time_execution = start.elapsed().as_millis();
        execution_data.valid_result = status.success();
        execution_data.timezone = Local::now().offset().fix().to_string();
        execution_data.version = version.to_string();
        execution_data.id_docker = image_id(version)?;
        debug!("ID docker récupéré: {}", execution_data.id_docker);

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| {
                debug!("Erreur lors de la création du runtime tokio: {}", e);
                CaveError::TelemetryError(e.to_string())
            })?;

        debug!("Runtime tokio créé, envoi des données...");

        rt.block_on(async {
            debug!("Appel de send_execution_data()");
            let local_telemetry = env::var("LOCAL_TELEMETRY").map(|v| v == "true").unwrap_or(false);
            let _ = send_execution_data(execution_data, local_telemetry).await;
            debug!("Fin de send_execution_data()");
        });

        debug!("Collecte et envoi des données terminés");
    }


    if !status.success() {
        return Err(CaveError::CodeAsterError(format!(
            "run failed for version: {}",
            version
        )));
    }

    Ok(())
}


pub fn image_id(version: &str) -> Result<String, CaveError> {
    let reference = format!("simvia/code_aster:{}", version);

    let output = Command::new("docker")
        .arg("images")
        .arg("-q")
        .arg(&reference)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CaveError::NoDocker
            } else {
                CaveError::IoError(e)
            }
        })?;

    if !output.status.success() {
        return Err(CaveError::DockerError(format!(
            "Failed to run `docker images` for {}",
            reference
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout.lines()
    .map(str::trim)
    .find(|l| !l.is_empty())
    .ok_or_else(|| CaveError::DockerError(format!("No image found for {}", reference)))?;

    Ok(id.to_string())
}


/// Returns the version associated with a given tag (`stable` or `testing`).
///
/// # Example
/// ```
/// let version = version_under_tag("stable".to_string()).unwrap();
/// println!("Stable version: {}", version);
/// ```
pub fn version_under_tag(tag : String) -> Result<String, CaveError> {
    let (stable_version, testing_version) = get_stable_and_testing()?;
    if tag == "stable" {
        return Ok(stable_version);
    }

    if tag == "testing" {
        return Ok(testing_version);
    }   

    Ok("".to_string())
}

#[derive(Debug, Deserialize)]
struct StabTestImage {
    digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StabTestTag {
    name: String,
    images: Vec<StabTestImage>,
}

#[derive(Debug, Deserialize)]
struct StabTestTagsResponse {
    results: Vec<StabTestTag>,
    next: Option<String>,
}


/// Returns the latest `stable` and `testing` versions from Docker Hub.
///
/// # Example
/// ```
/// let (stable, testing) = get_stable_and_testing().unwrap();
/// println!("Stable: {}, Testing: {}", stable, testing);
/// ```
pub fn get_stable_and_testing() -> Result<(String, String), CaveError> {
    let mut all_versions = Vec::new();
    let mut url = "https://hub.docker.com/v2/repositories/simvia/code_aster/tags?page_size=100".to_string();
    loop {
        let resp = reqwest::blocking::get(&url)
            .map_err(|e| CaveError::HttpError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CaveError::HttpError(format!(
                "Failed to fetch Docker tags: {}",
                resp.status()
            )));
        }

        let tags_response: StabTestTagsResponse =
            resp.json().map_err(|e| CaveError::HttpError(e.to_string()))?;

        for tag in tags_response.results {
            let digest = tag
                .images
                .get(0)
                .and_then(|img| img.digest.clone())
                .unwrap_or_else(|| "unknown".to_string());

            all_versions.push((tag.name, digest));
        }

        if let Some(next_url) = tags_response.next {
            url = next_url;
        } else {
            break;
        }
    }
    let mut stable_digest = None;
    let mut testing_digest = None;

    for (tag, digest) in &all_versions {
        if tag == "stable" {
            stable_digest = Some(digest.clone());
        }
        if tag == "testing" {
            testing_digest = Some(digest.clone());
        }
    }
    let mut stable_tag = String::new();
    let mut testing_tag = String::new();

    for (tag, digest) in &all_versions {
        if Some(digest) == stable_digest.as_ref() && tag != "stable" {
            stable_tag = tag.clone();
        }
        if Some(digest) == testing_digest.as_ref() && tag != "testing" {
            testing_tag = tag.clone();
        }
    }
    Ok((stable_tag, testing_tag))
}

// TODO : uncomment to have registry option
//
// fn docker_login(registry_cfg: &Registry) -> Result<(), CaveError> {
//     let registry = "registry.gitlab.com";
//     let user = &registry_cfg.user;
//     let token = &registry_cfg.token; 

//     let login_status = Command::new("docker")
//         .arg("login")
//         .arg(registry)
//         .arg("-u")
//         .arg(user)
//         .arg("--password-stdin")
//         .stdin(std::process::Stdio::piped())
//         .spawn()
//         .and_then(|mut child| {
//             use std::io::Write;
//             if let Some(stdin) = &mut child.stdin {
//                 stdin.write_all(token.as_bytes())?;
//             }
//             child.wait()
//         })
//         .map_err(|e| CaveError::IoError(e))?;

//     if !login_status.success() {
//         return Err(CaveError::DockerError("Docker login failed".into()));
//     }
//     Ok(())
// }


// TODO : uncomment to have registry option
//
// / Returns a list of tags available in the private registry.
// / 
// / Each time, it processes a docker login with the registry_cf (call to docker_login),
// / then pull the available versions on the registry and finally logout.
// /
// / # Example
// / ```
// / let registry_cfg = Registry {
// /     repo: "myrepo".to_string(),
// /     user: "username".to_string(),
// /     token: "mytoken".to_string(),
// / };
// / let tags = registry_versions(&registry_cfg).expect("Failed to fetch registry tags");
// / println!("Registry tags: {:?}", tags);
// / ```
// pub fn registry_versions(registry_cfg: &Registry) -> Result<Vec<String>, CaveError> {
//     docker_login(registry_cfg)?;

//     let registry = "registry.gitlab.com";
//     let repo = &registry_cfg.repo;
//     let token = &registry_cfg.token; 


//     let auth_header = reqwest::blocking::Client::new()
//         .head(&format!("https://{}/v2/{}/tags/list", registry, repo))
//         .send()
//         .map_err(|e| CaveError::HttpError(e.to_string()))?
//         .headers()
//         .get("www-authenticate")
//         .ok_or_else(|| CaveError::DockerError("No www-authenticate header".into()))?
//         .to_str()
//         .map_err(|e| CaveError::HttpError(e.to_string()))?
//         .to_string();

//     let realm = Regex::new(r#"realm="([^"]+)""#).unwrap()
//         .captures(&auth_header)
//         .and_then(|c| c.get(1))
//         .ok_or_else(|| CaveError::DockerError("No realm found".into()))?
//         .as_str()
//         .to_string();

//     let service = Regex::new(r#"service="([^"]+)""#).unwrap()
//         .captures(&auth_header)
//         .and_then(|c| c.get(1))
//         .ok_or_else(|| CaveError::DockerError("No service found".into()))?
//         .as_str()
//         .to_string();

//     let scope = Regex::new(r#"scope="([^"]+)""#).unwrap()
//         .captures(&auth_header)
//         .and_then(|c| c.get(1))
//         .ok_or_else(|| CaveError::DockerError("No scope found".into()))?
//         .as_str()
//         .to_string();

//     let jwt_resp: serde_json::Value = reqwest::blocking::Client::new()
//         .get(&format!("{}?service={}&scope={}", realm, service, scope))
//         .basic_auth("oauth2", Some(token))
//         .send()
//         .map_err(|e| CaveError::HttpError(e.to_string()))?
//         .json()
//         .map_err(|e| CaveError::HttpError(e.to_string()))?;

//     let jwt = jwt_resp.get("token")
//         .and_then(|t| t.as_str())
//         .ok_or_else(|| CaveError::DockerError("No token in JWT response".into()))?;

//     let tags_resp: serde_json::Value = reqwest::blocking::Client::new()
//         .get(&format!("https://{}/v2/{}/tags/list", registry, repo))
//         .bearer_auth(jwt)
//         .send()
//         .map_err(|e| CaveError::HttpError(e.to_string()))?
//         .json()
//         .map_err(|e| CaveError::HttpError(e.to_string()))?;

//     let tags = tags_resp.get("tags")
//         .and_then(|t| t.as_array())
//         .ok_or_else(|| CaveError::DockerError("No tags found".into()))?
//         .iter()
//         .filter_map(|t| t.as_str().map(|s| s.to_string()))
//         .collect::<Vec<String>>();

//     let _ = Command::new("docker")
//         .arg("logout")
//         .arg(registry)
//         .status();

//     Ok(tags)
// }
