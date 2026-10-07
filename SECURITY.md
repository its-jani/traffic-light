# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in `traffic-status`, please report it privately by opening a Security Advisory on GitHub at https://github.com/its-jani/traffic-status/security/advisories.

Please do not disclose security issues in public issue trackers or discussion forums until a fix has been released.

## Security Architecture & Localhost Boundary

- `traffic-status` binds strictly to the loopback interface (`127.0.0.1`) and does not listen on public network interfaces.
- The IPC server requires `Content-Type: application/json` and `POST` requests for any state-mutating operations, preventing cross-origin simple requests from web browsers.
- No remote telemetry or tracking is embedded in the application.
