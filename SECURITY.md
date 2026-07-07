# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Monogatari, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please email: sakalioling@rankchord.com

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

- Acknowledgment within 48 hours
- Status update within 7 days
- Fix released within 30 days (for confirmed vulnerabilities)

## Scope

The following are in scope:
- Tauri desktop application vulnerabilities
- Rust backend code injection or memory safety issues
- API key exposure or credential leakage
- Cross-site scripting in the Vue frontend
- Dependency supply chain vulnerabilities

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.6.x   | Yes |
| < 0.6   | No |

## Best Practices for Users

- Keep your API keys secure and never commit them to version control
- Use environment variables for sensitive configuration
- Regularly update to the latest version
- Review the release checklist before deploying to production