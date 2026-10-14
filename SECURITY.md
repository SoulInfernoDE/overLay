# Security policy

> English original. German version: [SECURITY.de.md](SECURITY.de.md)

## Reporting a vulnerability

Please **do not** open a public issue. Report vulnerabilities privately through GitHub: repository → *Security* → *Report a vulnerability*.

We aim to confirm a report within 7 days and to agree on a disclosure date with you.

## Scope

Especially relevant are:

- escaping the mod sandbox (file system, process or network access beyond the granted permissions)
- bypassing the network allowlist
- the update chain: index signatures, artifact hashes, staging and activation
- the memory reader and other game providers
- the mod index CI (e.g. ways to get a non-reproducible artifact accepted)

Vulnerabilities in a mod belong in the mod's own repository. If a mod in the official index is malicious, report it here as well so it can be yanked.

## Supported versions

overLay is pre-alpha. Until 1.0 only the latest commit on `main` receives fixes.
