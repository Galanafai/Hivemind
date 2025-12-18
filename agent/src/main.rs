use anyhow::Result;
use opencv::{
    core::{Mat, Vector},
    objdetect::CascadeClassifier,
    prelude::*,
    videoio::{self, VideoCapture, CAP_ANY},
    imgproc,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};
use zenoh::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct HazardPacket {
    id: String,
    timestamp: i64,
    pos: [f32; 3],
    #[serde(rename = "type")]
    hazard_type: String,
}

// Constants for 3D projection math
const FOCAL_LENGTH_CONST: f32 = 500.0; // Approximate focal length in pixels
const REAL_FACE_WIDTH_M: f32 = 0.15; // Average human face width in meters (~15cm)

#[tokio::main]
async fn main() -> Result<()> {
    println!("[X-RAY EMITTER] Initializing GodView Agent...");

    // 1. Initialize Zenoh Session
    let config = zenoh::Config::default();
    let session = zenoh::open(config).await?;
    println!("[X-RAY EMITTER] Zenoh session established");

    let key = "godview/zone1/hazards";
    println!("[X-RAY EMITTER] Publishing to key: {}", key);

    // 2. Open Webcam
    let mut cam = VideoCapture::new(0, CAP_ANY)?;
    if !cam.is_opened()? {
        anyhow::bail!("Failed to open webcam (device 0)");
    }
    println!("[X-RAY EMITTER] Webcam opened successfully");

    // 3. Load Haar Cascade Classifier
    let cascade_path = "haarcascade_frontalface_alt.xml";
    let mut face_cascade = CascadeClassifier::new(cascade_path)?;
    if face_cascade.empty() {
        anyhow::bail!("Failed to load Haar Cascade from {}", cascade_path);
    }
    println!("[X-RAY EMITTER] Haar Cascade loaded: {}", cascade_path);

    // 4. Main Detection Loop (30 Hz)
    let mut frame = Mat::default();
    let mut gray = Mat::default();
    let mut hazard_counter = 0u64;

    println!("[X-RAY EMITTER] Starting detection loop (30 Hz)...");
    
    loop {
        // Capture frame
        cam.read(&mut frame)?;
        if frame.empty() {
            println!("[X-RAY EMITTER] Warning: Empty frame captured");
            sleep(Duration::from_millis(33)).await;
            continue;
        }

        // Convert to grayscale
        imgproc::cvt_color(&frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;

        // Detect faces
        let mut faces = Vector::<opencv::core::Rect>::new();
        face_cascade.detect_multi_scale(
            &gray,
            &mut faces,
            1.1,  // scale factor
            3,    // min neighbors
            0,    // flags
            opencv::core::Size::new(30, 30), // min size
            opencv::core::Size::new(0, 0),   // max size (0 = no limit)
        )?;

        // Process each detected face
        for face in faces.iter() {
            let frame_width = frame.cols() as f32;
            let frame_height = frame.rows() as f32;
            let center_x = frame_width / 2.0;
            
            // Get face bounding box
            let face_x = face.x as f32 + (face.width as f32 / 2.0);
            let face_y = face.y as f32 + (face.height as f32 / 2.0);
            let face_width_px = face.width as f32;

            // 3D PROJECTION MATH (The Core Innovation)
            // Z (Depth) = (Focal Length * Real Object Width) / Pixel Width
            let z = (FOCAL_LENGTH_CONST * REAL_FACE_WIDTH_M) / face_width_px;
            
            // X (Lateral Position) = (Pixel X - Center X) * Z / Focal Length
            let x = (face_x - center_x) * z / FOCAL_LENGTH_CONST;
            
            // Y is fixed at 0.0 for this MVP (assumes faces at same height)
            let y = 0.0;

            // Create Hazard Packet
            hazard_counter += 1;
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis() as i64;

            let packet = HazardPacket {
                id: format!("hazard_{}", hazard_counter),
                timestamp,
                pos: [x, y, z],
                hazard_type: "human_face".to_string(),
            };

            // Serialize to JSON
            let json_payload = serde_json::to_string(&packet)?;

            // Publish via Zenoh
            session
                .put(key, json_payload.clone())
                .await?;

            println!(
                "[X-RAY EMITTER] Sent Hazard at pos: [{:.2}, {:.2}, {:.2}] | Depth: {:.2}m",
                x, y, z, z
            );
        }

        // 30 Hz = ~33ms per frame
        sleep(Duration::from_millis(33)).await;
    }
}
