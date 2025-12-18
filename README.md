# GodView MVP - The Live Reality Protocol

**Distributed X-Ray Vision System for Industrial Safety**

GodView is a real-time hazard detection and visualization system that decouples sight from location. Stationary sensors detect hazards and transmit semantic 3D data (not raw video) to mobile agents who can "see through walls" with <50ms latency.

## 🎯 Core Innovation

Instead of streaming video, GodView transmits **3D semantic coordinates** of detected hazards. A webcam detects faces, converts them to 3D positions using projection math, and publishes them via Zenoh. The web viewer renders these as "Red Ghost" avatars in a 3D space.

## 🏗️ Architecture

```
┌─────────────────┐         ┌──────────────┐         ┌─────────────────┐
│  Rust Agent     │         │    Zenoh     │         │   Web Viewer    │
│  (X-Ray Emit)   │────────▶│   Router     │────────▶│  (God View)     │
│                 │         │   (v1.0)     │         │                 │
│ • OpenCV        │         │              │         │ • Three.js      │
│ • Face Detect   │         │ WS:8000      │         │ • Zenoh-TS      │
│ • 3D Projection │         │ TCP:7447     │         │ • Red Ghost     │
└─────────────────┘         └──────────────┘         └─────────────────┘
```

## 📦 Technology Stack

- **Backend**: Rust 2021 Edition
- **Vision**: OpenCV 0.92 (Haar Cascade face detection)
- **Middleware**: Eclipse Zenoh 1.0.0 (Protocol v1)
- **Frontend**: Vanilla JavaScript + Vite + Three.js
- **OS**: Ubuntu 22.04

## 🚀 Quick Start

### Prerequisites

1. **Install Zenoh Router**:
   ```bash
   # Download and install zenohd
   curl -L https://github.com/eclipse-zenoh/zenoh/releases/download/1.0.0/zenohd-1.0.0-x86_64-unknown-linux-gnu.zip -o zenohd.zip
   unzip zenohd.zip
   sudo mv zenohd /usr/local/bin/
   ```

2. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Install Node.js 18+** (if not already installed):
   ```bash
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt-get install -y nodejs
   ```

4. **Install OpenCV** (system dependency):
   ```bash
   sudo apt-get update
   sudo apt-get install -y libopencv-dev clang libclang-dev
   ```

### Launch GodView

```bash
cd ~/godview
./run_godview.sh
```

This will:
1. Start Zenoh router on ports 7447 (TCP) and 8000 (WebSocket)
2. Compile and run the Rust agent (face detection + publishing)
3. Install dependencies and start the Vite dev server

### Access the Viewer

Open your browser to the URL shown in the terminal (typically `http://localhost:5173`).

Position yourself in front of your webcam. You should see a **red sphere** appear in the 3D view, tracking your face position in real-time.

## 📁 Project Structure

```
godview/
├── agent/                          # Rust backend
│   ├── Cargo.toml                  # Dependencies (Zenoh 1.0, OpenCV)
│   ├── src/
│   │   └── main.rs                 # Face detection + 3D projection
│   └── haarcascade_frontalface_alt.xml  # Haar Cascade model
│
├── viewer/                         # Web frontend
│   ├── package.json                # Dependencies (Three.js, Zenoh-TS)
│   ├── index.html                  # Cyberpunk HUD interface
│   └── src/
│       └── main.js                 # 3D scene + Zenoh subscriber
│
└── run_godview.sh                  # Orchestration script
```

## 🔬 How It Works

### 1. Face Detection (Rust Agent)

The agent captures webcam frames at 30 FPS and uses OpenCV's Haar Cascade classifier to detect faces:

```rust
// Detect faces in grayscale frame
face_cascade.detect_multi_scale(&gray, &mut faces, ...)?;
```

### 2. 3D Projection Math

For each detected face, we calculate real-world 3D coordinates:

```rust
// Z (Depth) = (Focal Length × Real Face Width) / Pixel Width
let z = (FOCAL_LENGTH_CONST * REAL_FACE_WIDTH_M) / face_width_px;

// X (Lateral) = (Pixel X - Center X) × Z / Focal Length
let x = (face_x - center_x) * z / FOCAL_LENGTH_CONST;
```

### 3. Zenoh Publishing

The 3D coordinates are serialized to JSON and published:

```rust
let packet = HazardPacket {
    id: "hazard_1",
    timestamp: 1234567890,
    pos: [x, 0.0, z],
    hazard_type: "human_face"
};

session.put("godview/zone1/hazards", json_payload).await?;
```

### 4. Web Visualization

The viewer subscribes to the Zenoh topic and renders a red sphere:

```javascript
// Subscribe to hazard data
await session.declareSubscriber('godview/zone1/hazards', {
  callback: (sample) => {
    const data = JSON.parse(sample.payload.deserialize());
    targetPosition.set(data.pos[0], data.pos[1], data.pos[2]);
    ghostMesh.visible = true;
  }
});

// Smooth interpolation in animation loop
ghostMesh.position.lerp(targetPosition, 0.1);
```

## 🎨 Visual Design

The viewer features a **cyberpunk "God View" aesthetic**:
- Neon green HUD with pulsing status indicators
- Dark fog atmosphere with grid floor
- Wireframe room boundaries
- Red "Ghost" avatar with glow effect
- Real-time latency display

## ⚡ Performance

- **Latency**: <50ms end-to-end (detection → visualization)
- **Frame Rate**: 30 FPS (agent) / 60 FPS (viewer)
- **Network**: Zenoh peer-to-peer (no broker overhead)

## 🔧 Configuration

### Adjust Camera Position

Edit `viewer/src/main.js`:
```javascript
camera.position.set(2, 2, 3);  // [X, Y, Z]
```

### Change Detection Sensitivity

Edit `agent/src/main.rs`:
```rust
face_cascade.detect_multi_scale(
    &gray,
    &mut faces,
    1.1,  // Scale factor (lower = more sensitive)
    3,    // Min neighbors (higher = fewer false positives)
    ...
)?;
```

### Modify Zenoh Topic

Both files use `godview/zone1/hazards`. Change this key in:
- `agent/src/main.rs` (line ~40)
- `viewer/src/main.js` (line ~120)

## 🐛 Troubleshooting

### "Failed to open webcam"
- Ensure `/dev/video0` exists: `ls -l /dev/video0`
- Check permissions: `sudo usermod -a -G video $USER` (logout/login required)

### "Zenoh connection failed"
- Verify `zenohd` is running: `ps aux | grep zenohd`
- Check port 8000 is available: `netstat -tuln | grep 8000`

### "Haar Cascade not found"
- The model should be downloaded automatically to `agent/haarcascade_frontalface_alt.xml`
- If missing, run: `cd agent && wget https://raw.githubusercontent.com/opencv/opencv/4.x/data/haarcascades/haarcascade_frontalface_alt.xml`

### No red sphere appears
- Check browser console for Zenoh connection errors
- Verify Rust agent is publishing: look for `[X-RAY EMITTER] Sent Hazard` logs
- Ensure you're visible to the webcam (good lighting helps)

## 📝 License

MIT License - Built for Project GodView MVP

## 🚀 Next Steps

This MVP demonstrates the core concept. Future enhancements:
- Multiple hazard types (not just faces)
- Multi-camera support
- Gaussian Splat rendering for photorealistic ghosts
- Mobile AR viewer (iPad/iPhone)
- Industrial sensor integration (thermal, LiDAR)

---

**The Live Reality Protocol** - Seeing through walls, one hazard at a time. 👁️
