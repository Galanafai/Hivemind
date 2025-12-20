# GodView + CARLA Integration - Phase 1

**Status:** ✅ Implementation Complete  
**Branch:** `carla-integration`  
**Hardware:** Optimized for GTX 1050 Ti (4GB VRAM)

---

## What This Does

Connects GodView v3 to CARLA simulator to test collaborative perception in a realistic environment:

- **3 autonomous vehicles** with cameras and GPS
- **YOLO object detection** (vehicles, pedestrians)
- **GodView Core v3** processing (AS-EKF, H3+Octree, CapBAC)
- **Ground truth validation** from CARLA

---

## Architecture

```
CARLA Simulator
  ↓ (Camera frames + GPS)
Python Bridge (godview_carla_bridge.py)
  ↓ (YOLO detections + JSON)
Rust Agent (carla_mode.rs)
  ↓ (Signed packets)
Zenoh → godview/carla/hazards
```

---

## Quick Start

### 1. Install CARLA

```bash
# Download CARLA 0.9.15
wget https://carla-releases.s3.us-east-005.backblazeb2.com/Linux/CARLA_0.9.15.tar.gz
tar -xzf CARLA_0.9.15.tar.gz -C ~/

# Add Python API to path
echo 'export PYTHONPATH=$PYTHONPATH:~/CARLA_0.9.15/PythonAPI/carla/dist/carla-0.9.15-py3.7-linux-x86_64.egg' >> ~/.bashrc
source ~/.bashrc
```

### 2. Install Python Dependencies

```bash
cd /home/ubu/godview
pip install -r carla_bridge/requirements.txt
```

### 3. Start CARLA Server

**Terminal 1:**
```bash
cd ~/CARLA_0.9.15
./CarlaUE4.sh -quality-level=Low -RenderOffScreen
```

**Wait for:** `Waiting for the client to connect...`

### 4. Run GodView Bridge

**Terminal 2:**
```bash
cd /home/ubu/godview
./run_carla_phase1.sh
```

---

## What Gets Created

### Files Modified:
- `agent/src/main.rs` - Added CARLA mode detection
- `agent/src/carla_mode.rs` - NEW: CARLA stdin handler

### Files Created:
- `carla_bridge/godview_carla_bridge.py` - Python bridge
- `carla_bridge/requirements.txt` - Dependencies
- `run_carla_phase1.sh` - Launch script

---

## How It Works

### Python Bridge (`godview_carla_bridge.py`)

1. **Connects to CARLA** at localhost:2000
2. **Spawns 3 vehicles** (Tesla Model 3) with autopilot
3. **Attaches sensors:**
   - RGB Camera (640x480 for GTX 1050 Ti)
   - GPS sensor
4. **Processes frames:**
   - Runs YOLOv8n (nano model, fastest)
   - Detects: cars, trucks, buses, people, bicycles, motorcycles
5. **Sends to Rust:**
   - JSON format via stdin
   - Includes: bbox, confidence, class, GPS, heading

### Rust Agent (`carla_mode.rs`)

1. **Reads from stdin** (JSON lines)
2. **Parses detections** into `CARLADetection` struct
3. **Creates entities** with global GPS coordinates
4. **Updates engines:**
   - AS-EKF for sensor fusion
   - H3+Octree for spatial indexing
5. **Signs packets** with Ed25519
6. **Publishes** to `godview/carla/hazards`

---

## GTX 1050 Ti Optimizations

**Applied automatically:**

1. **Low quality rendering** in CARLA
2. **Reduced camera resolution** (640x480 vs 1280x720)
3. **YOLOv8n model** (smallest, fastest)
4. **20 FPS** instead of 30 FPS
5. **Synchronous mode** for stability

**VRAM Usage:**
- CARLA: ~2.5 GB
- YOLO: ~0.5 GB
- Total: ~3 GB (safe for 4GB card)

---

## Expected Output

### Python Bridge:
```
╔════════════════════════════════════════════╗
║   GODVIEW + CARLA INTEGRATION (Phase 1)   ║
╚════════════════════════════════════════════╝

🔌 Connecting to CARLA at localhost:2000...
✅ Connected to CARLA world: Town03
🎮 Graphics: Low Quality (GTX 1050 Ti optimized)

🤖 Loading YOLOv8n (nano) model...
✅ YOLO model loaded

🚗 Spawning 3 vehicles...
  ✅ Vehicle 0: Spawned at (120.5, -50.2, 0.3)
  ✅ Vehicle 1: Spawned at (130.1, -45.8, 0.3)
  ✅ Vehicle 2: Spawned at (140.7, -52.3, 0.3)
✅ Spawned 3 vehicles

🚀 Starting GodView Rust agents...
  ✅ Started agent for carla_vehicle_0 (PID: 12345)
  ✅ Started agent for carla_vehicle_1 (PID: 12346)
  ✅ Started agent for carla_vehicle_2 (PID: 12347)
✅ Started 3 agents

🎬 Running simulation for 60 seconds...
Press Ctrl+C to stop early

⏱️  1.0s | Ticks: 20 | Frames: 60 | FPS: 60.0
⏱️  2.0s | Ticks: 40 | Frames: 120 | FPS: 60.0
...
```

### Rust Agent:
```
╔════════════════════════════════════════════╗
║   GODVIEW AGENT V3 (CARLA MODE)           ║
╚════════════════════════════════════════════╝

📍 Agent Configuration:
   ID: carla_vehicle_0
   Mode: CARLA (stdin input)

🔧 Initializing GodView Core v3 engines...
   ✅ AS-EKF initialized (lag depth: 20 states)
   ✅ Spatial Engine initialized (H3 Resolution 10)
   ✅ Security initialized (Ed25519)

🌐 Zenoh session established
📡 Publishing to: godview/carla/hazards

🎬 Waiting for detections from CARLA bridge...
════════════════════════════════════════════

📤 [Detection 10] car detected:
   GPS: [48.858370, 2.294481, 0.30]
   Confidence: 0.92
   Entity ID: 550e8400-e29b-41d4-a716-446655440000
```

---

## Testing Checklist

### Phase 1 Goals:
- [ ] CARLA server starts successfully
- [ ] Python bridge connects to CARLA
- [ ] 3 vehicles spawn with sensors
- [ ] YOLO detects objects
- [ ] JSON sent to Rust agents
- [ ] Rust agents parse detections
- [ ] Entities published to Zenoh
- [ ] No crashes for 60 seconds

### Success Criteria:
- ✅ All 3 agents running
- ✅ Detections appearing in logs
- ✅ GPS coordinates reasonable
- ✅ VRAM usage < 3.5 GB
- ✅ FPS > 15

---

## Troubleshooting

### "Failed to connect to CARLA"
**Solution:** Start CARLA server first:
```bash
cd ~/CARLA_0.9.15
./CarlaUE4.sh -quality-level=Low -RenderOffScreen
```

### "CARLA Python API not found"
**Solution:** Add to PYTHONPATH:
```bash
export PYTHONPATH=$PYTHONPATH:~/CARLA_0.9.15/PythonAPI/carla/dist/carla-0.9.15-py3.7-linux-x86_64.egg
```

### "YOLOv8 not installed"
**Solution:**
```bash
pip install ultralytics
```

### "Rust agent won't compile"
**Solution:**
```bash
cd agent
cargo clean
cargo build --release
```

### "Out of VRAM"
**Solutions:**
1. Reduce camera resolution in `godview_carla_bridge.py` (line 95-96)
2. Use even lower quality: `./CarlaUE4.sh -quality-level=Epic` → `-quality-level=Low`
3. Reduce number of vehicles: `--vehicles 2` instead of 3

---

## Next Steps (Phase 2)

After Phase 1 works:

1. **Add validation system** (compare vs ground truth)
2. **Implement "seeing around corners" test**
3. **Add network latency simulation**
4. **Test AS-EKF with delays**
5. **Measure position accuracy**

See `CARLA_INTEGRATION_PLAN.md` for full roadmap.

---

## File Structure

```
godview/
├── carla_bridge/
│   ├── godview_carla_bridge.py    # Main Python bridge
│   ├── requirements.txt            # Python dependencies
│   └── scenarios/                  # Test scenarios (Phase 2)
├── agent/
│   └── src/
│       ├── main.rs                 # Modified for CARLA mode
│       └── carla_mode.rs           # NEW: CARLA stdin handler
├── run_carla_phase1.sh             # Quick start script
└── CARLA_INTEGRATION_PLAN.md       # Full plan
```

---

**Ready to test!** 🚗🚀

**Branch:** `carla-integration`  
**Commit:** Ready for testing
