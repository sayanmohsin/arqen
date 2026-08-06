# Security policy

Arqen is early-stage. Do not disclose credentials, private endpoints, or
customer data in issues or pull requests.

Only the latest release on `main` receives active security fixes. Pin a version
in production and review the changelog before upgrading.

For a suspected vulnerability, use [GitHub private vulnerability reporting](https://github.com/sayanmohsin/arqen/security/advisories/new).
Include the affected version, reproduction steps, impact, and any safe
mitigation. Please allow time for triage before public disclosure.

Keep provider and cloud credentials server-side, use explicit authorization
metadata for tools, and review logs for secret leakage before sharing them.
