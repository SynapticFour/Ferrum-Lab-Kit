//! Authentication adapters for **ELIXIR Life Science Login** and pluggable IdPs.
//! GA4GH Passport / visa evaluation helpers live in [`passport`].

#![forbid(unsafe_code)]

mod error;
mod ls_login;
mod passport;
mod provider;

pub use error::AuthError;
pub use ls_login::{KeycloakOidc, LdapAuth, LsLoginOidc};
pub use passport::{
    access_tier_for_beacon, access_tier_for_beacon_with, dataset_matches, verify_visa_jwt,
    visa_from_verified_payload, BeaconAccessTier, HttpJwks, PassportVisa, VisaEvaluator,
    VisaKeySource, VisaType,
};
pub use provider::AuthProvider;
