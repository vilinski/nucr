# Security Policy

## Supported Versions

We actively support the latest release and provide security updates for it.

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it privately to **<vilinski@yahoo.de>**.

**Please do not** report security vulnerabilities through public GitHub issues.

When reporting, please include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will acknowledge receipt of your report within 48 hours and provide a detailed response indicating the next steps in handling your report within 7 days.

## Security Considerations

This tool handles sensitive credentials (NuGet usernames and passwords). Security measures include:

- Credentials are stored in the operating system's secure keyring/credential store
- Credentials are never logged or printed to console
- Git tracking is automatically configured to prevent accidental commits of credentials
