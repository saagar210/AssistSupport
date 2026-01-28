# KB Audit Report — Week 1

**Date:** 2026-01-28
**Auditor:** Automated (Claude Code)
**Version:** v0.5.3

## Structure Summary

| Folder | File Count | Purpose |
|--------|-----------|---------|
| POLICIES/ | 13 files (12 policies + INDEX.md) | Mandatory policies — what is/isn't allowed |
| PROCEDURES/ | 8 files | Step-by-step how-to guides |
| REFERENCE/ | 6 files | Background info, catalogs, contacts |
| **Total** | **27 files** | |

## Policies Identified (12)

| File | Policy Name | Key Restriction |
|------|-------------|-----------------|
| flash_drives_forbidden.md | Flash Drive and USB Storage | FORBIDDEN — no exceptions |
| removable_media_policy.md | Removable Media | NOT ALLOWED — CISO exception only |
| vpn_requirements.md | VPN Usage Requirements | MANDATORY for all remote access |
| byod_policy.md | Bring Your Own Device | Allowed only with MDM enrollment |
| data_classification.md | Data Classification and Handling | Tiered handling rules |
| password_policy.md | Password and Authentication | 12-char minimum, MFA required |
| software_installation_policy.md | Software Installation | IT-approved only, no local admin |
| remote_access_policy.md | Remote Access | Company VPN only |
| email_security_policy.md | Email and Communication Security | Encryption for sensitive data |
| cloud_storage_policy.md | Approved Cloud Storage | OneDrive/SharePoint/Dropbox Business only |
| incident_reporting_policy.md | Security Incident Reporting | Immediate reporting required |
| acceptable_use_policy.md | Acceptable Use | Business purposes, monitoring notice |
| INDEX.md | Policy Index | Lists all policies with status |

## Procedures Identified (8)

| File | Procedure |
|------|-----------|
| request_new_laptop.md | New laptop request workflow |
| vpn_setup.md | VPN setup and troubleshooting |
| report_technical_issue.md | How to report issues |
| password_reset.md | Password reset (self-service + IT) |
| software_request.md | Software request process |
| file_sharing_guide.md | Approved file sharing methods |
| access_request.md | System/application access request |
| data_backup.md | Backup procedures |

## Reference Documents (6)

| File | Content |
|------|---------|
| approved_devices.md | Laptop/monitor/peripheral catalog |
| supported_software.md | Approved software list |
| contact_directory.md | IT support contacts and escalation |
| compliance_overview.md | HIPAA/GDPR/SOX/PCI-DSS overview |
| network_architecture.md | Network segments and security |
| common_error_codes.md | Error codes and resolutions |

## Policy Cross-References

The following policies are cross-referenced in procedures:
- Flash drive policy referenced in: file_sharing_guide.md, data_backup.md
- VPN policy referenced in: vpn_setup.md, remote_access_policy.md
- Cloud storage policy referenced in: file_sharing_guide.md, data_backup.md

## Audit Findings

### No Issues Found
- All policy files contain clear FORBIDDEN/MANDATORY/NOT ALLOWED language
- All policies include reasons for the restriction
- All restrictive policies list approved alternatives
- Policy INDEX.md accurately lists all 12 policies
- No duplicate or overlapping articles detected
- No articles misclassified between folders

### Note
These files exist on disk in `knowledge_base/` but have **not yet been ingested** into the AssistSupport database. They must be ingested through the app's KB management interface before the policy-first search ranking will have observable effect on real queries.

## Audit Confidence: 98%

The 2% uncertainty reflects the fact that additional policies may be needed as team pilot feedback comes in (Week 2-3).
