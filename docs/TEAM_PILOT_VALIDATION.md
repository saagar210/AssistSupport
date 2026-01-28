# Team Pilot Validation Test Queries

20 structured test queries across 4 groups for validating KB ingestion, search ranking, and policy enforcement.

## Group 1: Policy Enforcement (5 queries)

Test that policy-related questions surface POLICIES/ documents first.

| # | Query | Expected Top Result | Expected Behavior |
|---|-------|-------------------|-------------------|
| 1 | "Can I use a USB flash drive?" | POLICIES/removable_media_policy | Denied — flash drives prohibited, suggest OneDrive |
| 2 | "Are external hard drives allowed?" | POLICIES/removable_media_policy | Denied — all removable media prohibited |
| 3 | "Can I install Slack on my laptop?" | POLICIES/software_installation_policy | Denied unless approved — check approved catalog |
| 4 | "Is it okay to use a personal VPN?" | POLICIES/network_security_policy | Denied — only corporate VPN permitted |
| 5 | "Can I bring my own keyboard?" | POLICIES/hardware_policy | Check policy — BYOD peripherals may be restricted |

## Group 2: No Exceptions (5 queries)

Test that exception-seeking queries still enforce policy.

| # | Query | Expected Top Result | Expected Behavior |
|---|-------|-------------------|-------------------|
| 6 | "Can I use a flash drive just for today?" | POLICIES/removable_media_policy | Denied — no temporary exceptions |
| 7 | "My manager approved a USB stick, is that okay?" | POLICIES/removable_media_policy | Denied — manager approval does not override policy |
| 8 | "Is there an exception for encrypted USB drives?" | POLICIES/removable_media_policy | Denied — no exceptions regardless of encryption |
| 9 | "Can someone else use their USB on my behalf?" | POLICIES/removable_media_policy | Denied — policy applies to all personnel |
| 10 | "What if I only need to copy one small file via USB?" | POLICIES/removable_media_policy | Denied — use OneDrive instead |

## Group 3: Procedures (5 queries)

Test that how-to questions surface PROCEDURES/ documents first.

| # | Query | Expected Top Result | Expected Behavior |
|---|-------|-------------------|-------------------|
| 11 | "How do I request a new laptop?" | PROCEDURES/laptop_request | Step-by-step process via IT portal |
| 12 | "How do I share files with a colleague?" | PROCEDURES/file_transfer_procedure | OneDrive/SharePoint sharing steps |
| 13 | "How do I request software installation?" | PROCEDURES/software_request_procedure | Self-service portal steps |
| 14 | "How do I reset my password?" | PROCEDURES/password_reset | Self-service portal or IT help desk |
| 15 | "How do I set up VPN access?" | PROCEDURES/vpn_setup | VPN client download and configuration steps |

## Group 4: Alternatives & Reference (5 queries)

Test that general/reference queries return appropriate results.

| # | Query | Expected Top Result | Expected Behavior |
|---|-------|-------------------|-------------------|
| 16 | "What cloud storage options do we have?" | REFERENCE/cloud_storage_options | List of approved platforms |
| 17 | "What laptop models are available?" | REFERENCE/device_specs | Approved laptop models and specs |
| 18 | "What is the alternative to USB drives?" | PROCEDURES/file_transfer_procedure | OneDrive, SharePoint, Teams |
| 19 | "Who do I contact for IT support?" | REFERENCE/contact_info | Help desk contact information |
| 20 | "What peripherals are provided?" | REFERENCE/device_specs | Monitor, keyboard, mouse |

## Scoring Criteria

For each query, evaluate:

1. **Relevance** (1-5): Does the top result answer the question?
2. **Category Match** (Pass/Fail): Is the result from the expected category (POLICIES/PROCEDURES/REFERENCE)?
3. **Policy Enforcement** (Pass/Fail): For Groups 1-2, does the response deny the request and cite policy?
4. **Alternatives Provided** (Pass/Fail): Does the response suggest approved alternatives?

## Pass Criteria

- Groups 1-2: 90% of queries return POLICIES/ as top result
- Group 3: 80% of queries return PROCEDURES/ as top result
- Group 4: 80% of queries return relevant REFERENCE/ or PROCEDURES/ result
- Overall relevance score average >= 3.5/5
