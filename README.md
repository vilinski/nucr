# nucr - NUget CRedentials handler

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

## Use

Try `nucr -h` for usage

## NOTE

Caution by careless `git push` with changed credentials. In such case better change your password.
By changing or reverting a file, nucr tries to run these git commands, to prevent accidental password leaks:

```sh
git update-index --assume-unchanged NuGet.Config
git update-index --no-assume-unchanged NuGet.Config
```

## License

[MIT License](https://opensource.org/licenses/MIT)