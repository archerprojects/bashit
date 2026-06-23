#!/bin/bash
# BashIt — .deb package build script
# Developer: archerprojects <archer.projects@proton.me>
#
# Run from app root: ./build.sh
# Reads current version from Cargo.toml, builds .deb to dist/
# Version is managed manually in Cargo.toml — bump it there before release.
#
# Requirements:
#   - Rust stable 1.77+
#   - dpkg-deb (sudo apt install dpkg)

set -e

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

echo "[0/5] Building version: ${VERSION}"

PKG="bashit_${VERSION}_amd64"
BUILD_DIR="/tmp/${PKG}"
DIST_DIR="dist"

echo "[1/5] Building release binary..."
cargo build --release

echo "[2/5] Staging package tree..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/DEBIAN"
mkdir -p "$BUILD_DIR/usr/bin"
mkdir -p "$BUILD_DIR/usr/share/applications"

for size in 16 22 24 32 48 64 128 256; do
    mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/${size}x${size}/apps"
done
mkdir -p "$BUILD_DIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$BUILD_DIR/usr/share/icons/lean-icons/apps/scalable"

cp target/release/bashit                    "$BUILD_DIR/usr/bin/bashit"
cp packaging/debian/control                 "$BUILD_DIR/DEBIAN/control"
cp packaging/debian/postinst                "$BUILD_DIR/DEBIAN/postinst"
cp packaging/desktop/bashit.desktop         "$BUILD_DIR/usr/share/applications/bashit.desktop"

for size in 16 22 24 32 48 64 128 256; do
    cp "packaging/icons/bashit-${size}.png" \
       "$BUILD_DIR/usr/share/icons/hicolor/${size}x${size}/apps/bashit.png"
done

cp packaging/icons/bashit.svg "$BUILD_DIR/usr/share/icons/hicolor/scalable/apps/bashit.svg"
cp packaging/icons/bashit.svg "$BUILD_DIR/usr/share/icons/lean-icons/apps/scalable/bashit.svg"

echo "[3/5] Setting permissions..."
chmod 755 "$BUILD_DIR/DEBIAN/postinst"
chmod 755 "$BUILD_DIR/usr/bin/bashit"

echo "[4/5] Building .deb..."
mkdir -p "$DIST_DIR"
rm -f ${DIST_DIR}/bashit_*.deb
dpkg-deb --build "$BUILD_DIR" "${DIST_DIR}/${PKG}.deb"

echo "[5/5] Done: ${DIST_DIR}/${PKG}.deb  (v${VERSION})"
