# 👁️ GodView - Distributed Gaussian Perception Protocol

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Rerun](https://img.shields.io/badge/Rerun-0.28-blue.svg)](https://rerun.io/)

**A Research Framework for Multi-Agent Sensor Fusion in Autonomous Systems**

> *"Solving the fundamental problems of distributed spatial computing: time synchronization, 3D spatial indexing, and Byzantine fault tolerance."*

---

## 🎯 Research Problem

Distributed sensor networks face three critical challenges:

| Problem | Traditional Approach | GodView Solution |
|---------|---------------------|------------------|
| **Out-of-Sequence Measurements** | Drop or FIFO queue | Augmented State EKF with retrodiction |
| **Vertical Aliasing** | 2D Geohash | H3 + Sparse Voxel Octrees |
| **Sybil Attacks** | Trust all publishers | CapBAC + Ed25519 signatures |

---

## 📐 Core Algorithms

### 1. Augmented State Extended Kalman Filter (AS-EKF)

Handles network latency (100-500ms) by maintaining a rolling window of past states:

```
State: x̂_aug = [x̂_k, x̂_{k-1}, ..., x̂_{k-L}]
Covariance: P_aug = [P_{k,k}, P_{k,k-1}, ...; P_{k-1,k}, P_{k-1,k-1}, ...]
```

**Key Innovation:** O(1) OOSM updates via cross-correlation matrices.

📄 **Implementation:** [`godview_core/src/godview_time.rs`](godview_core/src/godview_time.rs)

### 2. Covariance Intersection (CI)

Fuses observations from multiple agents without requiring cross-correlation knowledge:

```
P_fused⁻¹ = ω₁P₁⁻¹ + ω₂P₂⁻¹ + ... + ωₙPₙ⁻¹
x̂_fused = P_fused(ω₁P₁⁻¹x̂₁ + ω₂P₂⁻¹x̂₂ + ... + ωₙPₙ⁻¹x̂ₙ)
```

**Key Innovation:** Conservative fusion that never underestimates uncertainty.

📄 **Implementation:** [`godview_core/src/godview_tracking.rs`](godview_core/src/godview_tracking.rs)

### 3. Highlander CRDT Track Merging

*"There can be only one."* Deterministic merge of duplicate tracks:

```
Canonical ID = min(track_id₁, track_id₂, ..., track_idₙ)
```

📄 **Implementation:** [`godview_core/src/godview_tracking.rs`](godview_core/src/godview_tracking.rs)

---

## 🗂️ Repository Structure

```
godview/
├── godview_core/          # Rust library - core algorithms
│   ├── src/
│   │   ├── godview_time.rs      # AS-EKF implementation
│   │   ├── godview_space.rs     # H3 + Octree spatial indexing
│   │   ├── godview_trust.rs     # CapBAC + Ed25519 security
│   │   ├── godview_tracking.rs  # CI fusion + Highlander CRDT
│   │   └── visualization.rs     # Rerun.io integration
│   └── examples/
│       ├── rerun_demo.rs        # Synthetic multi-agent demo
│       └── kitti_demo.rs        # Real KITTI dataset demo
├── Dockerfile             # GPU-enabled container
├── docker-compose.yml     # Full stack orchestration
└── docs/                  # Additional documentation
```

---

## 🚀 Quick Start (Docker)

### Prerequisites
- Docker with NVIDIA Container Toolkit
- GPU with CUDA support

### Run Demo

```bash
# Clone and enter
git clone https://github.com/Galanafai/Hivemind.git
cd Hivemind

# Build Docker image
sudo docker build -t godview:latest .

# Run synthetic multi-agent demo
sudo docker run --rm --gpus all \
  -v $(pwd):/workspace \
  -w /workspace/godview_core \
  godview:latest \
  cargo run --example kitti_demo --features visualization,kitti -- \
  --save /workspace/godview_demo.rrd

# View in Rerun
rerun godview_demo.rrd
```

### Run with Real nuScenes Data

```bash
# Download nuScenes mini (3.9GB) and run
sudo docker run --rm --gpus all \
  -v $(pwd):/workspace \
  --network host \
  godview:latest bash -c "
    pip install 'rerun-sdk>=0.28' --quiet &&
    mkdir -p /workspace/data/nuscenes &&
    cd /workspace/data/nuscenes &&
    wget https://www.nuscenes.org/data/v1.0-mini.tgz &&
    tar -xzf v1.0-mini.tgz &&
    python3 -m nuscenes_dataset --root-dir /workspace/data/nuscenes \
      --seconds 30 --save /workspace/nuscenes_demo.rrd
  "

# View
rerun nuscenes_demo.rrd
```

---

## 📊 Performance

| Component | Operation | Time | Notes |
|-----------|-----------|------|-------|
| AS-EKF | Prediction (9D) | ~50 µs | 20-state augmented window |
| AS-EKF | OOSM Update | ~100 µs | O(1) via cross-correlation |
| Covariance Intersection | 5 agents | ~20 µs | Matrix operations |
| H3+Octree | Query (50m) | ~50 µs | Includes altitude filtering |
| Ed25519 | Sign/Verify | 15/40 µs | Per-packet |

*Benchmarked on AMD Ryzen 9 5950X*

---

## 📚 References

### Academic Papers

1. **Augmented State EKF:** Challa, S., & Bar-Shalom, Y. (2000). "OOSM Problem in Tracking"
2. **Covariance Intersection:** Julier, S., & Uhlmann, J. (1997). "A Non-divergent Estimation Algorithm in the Presence of Unknown Correlations"
3. **H3 Geospatial Indexing:** Uber Engineering. "H3: Uber's Hexagonal Hierarchical Spatial Index"
4. **Biscuit Tokens:** CleverCloud. "Biscuit: Datalog-Based Authorization"

### Key Dependencies

- [`nalgebra`](https://nalgebra.org/) - Linear algebra
- [`h3o`](https://docs.rs/h3o) - H3 geospatial indexing
- [`biscuit-auth`](https://docs.rs/biscuit-auth) - Capability-based access control
- [`rerun`](https://rerun.io/) - Visualization

---

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [`godview_core/README.md`](godview_core/README.md) | **Core Library.** API reference and algorithm details |
| [`TECHNICAL_DOCUMENTATION.md`](TECHNICAL_DOCUMENTATION.md) | **Deep Dive.** Mathematical foundations |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | **System Design.** Module interactions |

---

## 🤝 Contributing

1. Read the core library documentation
2. Run `cargo test` in `godview_core/`
3. Follow Rust API guidelines
4. Submit PR with tests

**License:** MIT

---

**Built for research in distributed perception and autonomous systems**
