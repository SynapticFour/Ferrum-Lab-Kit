//! Authentication adapters for **ELIXIR Life Science Login** and pluggable IdPs.
//! GA4GH Passport / visa evaluation helpers live in [`passport`].

#![forbid(unsafe_code)]

mod error;
mod keycloak;
mod ls_login;
mod passport;
mod provider;

pub use error::AuthError;
pub use keycloak::KeycloakAuthAdapter;
pub use ls_login::LsLoginOidc;
pub use passport::{
    access_tier_for_beacon, BeaconAccessTier, PassportVisa, VisaEvaluator, VisaType,
};
pub use provider::AuthProvider;
