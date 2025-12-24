# GodView v3 - Testing Guide for Limited Environments

**Reality Check:** You probably don't have GPS hardware, multiple cameras, or a fancy lab setup.  
**Good News:** We can still test the HELL out of this system with what you have!

---

## 🎯 Testing Philosophy

**We're testing THREE things:**
1. **Does the code compile and run?** (Rust is strict, if it compiles it probably works)
2. **Do the math engines work?** (AS-EKF, H3+Octree, coordinate transforms)
3. **Does the integration work?** (Agent → Core library → Zenoh → Viewer)

---

## 🔧 What You Actually Need

### Minimum Requirements:
- ✅ **One webcam** (built-in laptop camera is fine)
- ✅ **Rust installed** (`./install_dependencies.sh`)
- ✅ **WiFi/Ethernet** (for Zenoh localhost)
- ✅ **Your face** (for detection testing)

### Nice to Have:
- ⚠️ Second camera (USB webcam, phone camera via IP)
- ⚠️ Second computer/laptop (for multi-agent testing)
- ⚠️ Printed face photo (for static testing)

---

## 📋 Test Plan (Ordered by Difficulty)

### Level 1: Unit Tests (5 minutes)

**Test the core library in isolation:**

```bash
cd /home/ubu/godview/godview_core
cargo test -- --nocapture
```

**What this tests:**
- ✅ AS-EKF initialization
- ✅ Prediction step math
- ✅ OOSM update logic
- ✅ H3 cell assignment
- ✅ Vertical separation (drone vs car)
- ✅ Ed25519 signature creation/verification
- ✅ Biscuit token authorization

**Expected output:**
```
test godview_time::tests::test_filter_initialization ... ok
test godview_time::tests::test_prediction_step ... ok
test godview_space::tests::test_entity_insertion ... ok
test godview_space::tests::test_vertical_separation ... ok
test godview_trust::tests::test_signed_packet_creation ... ok
test godview_trust::tests::test_signature_verification_fails_on_tampering ... ok
test godview_trust::tests::test_biscuit_authorization ... ok
```

**If this passes:** Core library is solid! 🎉

---

### Level 2: Agent Build Test (10 minutes)

**Test that the agent compiles with all dependencies:**

```bash
cd /home/ubu/godview/agent
cargo build --release
```

**What this tests:**
- ✅ All dependencies resolve (godview_core, nalgebra, h3o, etc.)
- ✅ No syntax errors
- ✅ Type system is happy
- ✅ Linking works

**Expected output:**
```
   Compiling godview_core v0.3.0
   Compiling godview_agent v0.3.0
    Finished release [optimized] target(s) in 45.2s
```

**If this passes:** Integration is solid! 🎉

---

### Level 3: Single Agent Test (15 minutes)

**Run the agent with virtual GPS and webcam:**

```bash
cd /home/ubu/godview/agent

# Set virtual GPS (San Francisco coordinates)
export AGENT_ID="test_agent_1"
export AGENT_GPS_LAT=37.7749
export AGENT_GPS_LON=-122.4194
export AGENT_GPS_ALT=10.0
export AGENT_HEADING=0.0

# Run agent
cargo run --release
```

**What to do:**
1. Position your face in front of webcam
2. Move around slowly
3. Watch terminal output

**Expected output:**
```
╔════════════════════════════════════════════╗
║   GODVIEW AGENT V3 (GLOBAL MODE)          ║
╚════════════════════════════════════════════╝

📍 Agent Configuration:
   GPS: (37.774900, -122.419400, 10.0m)
   Heading: 0.0° (0°=North)
   ID: test_agent_1

🔧 Initializing GodView Core v3 engines...
   ✅ AS-EKF initialized (lag depth: 20 states)
   ✅ Spatial Engine initialized (H3 Resolution 10)
   ✅ Security initialized (Ed25519)

🚀 Starting detection loop (30 Hz)...

📤 [Frame 42] Hazard detected:
   Camera: [0.25, 0.00, 1.20]m
   Global: [37.774911, -122.419398, 10.00]
   Entity ID: 550e8400-e29b-41d4-a716-446655440000
```

**What this tests:**
- ✅ Webcam capture works
- ✅ Face detection works
- ✅ 3D projection math works
- ✅ Camera-to-global transform works
- ✅ AS-EKF updates without crashing
- ✅ H3+Octree indexing works
- ✅ Ed25519 signing works
- ✅ Zenoh publishing works

**Success criteria:**
- No crashes
- Detections appear when face is visible
- Global GPS coordinates change slightly as you move
- Camera coordinates make sense (Z increases as you move away)

---

### Level 4: Coordinate Accuracy Test (20 minutes)

**Verify the coordinate transform math is correct:**

**Setup:**
1. Measure distance from camera to your face (use tape measure or estimate)
2. Note the camera-relative Z coordinate from output
3. Compare to actual distance

**Example:**
```
Actual distance: 1.5 meters
Camera Z output: 1.48 meters
Error: 1.3% ✅ Good!
```

**Test lateral position:**
1. Stand directly in front of camera (centered)
   - Expected: Camera X ≈ 0.0
2. Move 0.5m to the right
   - Expected: Camera X ≈ +0.5
3. Move 0.5m to the left
   - Expected: Camera X ≈ -0.5

**Success criteria:**
- Depth (Z) within 10% of actual
- Lateral (X) within 20% of actual
- Global GPS changes consistently with movement

---

### Level 5: Virtual Multi-Agent Test (30 minutes)

**Simulate two agents with different positions/headings:**

**Terminal 1 - Agent Northwest (facing North):**
```bash
cd /home/ubu/godview/agent

export AGENT_ID="agent_nw"
export AGENT_GPS_LAT=37.7749
export AGENT_GPS_LON=-122.4194
export AGENT_GPS_ALT=10.0
export AGENT_HEADING=0.0  # North

cargo run --release
```

**Terminal 2 - Agent Northeast (facing East):**
```bash
cd /home/ubu/godview/agent

export AGENT_ID="agent_ne"
export AGENT_GPS_LAT=37.7749
export AGENT_GPS_LON=-122.4193  # 100m east
export AGENT_GPS_ALT=10.0
export AGENT_HEADING=90.0  # East

cargo run --release
```

**What to test:**
1. Both agents detect your face
2. Both publish to `godview/global/hazards`
3. Global GPS coordinates are DIFFERENT (different camera positions/headings)
4. But both should be detecting the SAME physical location

**Expected behavior:**
- Agent NW: Global GPS ≈ (37.7749, -122.4194, 10.X)
- Agent NE: Global GPS ≈ (37.7749, -122.4193, 10.X)
- Both should converge to similar coordinates if you're equidistant

**Success criteria:**
- Both agents run simultaneously
- Both detect faces
- Global coordinates make geometric sense
- No crashes

---

### Level 6: AS-EKF Latency Test (Advanced)

**Test that AS-EKF handles delayed measurements:**

**Modify agent temporarily to add artificial delay:**

```rust
// In agent/src/main.rs, in the detection loop:

// Simulate 200ms processing delay
tokio::time::sleep(Duration::from_millis(200)).await;

// Then update EKF with OLD timestamp
let old_timestamp = current_time - 0.2;  // 200ms ago
ekf.update_oosm(measurement, old_timestamp);
```

**What to test:**
- Agent still runs smoothly
- No "time travel" artifacts
- Predictions remain stable

**Success criteria:**
- No crashes
- Smooth operation despite artificial delay

---

### Level 7: Stress Test (Advanced)

**Push the system to its limits:**

**Test 1: Rapid Movement**
- Wave your hands in front of camera rapidly
- Expected: Detections appear/disappear smoothly
- No crashes

**Test 2: Multiple Faces**
- Show multiple faces (photos, other people)
- Expected: Multiple entities tracked
- Unique IDs for each

**Test 3: Long Duration**
- Run agent for 30+ minutes
- Expected: No memory leaks
- Stable performance

**Test 4: Network Stress**
- Run agent with poor WiFi
- Expected: Graceful degradation
- No crashes

---

## 🐛 Debugging Guide

### Problem: Agent won't compile

**Check:**
```bash
# Verify Rust version
rustc --version  # Should be 1.70+

# Clean and rebuild
cargo clean
cargo build --release
```

### Problem: "Failed to open webcam"

**Fix:**
```bash
# Check webcam device
ls /dev/video*

# Try different device number
# In agent/src/main.rs, change:
let mut cam = VideoCapture::new(0, CAP_ANY)?;
# to:
let mut cam = VideoCapture::new(1, CAP_ANY)?;  // or 2, 3, etc.
```

### Problem: No face detections

**Check:**
1. Haar cascade file exists: `ls haarcascade_frontalface_alt.xml`
2. Lighting is good (face well-lit)
3. Face is 0.5-3m from camera
4. Face is roughly frontal (not profile)

### Problem: Coordinates seem wrong

**Verify:**
1. FOCAL_LENGTH_CONST matches your camera (500 is generic)
2. REAL_FACE_WIDTH_M is reasonable (0.15m = 15cm)
3. Heading is correct (0° = North, 90° = East)

### Problem: Zenoh errors

**Fix:**
```bash
# Check if Zenoh router is needed
# For localhost testing, it should work without router

# If needed, start router:
zenohd
```

---

## 📊 Success Metrics

### Minimum Viable Test:
- ✅ Unit tests pass
- ✅ Agent compiles
- ✅ Single agent detects faces
- ✅ Coordinates are reasonable

### Good Test:
- ✅ All of above
- ✅ Coordinate accuracy within 20%
- ✅ Virtual multi-agent works
- ✅ Runs for 10+ minutes stable

### Excellent Test:
- ✅ All of above
- ✅ AS-EKF latency test passes
- ✅ Stress tests pass
- ✅ Real multi-agent (2 cameras)

---

## 🚀 Next Level Testing (If You Get Hardware)

### With Second Camera:
- USB webcam
- Phone camera via IP (use app like "IP Webcam")
- Second laptop with webcam

### With GPS Hardware:
- USB GPS dongle ($20-50)
- Replace virtual GPS with real coordinates
- Test outdoor with actual movement

### With Multiple Computers:
- Deploy agents on different machines
- Test real network latency
- Verify distributed operation

---

## 📝 Test Log Template

**Copy this for each test session:**

```markdown
## Test Session: [Date]

### Environment:
- OS: Ubuntu 22.04
- Rust: 1.75.0
- Camera: Built-in laptop webcam
- Network: WiFi localhost

### Tests Run:
- [ ] Unit tests
- [ ] Agent build
- [ ] Single agent
- [ ] Coordinate accuracy
- [ ] Virtual multi-agent
- [ ] AS-EKF latency
- [ ] Stress test

### Results:
- Passes: X/7
- Failures: Y/7
- Notes: [observations]

### Issues Found:
1. [Issue description]
2. [Issue description]

### Next Steps:
- [What to test next]
```

---

## 🎯 The Bottom Line

**You can thoroughly test GodView v3 with:**
- One laptop
- One webcam
- Your face
- 1-2 hours

**This tests:**
- 90% of the core functionality
- All three engines (Time, Space, Trust)
- Integration between components
- Real-world detection and tracking

**You DON'T need:**
- Multiple cameras (nice to have)
- GPS hardware (virtual GPS works)
- Fancy lab setup
- Multiple computers (can simulate)

**Start with Level 1-3, then go deeper as you get comfortable!**

---

**Happy Testing!** 🧪🚀

**Repository:** https://github.com/Galanafai/Hivemind  
**Issues:** https://github.com/Galanafai/Hivemind/issues
