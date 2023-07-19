use anyhow::{ensure, Context, Error};
use clap::{Parser, Subcommand};
use std::{env, fs, io::Write, path::Path};
extern crate keyring;

static CI_USER: &str = "#CI_USER#";
static CI_USER_PASS: &str = "#CI_USER_PASSWORD#";

/// reads NuGet.Config file
fn read_file(path: &str) -> anyhow::Result<String, Error> {
    let cwd = env::current_dir()?;
    let data = fs::read_to_string(path).with_context(|| {
        format!(
            "File {} not found in current directory {}",
            path,
            cwd.display()
        )
    })?;
    ensure!(!data.is_empty(), "NuGet.Conf file found but is empty");
    Ok(data)
}

/// prompt for a value or password
fn prompt(name: &str, shadowed: bool) -> anyhow::Result<String, Error> {
    print!("{name}: ");
    std::io::stdout().flush().unwrap();

    if shadowed {
        let value = rpassword::read_password()?;
        Ok(value)
    } else {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let value = line.trim().to_string();
        Ok(value)
    }
}

/// reads or asks and saves the value for `CI_USER` or `CI_USER_PASS` to users key chain
fn get_or_set(username: &str, shadowed: bool) -> anyhow::Result<String, Error> {
    let entry = keyring::Entry::new("nucr", username)?;
    let p = entry.get_password();
    if let Ok(p) = p {
        Ok(p)
    } else {
        let value = prompt(username, shadowed)?;
        entry.set_password(&value)?;
        Ok(value)
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// What mode to run the program in
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// (default) Replace credentials in NuGet.Conf, prompt if not yet known
    Replace,
    /// Replace credentials with placeholders, prompt if not yet known
    Undo,
    /// Forget saved credentials
    Forget,
}

fn forget() -> Result<(), Error> {
    for name in &["CI_USER", "CI_USER_PASSWORD"] {
        let entry = keyring::Entry::new("nucr", name)?;
        if entry.get_password().is_ok() {
            entry.delete_password()?;
            println!("{name} deleted");
        } else {
            println!("{name} not found");
        }
    }
    Ok(())
}

fn replace(path: &str) -> Result<(), Error> {
    let nuget_config = read_file(path)?;
    let user = get_or_set("CI_USER", false)?;
    let pass = get_or_set("CI_USER_PASSWORD", true)?;
    let new_data = nuget_config
        .replace(CI_USER, &user)
        .replace(CI_USER_PASS, &pass);
    if nuget_config == new_data {
        println!("Credentials are already set to {path}");
    } else {
        fs::write(path, new_data)?;
        println!("Credentials are set to {path}");
    };
    Ok(())
}

fn undo(path: &str) -> Result<(), Error> {
    let nuget_config = read_file(path)?;
    let user = get_or_set("CI_USER", false)?;
    let pass = get_or_set("CI_USER_PASSWORD", true)?;
    let new_data = nuget_config
        .replace(&*user, CI_USER)
        .replace(&*pass, CI_USER_PASS);
    if nuget_config == new_data {
        println!("No credentials to remove from {path}");
    } else {
        fs::write(path, new_data)?;
        println!("Credentials are removed from {path}");
    };
    Ok(())
}

mod tests {

    #[test]
    fn test_get_or_set_var() {
        let entry = keyring::Entry::new("unittest_nucr", "CI_USER").unwrap();
        let p = entry.get_password();
        if p.is_ok() {
            entry.delete_password().unwrap();
        }
        let password = "password";
        entry.set_password(password).unwrap();
        let p = entry.get_password().unwrap();
        assert_eq!(p, password);
        entry.delete_password().unwrap();
    }
}

/// main function
/// # nucr
///
/// At the first usage it will ask for your nuget source credentials.
///
/// It will set the `CI_ARTIFACTORY_USER` and `CI_ARTIFACTORY_USER_PASS` values to your user key chain and not ask next time.
///
/// Exits with an error if credentials are not provided.
fn main() -> anyhow::Result<(), Error> {
    let cli = Cli::parse();
    if !Path::new("./NuGet.Config").exists() && !Path::new("./NuGet.Config.Debug").exists() {
        println!("NuGet.Config or NuGet.Config.Debug are not found in current directory");
        return Ok(());
    }

    for path in &["./NuGet.Config", "./NuGet.Config.Debug"] {
        if Path::new(path).exists() {
            match cli.command {
                Some(Commands::Replace) | None => replace(path)?,
                Some(Commands::Undo) => undo(path)?,
                Some(Commands::Forget) => forget()?,
            }
        }
    }

    Ok(())
}
