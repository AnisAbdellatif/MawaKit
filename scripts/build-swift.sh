#!/usr/bin/env bash
# Builds the Swift package's binary parts:
#   bindings/swift/MawaKitFFI.xcframework        universal (arm64 + x86_64) static library + C headers
#   bindings/swift/Sources/MawaKit/MawaKit.swift  generated Swift API (UniFFI)
#
# The XCFramework is assembled by hand, so only the Command Line Tools are needed
# (`xcodebuild -create-xcframework` would require Xcode).
set -euo pipefail
cd "$(dirname "$0")/.."

export MACOSX_DEPLOYMENT_TARGET=14.0
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
OUT=bindings/swift
FRAMEWORK="$OUT/MawaKitFFI.xcframework"
SLICE="macos-arm64_x86_64"
GENERATED="target/uniffi-swift"

for target in "${TARGETS[@]}"; do
    rustup target list --installed | grep -qx "$target" || rustup target add "$target"
    cargo build --quiet --release --lib --features uniffi --target "$target"
done

# The interface is read from the compiled library itself (UniFFI "library mode").
host_target="aarch64-apple-darwin"
[[ "$(uname -m)" == "x86_64" ]] && host_target="x86_64-apple-darwin"
rm -rf "$GENERATED"
cargo run --quiet --features uniffi-cli --bin uniffi-bindgen -- generate \
    --library "target/$host_target/release/libmawakit.dylib" \
    --language swift --out-dir "$GENERATED" >/dev/null

rm -rf "$FRAMEWORK"
mkdir -p "$FRAMEWORK/$SLICE/Headers"
lipo -create $(printf 'target/%s/release/libmawakit.a ' "${TARGETS[@]}") -output "$FRAMEWORK/$SLICE/libmawakit.a"
cp "$GENERATED/MawaKitFFI.h" "$FRAMEWORK/$SLICE/Headers/"
cp "$GENERATED/MawaKitFFI.modulemap" "$FRAMEWORK/$SLICE/Headers/module.modulemap"
cp "$GENERATED/MawaKit.swift" "$OUT/Sources/MawaKit/MawaKit.swift"

cat > "$FRAMEWORK/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AvailableLibraries</key>
	<array>
		<dict>
			<key>HeadersPath</key>
			<string>Headers</string>
			<key>LibraryIdentifier</key>
			<string>$SLICE</string>
			<key>LibraryPath</key>
			<string>libmawakit.a</string>
			<key>SupportedArchitectures</key>
			<array>
				<string>arm64</string>
				<string>x86_64</string>
			</array>
			<key>SupportedPlatform</key>
			<string>macos</string>
		</dict>
	</array>
	<key>CFBundlePackageType</key>
	<string>XFWK</string>
	<key>XCFrameworkFormatVersion</key>
	<string>1.0</string>
</dict>
</plist>
PLIST

echo "Built $FRAMEWORK ($(lipo -archs "$FRAMEWORK/$SLICE/libmawakit.a")) and $OUT/Sources/MawaKit/MawaKit.swift"
