# Data Classification and Handling Policy

**Status:** ACTIVE
**Classification:** MANDATORY
**Last Updated:** 2025-01-08

## Classification Levels

### Confidential
- PII (Social Security numbers, financial records)
- Healthcare data (HIPAA protected)
- Trade secrets and intellectual property
- **Handling**: Encrypted at rest and in transit. Access logged. No removable media.

### Internal
- Internal communications
- Project documentation
- Employee directory
- **Handling**: Encrypted in transit. Access controlled by department.

### Public
- Marketing materials
- Published documentation
- Press releases
- **Handling**: No special controls required.

## Transfer Rules

- Confidential data: Encrypted channel ONLY (VPN + encrypted storage)
- Internal data: Company-approved tools only
- NEVER transfer any classified data via removable media or personal email
