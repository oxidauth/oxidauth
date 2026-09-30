# oxidauth-services

Service layer for [oxidauth](https://oxidauth.rs).

One service struct per kernel contract (auth, users, roles, authorities,
permissions, grants, refresh tokens, public keys, invitations, settings,
TOTP, bootstrap). Services take the `oxidauth-repository` traits (storage
ports) and implement the `oxidauth-kernel` named service traits; the HTTP
server (`oxidauth-api`) and the client (`oxidauth`) ride those traits.

`bootstrap/` is the first-boot provisioning path (admin authority, signing
keys, default settings); `dev_prelude/` re-exports the mock-friendly set.
