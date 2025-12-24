#!/bin/bash
# GodView Docker Quick Start
#
# Usage:
#   ./docker_run.sh          # Start interactive shell
#   ./docker_run.sh build    # Build the container
#   ./docker_run.sh rust     # Run Rust Rerun demo
#   ./docker_run.sh nuscenes # Run nuScenes demo

set -e

IMAGE_NAME="godview:latest"
CONTAINER_NAME="godview-dev"

# Allow X11 forwarding
xhost +local:docker 2>/dev/null || true

case "${1:-shell}" in
    build)
        echo "🔨 Building GodView Docker container..."
        docker build -t $IMAGE_NAME .
        echo "✅ Build complete!"
        ;;
    
    shell)
        echo "🚀 Starting GodView development shell..."
        docker run -it --rm \
            --gpus all \
            --name $CONTAINER_NAME \
            -e DISPLAY=$DISPLAY \
            -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
            -v $(pwd):/workspace \
            --network host \
            $IMAGE_NAME \
            /bin/bash
        ;;
    
    rust)
        echo "🦀 Running Rust Rerun demo..."
        docker run -it --rm \
            --gpus all \
            -e DISPLAY=$DISPLAY \
            -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
            -v $(pwd):/workspace \
            --network host \
            $IMAGE_NAME \
            bash -c "cd godview_core && cargo run --example rerun_demo --features visualization"
        ;;
    
    nuscenes)
        echo "🚗 Running nuScenes demo..."
        docker run -it --rm \
            --gpus all \
            -e DISPLAY=$DISPLAY \
            -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
            -v $(pwd):/workspace \
            --network host \
            $IMAGE_NAME \
            python3 -m nuscenes_dataset --seconds 30
        ;;
    
    *)
        echo "Usage: $0 {build|shell|rust|nuscenes}"
        exit 1
        ;;
esac
