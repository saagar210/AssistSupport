# VPN Usage Requirements

**Status:** ACTIVE
**Classification:** MANDATORY
**Last Updated:** 2025-01-10

## Policy Statement

All employees accessing company resources from outside the corporate network **MUST** use the company-provided VPN. No exceptions for any role or access method.

## Requirements

1. **Always-on VPN**: The VPN client must be connected before accessing any company resource
2. **Split tunneling**: Disabled — all traffic routes through VPN when connected
3. **MFA required**: Multi-factor authentication is mandatory for VPN login
4. **Approved client only**: Only the company-issued VPN client (GlobalProtect) is permitted
5. **Auto-connect on boot**: VPN must be configured to connect automatically on system startup
6. **Timeout policy**: Sessions auto-disconnect after 12 hours of inactivity

## Who Must Use VPN

- All remote employees
- All employees working from home
- All employees at client sites
- Contractors with network access
- Temporary staff

## Compliance

Failure to use VPN when accessing company resources remotely constitutes a security violation and will be escalated to management.
