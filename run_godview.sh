#!/bin/bash

# ============================================
# GODVIEW MVP LAUNCHER
# The Live Reality Protocol - Orchestration
# ============================================

set -e  # Exit on error

echo ""
echo "╔════════════════════════════════════════════╗"
echo "║     GODVIEW - LIVE REALITY PROTOCOL        ║"
echo "║     Distributed X-Ray Vision System        ║"
echo "╚════════════════════════════════════════════╝"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo "[GODVIEW] Shutting down all components..."
    kill $PID_ZENOH $PID_AGENT $PID_VIEWER 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

# ============================================
# STEP 1: START ZENOH ROUTER (V1)
# ============================================

echo "--- [GODVIEW] STARTING ZENOH ROUTER (V1) ---"
echo "    TCP Port: 7447"
echo "    WebSocket Port: 8000"
echo ""

zenohd --listen tcp/0.0.0.0:7447 --rest-http-port 8000 &
PID_ZENOH=$!

echo "[GODVIEW] Zenoh Router PID: $PID_ZENOH"
echo "[GODVIEW] Waiting for router to initialize..."
sleep 2

# ============================================
# STEP 2: START RUST AGENT (X-RAY EMITTER)
# ============================================

echo ""
echo "--- [GODVIEW] STARTING RUST AGENT ---"
echo "    Component: X-Ray Emitter"
echo "    Vision: OpenCV Haar Cascade"
echo "    Publisher: godview/zone1/hazards"
echo ""

cd agent
cargo run --release &
PID_AGENT=$!
cd ..

echo "[GODVIEW] Rust Agent PID: $PID_AGENT"
sleep 2

# ============================================
# STEP 3: START WEB VIEWER (GOD VIEW)
# ============================================

echo ""
echo "--- [GODVIEW] STARTING WEB VIEWER ---"
echo "    Framework: Vite + Three.js"
echo "    Subscriber: godview/zone1/hazards"
echo "    Mode: X-Ray Vision"
echo ""

cd viewer

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "[GODVIEW] Installing npm dependencies..."
    npm install
fi

npm run dev &
PID_VIEWER=$!
cd ..

echo "[GODVIEW] Web Viewer PID: $PID_VIEWER"

# ============================================
# SYSTEM READY
# ============================================

echo ""
echo "╔════════════════════════════════════════════╗"
echo "║          GODVIEW SYSTEM ONLINE             ║"
echo "╚════════════════════════════════════════════╝"
echo ""
echo "📡 Zenoh Router:  Running (PID: $PID_ZENOH)"
echo "🦀 Rust Agent:    Running (PID: $PID_AGENT)"
echo "🌐 Web Viewer:    Running (PID: $PID_VIEWER)"
echo ""
echo "🔗 Open your browser to the Vite URL shown above"
echo "   (typically http://localhost:5173)"
echo ""
echo "👁️  Position yourself in front of the webcam"
echo "   to see the Red Ghost avatar in 3D space"
echo ""
echo "Press Ctrl+C to shutdown all components"
echo ""

# Wait for all processes
wait $PID_ZENOH $PID_AGENT $PID_VIEWER
