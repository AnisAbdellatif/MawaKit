.PHONY: test test-rust test-swift lint swift fmt hooks clean

# Work around SDKs the Command Line Tools can't build Swift with (see scripts/toolchain.sh).
FALLBACK_SDK := $(shell scripts/toolchain.sh)
ifneq ($(FALLBACK_SDK),)
export SDKROOT := $(FALLBACK_SDK)
endif

# With only the Command Line Tools, SwiftPM needs help finding swift-testing.
CLT := /Library/Developer/CommandLineTools
ifeq ($(shell xcode-select -p),$(CLT))
SWIFT_TEST_FLAGS := -Xswiftc -F -Xswiftc $(CLT)/Library/Developer/Frameworks \
	-Xswiftc -plugin-path -Xswiftc $(CLT)/usr/lib/swift/host/plugins/testing \
	-Xlinker -F -Xlinker $(CLT)/Library/Developer/Frameworks \
	-Xlinker -rpath -Xlinker $(CLT)/Library/Developer/Frameworks \
	-Xlinker -rpath -Xlinker $(CLT)/Library/Developer/usr/lib
endif

# Everything: the Rust core against the spec, then the Swift bindings against it.
test: test-rust test-swift

test-rust:
	cargo test --quiet

test-swift: swift
	swift test $(SWIFT_TEST_FLAGS)

lint:
	cargo fmt --check
	cargo clippy --quiet --all-targets --features uniffi-cli -- -D warnings

fmt:
	cargo fmt

# Universal static library + generated Swift API for the Swift package.
swift:
	scripts/build-swift.sh

# Lint and test before commits; the Swift bindings too before pushes.
hooks:
	git config core.hooksPath .githooks
	@echo "Git hooks enabled (.githooks)."

clean:
	cargo clean
	rm -rf .build bindings/swift/MawaKitFFI.xcframework bindings/swift/Sources/MawaKit/MawaKit.swift
