# nucr - NUget CRedentials handler

NuGet Credentials Handler.
Utility to replace placeholders `#CI_USER#` and `#CI_USER_PASSWORD#` in the project's `NuGet.Config` and `NuGet.Config.Debug` files to your credentials or back to placeholders.

## Use case

Some internal projects include `NuGet.Config` and sometimes also `NuGet.Config.Debug` files, containing placeholders for username and password for the NuGet source. Because of this the dotnet CLI tools are not working within a directory. For example `dotnet restore` fails with HTTP error 407

## Build / Install

Requires rust tool chain installed. For example using https://rustup.rs/

```sh
cargo build --release
cargo test --release
cp target/release/nucr ~/.local/bin/
```

## Use

Try `nucr -h` for usage

## NOTE

Caution by careless `git push` with changed credentials. Already happened to me at least once. In such cases better change your password

## TODO

Hide the `NuGet.Config` changes from git to prevent commit containing credentials.
For now you can do it manually:

```sh
git update-index --assume-unchanged NuGet.Config
git update-index --no-assume-unchanged NuGet.Config
```
