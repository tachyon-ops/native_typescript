#!/usr/bin/env bash
set -e

echo "Starting SDL3 compilation and installation..."
echo "This will clone SDL3 to a temporary directory, build it with CMake, and install it to /usr/local."
echo ""

# Ensure we have build dependencies
echo "Installing prerequisites (requires sudo)..."
sudo apt-get update && sudo apt-get install -y cmake ninja-build build-essential git libasound2-dev libpulse-dev libaudio-dev libjack-dev libsndio-dev libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev libxss-dev libxkbcommon-dev libdrm-dev libgbm-dev libgl1-mesa-dev libgles2-mesa-dev libegl1-mesa-dev libdbus-1-dev libibus-1.0-dev libudev-dev fcitx-libs-dev libxtst-dev

TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Cloning SDL3..."
git clone https://github.com/libsdl-org/SDL.git -b release-3.4.4
cd SDL

echo "Configuring with CMake..."
mkdir build
cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DSDL_STATIC=OFF -DSDL_X11_XTEST=OFF

echo "Building SDL3..."
cmake --build . --config Release --parallel "$(nproc)"

echo "Installing SDL3 (requires sudo)..."
sudo cmake --install . --config Release

echo "Refreshing shared library cache..."
sudo ldconfig

echo "SDL3 installation complete!"
echo "Cleaning up temporary directory..."
cd ~
rm -rf "$TEMP_DIR"
