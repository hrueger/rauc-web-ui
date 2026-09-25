//! Updates from a manifest on the web: which bundle is the newest for this
//! system, and installing it straight from its URL (RAUC streams it, so it
//! never has to fit into the upload directory).
//!
//! The manifest lives at `UPDATE_MANIFEST_URL` and lists one bundle per
//! compatible:
//!
//! ```json
//! {
//!   "bundles": [
//!     {
//!       "compatible": "my-device-raspberrypi5",
//!       "version": "1.4.0",
//!       "url": "https://downloads.example.com/my-device/1.4.0/bundle.raucb",
//!       "sha256": "…", "size": 123456789,
//!       "notes": "What changed", "publishedAt": "2026-09-25T12:00:00Z"
//!     }
//!   ]
//! }
//! ```
//!
//! Trust comes from RAUC: it installs only bundles signed by the keyring, so
//! the manifest and the download only need to be reachable.

use crate::rauc::RaucStatus;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::time::Duration;

pub const MANIFEST_URL_ENV: &str = "UPDATE_MANIFEST_URL";
/// Set to 1 to accept http:// bundle URLs, for a test server in the LAN.
pub const ALLOW_HTTP_ENV: &str = "ALLOW_HTTP_BUNDLE_URLS";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ManifestBundle {
    pub compatible: String,
    pub version: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub bundles: Vec<ManifestBundle>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub manifest_url: String,
    pub compatible: String,
    /// The slot's bootname, e.g. "A".
    pub booted_slot: Option<String>,
    pub booted_version: Option<String>,
    /// The manifest's bundle for this compatible, if it lists one.
    pub latest: Option<ManifestBundle>,
    /// `latest` is newer than what runs.
    pub available: bool,
}

pub fn manifest_url() -> Option<String> {
    std::env::var(MANIFEST_URL_ENV)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub async fn fetch_manifest(url: &str) -> Result<Manifest, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let response = client
        .get(url)
        .header("Cache-Control", "no-cache")
        .send()
        .await
        .map_err(|e| format!("Manifest not reachable: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("Manifest: HTTP {}", response.status()));
    }
    response
        .json::<Manifest>()
        .await
        .map_err(|e| format!("Manifest is not valid: {}", e))
}

pub fn check(manifest_url: String, manifest: Manifest, status: &RaucStatus) -> UpdateCheck {
    let (booted_slot, slot_version) = booted_slot(status);
    let booted_version = slot_version.or_else(os_release_version);
    let latest = manifest
        .bundles
        .into_iter()
        .find(|b| b.compatible == status.compatible);
    let available = match (&latest, &booted_version) {
        (Some(latest), Some(booted)) => compare_versions(&latest.version, booted) == Ordering::Greater,
        // Nothing says what runs: offer what there is.
        (Some(_), None) => true,
        (None, _) => false,
    };
    UpdateCheck {
        manifest_url,
        compatible: status.compatible.clone(),
        booted_slot,
        booted_version,
        latest,
        available,
    }
}

/// The bootname of the booted slot and the version of the bundle that was
/// installed into it. A slot written by the SD card image, not by RAUC, has
/// no bundle version.
fn booted_slot(status: &RaucStatus) -> (Option<String>, Option<String>) {
    for entry in &status.slots {
        let Some(map) = entry.as_object() else { continue };
        for slot in map.values() {
            if slot.get("state").and_then(|s| s.as_str()) != Some("booted") {
                continue;
            }
            // The bootloader partition is not the one with the bundle
            // version on systems that have one; the rootfs always is.
            if slot.get("bootname").is_none() && slot.get("class").and_then(|c| c.as_str()) != Some("rootfs") {
                continue;
            }
            let bootname = slot.get("bootname").and_then(|b| b.as_str()).map(str::to_string);
            let version = slot
                .pointer("/slot_status/bundle/version")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            return (bootname, version);
        }
    }
    (None, None)
}

/// `VERSION_ID` of /etc/os-release, what the image was built as.
fn os_release_version() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    content.lines().find_map(|line| {
        line.strip_prefix("VERSION_ID=")
            .map(|v| v.trim_matches('"').to_string())
            .filter(|v| !v.is_empty())
    })
}

/// "1.10.0" after "1.9.2": numeric parts numerically, anything else as text.
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let split = |v: &str| {
        v.split(|c: char| c == '.' || c == '-' || c == '+')
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    let (a, b) = (split(a), split(b));
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).map(String::as_str).unwrap_or("0");
        let y = b.get(i).map(String::as_str).unwrap_or("0");
        let order = match (x.parse::<u64>(), y.parse::<u64>()) {
            (Ok(x), Ok(y)) => x.cmp(&y),
            _ => x.cmp(y),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    Ordering::Equal
}

/// Only what RAUC can stream, and http:// only when allowed.
pub fn validate_bundle_url(url: &str) -> Result<(), String> {
    let allow_http = std::env::var(ALLOW_HTTP_ENV).is_ok_and(|v| v == "1");
    if url.starts_with("https://") || (allow_http && url.starts_with("http://")) {
        Ok(())
    } else if url.starts_with("http://") {
        Err(format!("http:// bundle URLs are off; set {}=1 to allow them", ALLOW_HTTP_ENV))
    } else {
        Err("The bundle URL must start with https://".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn status(slots: serde_json::Value) -> RaucStatus {
        serde_json::from_value(json!({
            "compatible": "station-rpi5",
            "variant": "",
            "booted": "A",
            "boot_primary": "rootfs.0",
            "slots": slots,
            "artifact-repositories": []
        }))
        .unwrap()
    }

    fn manifest(version: &str) -> Manifest {
        Manifest {
            bundles: vec![
                ManifestBundle {
                    compatible: "other".into(),
                    version: "9.9.9".into(),
                    url: "https://x/other.raucb".into(),
                    sha256: None,
                    size: None,
                    notes: None,
                    published_at: None,
                },
                ManifestBundle {
                    compatible: "station-rpi5".into(),
                    version: version.into(),
                    url: "https://x/station.raucb".into(),
                    sha256: None,
                    size: None,
                    notes: None,
                    published_at: None,
                },
            ],
        }
    }

    #[test]
    fn versions_compare_numerically() {
        assert_eq!(compare_versions("1.10.0", "1.9.2"), Ordering::Greater);
        assert_eq!(compare_versions("0.2", "0.2.0"), Ordering::Equal);
        assert_eq!(compare_versions("0.1.0", "0.2.0"), Ordering::Less);
    }

    #[test]
    fn picks_the_bundle_for_this_compatible_and_the_booted_slot() {
        let slots = json!([
            { "boot.0": { "class": "boot", "state": "booted" } },
            { "rootfs.0": { "class": "rootfs", "bootname": "A", "state": "booted",
                "slot_status": { "bundle": { "version": "0.1.0" } } } },
            { "rootfs.1": { "class": "rootfs", "bootname": "B", "state": "inactive" } }
        ]);
        let result = check("https://x/manifest.json".into(), manifest("0.2.0"), &status(slots));
        assert_eq!(result.booted_slot.as_deref(), Some("A"));
        assert_eq!(result.booted_version.as_deref(), Some("0.1.0"));
        assert_eq!(result.latest.unwrap().url, "https://x/station.raucb");
        assert!(result.available);
    }

    #[test]
    fn nothing_newer_is_not_available() {
        let slots = json!([
            { "rootfs.1": { "class": "rootfs", "bootname": "B", "state": "booted",
                "slot_status": { "bundle": { "version": "0.2.0" } } } }
        ]);
        let result = check("m".into(), manifest("0.2.0"), &status(slots));
        assert!(!result.available);
    }

    #[test]
    fn only_https_bundles_by_default() {
        assert!(validate_bundle_url("https://x/b.raucb").is_ok());
        assert!(validate_bundle_url("http://x/b.raucb").is_err());
        assert!(validate_bundle_url("/tmp/b.raucb").is_err());
    }
}
