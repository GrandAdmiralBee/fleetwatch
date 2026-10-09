# ADR-0002: Device certificate issuance

- **Status:** Proposed
- **Date:** 2026-10-09
- **Related:** [01 Containers](../architecture/01-containers.md), [02 Identity](../architecture/02-agent-identity.md)

## Context

Every agent needs an individual, cryptographic identity so that metrics and
commands cannot be mixed up between devices, and so that access can be revoked
for a single device (see 02 Device Identity).

Constraints that shape this decision:

- Agents connect to **two** services over mTLS: `control-plane` (control
  stream) and `ingest` (metrics stream). Both must verify device certificates.
- Device certificates are valid for **90 days** and are renewed over an
  already authenticated channel.
- The system is operated by a single person, so every additional component
  (backups, upgrades, access control) is a recurring cost.
- A compromise of the server must not force re-trusting the whole fleet.
- The issuing mechanism should be replaceable by an industrial CA later
  without changing the protocol.

The question: **where and with which key are device certificates signed?**

## Decision Drivers

- A server compromise must not destroy the root of trust.
- Minimal number of components to run for a single operator.
- Ability to migrate to an industrial CA (step-ca, Vault PKI) later.
- Supported rotation of the signing key without re-enrolling the fleet.
- Signing must be reachable from as few code paths as possible.

## Considered Options

### Option 1: Single CA inside the control-plane process

One CA key stored on the server signs everything.

- Good: simplest possible setup, no extra material to manage.
- Bad: compromising the server compromises the root of trust. Recovery means
  re-enrolling every device.

### Option 2: Offline root and an intermediate CA in the control-plane

The root key is generated once and kept off the server. It signs an
intermediate CA. The intermediate key lives on the server and signs device
certificates in-process. Signing sits behind a `CertificateIssuer` trait.

- Good: a server compromise requires replacing the intermediate, not the root.
  The intermediate can be rotated while the old one is still valid.
- Good: no new component to operate.
- Bad: the intermediate key still lives on the server, readable by a process
  that is exposed to the network.
- Bad: needs a one-time manual ceremony and a safe place for the root key.

### Option 3: Dedicated CA service (step-ca, Vault PKI)

The control-plane asks a separate CA service to issue certificates.

- Good: best isolation of the signing key, built-in audit and API, mature
  tooling for renewal and revocation.
- Bad: one more component to deploy, back up, upgrade and secure. Disproportionate
  for a small fleet and a single operator.

## Decision

We will use **Option 2**: an offline root CA and an intermediate CA held by
the `control-plane`, with signing performed in-process behind the
`CertificateIssuer` trait.

```rust
#[async_trait]
trait CertificateIssuer: Send + Sync {
    async fn issue(&self, req: IssueRequest) -> Result<IssuedCert, IssueError>;
}
// v1: in-process implementation (rcgen)
// later, if needed: StepCaIssuer, VaultIssuer
```

Rules that are part of the decision:

1. **Trust anchor.** Agents, `control-plane` and `ingest` trust the **root**
   certificate. Its fingerprint is embedded in the join string and used to pin
   the server on first contact.
2. **Key storage.** The intermediate private key is delivered to the
   `control-plane` through `LoadCredential` with mode `0600`, never through
   environment variables. The root private key is not stored on any server.
3. **Two issuing paths only.** `issue` is called from `Enroll` and `Renew`
   and nowhere else. Both paths are covered by tests.
4. **Only the public key is taken from the CSR.** Subject, SAN, validity and
   key usage are set by the server. Otherwise an agent could request a
   certificate for another `device_id`.
5. **Certificate profile.**
   - SAN: URI `spiffe://fleetwatch/device/<uuid>` (a naming convention only,
     SPIRE is not used).
   - Validity: 90 days.
   - `extendedKeyUsage`: `clientAuth` only. `basicConstraints`: `CA:FALSE`.
   - Random, unique serial number, stored with the `device_id` and issue date.
6. **Intermediate constraints.** `pathLenConstraint = 0`, so the intermediate
   cannot issue subordinate CAs.
7. **Renewal.** The agent renews around day 60 of 90 over its existing mTLS
   channel with a new CSR and a new key.
8. **Revocation is not done per certificate.** A device is revoked by its
   status in PostgreSQL, distributed through Redis (see 02 Device Identity).

## Consequences

### Positive

- Compromise of the server is recoverable: issue a new intermediate from the
  root, devices obtain new certificates through `Renew` while the old
  intermediate is still trusted.
- No new service to run. The signing code stays small and testable.
- Moving to step-ca or Vault later means adding one implementation of
  `CertificateIssuer`, with no protocol change.
- A narrow certificate profile and two issuing paths keep the attack surface
  of the issuing code small.

### Negative

- The intermediate key is still on the server. An attacker with access to the
  `control-plane` can issue valid device certificates until the intermediate is
  replaced.
- Rotating the **root** is not supported in v1 and would require
  re-enrolling every device.
- A one-time key ceremony and a secure place for the root key are required.
  Losing the root key means starting over.
- `ingest` must also trust the root and know about device revocation, which
  adds a dependency on Redis for it.

## Revisit When

- The fleet grows to thousands of devices, or issuing becomes a throughput
  concern.
- Audit of every signing operation becomes a requirement.
- The intermediate key needs hardware protection (HSM or a managed KMS).
- Root rotation becomes necessary, for example due to an expiring root or a
  suspected root compromise.

## Open Questions

- Keep the SAN as a SPIFFE-style URI, or use another format?
- Add `nameConstraints` to the intermediate, or is `pathLenConstraint = 0`
  enough?
- Where exactly is the root key stored (encrypted file, password manager,
  hardware token)?
- Should the certificate history live in a separate table
  (`device_certificates`) or only the active serial in `devices`?
- What happens to a device whose certificate expired while it was offline?
  v1 assumes re-enrollment, to be confirmed in 02.
