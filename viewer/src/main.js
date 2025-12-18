import * as THREE from 'three';
import { Session, Config, Subscriber } from '@eclipse-zenoh/zenoh-ts';

// ============================================
// GODVIEW VIEWER - The Live Reality Protocol
// ============================================

console.log('[GODVIEW] Initializing X-Ray Vision System...');

// Three.js Scene Setup
const scene = new THREE.Scene();
scene.background = new THREE.Color(0x000000);
scene.fog = new THREE.Fog(0x000000, 1, 15);

// Camera (positioned for room overview)
const camera = new THREE.PerspectiveCamera(
  75,
  window.innerWidth / window.innerHeight,
  0.1,
  1000
);
camera.position.set(2, 2, 3);
camera.lookAt(0, 0, 0);

// Renderer
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
renderer.setPixelRatio(window.devicePixelRatio);
document.getElementById('app').appendChild(renderer.domElement);

// Lighting
const ambientLight = new THREE.AmbientLight(0x404040, 1);
scene.add(ambientLight);

const directionalLight = new THREE.DirectionalLight(0xffffff, 0.5);
directionalLight.position.set(5, 10, 5);
scene.add(directionalLight);

// Floor Grid (The "God View" Floor)
const gridHelper = new THREE.GridHelper(10, 20, 0x00ff00, 0x003300);
gridHelper.position.y = -1;
scene.add(gridHelper);

// The Room (Wireframe Box)
const roomGeometry = new THREE.BoxGeometry(4, 3, 4);
const roomEdges = new THREE.EdgesGeometry(roomGeometry);
const roomLine = new THREE.LineSegments(
  roomEdges,
  new THREE.LineBasicMaterial({ color: 0x00ff00, linewidth: 2 })
);
roomLine.position.y = 0.5;
scene.add(roomLine);

// ============================================
// MULTI-AGENT ENTITY SYSTEM
// ============================================

// Dictionary-based entity management for multiple simultaneous hazards
const ghosts = new Map();

// Configuration
const GHOST_TIMEOUT = 2000; // ms - Remove ghost after 2 seconds of no updates
const LERP_FACTOR = 0.1; // Smoothing factor for position interpolation

// HUD Elements
const statusElement = document.getElementById('status');
const latencyElement = document.getElementById('latency');

// ============================================
// GHOST FACTORY
// ============================================

/**
 * Creates a new Red Ghost hazard avatar with glow effect
 * Each ghost has its own material instances for independent opacity control
 * @returns {THREE.Mesh} Ghost mesh with userData for state tracking
 */
function createGhost() {
  // Main ghost sphere
  const ghostGeometry = new THREE.SphereGeometry(0.2, 32, 32);
  const ghostMaterial = new THREE.MeshBasicMaterial({
    color: 0xff0000,
    transparent: true,
    opacity: 0.8,
    wireframe: false,
  });
  const ghostMesh = new THREE.Mesh(ghostGeometry, ghostMaterial);

  // Glow effect (outer sphere)
  const ghostGlowGeometry = new THREE.SphereGeometry(0.25, 32, 32);
  const ghostGlowMaterial = new THREE.MeshBasicMaterial({
    color: 0xff0000,
    transparent: true,
    opacity: 0.3,
    side: THREE.BackSide,
  });
  const ghostGlow = new THREE.Mesh(ghostGlowGeometry, ghostGlowMaterial);
  ghostMesh.add(ghostGlow);

  // Store references to materials for easy access
  ghostMesh.userData.mainMaterial = ghostMaterial;
  ghostMesh.userData.glowMaterial = ghostGlowMaterial;
  ghostMesh.userData.glowMesh = ghostGlow;

  // Initialize state
  ghostMesh.userData.targetPos = new THREE.Vector3(0, 0, 0);
  ghostMesh.userData.lastUpdate = Date.now();

  return ghostMesh;
}

// ============================================
// ZENOH NETWORK SETUP
// ============================================

async function initializeZenoh() {
  try {
    console.log('[GODVIEW] Connecting to Zenoh Router...');

    // Create Zenoh config for WebSocket connection
    const config = new Config();
    // Connect to Zenoh router's WebSocket endpoint
    config.connect.endpoints = ['ws://localhost:8000'];

    // Open Zenoh session
    const session = await Session.open(config);
    console.log('[GODVIEW] Zenoh session established');

    statusElement.textContent = 'CONNECTED';
    statusElement.style.color = '#00ff00';

    // Subscribe to hazard data
    const key = 'godview/zone1/hazards';
    console.log(`[GODVIEW] Subscribing to: ${key}`);

    const subscriber = await session.declareSubscriber(key, {
      callback: (sample) => {
        try {
          // Parse JSON payload
          const payload = sample.payload.deserialize();
          const data = JSON.parse(payload);

          console.log('[GODVIEW] Received Hazard:', data);

          // Extract agent ID from hazard packet
          const agentId = data.id;

          // Check if ghost already exists for this agent
          if (!ghosts.has(agentId)) {
            // Spawn new ghost
            console.log(`[GODVIEW] Spawning new ghost for agent: ${agentId}`);
            const newGhost = createGhost();
            scene.add(newGhost);
            ghosts.set(agentId, newGhost);
          }

          // Get the ghost for this agent
          const ghost = ghosts.get(agentId);

          // Update target position
          ghost.userData.targetPos.set(data.pos[0], data.pos[1], data.pos[2]);

          // Update timestamp
          const now = Date.now();
          ghost.userData.lastUpdate = now;

          // Calculate latency
          const latency = now - data.timestamp;
          latencyElement.textContent = latency;

          // Update status
          statusElement.textContent = `TRACKING ${ghosts.size} HAZARD(S)`;
          statusElement.style.color = '#ff0000';

        } catch (error) {
          console.error('[GODVIEW] Error processing message:', error);
        }
      }
    });

    console.log('[GODVIEW] Subscriber active. Waiting for hazards...');

  } catch (error) {
    console.error('[GODVIEW] Zenoh initialization failed:', error);
    statusElement.textContent = 'CONNECTION FAILED';
    statusElement.style.color = '#ff0000';
  }
}

// ============================================
// ANIMATION LOOP
// ============================================

function animate() {
  requestAnimationFrame(animate);

  const now = Date.now();
  const ghostsToRemove = [];

  // Iterate through all active ghosts
  for (const [agentId, ghost] of ghosts.entries()) {
    // Smooth interpolation (LERP) to target position
    ghost.position.lerp(ghost.userData.targetPos, LERP_FACTOR);

    // Check timeout
    const timeSinceUpdate = now - ghost.userData.lastUpdate;

    if (timeSinceUpdate > GHOST_TIMEOUT) {
      // Mark for removal (garbage collection)
      ghostsToRemove.push(agentId);
      console.log(`[GODVIEW] Removing stale ghost: ${agentId}`);
    } else {
      // Pulse effect for active ghosts
      const pulse = Math.sin(now * 0.005) * 0.05 + 1;
      ghost.userData.glowMesh.scale.set(pulse, pulse, pulse);

      // Fade in/out based on age
      if (timeSinceUpdate > GHOST_TIMEOUT - 500) {
        // Start fading out in last 500ms
        const fadeProgress = (timeSinceUpdate - (GHOST_TIMEOUT - 500)) / 500;
        ghost.userData.mainMaterial.opacity = Math.max(0, 0.8 * (1 - fadeProgress));
        ghost.userData.glowMaterial.opacity = Math.max(0, 0.3 * (1 - fadeProgress));
      } else {
        // Fade in
        ghost.userData.mainMaterial.opacity = Math.min(0.8, ghost.userData.mainMaterial.opacity + 0.05);
        ghost.userData.glowMaterial.opacity = Math.min(0.3, ghost.userData.glowMaterial.opacity + 0.02);
      }
    }
  }

  // Remove stale ghosts
  for (const agentId of ghostsToRemove) {
    const ghost = ghosts.get(agentId);
    scene.remove(ghost);
    ghosts.delete(agentId);
  }

  // Update status if no active ghosts
  if (ghosts.size === 0) {
    statusElement.textContent = 'SCANNING...';
    statusElement.style.color = '#00ff00';
  }

  // Render scene
  renderer.render(scene, camera);
}

// ============================================
// WINDOW RESIZE HANDLER
// ============================================

window.addEventListener('resize', () => {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(window.innerWidth, window.innerHeight);
});

// ============================================
// INITIALIZE SYSTEM
// ============================================

initializeZenoh();
animate();

console.log('[GODVIEW] System Online. X-Ray Vision Active.');
