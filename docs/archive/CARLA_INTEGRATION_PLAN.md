# GodView v3 + CARLA Integration Plan

**Mission:** Test GodView v3 in a realistic autonomous vehicle simulation with ground truth validation

**Why CARLA is Perfect:**
- ✅ Multiple vehicles with real GPS coordinates
- ✅ Simulated cameras, LiDAR, radar
- ✅ Realistic physics and traffic
- ✅ Network latency simulation
- ✅ Ground truth for validation
- ✅ Python API (easy to integrate)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    CARLA Simulator                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │ Vehicle 1│  │ Vehicle 2│  │ Vehicle 3│             │
│  │ (Ego)    │  │          │  │          │             │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘             │
│       │             │             │                     │
│   ┌───▼─────────────▼─────────────▼───┐                │
│   │   CARLA Sensor Manager             │                │
│   │   - Camera feeds                   │                │
│   │   - GPS data                       │                │
│   │   - Ground truth                   │                │
│   └───┬────────────────────────────────┘                │
└───────┼─────────────────────────────────────────────────┘
        │
        │ Python API
        ▼
┌─────────────────────────────────────────────────────────┐
│          GodView CARLA Bridge (Python)                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  For each vehicle:                                │  │
│  │  1. Get camera frame                             │  │
│  │  2. Get GPS position                             │  │
│  │  3. Detect objects (YOLO/OpenCV)                 │  │
│  │  4. Convert to GodView format                    │  │
│  │  5. Call Rust agent via FFI or subprocess        │  │
│  └──────────────────────────────────────────────────┘  │
└───────┬─────────────────────────────────────────────────┘
        │
        │ Zenoh
        ▼
┌─────────────────────────────────────────────────────────┐
│              GodView Core v3 (Rust)                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
│  │  AS-EKF  │  │H3+Octree │  │  CapBAC  │             │
│  └──────────┘  └──────────┘  └──────────┘             │
└───────┬─────────────────────────────────────────────────┘
        │
        │ Zenoh
        ▼
┌─────────────────────────────────────────────────────────┐
│          Validation & Visualization                     │
│  - Compare GodView output vs CARLA ground truth        │
│  - Measure accuracy, latency, bandwidth                │
│  - 3D visualization (Three.js or CARLA spectator)      │
└─────────────────────────────────────────────────────────┘
```

---

## What Needs to Change

### 1. **Agent Input Source** (Major Change)

**Current:** OpenCV webcam capture
```rust
let mut cam = VideoCapture::new(0, CAP_ANY)?;
cam.read(&mut frame)?;
```

**CARLA:** Python bridge provides frames
```rust
// Option A: FFI from Python
#[no_mangle]
pub extern "C" fn process_frame(
    image_data: *const u8,
    width: u32,
    height: u32,
    gps_lat: f64,
    gps_lon: f64,
    gps_alt: f32,
    heading: f32,
) -> *const c_char { ... }

// Option B: Subprocess with stdin/stdout
// Python sends: frame + GPS → Rust agent
// Rust agent sends: detections → Python
```

### 2. **GPS Source** (Minor Change)

**Current:** Environment variables (virtual GPS)
```rust
let agent_lat: f64 = std::env::var("AGENT_GPS_LAT")
    .unwrap_or("37.7749".to_string()).parse()?;
```

**CARLA:** Real GPS from simulator
```rust
// Receive from CARLA bridge
let agent_lat: f64 = carla_gps.latitude;
let agent_lon: f64 = carla_gps.longitude;
let agent_alt: f32 = carla_gps.altitude;
let agent_heading: f32 = carla_vehicle.get_transform().rotation.yaw;
```

### 3. **Detection Method** (Optional Upgrade)

**Current:** Haar Cascade (fast but limited)
```rust
face_cascade.detect_multi_scale(&gray, &mut faces, ...)?;
```

**CARLA:** YOLO or Mask R-CNN (better for vehicles/pedestrians)
```python
# In Python bridge
detections = yolo_model.detect(frame)
for det in detections:
    if det.class_name in ['car', 'pedestrian', 'cyclist']:
        # Send to Rust agent
        send_detection(det.bbox, det.confidence, det.class_name)
```

---

## Required Components

### Component 1: CARLA Bridge (Python)

**File:** `carla_bridge/godview_carla_bridge.py`

**Purpose:** Connect CARLA to GodView agents

**Responsibilities:**
1. Spawn vehicles in CARLA
2. Attach sensors (camera, GPS)
3. Process sensor data
4. Detect objects (YOLO)
5. Send to GodView agent
6. Collect ground truth
7. Validate results

**Pseudocode:**
```python
import carla
import cv2
import numpy as np
from ultralytics import YOLO  # YOLOv8
import subprocess
import json

class GodViewCARLABridge:
    def __init__(self, carla_host='localhost', carla_port=2000):
        # Connect to CARLA
        self.client = carla.Client(carla_host, carla_port)
        self.world = self.client.get_world()
        
        # Load YOLO model
        self.yolo = YOLO('yolov8n.pt')
        
        # Spawn vehicles
        self.vehicles = []
        self.spawn_vehicles(num_vehicles=3)
        
        # Start GodView agents (one per vehicle)
        self.agents = []
        for vehicle in self.vehicles:
            agent = self.start_godview_agent(vehicle)
            self.agents.append(agent)
    
    def spawn_vehicles(self, num_vehicles=3):
        blueprint_library = self.world.get_blueprint_library()
        vehicle_bp = blueprint_library.filter('vehicle.tesla.model3')[0]
        
        spawn_points = self.world.get_map().get_spawn_points()
        
        for i in range(num_vehicles):
            vehicle = self.world.spawn_actor(vehicle_bp, spawn_points[i])
            
            # Attach camera
            camera_bp = blueprint_library.find('sensor.camera.rgb')
            camera_bp.set_attribute('image_size_x', '1280')
            camera_bp.set_attribute('image_size_y', '720')
            camera_bp.set_attribute('fov', '90')
            
            camera_transform = carla.Transform(
                carla.Location(x=2.0, z=1.5)  # Front bumper, 1.5m high
            )
            camera = self.world.spawn_actor(
                camera_bp, 
                camera_transform, 
                attach_to=vehicle
            )
            
            # Attach GPS
            gps_bp = blueprint_library.find('sensor.other.gnss')
            gps = self.world.spawn_actor(gps_bp, camera_transform, attach_to=vehicle)
            
            self.vehicles.append({
                'actor': vehicle,
                'camera': camera,
                'gps': gps,
                'id': f'carla_vehicle_{i}'
            })
    
    def start_godview_agent(self, vehicle_data):
        # Start Rust agent as subprocess
        agent_process = subprocess.Popen(
            ['cargo', 'run', '--release'],
            cwd='/home/ubu/godview/agent',
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            env={
                'AGENT_ID': vehicle_data['id'],
                'CARLA_MODE': 'true'
            }
        )
        return agent_process
    
    def process_frame(self, vehicle_data, image):
        # Convert CARLA image to OpenCV format
        array = np.frombuffer(image.raw_data, dtype=np.uint8)
        array = array.reshape((image.height, image.width, 4))
        frame = array[:, :, :3]  # RGB
        
        # Run YOLO detection
        results = self.yolo(frame)
        
        # Get GPS data
        gps_data = vehicle_data['gps'].get_data()
        transform = vehicle_data['actor'].get_transform()
        
        # For each detection, send to GodView agent
        for detection in results[0].boxes:
            bbox = detection.xyxy[0].cpu().numpy()
            confidence = detection.conf[0].cpu().numpy()
            class_id = int(detection.cls[0].cpu().numpy())
            
            # Send to Rust agent
            detection_data = {
                'bbox': bbox.tolist(),
                'confidence': float(confidence),
                'class_name': results[0].names[class_id],
                'gps_lat': gps_data.latitude,
                'gps_lon': gps_data.longitude,
                'gps_alt': gps_data.altitude,
                'heading': transform.rotation.yaw
            }
            
            # Send via stdin to Rust agent
            agent = self.agents[vehicle_data['id']]
            agent.stdin.write(json.dumps(detection_data).encode() + b'\n')
            agent.stdin.flush()
    
    def run(self):
        # Main loop
        while True:
            self.world.tick()  # Advance simulation
            
            # Process each vehicle
            for vehicle_data in self.vehicles:
                # Camera callback will trigger process_frame
                pass
```

---

### Component 2: Modified Rust Agent

**File:** `agent/src/carla_mode.rs` (new module)

**Changes to `main.rs`:**
```rust
// Add CARLA mode detection
let carla_mode = std::env::var("CARLA_MODE").is_ok();

if carla_mode {
    // Read from stdin instead of webcam
    run_carla_mode().await?;
} else {
    // Original webcam mode
    run_webcam_mode().await?;
}
```

**New CARLA mode:**
```rust
async fn run_carla_mode() -> Result<()> {
    use std::io::{self, BufRead};
    
    // Initialize engines (same as before)
    let mut ekf = AugmentedStateFilter::new(...);
    let mut spatial_engine = SpatialEngine::new(Resolution::Ten);
    let signing_key = SigningKey::generate(&mut OsRng);
    
    let session = zenoh::open(zenoh::Config::default()).await?;
    let key = "godview/carla/hazards";
    
    // Read from stdin (Python bridge sends JSON)
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let detection: CARLADetection = serde_json::from_str(&line)?;
        
        // Process detection
        let entity = Entity {
            id: Uuid::new_v4(),
            position: [detection.gps_lat, detection.gps_lon, detection.gps_alt],
            velocity: [0.0, 0.0, 0.0],  // TODO: Calculate from tracking
            entity_type: detection.class_name,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64,
            confidence: detection.confidence,
        };
        
        // Update engines
        spatial_engine.update_entity(entity.clone())?;
        ekf.update_oosm(measurement, current_time);
        
        // Sign and publish
        let packet = GlobalHazardPacket { entity, ... };
        let signed_packet = SignedPacket::new(payload, &signing_key, None);
        session.put(key, signed_payload).await?;
    }
    
    Ok(())
}

#[derive(Deserialize)]
struct CARLADetection {
    bbox: [f32; 4],
    confidence: f32,
    class_name: String,
    gps_lat: f64,
    gps_lon: f64,
    gps_alt: f32,
    heading: f32,
}
```

---

### Component 3: Validation System

**File:** `carla_bridge/validation.py`

**Purpose:** Compare GodView output vs CARLA ground truth

```python
class GodViewValidator:
    def __init__(self):
        # Subscribe to GodView output
        self.zenoh_session = zenoh.open(zenoh.Config())
        self.subscriber = self.zenoh_session.declare_subscriber(
            'godview/carla/hazards',
            self.on_godview_detection
        )
        
        # Store ground truth from CARLA
        self.ground_truth = {}
        
        # Metrics
        self.metrics = {
            'position_errors': [],
            'detection_latencies': [],
            'false_positives': 0,
            'false_negatives': 0,
            'true_positives': 0
        }
    
    def update_ground_truth(self, carla_world):
        # Get all actors in CARLA
        actors = carla_world.get_actors()
        
        self.ground_truth = {}
        for actor in actors.filter('vehicle.*'):
            location = actor.get_location()
            gps = self.carla_to_gps(location)
            
            self.ground_truth[actor.id] = {
                'position': gps,
                'type': 'vehicle',
                'timestamp': time.time()
            }
    
    def on_godview_detection(self, sample):
        # Parse GodView detection
        data = json.loads(sample.payload.decode())
        godview_pos = data['entity']['position']
        
        # Find closest ground truth
        min_distance = float('inf')
        closest_gt = None
        
        for gt_id, gt_data in self.ground_truth.items():
            distance = self.gps_distance(godview_pos, gt_data['position'])
            if distance < min_distance:
                min_distance = distance
                closest_gt = gt_data
        
        # Record metrics
        if min_distance < 5.0:  # Within 5 meters = true positive
            self.metrics['true_positives'] += 1
            self.metrics['position_errors'].append(min_distance)
        else:
            self.metrics['false_positives'] += 1
        
        # Calculate latency
        latency = time.time() - data['entity']['timestamp'] / 1000.0
        self.metrics['detection_latencies'].append(latency)
    
    def print_metrics(self):
        print(f"""
        === GodView Validation Metrics ===
        True Positives: {self.metrics['true_positives']}
        False Positives: {self.metrics['false_positives']}
        False Negatives: {self.metrics['false_negatives']}
        
        Avg Position Error: {np.mean(self.metrics['position_errors']):.2f}m
        Max Position Error: {np.max(self.metrics['position_errors']):.2f}m
        
        Avg Latency: {np.mean(self.metrics['detection_latencies'])*1000:.1f}ms
        Max Latency: {np.max(self.metrics['detection_latencies'])*1000:.1f}ms
        """)
```

---

## Testing Scenarios

### Scenario 1: Basic Multi-Vehicle Detection

**Setup:**
- 3 vehicles in CARLA
- Driving in same direction
- 50m spacing

**Test:**
- Each vehicle detects vehicles ahead
- Verify global GPS coordinates match
- Validate AS-EKF handles movement

**Success Criteria:**
- Position error < 2m
- Detection rate > 90%
- No crashes

---

### Scenario 2: "Seeing Around Corners"

**Setup:**
- Vehicle A at intersection
- Vehicle B around corner (not visible to A)
- Vehicle C can see both

**Test:**
- Vehicle C detects both A and B
- Publishes to GodView
- Vehicle A receives B's position
- Vehicle A knows about B **before seeing it**

**Success Criteria:**
- A receives B's position from C
- Position accurate within 3m
- Latency < 200ms

---

### Scenario 3: Network Latency Stress Test

**Setup:**
- Add artificial network delay (100-500ms)
- 5 vehicles in traffic

**Test:**
- AS-EKF handles delayed measurements
- No "time travel" artifacts
- Predictions remain stable

**Success Criteria:**
- Position error < 5m with 500ms delay
- No crashes or divergence
- Smooth tracking

---

### Scenario 4: 3D Indexing (Bridges/Tunnels)

**Setup:**
- CARLA map with multi-level roads
- Vehicles on bridge (10m altitude)
- Vehicles under bridge (0m altitude)

**Test:**
- H3+Octree correctly separates layers
- No false collision warnings
- Queries respect altitude

**Success Criteria:**
- Vehicles on bridge don't "see" vehicles under bridge
- Spatial queries return correct results
- No vertical aliasing

---

### Scenario 5: Security (Phantom Hazard Injection)

**Setup:**
- Rogue agent injects fake vehicle
- Legitimate agents have valid tokens

**Test:**
- Validator rejects unsigned packets
- Validator rejects packets from unauthorized agents
- System continues operating normally

**Success Criteria:**
- Fake detections rejected
- No impact on legitimate agents
- Security logs show rejection

---

## Required Plugins/Dependencies

### Python Side:
```bash
pip install carla
pip install ultralytics  # YOLOv8
pip install opencv-python
pip install numpy
pip install zenoh
```

### Rust Side:
```toml
# No changes needed! Already have everything
```

### CARLA Setup:
```bash
# Download CARLA 0.9.15
wget https://carla-releases.s3.us-east-005.backblazeb2.com/Linux/CARLA_0.9.15.tar.gz
tar -xzf CARLA_0.9.15.tar.gz

# Run CARLA server
./CarlaUE4.sh -quality-level=Low -RenderOffScreen
```

---

## Project Structure

```
godview/
├── agent/                      # Existing Rust agent
│   ├── src/
│   │   ├── main.rs            # Modified for CARLA mode
│   │   └── carla_mode.rs      # NEW: CARLA-specific logic
│   └── Cargo.toml
├── godview_core/              # Existing core library
├── carla_bridge/              # NEW: CARLA integration
│   ├── godview_carla_bridge.py
│   ├── validation.py
│   ├── scenarios/
│   │   ├── scenario_1_basic.py
│   │   ├── scenario_2_corners.py
│   │   ├── scenario_3_latency.py
│   │   ├── scenario_4_3d.py
│   │   └── scenario_5_security.py
│   ├── requirements.txt
│   └── README.md
└── carla_results/             # NEW: Test results
    ├── metrics.json
    ├── visualizations/
    └── logs/
```

---

## Implementation Timeline

### Week 1: Basic Integration
- [ ] Install CARLA
- [ ] Create Python bridge skeleton
- [ ] Modify Rust agent for stdin mode
- [ ] Test single vehicle detection

### Week 2: Multi-Agent
- [ ] Spawn multiple vehicles
- [ ] Test collaborative perception
- [ ] Implement validation system
- [ ] Run Scenario 1 & 2

### Week 3: Advanced Testing
- [ ] Add network latency simulation
- [ ] Test AS-EKF under stress
- [ ] Run Scenario 3 & 4
- [ ] Collect metrics

### Week 4: Security & Polish
- [ ] Implement security testing
- [ ] Run Scenario 5
- [ ] Create visualizations
- [ ] Write results paper

---

## Expected Results

### Metrics to Measure:
1. **Position Accuracy:** < 2m average error
2. **Detection Rate:** > 95% of ground truth
3. **Latency:** < 100ms end-to-end
4. **Bandwidth:** Confirm 99.25% reduction
5. **AS-EKF Performance:** Handles 500ms delays
6. **3D Accuracy:** No vertical aliasing
7. **Security:** 100% rejection of fake data

### Validation Against Ground Truth:
- CARLA provides perfect ground truth
- Can measure exact errors
- Can prove "seeing around corners" works
- Can demonstrate security effectiveness

---

## Why This Is HUGE

**CARLA testing proves:**
1. ✅ GodView works in realistic scenarios
2. ✅ Collaborative perception is real
3. ✅ AS-EKF handles real network conditions
4. ✅ 3D indexing works with real geometry
5. ✅ Security prevents attacks
6. ✅ System is production-ready

**This becomes:**
- Academic paper material
- Industry demo
- Proof of concept for deployment
- Validation for investors/partners

---

**Let's build this!** 🚗🚀

**Repository:** https://github.com/Galanafai/Hivemind  
**CARLA:** https://carla.org
