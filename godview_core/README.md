# GodView Core - Research Library

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**High-Precision Distributed Spatial Computing Protocol for Autonomous Systems**

> A production-grade Rust library implementing state-of-the-art algorithms for multi-agent sensor fusion, spatial indexing, and Byzantine fault tolerance.

---

## 📚 Table of Contents

1. [Research Problem](#-research-problem)
2. [Mathematical Foundations](#-mathematical-foundations)
3. [Algorithm Logic](#-algorithm-logic)
4. [Module Reference](#-module-reference)
5. [Visualization](#-visualization)
6. [Performance](#-performance)
7. [References](#-references)

---

## 🎯 Research Problem

Distributed sensor networks in autonomous systems face three critical challenges:

| Problem | Symptom | Impact |
|---------|---------|--------|
| **Out-of-Sequence Measurements (OOSM)** | 100-500ms network latency | Corrupted trajectories, planning errors |
| **Vertical Aliasing** | 2D spatial indices | Drones at 300m collide with cars at 0m |
| **Sybil Attacks** | Unauthenticated publishers | Phantom hazards, denial of service |

GodView Core solves these with three synergistic engines:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   TIME ENGINE   │    │   SPACE ENGINE  │    │   TRUST ENGINE  │
│   (AS-EKF)      │    │   (H3+Octree)   │    │   (CapBAC)      │
│                 │    │                 │    │                 │
│ • Retrodiction  │    │ • 3D Indexing   │    │ • Ed25519 Sigs  │
│ • O(1) OOSM     │    │ • Altitude      │    │ • Biscuit Auth  │
│ • 20-state lag  │    │ • H3 Resolution │    │ • Datalog Rules │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                     │                      │
         └─────────────────────┴──────────────────────┘
                              │
                    ┌─────────────────┐
                    │ TRACKING ENGINE │
                    │ (CI + Highlander)│
                    │                 │
                    │ • Mahalanobis   │
                    │ • Cov. Intersect│
                    │ • CRDT Merge    │
                    └─────────────────┘
```

---

## 📐 Mathematical Foundations

### 1. Augmented State Extended Kalman Filter (AS-EKF)

**Purpose:** Handle network latency (100-500ms) without corrupting the world model.

**Core Insight:** Maintain a rolling window of L past states to enable O(1) out-of-sequence measurement updates.

#### State Representation

```
Augmented State:   x̂_aug = [x̂_k, x̂_{k-1}, ..., x̂_{k-L}]ᵀ    (dimension: n × L)

Augmented Covariance:
                   ┌ P_{k,k}    P_{k,k-1}   ... P_{k,k-L}  ┐
         P_aug =   │ P_{k-1,k}  P_{k-1,k-1} ... P_{k-1,k-L}│
                   │ ...        ...         ... ...        │
                   └ P_{k-L,k}  P_{k-L,k-1} ... P_{k-L,k-L}┘
```

#### Prediction Step

```
F = State Transition Matrix (constant velocity/acceleration model)

F_aug = ┌ F   0   0   ... 0 ┐
        │ I   0   0   ... 0 │
        │ 0   I   0   ... 0 │
        │ ... ... ... ... . │
        └ 0   0   0   ... I ┘

x̂_aug(k+1|k) = F_aug · x̂_aug(k|k)
P_aug(k+1|k) = F_aug · P_aug(k|k) · F_augᵀ + Q_aug
```

#### OOSM Update (Out-of-Sequence Measurement)

When a measurement z arrives with timestamp τ (lag index j):

```
1. Build measurement matrix H_aug with non-zero block at lag position j
2. Compute innovation: ν = z - H_aug · x̂_aug
3. Compute innovation covariance: S = H_aug · P_aug · H_augᵀ + R
4. Kalman Gain: K = P_aug · H_augᵀ · S⁻¹
5. Update: x̂_aug ← x̂_aug + K · ν
6. Joseph-form covariance: P_aug ← (I - K·H_aug) · P_aug · (I - K·H_aug)ᵀ + K·R·Kᵀ
```

📄 **Implementation:** [`src/godview_time.rs`](src/godview_time.rs) lines 90-180

---

### 2. Covariance Intersection (CI)

**Purpose:** Fuse observations from multiple agents without knowing cross-correlations.

**Problem it solves:** In peer-to-peer networks, Agent A's data may loop back through B → C → A. Naive Kalman fusion would over-trust this "echoed" data, collapsing covariance toward zero (false confidence).

#### Mathematical Formulation

Given two estimates (x̂₁, P₁) and (x̂₂, P₂) with unknown correlation:

```
Find ω ∈ [0,1] that minimizes trace(P_fused):

P_fused⁻¹ = ω·P₁⁻¹ + (1-ω)·P₂⁻¹
x̂_fused = P_fused · (ω·P₁⁻¹·x̂₁ + (1-ω)·P₂⁻¹·x̂₂)
```

#### Key Properties

1. **Conservative:** P_fused is never smaller than warranted, even with full correlation
2. **Loop-safe:** If P₁ = P₂ (identical data looped), ω=0.5 yields P_fused = P₁ (no artificial shrinking)
3. **Multi-source:** Extends to N sources via iterative pairwise application

📄 **Implementation:** [`src/godview_tracking.rs`](src/godview_tracking.rs) lines 150-220

---

### 3. Highlander CRDT Track Merging

**Purpose:** Deterministic distributed track identity resolution.

**Problem:** In a mesh network, the same physical object may be assigned different UUIDs by different agents. Need to converge to a single canonical ID without central coordination.

#### Algorithm

```
Rule: "There can be only one"

For each track, maintain: aliases = Set<UUID>

On receiving track with ID_remote:
    if spatial_match(local_track, remote_track):
        aliases.insert(ID_remote)
        canonical_id = min(aliases)  // Lexicographically smallest
```

#### CRDT Properties

1. **Commutative:** A merging with B = B merging with A
2. **Associative:** (A ⊕ B) ⊕ C = A ⊕ (B ⊕ C)
3. **Idempotent:** A ⊕ A = A
4. **Monotonic:** IDs only ever decrease (toward minimum)

📄 **Implementation:** [`src/godview_tracking.rs`](src/godview_tracking.rs) lines 280-350

---

### 4. Mahalanobis Distance Gating

**Purpose:** Scale-invariant distance for data association.

```
d_M = √[(z - Hx̂)ᵀ · S⁻¹ · (z - Hx̂)]

where S = H·P·Hᵀ + R (innovation covariance)
```

A measurement associates to a track if d_M < threshold (typically χ² with 3 DOF ≈ 7.81 for 95% confidence).

📄 **Implementation:** [`src/godview_tracking.rs`](src/godview_tracking.rs) lines 100-130

---

## 🔧 Algorithm Logic

### Track Processing Pipeline

```
┌──────────────────────────────────────────────────────────────┐
│                    INCOMING OBSERVATION                       │
│                    (position, covariance, timestamp)          │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│  1. TIMESTAMP VALIDATION                                      │
│     if |timestamp - now| > max_latency: REJECT                │
│     else: lag_index = (now - timestamp) / dt                  │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│  2. SPATIAL QUERY                                             │
│     candidates = H3_octree.query_radius(position, gate_dist)  │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│  3. DATA ASSOCIATION (Mahalanobis Gating)                     │
│     for each candidate:                                       │
│         d = mahalanobis_distance(obs, candidate)              │
│         if d < threshold: best_match = argmin(d)              │
│     if no match: CREATE_NEW_TRACK                             │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│  4. FUSION                                                    │
│     if same_agent: AS-EKF.update(obs, lag_index)              │
│     if different_agent: Covariance_Intersection(local, obs)   │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│  5. IDENTITY RESOLUTION (Highlander CRDT)                     │
│     track.aliases.insert(obs.id)                              │
│     track.id = min(track.aliases)                             │
└──────────────────────────────────────────────────────────────┘
```

### Pseudocode: Core Fusion Loop

```rust
fn process_observation(obs: Observation, tracks: &mut HashMap<Uuid, Track>) {
    // 1. Time validation
    let lag_idx = compute_lag_index(obs.timestamp);
    if lag_idx > MAX_LAG { return; }
    
    // 2. Find candidates via spatial index
    let candidates = spatial_index.query_radius(obs.position, GATE_RADIUS);
    
    // 3. Mahalanobis gating
    let mut best_match: Option<(Uuid, f64)> = None;
    for track_id in candidates {
        let track = &tracks[track_id];
        let predicted_pos = track.filter.predict_at(obs.timestamp);
        let d = mahalanobis_distance(&obs, &predicted_pos, &track.filter.covariance);
        
        if d < CHI2_THRESHOLD_95 {
            if best_match.is_none() || d < best_match.unwrap().1 {
                best_match = Some((track_id, d));
            }
        }
    }
    
    // 4. Associate or create
    match best_match {
        Some((track_id, _)) => {
            let track = tracks.get_mut(&track_id).unwrap();
            
            if obs.source_agent == local_agent_id {
                // Own observation: AS-EKF update
                track.filter.update_oosm(obs.measurement, lag_idx);
            } else {
                // Remote observation: Covariance Intersection
                track.state = covariance_intersection(
                    track.state, track.covariance,
                    obs.state, obs.covariance
                );
            }
            
            // 5. Highlander merge
            track.aliases.insert(obs.id);
            track.id = track.aliases.iter().min().unwrap().clone();
        }
        None => {
            // Create new track
            tracks.insert(obs.id, Track::new(obs));
        }
    }
}
```

---

## 📦 Module Reference

### [`godview_time`](src/godview_time.rs) - Time Engine

```rust
use godview_core::AugmentedStateFilter;

let mut filter = AugmentedStateFilter::new(
    initial_state,    // DVector<f64>
    initial_cov,      // DMatrix<f64>
    process_noise,    // DMatrix<f64>
    measurement_noise,// DMatrix<f64>
    lag_buffer_size,  // usize (e.g., 20)
);

filter.predict(dt, current_time);               // Prediction step
filter.update_oosm(measurement, lag_index);     // OOSM update
let state = filter.get_current_state();         // Extract current estimate
```

### [`godview_space`](src/godview_space.rs) - Space Engine

```rust
use godview_core::SpatialEngine;
use h3o::Resolution;

let mut engine = SpatialEngine::new(Resolution::Ten);  // ~66m cells

engine.update_entity(entity)?;                         // Insert/update
let results = engine.query_radius([lat, lon, alt], radius_m);  // 3D query
```

### [`godview_trust`](src/godview_trust.rs) - Trust Engine

```rust
use godview_core::{TokenFactory, SecurityContext, SignedPacket};

// Authority creates tokens
let factory = TokenFactory::new(root_keypair);
let token = factory.create_publish_token("zone_id")?;

// Agent signs data
let packet = SignedPacket::new(payload, &signing_key, Some(token));

// Verifier checks both signature AND capability
let context = SecurityContext::new(root_public_key);
context.verify_packet(&packet, "topic", "action")?;
```

### [`godview_tracking`](src/godview_tracking.rs) - Tracking Engine

```rust
use godview_core::TrackManager;

let mut manager = TrackManager::new(resolution, gating_threshold);

// Process incoming observation
manager.process_observation(obs)?;

// Get all active tracks
let tracks = manager.get_all_tracks();
```

---

## 🎨 Visualization

GodView Core includes Rerun.io integration for 3D visualization.

```rust
use godview_core::visualization::RerunVisualizer;

let viz = RerunVisualizer::new("My Demo")?;

// Scene setup
viz.log_ground_plane(100.0, 20)?;
viz.log_road([-50.0, 0.0], [50.0, 0.0], 8.0)?;

// Log track with uncertainty ellipsoid
viz.log_track(track_id, position, velocity, &covariance, "vehicle")?;

// Log colored per-agent detections
viz.log_track_colored(id, pos, vel, &cov, "Agent_A_det", [255, 0, 0, 150])?;
```

Run demos:
```bash
# Synthetic multi-agent demo
cargo run --example rerun_demo --features visualization

# KITTI autonomous driving demo
cargo run --example kitti_demo --features visualization,kitti
```

---

## ⚡ Performance

| Component | Operation | Complexity | Typical Time |
|-----------|-----------|------------|--------------|
| AS-EKF | Prediction (9D, L=20) | O(n²L²) | ~50 µs |
| AS-EKF | OOSM Update | O(n²L) | ~100 µs |
| CI | Fusion (9D) | O(n³) | ~20 µs |
| H3+Octree | Insert | O(log N) | ~10 µs |
| H3+Octree | Query 50m radius | O(log N + k) | ~50 µs |
| Ed25519 | Sign | O(1) | ~15 µs |
| Ed25519 | Verify | O(1) | ~40 µs |

*Benchmarked on AMD Ryzen 9 5950X, single-threaded*

---

## 📖 References

### Academic Papers

1. **Augmented State EKF for OOSM:**
   - Bar-Shalom, Y. (2002). "Update with out-of-sequence measurements in tracking"
   - Challa, S., et al. (2000). "OOSM Problem in Tracking: An Information Theoretic Approach"

2. **Covariance Intersection:**
   - Julier, S. & Uhlmann, J. (1997). "A Non-divergent Estimation Algorithm in the Presence of Unknown Correlations"
   - IEEE paper on conservative fusion: [arXiv:2209.01136](https://arxiv.org/abs/2209.01136)

3. **H3 Geospatial Indexing:**
   - Uber Engineering. "H3: Uber's Hexagonal Hierarchical Spatial Index"
   - [h3geo.org](https://h3geo.org/)

4. **CRDTs for Distributed Consensus:**
   - Shapiro, M., et al. (2011). "Conflict-free Replicated Data Types"

5. **Capability-Based Access Control:**
   - CleverCloud. "Biscuit: Datalog-Based Authorization"

### Crate Dependencies

| Crate | Purpose |
|-------|---------|
| [`nalgebra`](https://nalgebra.org/) | Linear algebra, SIMD-accelerated |
| [`h3o`](https://docs.rs/h3o) | Pure Rust H3 hexagonal indexing |
| [`biscuit-auth`](https://docs.rs/biscuit-auth) | Capability tokens |
| [`ed25519-dalek`](https://docs.rs/ed25519-dalek) | Ed25519 signatures |
| [`rerun`](https://rerun.io/) | 3D visualization |

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test godview_time
cargo test godview_tracking

# Generate documentation
cargo doc --open
```

---

## 📄 License

MIT License - See [LICENSE](../LICENSE)

---

**Built for research in distributed perception and autonomous systems**

*"Solving the hard problems in multi-agent sensor fusion"*
