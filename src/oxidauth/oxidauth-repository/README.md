# oxidauth-repository

Storage ports (repository traits) for [oxidauth](https://oxidauth.rs).

One trait per data operation (find/create/update/delete/list), grouped by
aggregate: users, roles, authorities, permissions, grants, refresh tokens,
public/private keys, invitations, settings, TOTP secrets. The traits are the
port half of the ports-and-adapters split: `oxidauth-postgres` implements
them, `oxidauth-services` consumes them through `oxidauth-kernel` contracts.

No axum/tokio/sqlx here — trait signatures speak kernel and std types only.
