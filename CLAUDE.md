# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based GUI application called "Search Devices" - a network ping scanner built with FLTK. The application allows users to scan network ranges (CIDR format) or specific IP addresses to check host availability.

## Key Architecture

### Project Structure
- `/search_devices/` - Main Rust project directory containing the application
- `src/main.rs` - Application entry point, initializes FLTK GUI and manages the event loop
- `src/cidr_tab.rs` - Handles CIDR network scanning functionality (e.g., 192.168.1.0/24)
- `src/ip_list_tab.rs` - Handles individual IP address list scanning

### Core Dependencies
- `fltk` - GUI framework with bundled feature for cross-platform compatibility
- `ipnetwork` - IP network manipulation and CIDR parsing
- `dns-lookup` - Hostname resolution functionality
- `clap` - Command-line argument parsing (though not actively used in GUI mode)

### Platform Considerations
- Primary target is Windows (`x86_64-pc-windows-msvc`)
- Uses `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to hide console in release builds
- Windows-specific process creation flags may be used for ping operations

## Common Development Commands

### Building and Running
```bash
# Development build with console output
cd search_devices
cargo run

# Release build (no console window on Windows)
cargo build --release
cargo run --release

# The executable will be at: target/release/search_devices.exe
```

### Documentation
```bash
# Generate documentation
cd search_devices
cargo doc --no-deps --open
```

### Checking Code
```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy
```

## Testing Approach
Currently, the project lacks unit tests. When adding tests:
- Place unit tests in the same file using `#[cfg(test)]` modules
- Integration tests would go in `search_devices/tests/` directory
- Run tests with `cargo test`

## GUI Development Notes
- The application uses FLTK's channel system for thread communication
- CIDR tab results are sent via channels: `(Ipv4Addr, bool, String)` representing (IP, alive status, hostname)
- Both tabs have independent scanning logic with Stop/Clear functionality
- Japanese language is used in the UI and documentation

## Release Process
- GitHub Actions workflow triggers on version tags (v*)
- Builds both debug and release versions
- Generates documentation archive
- Creates GitHub release with artifacts

## Important Patterns
- Always maintain Windows compatibility
- Use channels for thread-safe GUI updates
- Follow existing Japanese naming conventions in UI strings
- Respect the existing tab-based architecture when adding features