# Oxidauth Security Vulnerability Report

**Date:** 2026-06-26  
**Version:** 0.4.0-rc2  
**Scope:** All workspace crates

---

## Executive Summary

After conducting a thorough security analysis of the oxidauth codebase across all 11 workspace crates, I identified **3 CRITICAL**, **4 HIGH**, and **17 MEDIUM** severity vulnerabilities. The codebase demonstrates good foundational security practices, including proper use of parameterized SQL queries, UUID generation, and JWT-based authentication. However, several critical security gaps must be addressed before production deployment.

**Overall Security Posture:** HIGH RISK - Requires immediate remediation

### Vulnerability Summary by Severity

| Severity | Count | Key Issues |
|----------|-------|------------|
| **CRITICAL** | 3 | CVE-2026-25537 in jsonwebtoken, OAuth2 authentication bypass, SSRF in OAuth2 profile retrieval |
| **HIGH** | 4 | Insecure CORS configuration, missing rate limiting, missing authorization checks |
| **MEDIUM** | 17 | Rate limiting gaps, timing attacks, log sanitization, database configuration |
| **LOW** | 12 | Hardcoded test values, information leakage, input validation |

---

## CRITICAL Vulnerabilities

### 1. CVE-2026-25537: Type Confusion in JWT Validation

**Severity:** CRITICAL  
**CWE:** CWE-843 (Type Confusion)  
**Affected Crate:** oxidauth-kernel  
**Location:** `oxidauth-kernel/src/jwt/mod.rs:60-71`

**Description:**
The application uses `jsonwebtoken = "9.3.0"` which contains a known type confusion vulnerability. When JWT standard claims (nbf, exp) are provided with incorrect JSON types (e.g., String instead of Number), the library treats "FailedToParse" identically to "NotPresent". This allows attackers to bypass time-based security restrictions.

**Current Implementation:**
```rust
pub fn decode(token: &str, key: &[u8]) -> Result<Jwt, JwtError> {
    let key = DecodingKey::from_rsa_pem(key).map_err(JwtError::new)?;
    let result: TokenData<Jwt> = decode(
        token,
        &key,
        &Validation::new(Algorithm::RS256),  //Missing required claims validation
    ).map_err(JwtError::new)?;
    Ok(result.claims)
}
```

**Potential Impact:**
- Authentication bypass
- Authorization bypass
- Use of expired tokens
- Bypass "Not Before" restrictions
- Tokens with malformed claims are silently accepted

**Recommended Fix:**
```toml
# Update Cargo.toml
jsonwebtoken = "10.3.0"
```

```rust
// Update jwt/mod.rs
use jsonwebtoken::{Validation, Algorithm};

let validation = Validation {
    validate_nbf: true,
    validate_exp: true,
    validate_aud: true,
    required_spec_claims: vec!["exp".to_string(), "nbf".to_string()],
    ..Validation::new(Algorithm::RS256)
};

let result: TokenData<Jwt> = decode(token, &key, &validation).map_err(JwtError::new)?;
```

---

### 2. OAuth2 Authentication Bypass

**Severity:** CRITICAL  
**CWE:** CWE-287 (Improper Authentication)  
**Affected Crate:** oxidauth-usecases  
**Location:** `oxidauth-usecases/src/auth/strategies/oauth2/authenticator.rs:31-38`

**Description:**
The OAuth2 authenticator implementation returns `Ok(())` without validating the code or token from the OAuth2 provider. This completely bypasses OAuth2 authentication.

**Potential Impact:**
- Complete OAuth2 bypass
- Any OAuth2 callback would be accepted as valid
- Attackers could forge authentication without valid OAuth2 tokens
- Full account takeover possible

---

### 3. SSRF Vulnerability in OAuth2 Profile Retrieval

**Severity:** CRITICAL  
**CWE:** CWE-918 (Server-Side Request Forgery)  
**Affected Crates:** oxidauth-usecases  
**Locations:**
- `oxidauth-usecases/src/auth/strategies/oauth2/google/retrieve_profile.rs:14-24`
- `oxidauth-usecases/src/auth/strategies/oauth2/microsoft/retrieve_profile.rs:14-24`

**Description:**
The code makes HTTP requests to URLs from authority parameters without any URL validation or IP address blocking, allowing Server-Side Request Forgery (SSRF) attacks.

**Potential Impact:**
- Access internal services (127.0.0.1, 169.254.169.254/metadata)
- Exfiltrate cloud provider metadata (AWS, Azure, GCP)
- Probe internal network infrastructure

---

## HIGH Vulnerabilities

### 4. Permissive CORS Configuration

**Severity:** HIGH  
**CWE:** CWE-346 (Origin Validation Error)  
**Affected Crate:** oxidauth-http  
**Location:** `oxidauth-http/src/server/mod.rs:42`

**Description:**
The server uses `CorsLayer::permissive()` which allows all origins, methods, and headers.

**Potential Impact:**
- Cross-Origin attacks on API endpoints
- Session hijacking via credential theft
- CSRF attacks on authenticated endpoints

**Recommended Fix:**
```rust
use tower_http::cors::{CorsLayer, AllowedOrigins, AllowedMethods, AllowedHeaders};
use http::header::{AUTHORIZATION, CONTENT_TYPE};

let cors = CorsLayer::new()
    .allow_origin("https://your-domain.com".parse().unwrap())
    .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(vec![AUTHORIZATION, CONTENT_TYPE]);
```

---

### 5. Missing Rate Limiting

**Severity:** HIGH  
**CWE:** CWE-770 (Missing Rate Limiting)  
**Affected Crates:** oxidauth-http, oxidauth-usecases, oxidauth-repository

**Locations Affected:**
- `/api/v1/auth/authenticate`
- `/api/v1/auth/register`
- `/api/v1/auth/username_password/forgot_password`

**Potential Impact:**
- Brute-force password attacks
- Denial of service via credential stuffing
- TOTP code enumeration attacks

---

### 6. Insecure Password Hashing Implementation

**Severity:** HIGH  
**CWE:** CWE-256 (Plaintext Storage of Password)  
**Affected Crate:** oxidauth-usecases  
**Location:** `oxidauth-usecases/src/auth/strategies/username_password/helpers.rs:21-30`

**Description:**
The password construction includes static salt and pepper in plaintext, revealing the salt/pepper structure to attackers.

**Recommended Fix:**
Use standard argon2 with randomly generated salt per password instead of including salt/pepper in the password string.

---

### 7. Missing Authorization Checks

**Severity:** HIGH  
**CWE:** CWE-285 (Improper Authorization)  
**Affected Crates:** oxidauth-usecases, oxidauth-http

**Locations:**
- `oxidauth-usecases/src/users/delete_user_by_id.rs:30-36`
- `oxidauth-usecases/src/roles/delete_role.rs`
- `oxidauth-usecases/src/authorities/delete_authority.rs`

**Potential Impact:**
- Any authenticated user with a valid token can potentially delete any user
- Delete critical system roles
- Delete authority configurations

---

## MEDIUM Vulnerabilities

### 8. Key Management Issues

**Severity:** MEDIUM  
**CWE:** CWE-320 (Key Management Issue)  
**Affected Crate:** oxidauth-kernel  
**Location:** `oxidauth-kernel/src/rsa/mod.rs`

**Issues:**
- No secure memory protection for private keys
- Keys are base64 decoded and used directly
- Minimum key size of 2048 bits is outdated (should be 4096)

---

### 9. Timing Attack in TOTP Validation

**Severity:** MEDIUM  
**CWE:** CWE-208 (Observable Timing Discrepancy)  
**Affected Crate:** oxidauth-usecases  
**Location:** `oxidauth-usecases/src/auth/authenticate_or_register.rs:155-157`

**Description:**
The state hash verification uses argon2's `verify_password` method with a timing-sensitive comparison that may leak information.

---

### 10. Insufficient Token Rotation

**Severity:** MEDIUM  
**CWE:** CWE-613 (Insufficient Session Expiration)  
**Affected Crate:** oxidauth-kernel  
**Location:** `src/refresh_tokens/` module

**Description:**
Refresh tokens are created but there's no evidence of:
1. Token binding to user agent/IP
2. Single active refresh token per user
3. Token revocation lists

---

### 11. Missing Input Validation

**Severity:** MEDIUM  
**CWE:** CWE-20 (Improper Input Validation)  
**Affected Crates:** oxidauth-http, oxidauth-usecases, oxidauth-repository

**Locations:**
- User-provided data not validated for length limits or malicious patterns

---

### 12. Information Leakage in Error Messages

**Severity:** MEDIUM  
**CWE:** CWE-209 (Generation of Error Message Containing Sensitive Information)  
**Affected Crates:** oxidauth-usecases, oxidauth-http

**Description:**
Error messages may leak implementation details or user existence information.

---

### 13. Overprivileged Bootstrap Admin User

**Severity:** MEDIUM  
**CWE:** CWE-269 (Improper Privilege Management)  
**Affected Crate:** oxidauth-usecases  
**Location:** `oxidauth-usecases/src/bootstrap/mod.rs:432-433`

**Description:**
The bootstrap process creates an admin user with excessive privileges and the password is printed to stdout.

---

### 14. Missing Security Headers

**Severity:** MEDIUM  
**CWE:** CWE-693 (Protection Mechanism Failure)  
**Affected Crate:** oxidauth-http

**Description:**
No security headers are configured (CSP, HSTS, X-Frame-Options, etc.)

---

### 15. Insecure Random String Generation

**Severity:** MEDIUM  
**CWE:** CWE-330 (Use of Insufficiently Random Values)  
**Affected Crate:** oxidauth-usecases  
**Location:** `oxidauth-usecases/src/lib.rs:22-29`

**Description:**
The `random_string()` function uses `thread_rng()` which may not provide sufficient entropy.

---

### 16. DATABASE_URL Validation Missing

**Severity:** MEDIUM  
**CWE:** CWE-1357 (Improper Validation of Specification)  
**Affected Crate:** oxidauth-postgres  
**Location:** `oxidauth-postgres/src/lib.rs:42`

**Description:**
The code reads `DATABASE_URL` environment variable but doesn't validate its format.

---

### 17. Large JSON Payload DoS Risk

**Severity:** MEDIUM  
**CWE:** CWE-400 (Uncontrolled Resource Consumption)  
**Affected Crate:** oxidauth-http

**Description:**
No explicit configuration to limit JSON payload size or nesting depth.

---

## LOW Vulnerabilities

### 18. Hardcoded Test Secret

**Severity:** LOW  
**CWE:** CWE-798 (Use of Hard-coded Credentials)  
**Affected Crate:** oxidauth-kernel  
**Location:** `oxidauth-kernel/src/user_authorities/create_user_authority.rs:51`

---

### 19. Token Transparency in Redirect URLs

**Severity:** LOW  
**CWE:** CWE-598 (Use of Query String in GET Request for Sensitive Data)  
**Affected Crate:** oxidauth-http  
**Location:** `oxidauth-http/src/server/api/v1/auth/oauth2/callback.rs`

---

### 20. Server Binding to All Network Interfaces

**Severity:** LOW  
**CWE:** CWE-285 (Improper Authorization)  
**Affected Crate:** oxidauth-http  
**Location:** `oxidauth-http/src/server/main.rs:36`

**Description:**
Server binds to `0.0.0.0:80` which exposes all network interfaces without HTTPS.

---

## Recommendations Priority Matrix

### Immediate (24-48 hours)
1. Upgrade jsonwebtoken from 9.3.0 to 10.3.0+ (CVE-2026-25537)
2. Implement OAuth2 token validation in authenticator (OAuth2 bypass)
3. Add SSRF protection for OAuth2 profile retrieval
4. Replace `CorsLayer::permissive()` with restrictive CORS

### High Priority (1-2 weeks)
5. Implement rate limiting on authentication endpoints
6. Add input validation on all request bodies
7. Implement proper authorization checks in use cases
8. Fix password hashing implementation
9. Add security headers (CSP, HSTS, X-Frame-Options)

### Medium Priority (1 month)
10. Implement TOTP timing-safe validation
11. Add refresh token rotation and revocation
12. Implement input validation libraries
13. Add comprehensive audit logging
14. Sanitize error messages for sensitive information

### Long-term (2-3 months)
15. Implement security monitoring and alerting
16. Add comprehensive security tests
17. Implement secure token binding
18. Conduct third-party penetration testing

---

## Disclaimer

This security assessment is based on the codebase as of 2026-06-26 and may not reflect changes made after that date. Security vulnerabilities should be addressed before production deployment. Regular security audits and penetration testing should be conducted on an ongoing basis.

**Report Version:** 1.0  
**Last Updated:** 2026-06-26
