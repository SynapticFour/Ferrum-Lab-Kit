# ELIXIR Life Science Login (LS Login)

## „Out of the box“?

**Konfiguration ja, Browser-Login nein.** Lab Kit speichert OIDC-Parameter in `lab-kit.toml` und setzt Compose-/Helm-Env für den Ferrum-Gateway. Die Bibliothek `lab-kit-auth::LsLoginOidc` kann ID-Tokens (Discovery, JWKS, `iss`/`aud`) validieren — das ist **kein** Login-Button und **kein** Authorization-Code-Austausch.

**Ende-zu-Ende** (Browser-Redirect, Authorization Code + PKCE, Sessions, Schutz der GA4GH-Endpunkte) liegt in **Ferrum** (`ferrum-core` / `ferrum-gateway`). Lab Kit **führt keinen Code-Exchange aus** und implementiert **kein PKCE**. Du registrierst trotzdem einen **OIDC-Client** bei LS Login und setzt **Ingress/HTTPS** wie unten beschrieben.

This guide is written so an **IT administrator without Rust experience** can register Lab Kit as an **OpenID Connect Relying Party** against ELIXIR’s Life Science AAI.

## 1. Register an OIDC client

1. Use the ELIXIR Czech broker discovery document:
   `https://login.elixir-czech.org/oidc/.well-known/openid-configuration`
2. In your IdP admin UI (or via your institutional contact), register a **confidential** client for your Lab Kit base URL.
3. Set **redirect URI** to the **Ferrum gateway** callback, e.g. `https://lab.example.org/oauth/callback`.
4. Enable scopes: `openid`, `profile`, `email`, `offline_access`, `ga4gh_passport_v1` (Passport for controlled access).

## 2. Who owns the browser flow

Ferrum’s gateway is the relying party for **Authorization Code + PKCE (S256)** when you enable LS Login there. Lab Kit only:

- writes `auth.provider = "ls-login"` and `[auth.ls-login]` into config;
- refuses HMAC/`none` algorithms in `lab-kit-auth` token validation helpers;
- does **not** exchange authorization codes or store refresh tokens.

**eduGAIN:** Home institution login is handled **inside LS Login**; Lab Kit must not rewrite redirects or strip parameters.

## 3. Configure `lab-kit.toml`

```toml
[auth]
provider = "ls-login"

[auth.ls-login]
client_id = "…"
# Keep the secret in the environment (LS_LOGIN_CLIENT_SECRET), not in git.
client_secret = "${LS_LOGIN_CLIENT_SECRET}"
issuer = "https://login.elixir-czech.org/oidc/"
redirect_uri = "https://lab.example.org/oauth/callback"
scopes = ["openid", "profile", "email", "offline_access", "ga4gh_passport_v1"]
```

`auth.provider = "ldap"` is **rejected** at config validate (not implemented). Offline/field nodes use `provider = "local"` (Ferrum Passport path).

## 4. GA4GH Passport & Beacon tiers

- **Public:** no session.
- **Registered:** any valid LS Login session.
- **Controlled:** valid `ControlledAccessGrants` visa in `ga4gh_passport_v1` for the dataset. Without a grant, Lab Kit helpers return **Denied**, not Registered.
- Visa JWKS fetch is **fail-closed**: issuer allowlist required, 10-minute JWKS TTL, no unsigned `iss` SSRF.

ORCID / Google as upstream IdPs require **no extra Lab Kit config** when LS Login is the OIDC issuer.

## 5. Token validation (for integrators)

`LsLoginOidc` loads JWKS from discovery and validates `iss`/`aud` for **library callers**. Runtime enforcement is Ferrum’s gateway. Passport visa JWTs inside the claim array should be validated per GA4GH AAI policy (`lab-kit-auth::passport`).
