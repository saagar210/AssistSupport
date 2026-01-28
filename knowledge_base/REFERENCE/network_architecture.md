# Network Architecture Overview

## Network Segments

- **Corporate LAN**: Office workstations, printers, meeting rooms
- **Guest Wi-Fi**: Isolated from corporate network, internet-only
- **Server VLAN**: Internal servers, databases (restricted access)
- **DMZ**: Public-facing services (web servers, email gateways)
- **VPN Segment**: Remote access users (full corporate access via tunnel)

## Security Layers

1. **Perimeter firewall**: Blocks unauthorized inbound/outbound traffic
2. **IDS/IPS**: Intrusion detection and prevention at network edge
3. **NAC**: Network Access Control — only compliant devices connect
4. **DNS filtering**: Blocks known malicious domains
5. **DLP**: Data Loss Prevention monitors outbound data transfers

## Wi-Fi Networks

| SSID | Purpose | Auth | Internet | Corporate |
|------|---------|------|----------|-----------|
| Corp-Secure | Employees | 802.1X (cert) | Yes | Yes |
| Corp-Guest | Visitors | Captive portal | Yes | No |
| Corp-IoT | Devices | WPA2-PSK | Limited | No |
