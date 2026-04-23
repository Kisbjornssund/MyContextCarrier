# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| latest (pre-alpha) | Yes |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report vulnerabilities by opening a private security advisory on GitHub. Include:

- A descriptive title and severity assessment (low / medium / high / critical)
- The component affected (core daemon, MCP server, privacy rules engine, a specific collector, SDK)
- Steps to reproduce the issue
- What an attacker could achieve by exploiting it
- Your environment (OS, version, install method)
- If available, a suggested fix or mitigation

You will receive an acknowledgment within 48 hours and a fuller response within 7 days indicating next steps.

## Disclosure Policy

We follow coordinated disclosure. Once a fix is confirmed:

1. A patch is prepared and tested
2. A GitHub Security Advisory is published with the fix
3. A new release is tagged and announced
4. Credit is given to the reporter unless they prefer to remain anonymous

We ask that you give us a reasonable time to address the issue before any public disclosure.

## Scope

MyContextPort is local-first software. The primary security concern is ensuring that:

- Data never leaves the user's machine unless explicitly configured to do so
- Encryption keys are never transmitted or stored in plaintext
- Collectors do not make unauthorized network requests
- The MCP server does not expose more context than the user's privacy rules permit
- The install script does not execute unexpected code

Reports related to these properties are given highest priority.

## Out of Scope

- Issues in third-party dependencies (report upstream, we will track the fix)
- Attacks requiring physical access to an unlocked device
- Theoretical issues without a demonstrated or realistic impact path
