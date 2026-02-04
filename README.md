# nucr - NUget CRedentials handler

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/nucr)](https://crates.io/crates/nucr)
[![Build Status](https://github.com/vilinski/nucr/actions/workflows/rust.yml/badge.svg)](https://github.com/vilinski/nucr/actions/workflows/rust.yml)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

NuGet Credentials Handler.
Utility to replace placeholders `#CI_USER#` and `#CI_USER_PASSWORD#` in the project's `NuGet.Config` and `NuGet.Config.Debug` files to your credentials or back to placeholders.

## Use case

Some internal projects include `NuGet.Config` and sometimes also `NuGet.Config.Debug` files, containing placeholders for username and password for the NuGet source. Because of this the dotnet CLI tools are not working within a directory. For example `dotnet restore` fails with HTTP error 407

## Installation

### Via Cargo (Recommended)

```bash
cargo install --git https://github.com/vilinski/nucr
```

### Pre-built Binaries

Download from the [Releases page](https://github.com/vilinski/nucr/releases)

```sh
# Linux
curl -L https://github.com/vilinski/nucr/releases/latest/download/nucr-x86_64-unknown-linux-gnu -o nucr

# macOS (Intel)
curl -L https://github.com/vilinski/nucr/releases/latest/download/nucr-x86_64-apple-darwin -o nucr
# macOS (Apple Silicon)
curl -L https://github.com/vilinski/nucr/releases/latest/download/nucr-aarch64-apple-darwin -o nucr

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/vilinski/nucr/releases/latest/download/nucr-x86_64-pc-windows-msvc.exe" -OutFile "nucr.exe"
```

## Usage

### Basic Commands

Replace placeholders with your credentials (default action):
```bash
nucr
# or explicitly:
nucr replace
```

This will:
1. Prompt for your NuGet username (if not already saved)
2. Prompt for your NuGet password (if not already saved)
3. Replace `#CI_USER#` and `#CI_USER_PASSWORD#` in `NuGet.Config` and `NuGet.Config.Debug`
4. Store credentials securely in your system keyring for future use

Revert credentials back to placeholders:
```bash
nucr undo
```

Delete saved credentials from keyring:
```bash
nucr forget
```

### First-time Setup

On first run, `nucr` will prompt you for:
- **CI_USER**: Your NuGet feed username
- **CI_USER_PASSWORD**: Your NuGet feed password

These credentials are stored securely in your operating system's credential manager:
- **macOS**: Keychain
- **Windows**: Credential Manager
- **Linux**: Secret Service (GNOME Keyring, KDE Wallet, etc.)

Subsequent runs will use the stored credentials without prompting.

### Example Workflow

```bash
# Navigate to your .NET project directory
cd /path/to/your/project

# Replace placeholders with your credentials
nucr

# Now you can use dotnet commands
dotnet restore
dotnet build

# When done, revert to placeholders
nucr undo
```

## Security Notes

⚠️ **Important**: Be careful not to commit credentials to git. If you accidentally push credentials, **change your password immediately**.

To prevent accidental commits, `nucr` automatically configures git to ignore changes to credential files:
- When setting credentials: `git update-index --assume-unchanged NuGet.Config`
- When reverting: `git update-index --no-assume-unchanged NuGet.Config`

### Best Practices

- Always run `nucr undo` before committing changes
- Use `.gitignore` if you want to completely ignore these files
- Regularly review your git history for accidental credential commits
- Use `nucr forget` to remove credentials from your keyring when no longer needed

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

To report a security vulnerability, please see [SECURITY.md](SECURITY.md).

## License

[MIT License](https://opensource.org/licenses/MIT)

## Links

- [GitHub Repository](https://github.com/vilinski/nucr)
- [Crates.io](https://crates.io/crates/nucr)
- [Issues](https://github.com/vilinski/nucr/issues)