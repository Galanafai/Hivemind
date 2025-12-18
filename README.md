# 👁️ GodView - The Live Reality Protocol

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Three.js](https://img.shields.io/badge/Three.js-0.160-blue.svg)](https://threejs.org/)
[![Zenoh](https://img.shields.io/badge/Zenoh-1.0-green.svg)](https://zenoh.io/)

**Distributed X-Ray Vision System for Industrial Safety**

GodView is a real-time hazard detection and visualization system that decouples sight from location. Instead of streaming video, it transmits **semantic 3D coordinates** of detected objects, creating a lightweight, privacy-preserving safety monitoring system with <50ms latency.

![GodView Demo](https://via.placeholder.com/800x400/000000/00ff00?text=GodView+Demo+%28Add+Screenshot%29)

---

## 🎯 The Core Innovation

Traditional video surveillance:
- 📹 Streams raw pixels (high bandwidth)
- 💾 Requires video storage (privacy concerns)
- 🐌 High latency due to encoding/decoding

**GodView approach:**
- 📊 Transmits 3D positions only (1-2 KB/s)
- 🔒 No video recording (privacy-first)
- ⚡ <50ms end-to-end latency

Think of it like **air traffic control radar** for industrial hazards.

---

## ✨ Features

- ✅ **Real-time Detection**: OpenCV face detection at 30 FPS
- ✅ **3D Projection**: Converts 2D detections to real-world 3D coordinates
- ✅ **Multi-Agent Support**: Track unlimited simultaneous hazards
- ✅ **Smooth Rendering**: LERP interpolation for 60 FPS visualization
- ✅ **Auto Cleanup**: Ghosts fade after 2 seconds of no updates
- ✅ **Privacy-First**: No video storage, only anonymous 3D positions
- ✅ **Low Latency**: <50ms end-to-end via Zenoh peer-to-peer
- ✅ **Cyberpunk UI**: Neon green HUD with real-time status

---

## 🏗️ Architecture

```
┌──────────────────┐         ┌──────────────┐         ┌──────────────────┐
│   RUST AGENT     │         │    ZENOH     │         │   WEB VIEWER     │
│  (X-Ray Emit)    │────────▶│   ROUTER     │────────▶│   (God View)     │
│                  │         │   (v1.0)     │         │                  │
│ • OpenCV         │         │ WS:8000      │         │ • Three.js       │
│ • Face Detect    │         │ TCP:7447     │         │ • Zenoh-TS       │
│ • 3D Projection  │         │              │         │ • Red Ghost      │
└──────────────────┘         └──────────────┘         └──────────────────┘
```

**Data Flow:**
```
Webcam → OpenCV → 3D Math → JSON → Zenoh → Browser → Three.js → Ghost
```

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+
- **Node.js** 18+
- **Zenoh Router** 1.0+
- **OpenCV** 4.x
- **Webcam** (any USB camera)

### Installation

```bash
# Clone repository
git clone https://github.com/Galanafai/Hivemind.git
cd Hivemind

# Install all dependencies (automated)
./install_dependencies.sh

# Restart terminal or source Rust environment
source ~/.cargo/env

# Verify installation
./check_requirements.sh
```

### Launch

```bash
# Start all components
./run_godview.sh

# Open browser to http://localhost:5173
```

Position yourself in front of the webcam. You should see a **red sphere** appear in the 3D view, tracking your face in real-time!

---

## 📊 Tech Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Backend** | Rust 2021 | Face detection + 3D projection |
| **Vision** | OpenCV 0.92 | Haar Cascade face detection |
| **Middleware** | Zenoh 1.0 | Pub/sub messaging (<10ms latency) |
| **Frontend** | Three.js 0.160 | 3D rendering engine |
| **Network** | Zenoh-TS 1.0 | WebSocket client |
| **Dev Server** | Vite 5.0 | Lightning-fast HMR |

---

## 🎨 How It Works

### 1. Face Detection (Rust)

```rust
// Detect faces using Haar Cascade
face_cascade.detect_multi_scale(&gray, &mut faces, 1.1, 3, ...)?;
```

### 2. 3D Projection Math

```rust
// Convert 2D bounding box to 3D world coordinates
let z = (FOCAL_LENGTH_CONST * REAL_FACE_WIDTH_M) / face_width_px;
let x = (face_x - center_x) * z / FOCAL_LENGTH_CONST;
```

### 3. Zenoh Publishing

```rust
// Publish semantic 3D data
let packet = HazardPacket {
    id: "hazard_42",
    timestamp: 1702934400000,
    pos: [x, y, z],
    hazard_type: "human_face"
};
session.put("godview/zone1/hazards", json_payload).await?;
```

### 4. Web Visualization

```javascript
// Spawn red ghost at 3D position
const ghost = createGhost();
ghost.position.set(data.pos[0], data.pos[1], data.pos[2]);
scene.add(ghost);

// Smooth interpolation
ghost.position.lerp(targetPosition, 0.1);
```

---

## 📖 Documentation

- **[README.md](README.md)** - Quick start guide
- **[TECHNICAL_DOCUMENTATION.md](TECHNICAL_DOCUMENTATION.md)** - Deep dive (32 KB)
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Cheat sheet
- **[MULTI_AGENT_UPGRADE.md](MULTI_AGENT_UPGRADE.md)** - Multi-agent system
- **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)** - Navigation guide

**Total:** 58.7 KB of documentation, ~2100 lines

---

## 🎯 Use Cases

### Current: Face Detection Demo
Developer testing the core technology

### Future: Industrial Safety

**Warehouse:**
- 10 cameras at intersections
- Detect forklifts + people
- AR glasses for operators
- Prevent collisions

**Construction:**
- 50 cameras covering site
- Detect workers, vehicles, falling objects
- Control room monitoring
- Real-time safety alerts

**Elderly Care:**
- 3 cameras in home
- Detect falls, unusual movement
- Remote monitoring via app
- Privacy-preserving (no video)

**Retail Analytics:**
- 20 cameras in store
- Anonymized customer tracking
- Heatmap visualization
- Data-driven layout optimization

---

## ⚡ Performance

| Metric | Value |
|--------|-------|
| **End-to-End Latency** | <50ms |
| **Agent FPS** | 30 |
| **Viewer FPS** | 60 |
| **CPU Usage** | 8-12% (agent), 3-5% (viewer) |
| **Bandwidth** | 1-2 KB/s |
| **Max Hazards** | 50+ simultaneous |

---

## 🚀 Future Enhancements

- [ ] Multiple hazard types (forklifts, spills, fire)
- [ ] Persistent object tracking (reduce ID churn)
- [ ] Mobile AR viewer (iPad/iPhone)
- [ ] Gaussian Splat rendering (photorealistic ghosts)
- [ ] Multi-camera fusion (accurate 3D triangulation)
- [ ] Alert system (audio/haptic warnings)
- [ ] Historical playback (incident investigation)
- [ ] Color-coded hazards by type
- [ ] Ghost labels with IDs
- [ ] Configurable danger zones

---

## 🧪 Testing

```bash
# Test 1: Single hazard
# Sit in front of webcam → Ghost appears → Move → Ghost follows

# Test 2: Multiple hazards
# Show photo of face + your face → 2 ghosts appear

# Test 3: Timeout
# Leave frame → Ghost fades after 2 seconds
```

---

## 🔧 Configuration

### Change Ghost Color

```javascript
// viewer/src/main.js
const ghostMaterial = new THREE.MeshBasicMaterial({
    color: 0xff8800,  // Orange instead of red
    // ...
});
```

### Adjust Detection Sensitivity

```rust
// agent/src/main.rs
face_cascade.detect_multi_scale(
    &gray,
    &mut faces,
    1.1,  // Lower = more sensitive
    3,    // Higher = fewer false positives
    // ...
)?;
```

### Modify Timeout

```javascript
// viewer/src/main.js
const GHOST_TIMEOUT = 5000;  // 5 seconds instead of 2
```

---

## 🐛 Troubleshooting

### "Command 'cargo' not found"
```bash
./install_dependencies.sh
source ~/.cargo/env
```

### "Failed to open webcam"
```bash
ls -l /dev/video0
sudo usermod -a -G video $USER
# Logout and login
```

### "Zenoh connection failed"
```bash
ps aux | grep zenohd
netstat -tuln | grep 8000
./run_godview.sh
```

See [QUICK_REFERENCE.md](QUICK_REFERENCE.md) for more troubleshooting.

---

## 📁 Project Structure

```
godview/
├── agent/                              # Rust backend
│   ├── Cargo.toml
│   ├── src/main.rs
│   └── haarcascade_frontalface_alt.xml
├── viewer/                             # Web frontend
│   ├── package.json
│   ├── index.html
│   └── src/main.js
├── run_godview.sh                      # Main launcher
├── check_requirements.sh               # Dependency checker
├── install_dependencies.sh             # Auto-installer
└── *.md                                # Documentation
```

---

## 🤝 Contributing

Contributions welcome! Please read [TECHNICAL_DOCUMENTATION.md](TECHNICAL_DOCUMENTATION.md) first.

**Areas for contribution:**
- Additional hazard detection (YOLO integration)
- Mobile AR viewer (WebXR)
- Multi-camera fusion
- Alert system
- Historical playback
- Performance optimizations

---

## 📄 License

MIT License - See [LICENSE](LICENSE) for details

---

## 🙏 Acknowledgments

- **Eclipse Zenoh** - For the amazing pub/sub middleware
- **Three.js** - For making 3D in the browser easy
- **OpenCV** - For reliable computer vision
- **Rust Community** - For excellent async tooling

---

## 📞 Contact

- **GitHub:** [@Galanafai](https://github.com/Galanafai)
- **Project:** [Hivemind](https://github.com/Galanafai/Hivemind)

---

## ⭐ Star History

If you find this project useful, please consider giving it a star!

---

**The Live Reality Protocol** - Seeing through walls, one hazard at a time. 👁️

*Built with ❤️ for industrial safety*
