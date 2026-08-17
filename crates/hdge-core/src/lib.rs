//! HDGE semantic core for the documented, testable FGL subset.
//!
//! The source-to-geometry mapping below is an explicit research visualization
//! policy. It is not a claim that FGL symbols have uniquely determined physical
//! geometry, nor that this software implements literal five-dimensional physics.

use hdge_schema::{
    accepted, digest_hex, source_digest, validate_scene, Camera, Composition, Primitive,
    PrimitiveKind, Provenance, Scene, TdmEvent, TdmPhase, TdmRun, ValidationCheck, Vec3,
    SCENE_SCHEMA, TDM_RUN_SCHEMA,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const COMPILER: &str = "hdge-core-v0.1";
const POLICY: &str =
    "HDGE Studio documented-subset mapping; inspectable research visualization only.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FglRole {
    Subject,
    Verb,
    Object,
    Modifier,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FglSymbol {
    pub symbol: char,
    pub meaning: String,
    pub role: FglRole,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedFgl {
    pub source: String,
    pub symbols: Vec<FglSymbol>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Compilation {
    pub source: ParsedFgl,
    pub scene: Scene,
    pub validation: Vec<ValidationCheck>,
    pub scene_digest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProbeResult {
    pub point: Vec3,
    pub distance: f64,
    pub primitive_id: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompileError {
    #[error("FGL source cannot be empty")]
    Empty,
    #[error("unknown FGL symbol {symbol:?} at character offset {offset}")]
    UnknownSymbol { symbol: char, offset: usize },
    #[error("FGL source needs a documented subject symbol")]
    MissingSubject,
    #[error("FGL source needs a documented object symbol")]
    MissingObject,
    #[error("unsupported FGL subject symbol {0:?}")]
    UnsupportedSubject(char),
    #[error("unsupported FGL object symbol {0:?}")]
    UnsupportedObject(char),
    #[error("contract serialization failed: {0}")]
    Contract(String),
}

fn lookup(symbol: char) -> Option<FglSymbol> {
    let (meaning, role) = match symbol {
        '☉' => ("source", FglRole::Subject),
        '⊗' => ("transformation", FglRole::Verb),
        '⚘' => ("life", FglRole::Object),
        '∆' => ("change", FglRole::Modifier),
        '⟁' => ("balance", FglRole::Verb),
        '⟡' => ("light", FglRole::Object),
        '✶' => ("creation", FglRole::Verb),
        'Ϟ' => ("energy", FglRole::Subject),
        _ => return None,
    };
    Some(FglSymbol {
        symbol,
        meaning: meaning.into(),
        role,
    })
}

fn ignored(symbol: char) -> bool {
    symbol.is_whitespace() || matches!(symbol, '·' | '—' | '∎')
}

pub fn parse_fgl(source: &str) -> Result<ParsedFgl, CompileError> {
    let mut symbols = Vec::new();
    for (offset, character) in source.chars().enumerate() {
        if ignored(character) {
            continue;
        }
        symbols.push(lookup(character).ok_or(CompileError::UnknownSymbol {
            symbol: character,
            offset,
        })?);
    }
    if symbols.is_empty() {
        return Err(CompileError::Empty);
    }
    Ok(ParsedFgl {
        source: source.into(),
        symbols,
    })
}

fn primitive_for(symbol: char, scale: f64) -> Option<Primitive> {
    let (id, center, radius) = match symbol {
        '☉' => ("source-anchor", Vec3::ZERO, 1.0),
        'Ϟ' => (
            "energy-node",
            Vec3 {
                x: -1.4,
                y: 0.0,
                z: 0.0,
            },
            0.5,
        ),
        '⚘' => (
            "life-node",
            Vec3 {
                x: 1.4,
                y: 0.0,
                z: 0.0,
            },
            0.65,
        ),
        '⟡' => (
            "light-node",
            Vec3 {
                x: 1.4,
                y: 0.9,
                z: 0.0,
            },
            0.3,
        ),
        _ => return None,
    };
    Some(Primitive {
        id: id.into(),
        kind: PrimitiveKind::Sphere,
        source_symbol: Some(symbol.to_string()),
        center,
        radius: Some(radius * scale),
        half_extents: None,
    })
}

pub fn compile_fgl(source: &str, scene_id: &str) -> Result<Compilation, CompileError> {
    let parsed = parse_fgl(source)?;
    let subject = parsed
        .symbols
        .iter()
        .find(|item| item.role == FglRole::Subject)
        .ok_or(CompileError::MissingSubject)?;
    let object = parsed
        .symbols
        .iter()
        .rev()
        .find(|item| item.role == FglRole::Object)
        .ok_or(CompileError::MissingObject)?;
    let object_scale = if parsed.symbols.iter().any(|item| item.symbol == '∆') {
        1.25
    } else {
        1.0
    };
    let subject_primitive = primitive_for(subject.symbol, 1.0)
        .ok_or(CompileError::UnsupportedSubject(subject.symbol))?;
    let object_primitive = primitive_for(object.symbol, object_scale)
        .ok_or(CompileError::UnsupportedObject(object.symbol))?;
    let scene = Scene {
        schema: SCENE_SCHEMA.into(),
        scene_id: scene_id.into(),
        revision: 1,
        composition: Composition::Union,
        primitives: vec![subject_primitive, object_primitive],
        camera: Camera::default(),
        provenance: Provenance {
            compiler: COMPILER.into(),
            mapping_policy: POLICY.into(),
            source_digest: source_digest(source),
        },
    };
    let validation = validate_scene(&scene);
    let scene_digest =
        digest_hex(&scene).map_err(|error| CompileError::Contract(error.to_string()))?;
    Ok(Compilation {
        source: parsed,
        scene,
        validation,
        scene_digest,
    })
}

pub fn execute_tdm(compilation: &Compilation, base_scene_digest: &str) -> TdmRun {
    let accepted = accepted(&compilation.validation);
    TdmRun {
        schema: TDM_RUN_SCHEMA.into(),
        run_id: format!("run-{}", &compilation.scene_digest[..12]),
        base_scene_digest: base_scene_digest.into(),
        candidate_scene_digest: compilation.scene_digest.clone(),
        accepted,
        validation: compilation.validation.clone(),
        events: vec![
            TdmEvent {
                tick: 0,
                phase: TdmPhase::Anchor,
                detail: format!("Loaded base scene revision {base_scene_digest}."),
            },
            TdmEvent {
                tick: 1,
                phase: TdmPhase::Insert,
                detail: format!("Staged candidate scene {}.", compilation.scene_digest),
            },
            TdmEvent {
                tick: 2,
                phase: TdmPhase::Expand,
                detail: if accepted {
                    "Validation passed; candidate is eligible for commit.".into()
                } else {
                    "Validation failed; candidate must be rolled back.".into()
                },
            },
        ],
    }
}

fn sphere_distance(point: Vec3, center: Vec3, radius: f64) -> f64 {
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    let dz = point.z - center.z;
    (dx * dx + dy * dy + dz * dz).sqrt() - radius
}

fn box_distance(point: Vec3, center: Vec3, half: Vec3) -> f64 {
    let qx = (point.x - center.x).abs() - half.x;
    let qy = (point.y - center.y).abs() - half.y;
    let qz = (point.z - center.z).abs() - half.z;
    let outside = Vec3 {
        x: qx.max(0.0),
        y: qy.max(0.0),
        z: qz.max(0.0),
    };
    let outside_length =
        (outside.x * outside.x + outside.y * outside.y + outside.z * outside.z).sqrt();
    outside_length + qx.max(qy.max(qz)).min(0.0)
}

pub fn sample_scene(scene: &Scene, point: Vec3) -> ProbeResult {
    let mut closest: Option<(f64, String)> = None;
    for primitive in &scene.primitives {
        let distance = match primitive.kind {
            PrimitiveKind::Sphere => primitive
                .radius
                .map(|radius| sphere_distance(point, primitive.center, radius)),
            PrimitiveKind::Box => primitive
                .half_extents
                .map(|half| box_distance(point, primitive.center, half)),
            PrimitiveKind::Plane => Some(point.y - primitive.center.y),
        };
        if let Some(distance) = distance {
            if closest
                .as_ref()
                .map(|(best, _)| distance < *best)
                .unwrap_or(true)
            {
                closest = Some((distance, primitive.id.clone()));
            }
        }
    }
    let (distance, primitive_id) = closest
        .map(|(distance, id)| (distance, Some(id)))
        .unwrap_or((f64::INFINITY, None));
    ProbeResult {
        point,
        distance,
        primitive_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_fixture_compiles_and_is_accepted() {
        let compilation = compile_fgl("☉⊗⚘∎", "life-seed").unwrap();
        assert_eq!(compilation.scene.primitives.len(), 2);
        assert!(accepted(&compilation.validation));
        let run = execute_tdm(&compilation, "uncommitted");
        assert!(run.accepted);
        assert_eq!(run.events.len(), 3);
    }

    #[test]
    fn modifier_scales_the_object() {
        let compilation = compile_fgl("☉⊗∆⚘", "scaled-life").unwrap();
        assert_eq!(compilation.scene.primitives[1].radius, Some(0.8125));
    }

    #[test]
    fn unknown_source_is_rejected() {
        assert!(matches!(
            parse_fgl("☉@⚘"),
            Err(CompileError::UnknownSymbol { .. })
        ));
    }

    #[test]
    fn probe_finds_anchor_surface() {
        let compilation = compile_fgl("☉⊗⚘", "life-seed").unwrap();
        let sample = sample_scene(
            &compilation.scene,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        );
        assert_eq!(sample.primitive_id.as_deref(), Some("source-anchor"));
        assert_eq!(sample.distance, -1.0);
    }
}
