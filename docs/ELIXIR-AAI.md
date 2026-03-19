# ELIXIR Life Science Login (LS Login)

## „Out of the box“?

**Teilweise.** Im **Open-Source-Teil** von Lab Kit liegt die **Bibliotheks-Integration**:

- `lab-kit-auth::LsLoginOidc` — OIDC Discovery, JWKS, Validierung von ID-Tokens (Issuer/Audience), plus **GA4GH-Passport-Helpers** (Claims / Visa-Struktur).

Das ist **kein fertiger Login-Button** und kein gehosteter IdP. **Ende-zu-Ende** (Browser-Flow mit **Authorization Code + PKCE**, Redirects, Sessions, Schutz der GA4GH-Endpunkte) kommt aus **Ferrum** — insbesondere **Gateway und Auth-Middleware** in `ferrum-core` / `ferrum-gateway` ([Ferrum](https://github.com/SynapticFour/Ferrum)). Lab Kit **konfiguriert** (`lab-kit.toml`) und **verdrahtet** Deployments; du registrierst trotzdem einen **OIDC-Client** bei LS Login und setzt **Ingress/HTTPS** wie unten beschrieben.

This guide is written so an **IT administrator without Rust experience** can register Lab Kit as an **OpenID Connect Relying Party** against ELIXIR’s Life Science AAI.

## 1. Register an OIDC client

1. Use the ELIXIR Czech broker discovery document:  
   `https://login.elixir-czech.org/oidc/.well-known/openid-configuration`
2. In your IdP admin UI (or via your institutional contact), register a **confidential** client for your Lab Kit base URL.
3. Set **redirect URI** to your gateway callback, e.g. `https://lab.example.org/oauth/callback`.
4. Enable scopes: `openid`, `profile`, `email`, `offline_access`, `ga4gh_passport_v1` (Passport for controlled access).

## 2. Authorization Code + PKCE

Lab Kit’s open-source adapter (`lab-kit-auth::LsLoginOidc`) expects **standard OIDC**:

- Authorization Code Flow with **PKCE** (S256).
- `offline_access` for refresh tokens where your policy allows it.

**eduGAIN:** Home institution login is handled **inside LS Login**; Lab Kit must not rewrite redirects or strip parameters.

## 3. Configure `lab-kit.toml`

```toml
[auth]
provider = "ls-login"

[auth.ls-login]
client_id = "…"
client_secret = "…"
issuer = "https://login.elixir-czech.org/oidc/"
redirect_uri = "https://lab.example.org/oauth/callback"
scopes = ["openid", "profile", "email", "offline_access", "ga4gh_passport_v1"]
```

Deploy the **auth** fragment (`docker-compose.auth.yml`) or equivalent Ferrum gateway so callbacks terminate on your ingress.

## 4. GA4GH Passport & Beacon tiers

- **Public:** no session.
- **Registered:** any valid LS Login session.
- **Controlled:** valid `ControlledAccessGrants` visa in `ga4gh_passport_v1` for the dataset.

ORCID / Google as upstream IdPs require **no extra Lab Kit config** when LS Login is the OIDC issuer.

## 5. Token validation (for integrators)

`LsLoginOidc` loads JWKS from discovery, validates `iss`/`aud`, and exposes claims to Ferrum’s gateway. Passport visa JWTs inside the claim array should be validated per GA4GH AAI policy (Lab Kit provides parsing helpers in `lab-kit-auth::passport`).
