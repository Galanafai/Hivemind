# GodView - Quick Reference Guide

## 🚀 Quick Start

```bash
# Install dependencies
./install_dependencies.sh

# Check system requirements
./check_requirements.sh

# Launch GodView
./run_godview.sh

# Open browser to http://localhost:5173
```

---

## 📁 File Structure

```
godview/
├── agent/                              # Rust backend
│   ├── Cargo.toml                      # Dependencies
│   ├── src/main.rs                     # Detection + Publishing
│   └── haarcascade_frontalface_alt.xml # Face detection model
│
├── viewer/                             # Web frontend
│   ├── package.json                    # Dependencies
│   ├── index.html                      # UI
│   └── src/main.js                     # 3D scene + Networking
│
├── run_godview.sh                      # Main launcher
├── check_requirements.sh               # Dependency checker
├── install_dependencies.sh             # Auto-installer
├── README.md                           # User documentation
├── TECHNICAL_DOCUMENTATION.md          # Deep dive (this file)
└── MULTI_AGENT_UPGRADE.md             # Multi-agent upgrade notes
```

---

## 🔑 Key Concepts

### 1. The 3D Projection Formula

```
Z (depth) = (Focal Length × Real Face Width) / Pixel Width
X (lateral) = (Pixel X - Center X) × Z / Focal Length
```

### 2. The Data Flow

```
Webcam → OpenCV → 3D Math → JSON → Zenoh → Browser → Three.js → Ghost
```

### 3. The Multi-Agent System

```javascript
const ghosts = new Map();  // agentId → ghostMesh

// On new hazard
if (!ghosts.has(agentId)) {
    ghosts.set(agentId, createGhost());
}

// Update existing
ghosts.get(agentId).userData.targetPos.set(x, y, z);
```

---

## 🎨 Visual Guide

### What You See in the Browser

```
┌─────────────────────────────────────────────┐
│ GODVIEW SYSTEM STATUS: TRACKING 2 HAZARD(S) │ ← HUD (top-left)
├─────────────────────────────────────────────┤
│                                             │
│         ┌─────────────┐                     │
│         │             │  ← Wireframe room   │
│         │    ●        │  ← Red ghost #1     │
│         │             │                     │
│         │        ●    │  ← Red ghost #2     │
│         └─────────────┘                     │
│    ═══════════════════════  ← Grid floor    │
│                                             │
├─────────────────────────────────────────────┤
│ Protocol: Zenoh v1.0                        │ ← Info (bottom-left)
│ Latency: 45ms                               │
└─────────────────────────────────────────────┘
```

---

## 🔧 Configuration

### Adjust Camera Position

**File:** `viewer/src/main.js`

```javascript
camera.position.set(2, 2, 3);  // [X, Y, Z]
camera.lookAt(0, 0, 0);        // Look at origin
```

### Change Ghost Timeout

**File:** `viewer/src/main.js`

```javascript
const GHOST_TIMEOUT = 2000;  // milliseconds
```

### Adjust Detection Sensitivity

**File:** `agent/src/main.rs`

```rust
face_cascade.detect_multi_scale(
    &gray,
    &mut faces,
    1.1,  // Scale factor (lower = more sensitive, more false positives)
    3,    // Min neighbors (higher = fewer false positives)
    // ...
)?;
```

### Change Zenoh Topic

**Both files must match!**

**Rust:** `agent/src/main.rs`
```rust
let key = "godview/zone1/hazards";
```

**JavaScript:** `viewer/src/main.js`
```javascript
const key = 'godview/zone1/hazards';
```

---

## 🐛 Troubleshooting

### Problem: "Command 'cargo' not found"

**Solution:**
```bash
./install_dependencies.sh
source ~/.cargo/env
```

### Problem: "Failed to open webcam"

**Solution:**
```bash
# Check device exists
ls -l /dev/video0

# Add user to video group
sudo usermod -a -G video $USER
# Then logout and login
```

### Problem: "Zenoh connection failed"

**Solution:**
```bash
# Check if zenohd is running
ps aux | grep zenohd

# Check port 8000 is available
netstat -tuln | grep 8000

# Restart system
./run_godview.sh
```

### Problem: No ghost appears in browser

**Solution:**
1. Check browser console for errors (F12)
2. Verify Rust agent is publishing:
   ```bash
   # Look for "[X-RAY EMITTER] Sent Hazard" logs
   ```
3. Ensure good lighting for face detection
4. Try moving closer to webcam

---

## 📊 Performance Tuning

### Reduce CPU Usage

**Option 1: Lower FPS**

`agent/src/main.rs`:
```rust
sleep(Duration::from_millis(66)).await;  // 15 FPS instead of 30
```

**Option 2: Reduce Detection Frequency**

```rust
if frame_counter % 2 == 0 {  // Detect every other frame
    face_cascade.detect_multi_scale(...)?;
}
```

### Increase Smoothness

**Option 1: Higher LERP Factor**

`viewer/src/main.js`:
```javascript
const LERP_FACTOR = 0.2;  // Faster response (less smooth)
```

**Option 2: Lower LERP Factor**

```javascript
const LERP_FACTOR = 0.05;  // Slower response (more smooth)
```

---

## 🎯 Common Modifications

### Add New Hazard Type

**1. Update Rust packet:**

`agent/src/main.rs`:
```rust
let packet = HazardPacket {
    id: format!("forklift_{}", counter),
    timestamp: now,
    pos: [x, y, z],
    hazard_type: "forklift".to_string(),  // ← New type
};
```

**2. Update viewer colors:**

`viewer/src/main.js`:
```javascript
function createGhost(hazardType) {
    const colors = {
        'human_face': 0xff0000,  // Red
        'forklift': 0xff8800,    // Orange
    };
    
    const ghostMaterial = new THREE.MeshBasicMaterial({
        color: colors[hazardType] || 0xff0000,
        // ...
    });
}
```

### Add Ghost Labels

`viewer/src/main.js`:
```javascript
import { TextGeometry } from 'three/examples/jsm/geometries/TextGeometry';

function createGhost() {
    const ghost = /* ... existing code ... */;
    
    // Add text label
    const textGeometry = new TextGeometry(agentId, {
        font: font,
        size: 0.1,
        height: 0.01,
    });
    const textMesh = new THREE.Mesh(textGeometry, new THREE.MeshBasicMaterial({ color: 0xffffff }));
    textMesh.position.y = 0.3;  // Above ghost
    ghost.add(textMesh);
    
    return ghost;
}
```

### Change Ghost Shape

`viewer/src/main.js`:
```javascript
function createGhost() {
    // Instead of sphere, use cube
    const ghostGeometry = new THREE.BoxGeometry(0.3, 0.3, 0.3);
    
    // Or cone
    const ghostGeometry = new THREE.ConeGeometry(0.2, 0.4, 32);
    
    // Or custom shape
    const ghostGeometry = new THREE.TorusGeometry(0.2, 0.05, 16, 100);
    
    // ... rest of code
}
```

---

## 📈 Monitoring

### View Rust Agent Logs

```bash
cd agent
cargo run 2>&1 | tee agent.log

# Look for:
# [X-RAY EMITTER] Sent Hazard at pos: [0.25, 0.00, 1.20]
```

### View Browser Logs

Open browser console (F12) and look for:
```
[GODVIEW] Received Hazard: {id: "hazard_42", ...}
[GODVIEW] Spawning new ghost for agent: hazard_42
[GODVIEW] Removing stale ghost: hazard_42
```

### Monitor Zenoh Traffic

```bash
# Install Zenoh CLI tools
# Then monitor topic
zenoh-cli subscribe 'godview/zone1/hazards'
```

---

## 🧪 Testing Scenarios

### Test 1: Single Person

1. Sit in front of webcam
2. **Expected:** 1 red ghost appears
3. Move left/right
4. **Expected:** Ghost follows smoothly
5. Leave frame
6. **Expected:** Ghost fades after 2 seconds

### Test 2: Multiple People

1. Show webcam a photo of a face
2. **Expected:** Ghost 1 appears
3. Move your face into frame
4. **Expected:** Ghost 2 appears
5. Status: `TRACKING 2 HAZARD(S)`

### Test 3: Rapid Movement

1. Move head quickly left/right
2. **Expected:** Ghost follows with slight lag (LERP smoothing)
3. No jittering or jumping

### Test 4: Poor Lighting

1. Turn off lights
2. **Expected:** Detection may fail (Haar Cascade needs good lighting)
3. Turn on lights
4. **Expected:** Detection resumes

---

## 🔗 Important Links

- **Zenoh Docs:** https://zenoh.io/docs/
- **Three.js Docs:** https://threejs.org/docs/
- **OpenCV Rust:** https://docs.rs/opencv/latest/opencv/
- **Tokio Async:** https://tokio.rs/

---

## 💡 Pro Tips

1. **Use Good Lighting:** Haar Cascade works best with even, bright lighting
2. **Position Camera at Eye Level:** Better face detection accuracy
3. **Monitor CPU Usage:** If high, reduce FPS or detection frequency
4. **Test Network Latency:** Use `ping localhost` to verify low latency
5. **Clear Browser Cache:** If viewer behaves strangely after code changes

---

## 📞 Support

### Common Questions

**Q: Can I use a different camera?**
A: Yes, change `VideoCapture(0)` to `VideoCapture(1)` in `agent/src/main.rs`

**Q: Can I run agent and viewer on different machines?**
A: Yes, change `ws://localhost:8000` to `ws://<agent-ip>:8000` in `viewer/src/main.js`

**Q: How do I detect objects other than faces?**
A: Replace Haar Cascade with YOLO or other object detector (see TECHNICAL_DOCUMENTATION.md)

**Q: Can I record the 3D data?**
A: Yes, add a Zenoh subscriber that writes to database (see Future Enhancements)

---

**End of Quick Reference**

*For detailed explanations, see TECHNICAL_DOCUMENTATION.md*
