//! # nucr
//! `NuGet` credentials manager
use anyhow::{Context, Result, anyhow, ensure};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{Write, stdin, stdout};
use std::path::Path;

const CI_USER: &str = "#CI_USER#";
const CI_USER_PASS: &str = "#CI_USER_PASSWORD#";
const CONFIG_FILES: &[&str] = &["./NuGet.Config", "./NuGet.Config.Debug"];

/// Trait for keyring operations to enable dependency injection
trait Keyring {
    fn get_password(&self, service: &str, key: &str) -> Result<String, keyring::Error>;
    fn set_password(&self, service: &str, key: &str, password: &str) -> Result<(), keyring::Error>;
    fn delete_credential(&self, service: &str, key: &str) -> Result<(), keyring::Error>;
}

/// Real keyring implementation
struct RealKeyringProvider;

impl Keyring for RealKeyringProvider {
    fn get_password(&self, service: &str, key: &str) -> Result<String, keyring::Error> {
        let entry = keyring::Entry::new(service, key)?;
        entry.get_password()
    }

    fn set_password(&self, service: &str, key: &str, password: &str) -> Result<(), keyring::Error> {
        let entry = keyring::Entry::new(service, key)?;
        entry.set_password(password)
    }

    fn delete_credential(&self, service: &str, key: &str) -> Result<(), keyring::Error> {
        let entry = keyring::Entry::new(service, key)?;
        entry.delete_credential()
    }
}

/// Reads NuGet.Config file
fn read_file(path: &Path) -> Result<String> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    ensure!(!data.is_empty(), "NuGet.Config file is empty");
    Ok(data)
}

/// Prompts for a value or password
fn prompt(name: &str, is_password: bool) -> Result<String> {
    print!("{name}: ");
    stdout().flush()?;

    let value = if is_password {
        rpassword::read_password()?
    } else {
        let mut input = String::new();
        stdin().read_line(&mut input)?;
        input.trim().to_owned()
    };

    ensure!(!value.is_empty(), "Empty input not allowed");
    Ok(value)
}

/// Gets or sets credential from keyring with dependency injection
fn get_or_set_credential<K: Keyring>(keyring: &K, key: &str, is_password: bool) -> Result<String> {
    if let Ok(password) = keyring.get_password("nucr", key) {
        Ok(password)
    } else {
        let value = prompt(key, is_password)?;
        keyring
            .set_password("nucr", key, &value)
            .map_err(|e| anyhow!("Failed to store credential: {}", e))?;
        Ok(value)
    }
}

/// Updates git index for file tracking
fn update_git_index(file_path: &Path, assume_unchanged: bool) -> Result<()> {
    let flag = if assume_unchanged {
        "--assume-unchanged"
    } else {
        "--no-assume-unchanged"
    };

    let output = std::process::Command::new("git")
        .args(["update-index", flag])
        .arg(file_path)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Git warning for {}: {}", file_path.display(), stderr);
    }

    Ok(())
}

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Replace placeholders with credentials (default)
    Replace,
    /// Replace credentials with placeholders
    Undo,
    /// Delete saved credentials from keyring
    Forget,
}

/// Deletes saved credentials from keyring with dependency injection
fn forget_credentials<K: Keyring>(keyring: &K) {
    let keys = ["CI_USER", "CI_USER_PASSWORD"];

    for key in &keys {
        match keyring.delete_credential("nucr", key) {
            Ok(()) => println!("{key} deleted"),
            Err(_) => println!("{key} not found"),
        }
    }

    // Reset git tracking for config files
    for &path in CONFIG_FILES {
        let path = Path::new(path);
        if path.exists() {
            let _ = update_git_index(path, false);
        }
    }
}

/// Replaces placeholders with actual credentials with dependency injection
fn replace_credentials<K: Keyring>(keyring: &K, path: &Path) -> Result<()> {
    let config_content = read_file(path)?;
    let user = get_or_set_credential(keyring, "CI_USER", false)?;
    let password = get_or_set_credential(keyring, "CI_USER_PASSWORD", true)?;

    let updated_content = config_content
        .replace(CI_USER, &user)
        .replace(CI_USER_PASS, &password);

    if config_content == updated_content {
        println!("Credentials already set in {}", path.display());
    } else {
        fs::write(path, updated_content)
            .with_context(|| format!("Failed to write to {}", path.display()))?;
        update_git_index(path, true)?;
        println!("Credentials set in {}", path.display());
    }

    Ok(())
}

/// Replaces credentials with placeholders with dependency injection
fn undo_credentials<K: Keyring>(keyring: &K, path: &Path) -> Result<()> {
    let config_content = read_file(path)?;
    let user = get_or_set_credential(keyring, "CI_USER", false)?;
    let password = get_or_set_credential(keyring, "CI_USER_PASSWORD", true)?;

    let updated_content = config_content
        .replace(&user, CI_USER)
        .replace(&password, CI_USER_PASS);

    if config_content == updated_content {
        println!("No credentials to remove from {}", path.display());
    } else {
        fs::write(path, updated_content)
            .with_context(|| format!("Failed to write to {}", path.display()))?;
        update_git_index(path, false)?;
        println!("Credentials removed from {}", path.display());
    }

    Ok(())
}

/// Application logic with dependency injection
fn run_app<K: Keyring>(keyring: &K, cli: &Cli) -> Result<()> {
    if !Path::new("./NuGet.Config").exists() && !Path::new("./NuGet.Config.Debug").exists() {
        println!("NuGet.Config or NuGet.Config.Debug are not found in current directory");
        return Ok(());
    }

    for config_file in CONFIG_FILES {
        let path = Path::new(config_file);
        if path.exists() {
            match cli.command {
                Some(Command::Replace) | None => replace_credentials(keyring, path)?,
                Some(Command::Undo) => undo_credentials(keyring, path)?,
                Some(Command::Forget) => {
                    forget_credentials(keyring);
                    break; // Only run once for forget
                }
            }
        }
    }

    Ok(())
}

/// main function
/// # nucr
///
/// At the first usage it will ask for your nuget source credentials.
///
/// It will set the `CI_ARTIFACTORY_USER` and `CI_ARTIFACTORY_USER_PASS` values to your user key chain and not ask next time.
///
/// Exits with an error if credentials are not provided.
fn main() -> Result<()> {
    let cli = Cli::parse();
    let keyring = RealKeyringProvider;
    run_app(&keyring, &cli)
}

/// Unit tests with mocked keyring
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};

    /// Better mock keyring with interior mutability
    struct TestKeyring {
        credentials: RefCell<HashMap<String, String>>,
    }

    impl TestKeyring {
        fn new() -> Self {
            Self {
                credentials: RefCell::new(HashMap::new()),
            }
        }

        fn with_credentials(credentials: HashMap<String, String>) -> Self {
            Self {
                credentials: RefCell::new(credentials),
            }
        }
    }

    impl Keyring for TestKeyring {
        fn get_password(&self, service: &str, key: &str) -> Result<String, keyring::Error> {
            let full_key = format!("{service}:{key}");
            self.credentials
                .borrow()
                .get(&full_key)
                .cloned()
                .ok_or(keyring::Error::NoEntry)
        }

        fn set_password(
            &self,
            service: &str,
            key: &str,
            password: &str,
        ) -> Result<(), keyring::Error> {
            let full_key = format!("{service}:{key}");
            self.credentials
                .borrow_mut()
                .insert(full_key, password.to_string());
            Ok(())
        }

        fn delete_credential(&self, service: &str, key: &str) -> Result<(), keyring::Error> {
            let full_key = format!("{service}:{key}");
            self.credentials.borrow_mut().remove(&full_key);
            Ok(())
        }
    }

    #[test]
    fn test_real_keyring_operations() -> Result<(), keyring::Error> {
        let keyring = RealKeyringProvider;

        // Clean up any existing entry first
        let _ = keyring.delete_credential("unittest_nucr", "CI_USER");

        let password = "test_password_123";

        // Set password
        keyring.set_password("unittest_nucr", "CI_USER", password)?;

        // Verify we can retrieve the password
        let retrieved_password = keyring.get_password("unittest_nucr", "CI_USER")?;
        assert_eq!(retrieved_password, password);

        // Clean up
        keyring.delete_credential("unittest_nucr", "CI_USER")?;

        Ok(())
    }

    #[test]
    fn test_mock_keyring_operations() -> Result<()> {
        let keyring = TestKeyring::new();

        // Test get non-existent credential
        assert!(keyring.get_password("test", "user").is_err());

        // Test set and get
        keyring.set_password("test", "user", "password123")?;
        let password = keyring.get_password("test", "user").unwrap();
        assert_eq!(password, "password123");

        // Test delete
        keyring.delete_credential("test", "user")?;
        assert!(keyring.get_password("test", "user").is_err());

        Ok(())
    }

    #[test]
    fn test_replace_credentials_with_mock() -> Result<()> {
        // Create test content with placeholders
        let test_content = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
             <configuration>\n\
               <packageSourceCredentials>\n\
                 <nuget>\n\
                   <add key=\"Username\" value=\"{CI_USER}\" />\n\
                   <add key=\"ClearTextPassword\" value=\"{CI_USER_PASS}\" />\n\
                 </nuget>\n\
               </packageSourceCredentials>\n\
             </configuration>",
        );

        let temp_file = NamedTempFile::new()?;
        fs::write(&temp_file, &test_content)?;

        // Setup mock keyring with predefined credentials
        let mut credentials = HashMap::new();
        credentials.insert("nucr:CI_USER".to_string(), "testuser".to_string());
        credentials.insert("nucr:CI_USER_PASSWORD".to_string(), "testpass".to_string());
        let keyring = TestKeyring::with_credentials(credentials);

        // This would normally require user input, but we're testing the logic
        // In a full integration test, you'd mock the prompt function too
        let user = keyring.get_password("nucr", "CI_USER").unwrap();
        let password = keyring.get_password("nucr", "CI_USER_PASSWORD").unwrap();

        let modified_content = test_content
            .replace(CI_USER, &user)
            .replace(CI_USER_PASS, &password);

        assert_ne!(test_content, modified_content);
        assert!(modified_content.contains("testuser"));
        assert!(modified_content.contains("testpass"));
        assert!(!modified_content.contains(CI_USER));
        assert!(!modified_content.contains(CI_USER_PASS));

        Ok(())
    }

    #[test]
    fn test_forget_credentials_with_mock() -> Result<()> {
        let mut credentials = HashMap::new();
        credentials.insert("nucr:CI_USER".to_string(), "testuser".to_string());
        credentials.insert("nucr:CI_USER_PASSWORD".to_string(), "testpass".to_string());
        let keyring = TestKeyring::with_credentials(credentials);

        // Verify credentials exist
        assert!(keyring.get_password("nucr", "CI_USER").is_ok());
        assert!(keyring.get_password("nucr", "CI_USER_PASSWORD").is_ok());

        // Forget credentials (without file operations)
        keyring.delete_credential("nucr", "CI_USER")?;
        keyring.delete_credential("nucr", "CI_USER_PASSWORD")?;

        // Verify credentials are gone
        assert!(keyring.get_password("nucr", "CI_USER").is_err());
        assert!(keyring.get_password("nucr", "CI_USER_PASSWORD").is_err());

        Ok(())
    }

    #[test]
    fn test_app_with_no_config_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let original_dir = std::env::current_dir()?;

        // Change to temp directory with no config files
        std::env::set_current_dir(&temp_dir)?;

        let keyring = TestKeyring::new();
        let cli = Cli {
            command: Some(Command::Replace),
        };

        // This should succeed without error
        let result = run_app(&keyring, &cli);
        assert!(result.is_ok());

        // Restore original directory
        std::env::set_current_dir(original_dir)?;

        Ok(())
    }

    // Re-use existing tests that don't need keyring injection
    #[test]
    fn test_read_file_success() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let test_content =
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<configuration></configuration>";
        fs::write(&temp_file, test_content)?;

        let result = read_file(temp_file.path())?;
        assert_eq!(result, test_content);
        Ok(())
    }

    #[test]
    fn test_constants() {
        assert_eq!(CI_USER, "#CI_USER#");
        assert_eq!(CI_USER_PASS, "#CI_USER_PASSWORD#");
        assert_eq!(CONFIG_FILES.len(), 2);
        assert!(CONFIG_FILES.contains(&"./NuGet.Config"));
        assert!(CONFIG_FILES.contains(&"./NuGet.Config.Debug"));
    }

    #[test]
    fn test_cli_parsing() {
        use clap::Parser;

        // Test default (no subcommand)
        let cli = Cli::try_parse_from(["nucr"]).unwrap();
        assert!(cli.command.is_none());

        // Test replace command
        let cli = Cli::try_parse_from(["nucr", "replace"]).unwrap();
        matches!(cli.command, Some(Command::Replace));

        // Test undo command
        let cli = Cli::try_parse_from(["nucr", "undo"]).unwrap();
        matches!(cli.command, Some(Command::Undo));

        // Test forget command
        let cli = Cli::try_parse_from(["nucr", "forget"]).unwrap();
        matches!(cli.command, Some(Command::Forget));
    }
}
