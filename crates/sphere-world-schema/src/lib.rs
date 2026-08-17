//! Canonical contracts for a sphere-first, declarative/procedural world.
//!
//! A `SphereWorld` is source data. Meshes, wireframes, collision surfaces, and
//! camera transforms are derived artifacts and are intentionally not persisted
//! in this contract.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use thiserror::Error;

pub const WORLD_SCHEMA: &str = "sphere-world/v1";

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

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length <= f64::EPSILON {
            Self::ZERO
        } else {
            self * (1.0 / length)
        }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CubeFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl CubeFace {
    pub const ALL: [Self; 6] = [
        Self::PosX,
        Self::NegX,
        Self::PosY,
        Self::NegY,
        Self::PosZ,
        Self::NegZ,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyKind {
    CubeSphere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionKind {
    NormalizedCube,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topology {
    pub kind: TopologyKind,
    pub projection: ProjectionKind,
    pub max_level: u8,
    pub patch_resolution: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LodPolicy {
    pub max_neighbor_level_delta: u8,
    pub use_edge_skirts: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Layer {
    RadialNoise {
        id: String,
        amplitude_m: f64,
        frequency: f64,
        seed_offset: u64,
    },
    LatitudeBand {
        id: String,
        min_degrees: f64,
        max_degrees: f64,
        behavior: String,
    },
}

impl Layer {
    pub fn id(&self) -> &str {
        match self {
            Self::RadialNoise { id, .. } | Self::LatitudeBand { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObserverAnchor {
    pub id: String,
    pub kind: String,
    pub face: CubeFace,
    pub level: u8,
    pub u: f64,
    pub v: f64,
    pub altitude_m: f64,
    pub heading_degrees: f64,
    pub pitch_degrees: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructureFootprint {
    pub id: String,
    pub face: CubeFace,
    pub level: u8,
    pub u: f64,
    pub v: f64,
    pub width_m: f64,
    pub depth_m: f64,
    pub heading_degrees: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedOutputs {
    pub render_mesh: bool,
    pub wireframe: bool,
    pub collision_mesh: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SphereWorld {
    pub schema: String,
    pub world_id: String,
    pub seed: u64,
    pub radius_m: f64,
    pub topology: Topology,
    pub lod: LodPolicy,
    pub layers: Vec<Layer>,
    pub anchors: Vec<ObserverAnchor>,
    pub structures: Vec<StructureFootprint>,
    pub derived_outputs: DerivedOutputs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PatchId {
    pub face: CubeFace,
    pub level: u8,
    pub x: u32,
    pub y: u32,
}

impl PatchId {
    pub fn root(face: CubeFace) -> Self {
        Self {
            face,
            level: 0,
            x: 0,
            y: 0,
        }
    }

    pub fn children(self) -> [Self; 4] {
        let level = self.level + 1;
        let base_x = self.x * 2;
        let base_y = self.y * 2;
        [
            Self {
                face: self.face,
                level,
                x: base_x,
                y: base_y,
            },
            Self {
                face: self.face,
                level,
                x: base_x + 1,
                y: base_y,
            },
            Self {
                face: self.face,
                level,
                x: base_x,
                y: base_y + 1,
            },
            Self {
                face: self.face,
                level,
                x: base_x + 1,
                y: base_y + 1,
            },
        ]
    }

    pub fn is_valid_for(self, topology: &Topology) -> bool {
        if self.level > topology.max_level || self.level >= 31 {
            return false;
        }
        let width = 1_u32 << self.level;
        self.x < width && self.y < width
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("failed to serialize canonical JSON: {0}")]
    Serialize(#[from] serde_json::Error),
}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, ContractError> {
    Ok(serde_json::to_string(value)?)
}

pub fn digest_hex<T: Serialize>(value: &T) -> Result<String, ContractError> {
    let mut hasher = Sha256::new();
    hasher.update(canonical_json(value)?.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn accepted(checks: &[ValidationCheck]) -> bool {
    checks.iter().all(|check| check.passed)
}

pub fn validate_world(world: &SphereWorld) -> Vec<ValidationCheck> {
    let mut checks = Vec::new();
    checks.push(ValidationCheck {
        name: "schema_version".into(),
        passed: world.schema == WORLD_SCHEMA,
        detail: format!("Expected schema {WORLD_SCHEMA}."),
    });
    checks.push(ValidationCheck {
        name: "world_identity".into(),
        passed: !world.world_id.trim().is_empty(),
        detail: "World identifier is non-empty.".into(),
    });
    checks.push(ValidationCheck {
        name: "positive_radius".into(),
        passed: world.radius_m.is_finite() && world.radius_m > 0.0,
        detail: "World radius is finite and positive.".into(),
    });
    checks.push(ValidationCheck {
        name: "topology_limits".into(),
        passed: world.topology.max_level <= 24
            && world.topology.patch_resolution >= 3
            && world.topology.patch_resolution % 2 == 1,
        detail:
            "Maximum level is bounded and patch resolution is an odd integer of at least three."
                .into(),
    });
    checks.push(ValidationCheck {
        name: "lod_seam_policy".into(),
        passed: world.lod.max_neighbor_level_delta <= 1,
        detail: "Neighbor level delta is at most one for seam-safe prototype stitching.".into(),
    });

    let mut ids = BTreeSet::new();
    for layer in &world.layers {
        let valid = !layer.id().trim().is_empty() && ids.insert(format!("layer:{}", layer.id()));
        let parameters_valid = match layer {
            Layer::RadialNoise {
                amplitude_m,
                frequency,
                ..
            } => amplitude_m.is_finite() && frequency.is_finite() && *frequency > 0.0,
            Layer::LatitudeBand {
                min_degrees,
                max_degrees,
                ..
            } => {
                min_degrees.is_finite()
                    && max_degrees.is_finite()
                    && *min_degrees >= -90.0
                    && *max_degrees <= 90.0
                    && min_degrees <= max_degrees
            }
        };
        checks.push(ValidationCheck {
            name: format!("valid_layer_{}", layer.id()),
            passed: valid && parameters_valid,
            detail: format!(
                "Layer {} has unique identity and supported finite parameters.",
                layer.id()
            ),
        });
    }

    for anchor in &world.anchors {
        let unique = !anchor.id.trim().is_empty() && ids.insert(format!("anchor:{}", anchor.id));
        let valid = anchor.level <= world.topology.max_level
            && anchor.u.is_finite()
            && (0.0..=1.0).contains(&anchor.u)
            && anchor.v.is_finite()
            && (0.0..=1.0).contains(&anchor.v)
            && anchor.altitude_m.is_finite()
            && anchor.heading_degrees.is_finite()
            && anchor.pitch_degrees.is_finite();
        checks.push(ValidationCheck {
            name: format!("valid_anchor_{}", anchor.id),
            passed: unique && valid,
            detail: format!(
                "Anchor {} uses a valid surface coordinate and finite observer values.",
                anchor.id
            ),
        });
    }

    for structure in &world.structures {
        let unique =
            !structure.id.trim().is_empty() && ids.insert(format!("structure:{}", structure.id));
        let valid = structure.level <= world.topology.max_level
            && structure.u.is_finite()
            && (0.0..=1.0).contains(&structure.u)
            && structure.v.is_finite()
            && (0.0..=1.0).contains(&structure.v)
            && structure.width_m.is_finite()
            && structure.width_m > 0.0
            && structure.depth_m.is_finite()
            && structure.depth_m > 0.0
            && structure.heading_degrees.is_finite();
        checks.push(ValidationCheck {
            name: format!("valid_structure_{}", structure.id),
            passed: unique && valid,
            detail: format!(
                "Structure {} is anchored to a supported patch coordinate.",
                structure.id
            ),
        });
    }
    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_world() -> SphereWorld {
        SphereWorld {
            schema: WORLD_SCHEMA.into(),
            world_id: "sample".into(),
            seed: 42,
            radius_m: 1000.0,
            topology: Topology {
                kind: TopologyKind::CubeSphere,
                projection: ProjectionKind::NormalizedCube,
                max_level: 8,
                patch_resolution: 5,
            },
            lod: LodPolicy {
                max_neighbor_level_delta: 1,
                use_edge_skirts: true,
            },
            layers: vec![],
            anchors: vec![],
            structures: vec![],
            derived_outputs: DerivedOutputs {
                render_mesh: true,
                wireframe: true,
                collision_mesh: true,
            },
        }
    }

    #[test]
    fn patch_children_are_stable() {
        let root = PatchId::root(CubeFace::PosZ);
        assert_eq!(
            root.children()[3],
            PatchId {
                face: CubeFace::PosZ,
                level: 1,
                x: 1,
                y: 1
            }
        );
    }

    #[test]
    fn world_digest_is_stable() {
        let world = sample_world();
        assert!(accepted(&validate_world(&world)));
        assert_eq!(digest_hex(&world).unwrap(), digest_hex(&world).unwrap());
    }
}
