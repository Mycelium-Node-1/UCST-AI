//! Versioned data contracts for HDGE Studio.
//!
//! These types represent ordinary, inspectable software artifacts. They do not
//! claim to model a literal five-dimensional or physical holographic system.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

pub const SCENE_SCHEMA: &str = "hdge.scene/v1";
pub const TDM_RUN_SCHEMA: &str = "hdge.tdm-run/v1";
pub const BACKEND_REPORT_SCHEMA: &str = "hdge.backend-report/v1";
pub const PHYSICS_RUN_SCHEMA: &str = "hdge.physics-run/v1";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveKind {
    Sphere,
    Box,
    Plane,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Primitive {
    pub id: String,
    pub kind: PrimitiveKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_symbol: Option<String>,
    pub center: Vec3,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub half_extents: Option<Vec3>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Composition {
    Union,
    Intersection,
    Subtraction,
    SmoothUnion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub vertical_fov_degrees: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3 {
                x: 0.0,
                y: 2.2,
                z: 7.5,
            },
            target: Vec3::ZERO,
            vertical_fov_degrees: 50.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub compiler: String,
    pub mapping_policy: String,
    pub source_digest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub schema: String,
    pub scene_id: String,
    pub revision: u32,
    pub composition: Composition,
    pub primitives: Vec<Primitive>,
    pub camera: Camera,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TdmPhase {
    Anchor,
    Insert,
    Expand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TdmEvent {
    pub tick: u64,
    pub phase: TdmPhase,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TdmRun {
    pub schema: String,
    pub run_id: String,
    pub base_scene_digest: String,
    pub candidate_scene_digest: String,
    pub accepted: bool,
    pub validation: Vec<ValidationCheck>,
    pub events: Vec<TdmEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendReport {
    pub schema: String,
    pub scene_digest: String,
    pub renderer: String,
    pub renderer_revision: String,
    pub mode: String,
    pub probe_count: u32,
    pub summary: String,
}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("failed to serialize canonical JSON: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Serializes a contract in the stable field order specified by its Rust type.
/// HDGE contracts deliberately avoid hash maps so the representation is stable.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, ContractError> {
    Ok(serde_json::to_string(value)?)
}

pub fn digest_hex<T: Serialize>(value: &T) -> Result<String, ContractError> {
    let mut hasher = Sha256::new();
    hasher.update(canonical_json(value)?.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn source_digest(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn validate_scene(scene: &Scene) -> Vec<ValidationCheck> {
    let ids: BTreeSet<&str> = scene
        .primitives
        .iter()
        .map(|primitive| primitive.id.as_str())
        .collect();
    let unique = ids.len() == scene.primitives.len();
    let non_empty = !scene.primitives.is_empty();
    let schema_ok = scene.schema == SCENE_SCHEMA;
    let camera_ok = scene.camera.position.is_finite()
        && scene.camera.target.is_finite()
        && scene.camera.vertical_fov_degrees.is_finite()
        && (1.0..179.0).contains(&scene.camera.vertical_fov_degrees);

    let mut checks = vec![
        ValidationCheck {
            name: "schema_version".into(),
            passed: schema_ok,
            detail: format!("Expected schema {SCENE_SCHEMA}."),
        },
        ValidationCheck {
            name: "non_empty_scene".into(),
            passed: non_empty,
            detail: "Scene contains at least one primitive.".into(),
        },
        ValidationCheck {
            name: "unique_primitive_ids".into(),
            passed: unique,
            detail: "Primitive identifiers are unique.".into(),
        },
        ValidationCheck {
            name: "valid_camera".into(),
            passed: camera_ok,
            detail: "Camera contains finite values and a supported field of view.".into(),
        },
    ];

    for primitive in &scene.primitives {
        let finite_center = primitive.center.is_finite();
        let valid_parameters = match primitive.kind {
            PrimitiveKind::Sphere => primitive
                .radius
                .map(|radius| radius.is_finite() && radius > 0.0)
                .unwrap_or(false),
            PrimitiveKind::Box => primitive
                .half_extents
                .map(|half| half.is_finite() && half.x > 0.0 && half.y > 0.0 && half.z > 0.0)
                .unwrap_or(false),
            PrimitiveKind::Plane => finite_center,
        };
        checks.push(ValidationCheck {
            name: format!("valid_{}", primitive.id),
            passed: finite_center && valid_parameters,
            detail: format!(
                "Primitive {} has finite, supported parameters.",
                primitive.id
            ),
        });
    }
    checks
}

pub fn accepted(checks: &[ValidationCheck]) -> bool {
    checks.iter().all(|check| check.passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_scene() -> Scene {
        Scene {
            schema: SCENE_SCHEMA.into(),
            scene_id: "sample".into(),
            revision: 1,
            composition: Composition::Union,
            primitives: vec![Primitive {
                id: "anchor".into(),
                kind: PrimitiveKind::Sphere,
                source_symbol: Some("☉".into()),
                center: Vec3::ZERO,
                radius: Some(1.0),
                half_extents: None,
            }],
            camera: Camera::default(),
            provenance: Provenance {
                compiler: "test".into(),
                mapping_policy: "test".into(),
                source_digest: source_digest("☉"),
            },
        }
    }

    #[test]
    fn canonical_scene_digest_is_stable() {
        let scene = sample_scene();
        assert_eq!(digest_hex(&scene).unwrap(), digest_hex(&scene).unwrap());
        assert!(canonical_json(&scene).unwrap().contains("hdge.scene/v1"));
    }

    #[test]
    fn invalid_sphere_is_rejected() {
        let mut scene = sample_scene();
        scene.primitives[0].radius = Some(0.0);
        assert!(!accepted(&validate_scene(&scene)));
    }
}
