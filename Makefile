# Makefile for alphaNES project

# Define the target executable name
TARGET = alphaNES.exe

# Define the Rust source directory
SRC_DIR = src

# Define the build directory
BUILD_DIR = target

# Define the Rust compiler
RUSTC = cargo

# Default target
all: build

# Build the project
build:
    $(RUSTC) build --release --target x86_64-pc-windows-gnu
    cp $(BUILD_DIR)/x86_64-pc-windows-gnu/release/$(TARGET) .

# Clean the build artifacts
clean:
    $(RUSTC) clean

# Run the project
run: build
    ./$(TARGET)

.PHONY: all build clean run