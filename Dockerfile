# GodView Development Container
# 
# Includes:
# - Rust toolchain + godview_core dependencies
# - Python 3.10 with Rerun SDK + nuScenes
# - All visualization libraries
# - CARLA Python API support (optional)
#
# Build: docker build -t godview:latest .
# Run:   docker run -it --rm -v $(pwd):/workspace -e DISPLAY=$DISPLAY godview:latest

FROM nvidia/cuda:12.2.0-runtime-ubuntu22.04

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    cmake \
    pkg-config \
    git \
    curl \
    wget \
    # Python
    python3.10 \
    python3.10-venv \
    python3-pip \
    # GUI support (for Rerun viewer)
    libxkbcommon0 \
    libxcb1 \
    libx11-6 \
    libgl1-mesa-glx \
    libglib2.0-0 \
    # Rust dependencies
    libssl-dev \
    libclang-dev \
    # Cleanup
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Create Python virtual environment with compatible versions
RUN python3 -m venv /opt/venv
ENV PATH="/opt/venv/bin:${PATH}"

# Install Python dependencies with pinned versions
# Rerun 0.28 requires NumPy 2.x
RUN pip install --upgrade pip && \
    pip install \
    "numpy>=2.0" \
    "scipy>=1.13" \
    "matplotlib>=3.7" \
    "pillow>=9.0" \
    "pyarrow>=18.0" \
    "attrs>=23.1" \
    "typing-extensions>=4.5"

# Install Rerun SDK 0.28 (latest, works with nuScenes example)
RUN pip install "rerun-sdk>=0.28"

# Install nuScenes devkit
RUN pip install nuscenes-devkit

# Install additional ML/CV libraries
RUN pip install \
    opencv-python-headless \
    scikit-learn \
    tqdm

# Clone Rerun examples and install nuScenes demo module
RUN git clone --depth 1 --branch latest https://github.com/rerun-io/rerun.git /opt/rerun && \
    pip install -e /opt/rerun/examples/python/nuscenes_dataset

# Force reinstall NumPy 2.x (nuscenes-devkit downgrades to 1.x which breaks Rerun 0.28)
RUN pip install "numpy>=2.0" --force-reinstall

# Optional: CARLA Python API placeholder
# To use CARLA, mount the CARLA PythonAPI folder and add to PYTHONPATH
# e.g., -v /path/to/CARLA/PythonAPI:/opt/carla-api
# ENV PYTHONPATH="/opt/carla-api/carla/dist/carla-0.9.15-py3.10-linux-x86_64.egg:${PYTHONPATH}"

# Set working directory
WORKDIR /workspace

# Build godview_core on first run (optional)
# The actual cargo build happens when you need it

# Default command
CMD ["/bin/bash"]
