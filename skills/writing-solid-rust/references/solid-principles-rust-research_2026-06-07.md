<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# solid principles

SOLID is a set of five object‑oriented design principles aimed at making code easier to understand, change, and extend over time: Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, and Dependency Inversion.[^1_1][^1_2]

## What SOLID Stands For

- Single Responsibility Principle (SRP)
- Open/Closed Principle (OCP)
- Liskov Substitution Principle (LSP)
- Interface Segregation Principle (ISP)
- Dependency Inversion Principle (DIP)[^1_2][^1_1]

The acronym was popularized by Michael Feathers based on design principles articulated by Robert C. Martin around 2000.[^1_1][^1_2]

## Single Responsibility Principle

SRP states that a class should have only one responsibility and therefore only one reason to change.[^1_2][^1_1]
In practice, this pushes you toward cohesive modules where unrelated concerns (e.g., business logic vs. logging vs. persistence) are separated, making refactoring and testing safer.[^1_1][^1_2]

Example intuition: a `InvoiceService` that calculates totals, talks to the DB, sends emails, and logs activity clearly has multiple reasons to change; SRP suggests splitting those roles.

## Open/Closed Principle

OCP says that software entities should be open for extension but closed for modification.[^1_2][^1_1]
The idea is that adding new behavior should primarily involve adding new code (e.g., new subclasses, strategies, or handlers) rather than editing stable, tested modules, reducing the risk of regressions.[^1_1][^1_2]

A common example is using polymorphism or plugins for new payment methods instead of stuffing more `if`/`match` branches into existing core logic.

## Liskov Substitution Principle

LSP requires that objects of a subtype should be usable anywhere their base type is expected without breaking correctness.[^1_2][^1_1]
Formally, if A is a subtype of B, then code depending on B should not need to know whether it’s actually dealing with A, preserving the expected behavior and invariants.[^1_1][^1_2]

Classic violations include subclasses that throw “not supported” for base operations, or that weaken preconditions / strengthen postconditions in incompatible ways.

## Interface Segregation Principle

ISP says clients should not be forced to depend on methods they do not use, encouraging smaller, more focused interfaces.[^1_2][^1_1]
By splitting large “god” interfaces into specific role-focused ones, you reduce unnecessary coupling and avoid implementing dummy or no‑op methods in consumers.[^1_1][^1_2]

This often manifests as splitting, say, a huge `Repository` interface into separate read/write/query abstractions so clients can depend only on what they actually need.

## Dependency Inversion Principle

DIP states that high-level modules and low-level modules should both depend on abstractions, and abstractions should not depend on details.[^1_2][^1_1]
In concrete terms, instead of a service directly constructing or calling concrete implementations, it depends on interfaces or abstract types that are supplied from outside, which supports testability and swapping implementations.[^1_1][^1_2]

This principle underlies dependency injection patterns and inversion-of-control containers common in modern application architectures.[^1_2]

Given your background, would it be most useful if I map these to concrete patterns in Python (e.g., protocols, ABCs, type hints) or to higher-level system design (e.g., hexagonal architecture and agents)?
<span style="display:none">[^1_3][^1_4][^1_5][^1_6][^1_7]</span>

<div align="center">⁂</div>

[^1_1]: <https://en.wikipedia.org/wiki/SOLID>

[^1_2]: <https://www.baeldung.com/solid-principles>

[^1_3]: <https://www.geeksforgeeks.org/system-design/solid-principle-in-programming-understand-with-real-life-examples/>

[^1_4]: <https://www.reddit.com/r/learnprogramming/comments/cr3m01/solid_design_principles_for_everyone/>

[^1_5]: <https://www.digitalocean.com/community/conceptual-articles/s-o-l-i-d-the-first-five-principles-of-object-oriented-design>

[^1_6]: <https://www.splunk.com/en_us/blog/learn/solid-design-principle.html>

[^1_7]: <https://stackoverflow.blog/2021/11/01/why-solid-principles-are-still-the-foundation-for-modern-software-architecture/>

---

## can it be applied in rust?

Yes, SOLID absolutely can be applied in Rust, but you express it through traits, modules, and composition rather than classical inheritance, and some principles map more “naturally” than others.[^2_1][^2_2][^2_3]

### High-level fit in Rust

Rust is multi‑paradigm and already pushes you toward small, composable units with strong boundaries, which aligns well with SOLID’s goals (cohesion, decoupling, substitutability).[^2_3]
Community resources and guides explicitly discuss applying SOLID in Rust, using traits, generics, and module boundaries instead of classes and inheritance hierarchies.[^2_2][^2_1][^2_3]

### SRP in Rust

Single Responsibility Principle maps very cleanly: small modules, focused structs, and narrow functions with one reason to change.[^2_1][^2_2][^2_3]
Rust’s ownership and borrowing rules plus idiomatic module organization already encourage splitting responsibilities into separate structs/enums and modules rather than “god types.”[^2_3]

Concrete pattern you already use: separate `FooRepository`, `FooService`, and `FooNotifier` as independent structs or traits instead of one giant `FooManager`.

### OCP with traits and enums

Open/Closed in Rust is mostly about “open via traits, closed via existing types.”[^2_2][^2_1][^2_3]
You make behavior extensible by defining traits and implementing them for new types, or by using enums + pattern matching where adding new variants is a deliberate, localized change.[^2_4][^2_3]

Example: a `PaymentMethod` trait plus separate concrete types lets you add new payment providers without touching existing call sites, just wiring new implementations into your composition root.

### LSP as trait substitutability

Liskov Substitution becomes “any type implementing this trait must behave according to the trait’s contract.”[^2_1][^2_2][^2_3]
With trait bounds or trait objects, any implementor should be usable wherever that trait is expected without surprising semantics (panic, partial behavior, broken invariants).[^2_2][^2_3]

So your “subtypes” are trait implementors; your LSP checks are essentially making sure trait contracts and invariants are sound and not violated by specific implementations.

### ISP via small traits

Interface Segregation aligns almost perfectly with idiomatic Rust: many small traits instead of one huge, do‑everything interface.[^2_3][^2_1][^2_2]
You design narrow traits (e.g., `Readable`, `Writable`, `Queryable`) and let types implement only what they need, so clients depend on the minimal trait bounds necessary.[^2_2][^2_3]

This plays nicely with generic functions that state their capabilities in the where‑clause instead of depending on a kitchen‑sink trait.

### DIP with generics and trait objects

Dependency Inversion in Rust is basically “depend on traits, not concrete types,” wired through generics or `dyn Trait`.[^2_1][^2_3][^2_2]
High‑level logic takes type parameters with trait bounds or boxed trait objects, and low‑level modules provide concrete implementations, which you plug in at construction/composition time.[^2_4][^2_3]

For example, your service might be `fn new<R: Repository, N: Notifier>(repo: R, notif: N) -> Service`, keeping the core independent of specific DB clients or message busses.

### Nuances and limitations

There is an active Rust-community discussion that SOLID was born in classical OOP and some parts (especially traditional LSP/OCP interpretations) do not map 1:1 to Rust’s type system and lack of inheritance.[^2_5][^2_6][^2_3]
The pragmatic view is: treat SOLID as a set of design heuristics—cohesion, substitutable behavior, narrow interfaces, and abstraction‑first dependencies—implemented via Rust traits, generics, and modules, not rigid OO dogma.[^2_7][^2_3][^2_1]

If you like, I can sketch a small Rust example (e.g., “report generator” or “payment service”) written first as naïve code and then refactored to demonstrate each SOLID principle in idiomatic Rust.
<span style="display:none">[^2_8][^2_9]</span>

<div align="center">⁂</div>

[^2_1]: <https://rust-unofficial.github.io/patterns/additional_resources/design-principles.html>

[^2_2]: <https://codesignal.com/learn/courses/applying-clean-code-principles-in-rust/lessons/applying-clean-code-principles-in-rust-understanding-and-implementing-solid-principles>

[^2_3]: <https://www.holeoftherabbit.com/2025/01/19/rust-and-the-solid-principals/>

[^2_4]: <https://blog.bagwanpankaj.com/architecture/12-design-principles-you-can-implement-in-rust>

[^2_5]: <https://www.reddit.com/r/programming/comments/1gbqedh/solid_design_principles_rust_with_examples/>

[^2_6]: <https://github.com/rust-unofficial/patterns/discussions/406>

[^2_7]: <https://rusting.substack.com/p/solid-rust>

[^2_8]: <https://www.linkedin.com/pulse/solid-srp-ocp-lsp-isp-dip-khalid-shaikh-0bc7c>

[^2_9]: <https://www.40tude.fr/docs/06_programmation/rust/022_solid/solid_02.html>

---

## yes provide a generic idiomatic rust example and explain each in comments

Below is a small, generic “report sending” example in idiomatic Rust, annotated to show where each SOLID principle comes in. This is not “perfect” architecture, just a compact illustration.

```rust
// A simple domain model.
// SRP: `Report` just holds data for a report and nothing else
// (no sending, no formatting, no persistence).
struct Report {
    title: String,
    body: String,
}

// ISP: Keep this trait narrow: it only knows how to send a string.
// Clients that only need sending don't depend on formatting, storage, etc.
trait MessageSender {
    fn send(&self, msg: &str) -> Result<(), String>;
}

// OCP + DIP: High-level code depends on this abstraction, not on concrete
// SMTP/HTTP/etc. We can extend by adding new implementors without changing
// the high-level logic.

// A concrete low-level implementation.
// SRP: only responsible for "sending over HTTP" details.
struct HttpSender {
    endpoint: String,
}

impl HttpSender {
    fn new(endpoint: impl Into<String>) -> Self {
        Self { endpoint: endpoint.into() }
    }
}

impl MessageSender for HttpSender {
    fn send(&self, msg: &str) -> Result<(), String> {
        // LSP: this implementation must respect the trait contract:
        // return Ok on success, Err with a message on failure.
        // It should not panic or "half succeed" in a surprising way.
        println!("POST {} with body: {}", self.endpoint, msg);
        Ok(())
    }
}

// Another concrete low-level implementation.
struct StdoutSender;

impl MessageSender for StdoutSender {
    fn send(&self, msg: &str) -> Result<(), String> {
        println!("STDOUT: {}", msg);
        Ok(())
    }
}

// ISP: A separate trait for formatting. Callers that only need sending
// don't depend on formatting details.
trait ReportFormatter {
    fn format(&self, report: &Report) -> String;
}

// SRP: Responsible only for turning `Report` into a String.
struct PlainTextFormatter;

impl ReportFormatter for PlainTextFormatter {
    fn format(&self, report: &Report) -> String {
        format!("{}\n\n{}", report.title, report.body)
    }
}

// Another formatter implementation, showing OCP: we extend by adding a new
// type that implements the trait, not by modifying callers.
struct MarkdownFormatter;

impl ReportFormatter for MarkdownFormatter {
    fn format(&self, report: &Report) -> String {
        format!("# {}\n\n{}", report.title, report.body)
    }
}

// High-level policy object.
// SRP: `ReportService` coordinates formatting + sending, nothing else.
// DIP: It depends on *traits* (`ReportFormatter`, `MessageSender`), not on
// concrete implementations. The concrete `HttpSender`, `StdoutSender`,
// `PlainTextFormatter`, etc. are injected from outside.
struct ReportService<F, S>
where
    F: ReportFormatter,
    S: MessageSender,
{
    formatter: F,
    sender: S,
}

impl<F, S> ReportService<F, S>
where
    F: ReportFormatter,
    S: MessageSender,
{
    // DIP: Constructor takes abstractions (by type parameter bound to traits).
    fn new(formatter: F, sender: S) -> Self {
        Self { formatter, sender }
    }

    fn send_report(&self, report: &Report) -> Result<(), String> {
        // SRP (for this method): orchestrate the steps for sending a report.
        let msg = self.formatter.format(report); // uses ReportFormatter
        self.sender.send(&msg)                  // uses MessageSender
    }
}

fn main() -> Result<(), String> {
    let report = Report {
        title: "Daily Summary".into(),
        body: "All systems operational.".into(),
    };

    // Here we wire together concrete pieces at the composition root.
    // OCP: To add a new transport or format, we plug a new type here
    // without changing `ReportService`.
    let formatter = MarkdownFormatter;
    let sender = HttpSender::new("https://example.com/report");

    let service = ReportService::new(formatter, sender);

    service.send_report(&report)
}
```

How each SOLID principle shows up here, in brief:

- SRP
  - `Report` only models data; `PlainTextFormatter` / `MarkdownFormatter` only format; `HttpSender` / `StdoutSender` only send; `ReportService` only orchestrates those steps.
- OCP
  - You add new senders (e.g., `SmtpSender`) or formatters (e.g., `HtmlFormatter`) by implementing the existing traits; `ReportService` and existing call sites stay untouched.
- LSP
  - Any `S: MessageSender` or `F: ReportFormatter` must behave according to the trait contracts; generic code in `ReportService` does not care which concrete type is substituted as long as it obeys those semantics.
- ISP
  - Separate `MessageSender` and `ReportFormatter` traits keep interfaces small; a type can implement one without being forced to implement the other.
- DIP
  - High-level `ReportService` is generic over abstractions (`F: ReportFormatter, S: MessageSender`); concrete implementations are provided by the caller (main / DI layer), so the high-level logic is decoupled from low-level details.

If you’d like, I can also show a version that leans more on `dyn Trait` and trait objects (for runtime-selected transports/formatters) and contrast it with this generic-based approach.
<span style="display:none">[^3_1][^3_10][^3_2][^3_3][^3_4][^3_5][^3_6][^3_7][^3_8][^3_9]</span>

<div align="center">⁂</div>

[^3_1]: <https://www.reddit.com/r/rust/comments/1ff00f5/how_to_write_idiomatic_rust_with_best_practices/>

[^3_2]: <https://www.youtube.com/watch?v=P2mooqNMxMs>

[^3_3]: <https://news.ycombinator.com/item?id=22818150>

[^3_4]: <https://se-radio.net/2025/03/se-radio-659-brenden-matthews-on-idiomatic-rust/>

[^3_5]: <https://github.com/mre/idiomatic-rust>

[^3_6]: <https://www.youtube.com/watch?v=UkLvbAAq80Q>

[^3_7]: <https://ianbull.com/posts/idiomatic-rust/>

[^3_8]: <https://rustic-chess.org/back_matter/idiomatic.html>

[^3_9]: <https://www.youtube.com/watch?v=pL9hjZAmdj4>

[^3_10]: <https://users.rust-lang.org/t/why-and-why-not-rust/98354>

---

## show me "perfect" architecture

There isn’t a single “perfect” architecture in Rust (or any language); instead there are a few composable patterns that play nicely with ownership, traits, async, and testing. For something SOLID‑flavored, the “sweet spot” is usually: functional core + trait‑based ports + thin async/adapters at the edges.[^4_1][^4_2]

Below is one concrete “pretty darn good” architecture for a small service, shown as a skeleton rather than a full app. It’s close to hexagonal / clean architecture but adapted to Rust’s strengths.

---

### High-level structure

Think in three rough layers:

- Domain (pure, synchronous, no I/O): entities, value objects, domain services.
- Ports (traits): abstractions for persistence, messaging, external APIs.
- Adapters (I/O, infra, wiring): HTTP handlers, DB clients, CLI, DI/composition root.[^4_3][^4_1]

Directory sketch:

```text
src/
  domain/
    mod.rs
    report.rs
    service.rs
  ports/
    mod.rs
    report_repository.rs
    message_sender.rs
  adapters/
    mod.rs
    http/
      handlers.rs
      routes.rs
    db/
      pg_report_repository.rs
    messaging/
      http_message_sender.rs
  main.rs
```

This gives you SRP at module level, clear DIP (domain depends on ports; adapters implement them), and good testability.[^4_1][^4_3]

---

### Domain layer (functional core)

```rust
// src/domain/report.rs
// SRP: `Report` is purely domain data + invariants, no I/O.
#[derive(Debug, Clone)]
pub struct Report {
    title: String,
    body: String,
}

impl Report {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        // place invariants / validation here if needed
        Self {
            title: title.into(),
            body: body.into(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

// Example domain logic.
// SRP: domain rules only, no HTTP/DB concerns.
pub fn summarize(report: &Report) -> String {
    format!("{}: {} chars", report.title(), report.body().len())
}
```

```rust
// src/domain/service.rs
use crate::domain::report::Report;
use crate::ports::{ReportRepository, MessageSender};

// High-level policy. DIP: depends on *traits* from `ports`.
pub struct ReportService<R, M>
where
    R: ReportRepository,
    M: MessageSender,
{
    repo: R,
    sender: M,
}

impl<R, M> ReportService<R, M>
where
    R: ReportRepository,
    M: MessageSender,
{
    // DIP: constructor takes abstractions (via type params).
    pub fn new(repo: R, sender: M) -> Self {
        Self { repo, sender }
    }

    // Pure-ish orchestration logic; still async because it crosses ports.
    pub async fn send_daily_report(&self) -> Result<(), ServiceError> {
        // Domain-oriented language here; infra details are hidden in ports.
        let report = self
            .repo
            .load_daily_report()
            .await
            .map_err(ServiceError::LoadFailed)?;

        let msg = format!("{}\n\n{}", report.title(), report.body());

        self.sender
            .send(&msg)
            .await
            .map_err(ServiceError::SendFailed)
    }
}

// Small domain-level error. Can be enriched later.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("failed to load daily report: {0}")]
    LoadFailed(String),
    #[error("failed to send report: {0}")]
    SendFailed(String),
}
```

Comments explain:

- SRP: domain types don’t know about HTTP/SQL.
- DIP: `ReportService` is generic over `R: ReportRepository, M: MessageSender`.
- ISP: traits in `ports` will be narrow.

---

### Ports (traits as boundaries)

```rust
// src/ports/report_repository.rs
use async_trait::async_trait;
use crate::domain::report::Report;

// ISP: Only what the domain needs to know about report storage.
// No connection details, transactions, etc.
#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn load_daily_report(&self) -> Result<Report, String>;
}
```

```rust
// src/ports/message_sender.rs
use async_trait::async_trait;

// ISP: A minimal sending interface.
// LSP: implementors must respect the contract: return Err on failure.
#[async_trait]
pub trait MessageSender: Send + Sync {
    async fn send(&self, msg: &str) -> Result<(), String>;
}
```

```rust
// src/ports/mod.rs
pub mod report_repository;
pub mod message_sender;

pub use report_repository::ReportRepository;
pub use message_sender::MessageSender;
```

This is your SOLID “interface” zone: small traits (ISP), substitutable implementations (LSP), used by domain services (DIP).[^4_3][^4_1]

---

### Adapters: DB, HTTP, messaging

#### DB adapter

```rust
// src/adapters/db/pg_report_repository.rs
use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::report::Report;
use crate::ports::ReportRepository;

// SRP: Talks to Postgres and maps rows <-> domain types.
// OCP: If we add a `RedisReportRepository`, we don't change domain.
pub struct PgReportRepository {
    pool: PgPool,
}

impl PgReportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReportRepository for PgReportRepository {
    async fn load_daily_report(&self) -> Result<Report, String> {
        let row = sqlx::query!(
            r#"SELECT title, body FROM reports WHERE kind = 'daily' ORDER BY created_at DESC LIMIT 1"#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(Report::new(row.title, row.body))
    }
}
```

#### Messaging adapter

```rust
// src/adapters/messaging/http_message_sender.rs
use async_trait::async_trait;

use crate::ports::MessageSender;

// SRP: Only knows how to send HTTP POSTs with a body.
// LSP: Does not panic on "normal" errors; returns Err instead.
pub struct HttpMessageSender {
    client: reqwest::Client,
    endpoint: String,
}

impl HttpMessageSender {
    pub fn new(client: reqwest::Client, endpoint: impl Into<String>) -> Self {
        Self {
            client,
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl MessageSender for HttpMessageSender {
    async fn send(&self, msg: &str) -> Result<(), String> {
        self.client
            .post(&self.endpoint)
            .body(msg.to_owned())
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
```

#### HTTP handler (edge of the system)

```rust
// src/adapters/http/handlers.rs
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;

use crate::domain::service::ReportService;

// Axum State for DI. Generally you wrap your service in an Arc.
#[derive(Clone)]
pub struct AppState<S> {
    pub report_service: S,
}

#[derive(Serialize)]
struct ApiResponse {
    status: String,
}

pub async fn send_daily_report_handler<S>(
    State(state): State<AppState<S>>,
) -> impl IntoResponse
where
    S: Send + Sync,
    // Bound on the underlying service behavior is usually on S's type,
    // or you expose trait objects. For brevity, we assume S has the method:
    //   async fn send_daily_report(&self) -> Result<(), ServiceError>
{
    match state.report_service.send_daily_report().await {
        Ok(_) => Json(ApiResponse {
            status: "ok".into(),
        }),
        Err(e) => {
            eprintln!("failed to send report: {e}");
            Json(ApiResponse {
                status: "error".into(),
            })
        }
    }
}
```

Here:

- Adapters depend on ports and domain, but not the other way around (DIP).
- Each adapter struct has a single responsibility (SRP).
- You can add new HTTP endpoints or transports without changing domain services (OCP).

---

### Composition root (main.rs)

```rust
// src/main.rs
mod domain;
mod ports;
mod adapters;

use std::net::SocketAddr;
use std::sync::Arc;

use adapters::db::pg_report_repository::PgReportRepository;
use adapters::http::handlers::{send_daily_report_handler, AppState};
use adapters::messaging::http_message_sender::HttpMessageSender;
use axum::{routing::post, Router};
use domain::service::ReportService;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Infra setup: DB pool, HTTP client, config, logging, etc.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:pass@localhost/db")
        .await?;

    let repo = PgReportRepository::new(pool);
    let client = reqwest::Client::new();
    let sender = HttpMessageSender::new(client, "https://example.com/report");

    // Wire the domain service with concrete ports (DIP).
    let service = ReportService::new(repo, sender);

    // Wrap in Arc if needed (cloned into handlers).
    let state = AppState {
        report_service: Arc::new(service),
    };

    let app = Router::new()
        .route("/reports/daily/send", post(send_daily_report_handler::<Arc<ReportService<_, _>>>))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    println!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

This file is intentionally “ugly” in one way: it knows about everything. That’s by design:

- It is the only place that wires concrete infra into abstract ports (classic composition root).
- All other code stays testable with in‑memory fakes/mocks that implement the same traits.

---

### Why this is close to “ideal” (for Rust)

From the Rust community’s perspective, “good architecture” is mostly about: simplicity, clear ownership flows, narrow traits, async at the boundaries, and testability, rather than chasing some canonical OO blueprint. This layout tends to satisfy:[^4_4][^4_2][^4_1]

- SOLID (in a Rust way): SRP (per module/struct), OCP via traits/implementors, LSP via trait contracts, ISP via small traits, DIP via ports.
- Idiomatic Rust: traits and generics instead of unnecessary dynamic dispatch; composition over inheritance; data‑oriented domain; dependencies passed explicitly.[^4_2][^4_1][^4_3]

If you share the shape of one of your existing Rust services (e.g., crates, layers, async stack), I can refactor its outline into this style and annotate tradeoffs for your specific context.
<span style="display:none">[^4_5][^4_6][^4_7][^4_8][^4_9]</span>

<div align="center">⁂</div>

[^4_1]: <https://users.rust-lang.org/t/rust-best-practices/40436>

[^4_2]: <https://rust-unofficial.github.io/patterns/idioms/>

[^4_3]: <https://codesignal.com/learn/courses/applying-clean-code-principles-in-rust/lessons/applying-clean-code-principles-in-rust-understanding-and-implementing-solid-principles>

[^4_4]: <https://www.reddit.com/r/rust/comments/1ff00f5/how_to_write_idiomatic_rust_with_best_practices/>

[^4_5]: <https://github.com/mre/idiomatic-rust>

[^4_6]: <https://www.oreilly.com/videos/idiomatic-rust-video/9781633437463VE/>

[^4_7]: <https://www.youtube.com/watch?v=P2mooqNMxMs>

[^4_8]: <https://corrode.dev/blog/idiomatic-rust-resources/>

[^4_9]: <https://se-radio.net/2025/03/se-radio-659-brenden-matthews-on-idiomatic-rust/>

---

## is this hexagonal?

Roughly yes: what I showed is a light hexagonal / ports-and-adapters layout, even if I didn’t draw the hexagon or add an explicit “application/use‑case” layer.[^5_1][^5_2]

### Hexagonal criteria vs. the example

Hexagonal architecture (a.k.a. ports and adapters / clean architecture) is about putting the domain at the center and pushing frameworks, databases, and transport details to the edges. The inner core should depend on abstractions (“ports”), while outer “adapters” implement those abstractions and depend inward, never the other way around.[^5_1][^5_2]

In the example:

- Domain (`domain::report`, `domain::service`) contains entities and orchestrating logic and has no knowledge of HTTP, SQL, or reqwest.[^5_2][^5_1]
- Ports (`ports::ReportRepository`, `ports::MessageSender`) are domain‑facing traits that model what the domain needs from the outside world.[^5_1][^5_2]
- Adapters (`adapters::db::PgReportRepository`, `adapters::messaging::HttpMessageSender`, `adapters::http::handlers`) implement those traits and depend on SQLx, Axum, reqwest, etc., not vice versa.[^5_2][^5_1]
- `main.rs` acts as the composition root, wiring domain + ports + adapters, which is a common hexagonal pattern.[^5_1][^5_2]

That matches the essence of hexagonal architecture in Rust as described in guides and examples: a core domain independent of technical dependencies, accessed via ports and surrounded by adapters.[^5_3][^5_4][^5_2]

### Where it’s “lite” rather than “strict”

A stricter hexagonal/Clean implementation usually:

- Separates “domain” and “application/use‑case” layers (e.g., `domain` vs. `app` or `usecase` crate), where application services orchestrate domain entities and ports.[^5_2][^5_4]
- Sometimes moves ports into the application layer instead of the domain crate, depending on taste and DDD interpretation.[^5_2]

In the sketch I kept it simple and collapsed “application services” and “domain services” into one `ReportService`, which is very common in smaller Rust services and still respects the core idea: domain + ports are inward, adapters + frameworks are outward.

If you want, I can re-slice the example into separate `domain` and `application` crates (or modules) to show what a fully “clean” / layered hexagonal setup looks like in Rust with cargo workspaces.
<span style="display:none">[^5_5][^5_6][^5_7][^5_8]</span>

<div align="center">⁂</div>

[^5_1]: <https://blog.eizinger.io/5835/rust-s-custom-derives-in-a-hexagonal-architecture-incompatible-ideas>

[^5_2]: <https://alexis-lozano.com/blog/hexagonal-architecture-in-rust-1/>

[^5_3]: <https://github.com/antoinecarton/hexagonal-rust>

[^5_4]: <https://www.40tude.fr/docs/06_programmation/rust/024_hexagonal/hexagonal_lite.html>

[^5_5]: <https://www.reddit.com/r/rust/comments/1dmqqo5/master_hexagonal_architecture_in_rust_parts_1_2/>

[^5_6]: <https://news.ycombinator.com/item?id=41518698>

[^5_7]: <https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust>

[^5_8]: <https://www.youtube.com/watch?v=1ddvwuf0YGw>

---

## yes

Yes. Let’s take the previous single-crate layout and “upgrade” it into a stricter, layered hexagonal architecture using a Cargo workspace with separate domain, application, and infrastructure crates.[^6_1][^6_2][^6_3]

Below is one way to structure it that stays idiomatic for Rust while keeping a clean hexagonal boundary.

---

### Workspace layout

```text
my-app/
  Cargo.toml        # workspace root
  domain/
    Cargo.toml
    src/
      lib.rs
      report.rs
  application/
    Cargo.toml
    src/
      lib.rs
      ports.rs
      report_service.rs
  infrastructure/
    Cargo.toml
    src/
      lib.rs
      db/
        pg_report_repository.rs
      messaging/
        http_message_sender.rs
      http/
        handlers.rs
  bin/
    http-api/
      Cargo.toml
      src/main.rs    # composition root for HTTP API
```

Key ideas:

- `domain`: pure business concepts and rules, no async, no frameworks.[^6_2][^6_3]
- `application`: use cases, ports (traits), and orchestration, depends on `domain` only.[^6_3][^6_2]
- `infrastructure`: all the tech stack — DB, HTTP, messaging, logging — depends on `application`.[^6_2][^6_3]
- `bin/http-api`: the actual runnable service; wires infra implementations into application services.[^6_3]

---

### Root Cargo.toml

```toml
[workspace]
members = [
    "domain",
    "application",
    "infrastructure",
    "bin/http-api",
]
```

Workspaces are the standard way in Rust to manage multiple crates in one repo and share a lockfile.[^6_4]

---

### Domain crate

`domain/Cargo.toml`:

```toml
[package]
name = "domain"
version = "0.1.0"
edition = "2021"

[dependencies]
thiserror = "1"
```

`domain/src/lib.rs`:

```rust
pub mod report;
pub mod error; // optional if you want domain-specific errors
```

`domain/src/report.rs`:

```rust
// Pure domain entity, no async, no IO.
#[derive(Debug, Clone)]
pub struct Report {
    title: String,
    body: String,
}

impl Report {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

pub fn summarize(report: &Report) -> String {
    format!("{}: {} chars", report.title(), report.body().len())
}
```

This crate knows nothing about traits for persistence or sending; that’s part of hexagonal’s “domain is ignorant of the outside world.”[^6_2][^6_3]

---

### Application crate (ports + use cases)

`application/Cargo.toml`:

```toml
[package]
name = "application"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
thiserror = "1"
domain = { path = "../domain" }
```

`application/src/lib.rs`:

```rust
pub mod ports;
pub mod report_service;
```

`application/src/ports.rs`:

```rust
use async_trait::async_trait;
use domain::report::Report;

// Ports are application-facing abstractions; they define what the app
// needs from the outside world (persistence, messaging, etc.).

#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn load_daily_report(&self) -> Result<Report, String>;
}

#[async_trait]
pub trait MessageSender: Send + Sync {
    async fn send(&self, msg: &str) -> Result<(), String>;
}
```

`application/src/report_service.rs`:

```rust
use crate::ports::{MessageSender, ReportRepository};
use domain::report::Report;

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("failed to load report: {0}")]
    Load(String),
    #[error("failed to send report: {0}")]
    Send(String),
}

// High-level use case. This is the "application layer" in hexagonal terms.
pub struct ReportService<R, M>
where
    R: ReportRepository,
    M: MessageSender,
{
    repo: R,
    sender: M,
}

impl<R, M> ReportService<R, M>
where
    R: ReportRepository,
    M: MessageSender,
{
    pub fn new(repo: R, sender: M) -> Self {
        Self { repo, sender }
    }

    pub async fn send_daily_report(&self) -> Result<(), ReportError> {
        let report: Report = self
            .repo
            .load_daily_report()
            .await
            .map_err(ReportError::Load)?;

        let msg = format!("{}\n\n{}", report.title(), report.body());

        self.sender
            .send(&msg)
            .await
            .map_err(ReportError::Send)
    }
}
```

This crate depends on `domain` and defines ports/use cases; it still knows nothing about SQLx, Axum, or reqwest.[^6_3][^6_2]

---

### Infrastructure crate (adapters)

`infrastructure/Cargo.toml`:

```toml
[package]
name = "infrastructure"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
axum = "0.7"
serde = { version = "1", features = ["derive"] }

application = { path = "../application" }
domain = { path = "../domain" } # optional; often infra needs domain types for mapping
```

`infrastructure/src/lib.rs`:

```rust
pub mod db;
pub mod messaging;
pub mod http;
```

#### DB adapter

`infrastructure/src/db/pg_report_repository.rs`:

```rust
use async_trait::async_trait;
use sqlx::PgPool;

use application::ports::ReportRepository;
use domain::report::Report;

// Hexagonal adapter: implements the ReportRepository port using Postgres.

pub struct PgReportRepository {
    pool: PgPool,
}

impl PgReportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReportRepository for PgReportRepository {
    async fn load_daily_report(&self) -> Result<Report, String> {
        let row = sqlx::query!(
            r#"SELECT title, body
               FROM reports
               WHERE kind = 'daily'
               ORDER BY created_at DESC
               LIMIT 1"#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(Report::new(row.title, row.body))
    }
}
```

#### Messaging adapter

`infrastructure/src/messaging/http_message_sender.rs`:

```rust
use async_trait::async_trait;
use reqwest::Client;

use application::ports::MessageSender;

// Another adapter: implements MessageSender using HTTP POST.
pub struct HttpMessageSender {
    client: Client,
    endpoint: String,
}

impl HttpMessageSender {
    pub fn new(client: Client, endpoint: impl Into<String>) -> Self {
        Self {
            client,
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl MessageSender for HttpMessageSender {
    async fn send(&self, msg: &str) -> Result<(), String> {
        self.client
            .post(&self.endpoint)
            .body(msg.to_owned())
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
```

#### HTTP adapter

`infrastructure/src/http/handlers.rs`:

```rust
use std::sync::Arc;

use application::report_service::ReportService;
use application::ports::{ReportRepository, MessageSender};
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;

// AppState holds the application service. We use trait bounds
// so we can still swap implementations in tests.
#[derive(Clone)]
pub struct AppState<R, M>
where
    R: ReportRepository,
    M: MessageSender,
{
    pub service: Arc<ReportService<R, M>>,
}

#[derive(Serialize)]
struct ApiResponse {
    status: String,
}

pub async fn send_daily_report_handler<R, M>(
    State(state): State<AppState<R, M>>,
) -> impl IntoResponse
where
    R: ReportRepository,
    M: MessageSender,
{
    match state.service.send_daily_report().await {
        Ok(_) => Json(ApiResponse {
            status: "ok".into(),
        }),
        Err(e) => {
            eprintln!("failed to send report: {e}");
            Json(ApiResponse {
                status: "error".into(),
            })
        }
    }
}
```

---

### HTTP binary (composition root)

`bin/http-api/Cargo.toml`:

```toml
[package]
name = "http-api"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
axum = "0.7"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
anyhow = "1"

domain = { path = "../../domain" }
application = { path = "../../application" }
infrastructure = { path = "../../infrastructure" }
```

`bin/http-api/src/main.rs`:

```rust
use std::net::SocketAddr;
use std::sync::Arc;

use application::report_service::ReportService;
use axum::{routing::post, Router};
use infrastructure::db::pg_report_repository::PgReportRepository;
use infrastructure::http::handlers::{send_daily_report_handler, AppState};
use infrastructure::messaging::http_message_sender::HttpMessageSender;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:pass@localhost/db")
        .await?;

    let repo = PgReportRepository::new(pool);
    let client = Client::new();
    let sender = HttpMessageSender::new(client, "https://example.com/report");

    let service = Arc::new(ReportService::new(repo, sender));
    let state = AppState { service };

    let app = Router::new()
        .route("/reports/daily/send", post(send_daily_report_handler::<_, _>))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:3000".parse()?;
    println!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

This binary knows about all three layers and is the only place that couples them together, which is exactly what a hexagonal composition root is supposed to do.[^6_2][^6_3]

---

### Why this is “full hexagonal”

- Domain is self-contained and oblivious to infrastructure concerns; `domain` crate has no dependency on HTTP, DB, or even async.[^6_3][^6_2]
- Application crate defines ports and use cases and only depends on domain; it knows “what to do,” not “how to talk HTTP/SQL.”[^6_1][^6_3]
- Infrastructure crate implements ports via adapters; it depends on application/domain but not the other way around.[^6_2][^6_3]
- The executable is a thin shell that instantiates concrete adapters and wires them into application services (DIP to the edge).[^6_3][^6_2]

This is about as “textbook hexagonal in Rust” as you can reasonably get without going overboard.[^6_5][^6_1]

If you describe the scale of your current service (LOC, team, deployment model), I can suggest where to simplify (e.g., drop the separate application crate for small tools, or compress domain/application into one crate but keep the port/adapter split).
<span style="display:none">[^6_10][^6_11][^6_12][^6_13][^6_6][^6_7][^6_8][^6_9]</span>

<div align="center">⁂</div>

[^6_1]: <https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust>

[^6_2]: <https://blog.eizinger.io/5835/rust-s-custom-derives-in-a-hexagonal-architecture-incompatible-ideas>

[^6_3]: <https://users.rust-lang.org/t/rust-hexagonal-architecture/98672>

[^6_4]: <https://rust-lang.github.io/rfcs/1525-cargo-workspace.html>

[^6_5]: <https://smithery.ai/skills/davincible/rust-architecture-patterns>

[^6_6]: <https://www.reddit.com/r/rust/comments/1k3ah0b/hexagonal_architecture_questions/>

[^6_7]: <https://crates.io/crates/hexser>

[^6_8]: <https://news.ycombinator.com/item?id=41518698>

[^6_9]: <https://www.youtube.com/watch?v=1ddvwuf0YGw>

[^6_10]: <https://github.com/antoinecarton/hexagonal-rust>

[^6_11]: <https://github.com/Sairyss/domain-driven-hexagon/discussions/115>

[^6_12]: <https://www.reddit.com/r/rust/comments/1dmqqo5/master_hexagonal_architecture_in_rust_parts_1_2/>

[^6_13]: <https://www.40tude.fr/docs/06_programmation/rust/026_monolith_to_distributed/monolith_to_distributed.html>

---

## ~500k LOC, team of at least 15 devs but most newer to the team. currently there are about 15 crates but i think they can be narrowed down to: common, api, cli, connectors, contracts, devcontainer, hooks, docker, playbooks, test-utils, k8s, test-macros, runtime, traits, and a typescript ui

the traits and contracts are new additions that exist solely for the refactor

For a 500k LOC system with 15+ devs, I would not optimize first for “pure hexagonal elegance”; I would optimize for boundaries that are obvious to newer team members, minimize abstraction-only crates, and keep the dependency direction impossible to misunderstand. Given that, `traits` and `contracts` existing solely for the refactor is a yellow flag: they may be useful temporarily, but abstraction-only crates often become organizational overhead unless they represent stable domain or protocol boundaries with multiple real consumers.[^7_1][^7_2][^7_3]

### Likely target shape

I’d aim for a narrower structure centered on business/runtime seams rather than tool/deployment artifacts:

- `core` or `domain`: core types, invariants, domain logic.
- `application`: use cases, orchestration, ports, service-level policies.
- `adapters` or split infra crates: connectors, persistence, external integrations, transport implementations.
- `api`, `cli`, `ui`: entrypoints/interfaces.
- `platform` or `ops` outside the main dependency graph: docker, k8s, playbooks, devcontainer, hooks.[^7_3][^7_4][^7_5]

That means many of your current items are probably not peer architecture crates at all; `docker`, `k8s`, `playbooks`, `hooks`, and `devcontainer` are delivery/ops concerns, not application-layer building blocks.[^7_5][^7_3]

### On `traits` and `contracts`

I would usually avoid separate `traits` and `contracts` crates unless both are clearly justified by dependency direction and reuse. In a hexagonal design, ports often belong near the application layer because they describe what the use cases need from the outside world, while protocol contracts belong near the interface or integration boundary that owns them, not in a global abstraction bucket.[^7_4][^7_6][^7_3]

A useful heuristic:

| Crate       | Keep separate when                                                                                                               | Fold back when                                                                                                                   |
| :---------- | :------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------- |
| `traits`    | Multiple adapters/consumers implement the same stable port, and placing traits in `application` would create cycles.[^7_1][^7_3] | It only exists to “be clean,” mirrors one implementation, or forces people to jump crates to understand one feature.[^7_2][^7_1] |
| `contracts` | It contains truly shared wire schemas or protocol DTOs consumed by multiple independent binaries/services.[^7_6][^7_3]           | It is just internal request/response plumbing for one app, or duplicates richer domain types.[^7_6]                              |

For most teams, I’d prefer `application::ports` over a standalone `traits` crate, and boundary-specific `contracts` under `api-contracts`, `connector-x-contracts`, or even colocated with the owning interface rather than one global `contracts` crate.[^7_3][^7_4]

### A practical consolidation

Based on your list, a more teachable structure could look like this:

| Current                                               | Likely destination                                                                                           |
| :---------------------------------------------------- | :----------------------------------------------------------------------------------------------------------- |
| `common`                                              | Split or delete; move real domain code to `core/domain`, pure helpers to small internal modules.[^7_2][^7_1] |
| `runtime`                                             | `application` if it orchestrates use cases; `platform-runtime` if it is bootstrap/execution infrastructure.  |
| `traits`                                              | Move into `application::ports` unless independently justified.[^7_4][^7_3]                                   |
| `contracts`                                           | Keep only if shared across API/UI/connectors as stable schemas; otherwise colocate by boundary.[^7_6][^7_3]  |
| `connectors`                                          | Keep, but likely split by integration family if large.                                                       |
| `api`, `cli`, TS `ui`                                 | Keep as entrypoints/adapters.                                                                                |
| `test-utils`, `test-macros`                           | Keep if widely reused; otherwise fold into workspace `tests/` support.                                       |
| `docker`, `k8s`, `playbooks`, `hooks`, `devcontainer` | Treat as repo/platform assets, not architectural crates.[^7_5]                                               |

A plausible end state for the Rust workspace might be:

- `domain`
- `application`
- `connectors-*` or `infra-*`
- `api`
- `cli`
- `contracts-*` only where cross-boundary sharing is real
- `testing` or `test-support`

### What I’d enforce

For a team with many newer members, the biggest win is not “more layers”; it’s hard rules:

1. Dependency direction is one-way: `domain <- application <- adapters/interfaces`, never backward.[^7_4][^7_3]
2. No new `common` code without naming the owning domain or boundary; “common” tends to become a junk drawer.[^7_2][^7_1]
3. No standalone abstraction crate unless there are at least two concrete consumers or a clear cycle-breaking reason; one source specifically highlights starting with concrete types and abstracting later, not earlier.[^7_2]
4. Ops assets are not architecture crates; document them separately so the codebase shape reflects runtime architecture, not repo miscellany.[^7_5]

### Recommended direction

If I were guiding this refactor, I would likely collapse toward:

- `domain`
- `application`
- `api`
- `cli`
- `connectors` or a few named integration crates
- optional, narrowly scoped `contracts-*`
- `test-support`

…and remove `traits` as a top-level concept unless it proves its value with multiple real implementations and cleaner dependency flow.[^7_2][^7_3][^7_4]

Your setup sounds less like “do we need hexagonal?” and more like “how do we stop the refactor from producing architecture theater?” The answer is to make boundaries map to business/use-case ownership and runtime edges, not to abstract nouns.[^7_1][^7_2]

Would you like a proposed before/after crate map with exact dependency arrows for your current list?

<div align="center">⁂</div>

[^7_1]: <https://www.reddit.com/r/rust/comments/1k3ah0b/hexagonal_architecture_questions/>

[^7_2]: <https://news.ycombinator.com/item?id=41518698>

[^7_3]: <https://users.rust-lang.org/t/rust-hexagonal-architecture/98672>

[^7_4]: <https://blog.eizinger.io/5835/rust-s-custom-derives-in-a-hexagonal-architecture-incompatible-ideas>

[^7_5]: <https://www.40tude.fr/docs/06_programmation/rust/026_monolith_to_distributed/monolith_to_distributed.html>

[^7_6]: <https://github.com/Sairyss/domain-driven-hexagon/discussions/115>

---

## we also want to create a framework so that our clients can use part of our system and our internal teams to integrate it and that our system can have full observability

Yes — that requirement changes the recommendation quite a bit. If you want both a client-consumable framework and strong internal observability, you should treat your codebase as a **platform** with a stable public SDK surface, an internal application/runtime layer, and a first-class observability foundation rather than as just one large service split into crates.[^8_1][^8_2][^8_3]

### Recommended split

I would separate the system into three product-facing layers:

- `sdk` or `framework`: the supported public API your clients and internal teams embed or call.
- `application/runtime`: your orchestration, policies, workflows, and internal use cases.
- `adapters/connectors/interfaces`: transport, persistence, integrations, API/CLI/UI entrypoints.[^8_2][^8_4]

That gives you a cleaner contract story than generic `traits` and `contracts` crates. The “framework” becomes the intentional external surface, while internal ports stay near the application layer and are not automatically promoted to public extension points.[^8_5][^8_6]

A practical workspace could look like:

| Layer           | Crates                                                |
| :-------------- | :---------------------------------------------------- |
| Public platform | `sdk-core`, `sdk-observability`, `sdk-contracts`      |
| Internal core   | `domain`, `application`, `runtime`                    |
| Adapters        | `connectors-*`, `api`, `cli`, `ui-backend`            |
| Support         | `test-support`, `test-macros`                         |
| Repo assets     | `docker`, `k8s`, `playbooks`, `hooks`, `devcontainer` |

### Public framework boundary

If clients should “use part of the system,” don’t expose your internal architecture directly. Instead, define a narrow, versioned public surface with stable types, extension hooks, and lifecycle APIs, because public APIs need to optimize for long-term compatibility and onboarding clarity rather than internal purity.[^8_7][^8_8]

Concretely, I’d model it like this:

- `sdk-core`: core client-facing types, builders, configuration, error model.
- `sdk-contracts`: stable DTOs/events/schemas if cross-process or UI/shared integrations need them.
- `sdk-observability`: tracing, metrics, context propagation, correlation helpers.
- Optional `sdk-connectors-*`: only for integrations you explicitly want third parties to adopt.

The mistake to avoid is making `traits` equal “framework.” Most internal traits are just internal seams; a framework API should be fewer, better documented, more stable, and intentionally versioned.

### Internal architecture

Inside the system, keep the hexagonal shape, but make the framework sit beside it, not above it. In practice:

- `domain`: business types and invariants.
- `application`: use cases and internal ports.
- `runtime`: workflow engine, task orchestration, composition primitives, background execution.
- `adapters`: infra implementations of internal ports.[^8_9][^8_5]

Then the public SDK either:

1. wraps your application/runtime capabilities for embedding, or
2. acts as a remote client against your API, depending on whether “framework” means in-process reuse or service integration.

If both are needed, support both explicitly: an embed SDK and a remote client SDK. Trying to blur those into one crate usually creates confusion.

### Observability as a platform concern

For full observability, I would create a dedicated observability foundation crate and require every binary, worker, connector, and SDK entrypoint to use it. OpenTelemetry in Rust is specifically designed to generate and export traces, metrics, and logs, and the Rust docs show it as the standard path for instrumented applications.[^8_4][^8_3][^8_1][^8_2]

A good internal split is:

- `observability-core`: tracing setup, OTLP config, env-driven initialization, common resource attributes.
- `observability-http` / `observability-grpc`: propagation middleware/interceptors.
- `observability-test`: test helpers for spans, structured logs, and assertions.
- `sdk-observability`: safe public subset for clients embedding your framework.[^8_10][^8_4]

You want all three signals where feasible: traces for request/workflow flow, metrics for SLOs/capacity, and logs correlated to trace/span context.[^8_1][^8_2][^8_4]

### What to instrument

For your kind of system, “full observability” should mean every major boundary emits correlated telemetry:

- API/gRPC/CLI entrypoints: request spans, caller identity, latency, status.
- Runtime/workflow steps: span per task/stage/agent/tool execution.
- Connectors: outbound request spans, retries, backoff, rate-limit events.
- Domain/application: business events and counters around key outcomes.
- Background jobs: queue wait time, execution time, failure classes.
- SDK/framework: operation spans and context propagation so client apps can join traces with your platform.[^8_11][^8_2][^8_4][^8_1]

For a platform product, the most valuable thing is not just emitting telemetry, but standardizing names and attributes. Otherwise every team invents different span names and the traces become noise.

### What this means for your crate list

Given your goals, I would likely evolve your current structure toward something like:

- `sdk-core`
- `sdk-contracts`
- `sdk-observability`
- `domain`
- `application`
- `runtime`
- `connectors-*`
- `api`
- `cli`
- `test-support`
- `test-macros`

…and then either remove or heavily shrink generic `traits` and `contracts`. Public-facing abstractions move into `sdk-*`; internal ports live in `application`; transport/wire contracts are boundary-owned and versioned intentionally.[^8_6][^8_5][^8_9]

### Decision rule

Use this simple rule for each crate:

- If external clients must rely on it, it belongs in `sdk-*`.
- If it defines internal use-case seams, it belongs in `application`.
- If it talks to the outside world, it belongs in adapters/connectors.
- If it only exists to make the refactor “look clean,” merge it.[^8_12][^8_13]

For your team size and system scale, this will be easier to teach, easier to document, and much safer for long-term compatibility than a broad abstraction-first workspace.[^8_13][^8_12]

Would you like me to propose an exact target workspace tree and dependency graph for your current crates, including where observability and the client framework should live?
<span style="display:none">[^8_14][^8_15][^8_16][^8_17][^8_18][^8_19][^8_20]</span>

<div align="center">⁂</div>

[^8_1]: <https://crates.io/crates/opentelemetry>

[^8_2]: <https://github.com/open-telemetry/opentelemetry-rust>

[^8_3]: <https://opentelemetry.io/docs/languages/rust/>

[^8_4]: <https://opentelemetry.io/docs/languages/rust/getting-started/>

[^8_5]: <https://blog.eizinger.io/5835/rust-s-custom-derives-in-a-hexagonal-architecture-incompatible-ideas>

[^8_6]: <https://github.com/Sairyss/domain-driven-hexagon/discussions/115>

[^8_7]: <https://rawkode.academy/read/building-rust-cue-library>

[^8_8]: <https://users.rust-lang.org/t/which-crates-are-mature-enough-to-be-sure-they-will-be-long-lived/45826>

[^8_9]: <https://users.rust-lang.org/t/rust-hexagonal-architecture/98672>

[^8_10]: <https://docs.rs/si-observability>

[^8_11]: <https://blog.logrocket.com/composing-underpinnings-observable-rust-application/>

[^8_12]: <https://news.ycombinator.com/item?id=41518698>

[^8_13]: <https://www.reddit.com/r/rust/comments/1k3ah0b/hexagonal_architecture_questions/>

[^8_14]: <https://www.reddit.com/r/rust/comments/17zxyku/what_are_the_rust_crates_you_use_in_almost_every/>

[^8_15]: <https://openobserve.ai/blog/rust-observability-platform/>

[^8_16]: <https://www.reddit.com/r/rust/comments/1rptgzy/implementing_opentelemetry_in_rust_applications/>

[^8_17]: <https://www.youtube.com/watch?v=5LdnfzFdWhE>

[^8_18]: <https://github.com/rust-embedded/awesome-embedded-rust>

[^8_19]: <https://www.datadoghq.com/blog/monitor-rust-otel/>

[^8_20]: <https://www.youtube.com/watch?v=JNZoo_8XeaE>

---

## sure there is also an e2e crate and infrastructure directory with various deployment docs. tf and helm, etc

### also a preview-proxy crate and build-service dir

these were all made by the infra guy and considered sound

Yes — with those additions, I’d explicitly separate the repo into **product code**, **platform/runtime code**, and **ops/delivery assets**, and I would not try to force everything into the same “architecture crate” taxonomy. For a workspace of your size, a flat, clearly named layout is generally easier to navigate than clever layering hidden behind generic names, and keeping repo assets distinct from Rust crates reduces confusion for newer contributors.[^9_1][^9_2]

### Recommended top-level map

I’d organize the repository roughly like this:

```text
/
  Cargo.toml              # virtual workspace root
  crates/
    domain
    application
    runtime
    sdk-core
    sdk-contracts
    sdk-observability
    api
    cli
    preview-proxy
    build-service
    connectors-...
    test-support
    test-macros
    e2e-support?          # only if it is reusable code
  ui/
    ...typescript ui...
  infrastructure/
    terraform/
    helm/
    docs/
    env/
  dev/
    devcontainer/
    hooks/
    scripts-or-xtask/
```

This keeps executable/deployable Rust packages under one obvious place, while `infrastructure/` remains a first-class repo area for Terraform, Helm, and deployment docs without pretending those are application-layer crates.[^9_2][^9_1]

### Where your new pieces fit

Based on your description:

- `preview-proxy` is likely an adapter/platform service crate and should stay a crate if it is an actual deployable or reusable runtime component.
- `build-service` also sounds like a real service/binary, so keeping it as a crate or service package makes sense.
- `e2e` should stay separate if it owns end-to-end test runners, fixtures, orchestration, or environment bootstrapping; if it is mostly test helpers, fold shared parts into `test-support` and keep `e2e` as a test harness/package.[^9_1][^9_2]

The infra-owned `infrastructure/` directory sounds perfectly reasonable as a non-crate repo area. Terraform, Helm, and deployment documentation are deployment assets, not evidence that your Rust workspace should mirror infra concerns as code crates.[^9_1]

### Suggested crate consolidation

Given everything you listed, I’d target something like this:

| Keep as Rust crate/package                                                                                                                        | Consider folding or relocating                                                                           |
| :------------------------------------------------------------------------------------------------------------------------------------------------ | :------------------------------------------------------------------------------------------------------- |
| `api`, `cli`, `preview-proxy`, `build-service`, `runtime`, `connectors-*`, `sdk-*`, `domain`, `application`, `test-macros`, `test-support`, `e2e` | `common`, `traits`, broad `contracts`, plus any ops/deployment folders masquerading as code architecture |
| `ui` as separate app area                                                                                                                         | `devcontainer`, `hooks`, Helm/TF docs should live outside `crates/`                                      |

`common` is usually the first thing I would challenge, because it often mixes unrelated utilities and weakens ownership boundaries. Likewise, `traits` and `contracts` should survive only if they are tied to stable public SDK or cross-boundary protocol needs, not just because the refactor introduced them.[^9_3][^9_2][^9_1]

### Concrete target structure

A more opinionated final shape could be:

- `crates/domain` — business entities, invariants, value objects.
- `crates/application` — use cases, ports, orchestration.
- `crates/runtime` — execution model, workflow/task engine, scheduling, lifecycle.
- `crates/sdk-core` — public embedding API.
- `crates/sdk-contracts` — stable public/shared schemas only.
- `crates/sdk-observability` — tracing/metrics/logging setup and propagation for embedders.
- `crates/api` — HTTP/gRPC interface.
- `crates/cli` — operator/developer CLI.
- `crates/preview-proxy` — preview-facing transport/service boundary.
- `crates/build-service` — build/deployment pipeline service if it is product/runtime code.
- `crates/connectors-*` — split by domain/vendor, not one giant integration bucket.
- `crates/test-support` and `crates/test-macros`.
- `crates/e2e` — end-to-end harness and suites.
- `ui/` — TS frontend.
- `infrastructure/` — Terraform, Helm, deployment docs, env manifests.
- `dev/` — devcontainer, git hooks, local workflows, `xtask` or internal automation.[^9_2][^9_1]

### Naming and workspace rules

For a codebase this large, I’d enforce a few boring but powerful rules:

- Use a virtual workspace root and keep the root free of app code; that makes the repository shape more uniform and reduces exceptions.[^9_1]
- Keep crate names aligned with folder names, because that makes navigation and rename operations easier in large workspaces.[^9_1]
- Publishable/public crates should be obviously separated from internal ones; one source suggests deliberately isolating crates intended for semver/public use from internal workspace crates.[^9_1]
- Avoid ad hoc scripts scattered across the repo; use a dedicated automation approach such as an `xtask`-style crate for developer workflows.[^9_1]

### Architecture call on your current repo

Given your constraints, I would **not** say “collapse everything into pure hexagonal layers.” I would say:

- Hexagonal boundaries should shape `domain`, `application`, `runtime`, and adapters.
- Public reuse goals should shape `sdk-*`.
- Deployability should shape `api`, `preview-proxy`, `build-service`, `cli`.
- Operational ownership should shape `infrastructure/` and `dev/`.

That gives each top-level area a clear reason to exist and makes the repo easier for newer teammates to mentally model.[^9_2][^9_1]

### My strongest recommendation

The biggest architectural win here is probably not another crate split, but replacing ambiguous buckets with explicit ownership:

- Replace `common` with owned modules/crates or delete it.
- Move internal ports into `application` unless there is a strong cycle/public-API reason.
- Rename `contracts` into public/boundary-specific contracts if they are real.
- Keep infra assets outside the Rust crate map.
- Create a deliberate `sdk-*` family if client consumption is a strategic goal.[^9_3][^9_1]

If you want, I can turn your current list into a concrete proposed before/after tree with “keep / merge / move / rename” decisions and dependency arrows.
<span style="display:none">[^9_10][^9_11][^9_12][^9_13][^9_14][^9_15][^9_16][^9_4][^9_5][^9_6][^9_7][^9_8][^9_9]</span>

<div align="center">⁂</div>

[^9_1]: <https://matklad.github.io/2021/08/22/large-rust-workspaces.html>

[^9_2]: <https://earthly.dev/blog/cargo-workspace-crates/>

[^9_3]: <https://news.ycombinator.com/item?id=41518698>

[^9_4]: <https://www.reddit.com/r/rust/comments/1lra6h4/structuring_a_rust_mono_repo/>

[^9_5]: <https://users.rust-lang.org/t/mono-repos-in-rust/134824>

[^9_6]: <https://www.youtube.com/watch?v=5LdnfzFdWhE>

[^9_7]: <https://stackoverflow.com/questions/62051925/why-does-rust-need-both-packages-and-workspaces>

[^9_8]: <https://github.com/flosse/clean-architecture-with-rust/blob/master/Cargo.toml>

[^9_9]: <https://kerkour.com/rust-large-cargo-workspace>

[^9_10]: <https://smithery.ai/skills/davincible/rust-architecture-patterns>

[^9_11]: <https://stackoverflow.com/questions/49849878/how-to-deal-with-multiple-nested-workspace-roots>

[^9_12]: <https://news.ycombinator.com/item?id=46689834>

[^9_13]: <https://www.reddit.com/r/rust/comments/1i4c1x5/when_should_a_dependency_be_in_the_workspace_vs/>

[^9_14]: <https://internals.rust-lang.org/t/brainstorm-request-how-to-get-benefits-of-small-and-large-crates/10585>

[^9_15]: <https://github.com/aaronmallen/doing/discussions/213>

[^9_16]: <https://www.youtube.com/watch?v=-ewL14Gr1UY>

---

## build-service is not a crate, as far as i'm aware. i think it is only shell scripts

That helps; then I’d treat `build-service` and similar script-y areas as **developer automation**, not as part of the crate/architecture graph. For a 500k‑LOC Rust workspace, the main thing is to keep those concerns clearly separated so newcomers don’t confuse “infra/scripts” with “runtime/application architecture.”[^10_1]

### Where build scripts should live

Given `build-service` is shell scripts, I’d put it alongside other dev/ops utilities rather than under the main `crates` tree:

```text
/
  Cargo.toml          # virtual workspace
  crates/
    domain
    application
    runtime
    sdk-*
    api
    cli
    preview-proxy
    connectors-*
    test-support
    test-macros
    e2e
  ui/
    ... TS UI ...
  infrastructure/
    terraform/
    helm/
    docs/
  dev/
    build-service/    # your current shell scripts
    hooks/
    devcontainer/
    scripts/
```

This matches advice for large workspaces: keep build/automation scripts in a dedicated place instead of scattering `prepare.sh` and similar files throughout the repo.[^10_1]

### Consider an `xtask` crate over shell

If those build scripts are becoming central to how the repo is built, tested, or deployed locally, it’s often worth migrating the most important ones into an `xtask`‑style Rust crate:

- Single binary (e.g., `xtask`) in the workspace that owns tasks like codegen, checking, local env spins, and bundling.[^10_2][^10_3][^10_1]
- Called as `cargo xtask <task>` via a cargo alias.
- Tasks are typed, testable Rust code instead of a pile of bash with implicit dependencies.[^10_3][^10_2][^10_1]

That pattern is explicitly recommended to avoid “ad-hoc automation bloat” in large Rust workspaces.[^10_1]

You don’t have to do this immediately — but if `build-service` keeps growing and becomes business‑critical, making it a real crate (`crates/xtask` or `crates/build-tools`) can actually simplify onboarding and cross‑platform behavior.

### How this affects your architecture thinking

The upshot is:

- `build-service` (shell) is part of your **developer workflow/ops story**, not part of the internal module/crate architecture we’ve been shaping.
- It shouldn’t influence decisions about where `traits`, `contracts`, `runtime`, or `sdk-*` live.
- Over time, if those scripts harden into workflows that every dev runs, consider promoting them into a first-class automation crate and documenting that as part of your platform, but still keep it clearly labeled as dev tooling.

If you’d like, next I can sketch a concrete “before → after” mapping for all your named crates/dirs (including `preview-proxy`, `e2e`, infra dirs, and `build-service`) into the target layout we discussed, with a one-line purpose and allowed dependencies for each.
<span style="display:none">[^10_10][^10_11][^10_12][^10_13][^10_14][^10_15][^10_4][^10_5][^10_6][^10_7][^10_8][^10_9]</span>

<div align="center">⁂</div>

[^10_1]: <https://matklad.github.io/2021/08/22/large-rust-workspaces.html>

[^10_2]: <https://github.com/matklad/cargo-xtask>

[^10_3]: <https://github.com/matklad/cargo-xtask/blob/master/README.md>

[^10_4]: <https://www.reddit.com/r/rust/comments/1lra6h4/structuring_a_rust_mono_repo/>

[^10_5]: <https://users.rust-lang.org/t/complex-build-advice/48779>

[^10_6]: <https://betterprogramming.pub/running-rust-tasks-with-xtask-and-xtaskops-a2193e67dc25>

[^10_7]: <https://discourse.nixos.org/t/using-buildrustcrate-to-build-a-project-within-a-cargo-workspace/15672>

[^10_8]: <https://www.reddit.com/r/rust/comments/p9dd7h/blog_post_large_rust_workspaces/>

[^10_9]: <https://www.youtube.com/watch?v=70_9IIsQfjs>

[^10_10]: <https://users.rust-lang.org/t/cargo-building-system-in-complex-projects/117827>

[^10_11]: <https://www.youtube.com/watch?v=5LdnfzFdWhE>

[^10_12]: <https://news.ycombinator.com/item?id=46689834>

[^10_13]: <https://dev.to/ghost/rust-project-structure-example-step-by-step-3ee>

[^10_14]: <https://www.youtube.com/watch?v=-ewL14Gr1UY>

[^10_15]: <https://crates.io/crates/tracel-xtask>

---

## how to migrate shell scripts to an xtask crate in rust

A good way to migrate shell scripts to `xtask` is to do it incrementally: create one `xtask` binary in your workspace, move the most important scripts first, keep calling external tools where needed, and only replace bash logic with Rust when it improves reliability, typing, portability, or testability.[^11_1][^11_2][^11_3]

### Set up `xtask`

The `xtask` pattern is a Cargo workspace convention: add a small binary crate to your workspace, then expose it through a Cargo alias so contributors can run commands like `cargo xtask lint` or `cargo xtask release`.[^11_2][^11_4][^11_1]

Minimal setup:

```toml
# Cargo.toml at workspace root
[workspace]
members = [
  "crates/*",
  "xtask",
]
```

```toml
# .cargo/config.toml
[alias]
xtask = "run --package xtask --"
```

```bash
cargo new --bin xtask
```

After that, `cargo xtask <subcommand>` runs your automation binary.[^11_4][^11_2]

### Start with a thin wrapper

Do **not** rewrite every shell script from scratch on day one. The easiest migration is to first wrap your existing workflows in Rust subcommands that still invoke `cargo`, `docker`, `helm`, `terraform`, or legacy shell scripts as external commands, then gradually absorb the brittle parts into Rust.[^11_5][^11_6][^11_7]

Example structure:

```rust
// xtask/src/main.rs
use std::process::Command;

fn main() -> anyhow::Result<()> {
    let cmd = std::env::args().nth(1).unwrap_or_default();

    match cmd.as_str() {
        "lint" => lint(),
        "test" => test(),
        "e2e" => e2e(),
        "preview" => preview(),
        _ => {
            eprintln!("usage: cargo xtask [lint|test|e2e|preview]");
            std::process::exit(1);
        }
    }
}

fn lint() -> anyhow::Result<()> {
    run("cargo", &["fmt", "--all", "--", "--check"])?;
    run("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    Ok(())
}

fn test() -> anyhow::Result<()> {
    run("cargo", &["test", "--workspace"])?;
    Ok(())
}

fn e2e() -> anyhow::Result<()> {
    run("./dev/build-service/run-e2e.sh", &[])?;
    Ok(())
}

fn preview() -> anyhow::Result<()> {
    run("docker", &["compose", "up", "-d"])?;
    Ok(())
}

fn run(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(program).args(args).status()?;
    anyhow::ensure!(status.success(), "command failed: {} {:?}", program, args);
    Ok(())
}
```

This already gives you one stable entrypoint and better error handling than scattered shell.[^11_5][^11_2]

### Recommended crate stack

For `xtask`, I’d usually use:

- `anyhow` for error propagation.
- `clap` for subcommands and help text.
- Optionally `xshell` for ergonomic command execution; the rust-analyzer `xtask` example uses a shell helper approach to run commands and manage directories/env cleanly.[^11_7]
- Optionally `camino` for UTF-8 paths, and `serde`/`toml` if you need config parsing.

Example `Cargo.toml`:

```toml
[package]
name = "xtask"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
xshell = "0.2"
```

And then:

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Lint,
    Test,
    E2e,
    Preview,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli.command {
        Commands::Lint => {
            cmd!(sh, "cargo fmt --all -- --check").run()?;
            cmd!(sh, "cargo clippy --workspace --all-targets -- -D warnings").run()?;
        }
        Commands::Test => {
            cmd!(sh, "cargo test --workspace").run()?;
        }
        Commands::E2e => {
            cmd!(sh, "./dev/build-service/run-e2e.sh").run()?;
        }
        Commands::Preview => {
            cmd!(sh, "docker compose up -d").run()?;
        }
    }

    Ok(())
}
```

### Migration plan

A practical migration path is:

1. Inventory your scripts.
   Group them into categories: dev bootstrap, lint/check, codegen, test/e2e, packaging, deploy, preview, release.
2. Create one subcommand per workflow.
   Example: `cargo xtask bootstrap`, `cargo xtask check`, `cargo xtask e2e`, `cargo xtask release`.
3. Wrap existing scripts first.
   Rust becomes the stable interface; bash stays underneath temporarily.[^11_6][^11_5]
4. Pull logic into Rust where shell is weak.
   Good candidates: path discovery, environment validation, config parsing, matrix execution, file generation, JSON/TOML manipulation, OS portability.
5. Leave true shell/native tooling as command invocations.
   Don’t reimplement `docker compose`, `helm`, or `terraform`; just orchestrate them cleanly from Rust.
6. Add tests for the logic you moved.
   This is where `xtask` becomes much better than bash: argument parsing, config resolution, and task planning can be unit tested.

### What to rewrite first

Best candidates for full Rust migration:

- Path-heavy scripts that break when run from the wrong directory.
- Env validation scripts (`check that X, Y, Z tools are installed`).
- Codegen and template expansion.
- Cross-platform setup logic.
- Scripts that parse JSON/YAML/TOML output and branch on it.
- Multi-step workflows with retries or structured failure messages.[^11_7][^11_5]

Keep as external tool calls:

- `cargo`, `docker`, `helm`, `terraform`, `kubectl`, `pnpm`, `npm`, `make`.
- Tiny one-liners that just invoke one tool with fixed args.
- Highly Unix-specific plumbing that gives no value when ported.

### Example conversions

Shell:

```bash
#!/usr/bin/env bash
set -euo pipefail

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Rust:

```rust
fn check(sh: &xshell::Shell) -> anyhow::Result<()> {
    xshell::cmd!(sh, "cargo fmt --all -- --check").run()?;
    xshell::cmd!(sh, "cargo clippy --workspace --all-targets -- -D warnings").run()?;
    xshell::cmd!(sh, "cargo test --workspace").run()?;
    Ok(())
}
```

Shell:

```bash
ENV=${1:-dev}
helm upgrade --install app ./infrastructure/helm/app -f "./infrastructure/helm/values-${ENV}.yaml"
```

Rust:

```rust
fn deploy(sh: &xshell::Shell, env: &str) -> anyhow::Result<()> {
    let values = format!("./infrastructure/helm/values-{env}.yaml");
    xshell::cmd!(sh, "helm upgrade --install app ./infrastructure/helm/app -f {values}").run()?;
    Ok(())
}
```

The value is not that Rust is shorter; the value is typed argument parsing, composability, testing, and better diagnostics.

### Suggested command surface

For your repo, I’d probably expose something like:

- `cargo xtask bootstrap`
- `cargo xtask check`
- `cargo xtask test`
- `cargo xtask e2e`
- `cargo xtask preview up`
- `cargo xtask preview down`
- `cargo xtask codegen`
- `cargo xtask release`
- `cargo xtask infra plan --env dev`
- `cargo xtask infra apply --env staging`

That gives the team one discoverable command surface instead of a scavenger hunt through shell files.[^11_3][^11_2]

### Repo placement

Given your current repo, I’d place things like this:

```text
/
  xtask/
    src/
      main.rs
      check.rs
      test.rs
      infra.rs
      preview.rs
      codegen.rs
  dev/
    build-service/      # legacy scripts during migration
  infrastructure/
    terraform/
    helm/
```

Then migrate one script family at a time from `dev/build-service` into `xtask`, deleting the shell script only after the Rust version is stable.

### Rule of thumb

Use `xtask` as the control plane for automation, not necessarily the implementation language for every single step. In other words:[^11_2][^11_7]

- Rust decides what to run, validates inputs, and reports errors.
- External tools still do their specialized jobs.
- Shell gradually disappears where it causes pain.

A solid next step is to migrate one concrete script family first — usually `check` or `bootstrap` — because it’s low risk and quickly proves the pattern.
<span style="display:none">[^11_10][^11_11][^11_12][^11_13][^11_14][^11_15][^11_8][^11_9]</span>

<div align="center">⁂</div>

[^11_1]: <https://betterprogramming.pub/running-rust-tasks-with-xtask-and-xtaskops-a2193e67dc25>

[^11_2]: <https://github.com/matklad/cargo-xtask>

[^11_3]: <https://blog.bryantluk.com/cargo-xtask/>

[^11_4]: <https://lib.rs/crates/xtask>

[^11_5]: <https://www.reddit.com/r/rust/comments/u1uz7n/how_to_improve_my_rust_workflow/>

[^11_6]: <https://github.com/sagiegurari/cargo-make/discussions/717>

[^11_7]: <https://rust-lang.github.io/rust-analyzer/src/xtask/main.rs.html>

[^11_8]: <https://internals.rust-lang.org/t/a-list-of-commands-to-pre-configure-the-project/17492>

[^11_9]: <https://stackoverflow.com/questions/78477411/how-to-cross-compile-rust-to-windows-using-xwin-and-xtask>

[^11_10]: <https://lib.rs/crates/xtasks>

[^11_11]: <https://github.com/nickgerace/cargo-xtask-example>

[^11_12]: <https://www.youtube.com/watch?v=93SS3VGsKx4>

[^11_13]: <https://github.com/matklad/cargo-xtask/blob/master/examples/hello-world/README.md>

[^11_14]: <https://users.rust-lang.org/t/cargo-auto-automation-tasks-written-in-rust-language-for-the-build-process-of-rust-projects/63588>

[^11_15]: <https://docs.rs/xtasks/latest/xtasks/>

---

## Strategies for handling shared state and complex build orchestration across a large Rust workspace using xtask

This is a meaty topic with several distinct sub-problems: shared configuration and paths, task orchestration, parallelism, CI integration, and managing the growing surface area of a large `xtask` binary. Here's how to approach each.

---

### 1. Workspace root discovery

The first problem is always: "where are we?" Shell scripts assume a working directory; `xtask` needs to locate the workspace root reliably regardless of where Cargo invokes it.[^12_1][^12_2]

```rust
// xtask/src/workspace.rs

use std::path::{Path, PathBuf};

/// Returns the workspace root, resolved from CARGO_MANIFEST_DIR.
/// Cargo sets this to the xtask crate manifest, so we walk up one level.
pub fn root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to `xtask/`, so parent is the workspace root.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set; are you running via cargo xtask?");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("xtask must live inside the workspace")
        .to_path_buf()
}

/// Convenience paths derived from the workspace root.
pub struct Workspace {
    pub root: PathBuf,
}

impl Workspace {
    pub fn new() -> Self {
        Self { root: root() }
    }

    pub fn crates_dir(&self) -> PathBuf       { self.root.join("crates") }
    pub fn infra_dir(&self) -> PathBuf        { self.root.join("infrastructure") }
    pub fn ui_dir(&self) -> PathBuf           { self.root.join("ui") }
    pub fn target_dir(&self) -> PathBuf       { self.root.join("target") }
    pub fn dev_dir(&self) -> PathBuf          { self.root.join("dev") }
}
```

This is the foundation everything else references.[^12_2][^12_1]

---

### 2. Shared state with a typed `Context`

Rather than passing environment variables and flags through function arguments everywhere, build a top-level `Context` that is constructed once and passed throughout.[^12_3][^12_2]

```rust
// xtask/src/context.rs

use crate::workspace::Workspace;

#[derive(Debug, Clone)]
pub struct Context {
    pub workspace: Workspace,
    pub env: Environment,
    pub dry_run: bool,
    pub verbose: bool,
    pub ci: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Staging,
    Production,
}

impl Context {
    pub fn from_cli(args: &GlobalArgs) -> Self {
        Self {
            workspace: Workspace::new(),
            env: match args.env.as_deref() {
                Some("staging")    => Environment::Staging,
                Some("production") => Environment::Production,
                _                  => Environment::Dev,
            },
            dry_run: args.dry_run,
            verbose: args.verbose,
            // Detect CI: most CI systems set CI=true
            ci: std::env::var("CI").is_ok(),
        }
    }
}
```

Every task receives `&Context`. No global mutable state, no environment leakage.[^12_2][^12_3]

---

### 3. Task structure with `clap` subcommands

Split tasks into modules and compose them under a single `clap`-derived CLI.[^12_3]

```rust
// xtask/src/main.rs
mod check;
mod codegen;
mod context;
mod deploy;
mod e2e;
mod infra;
mod preview;
mod release;
mod workspace;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use context::Context;

#[derive(Parser)]
#[command(name = "xtask", about = "Workspace automation")]
struct Cli {
    #[command(flatten)]
    global: GlobalArgs,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Clone)]
pub struct GlobalArgs {
    /// Target environment (dev, staging, production)
    #[arg(long, default_value = "dev", global = true)]
    pub env: Option<String>,

    /// Print commands without running them
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Increase verbosity
    #[arg(long, short, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run fmt, clippy, and doc checks
    Check(check::CheckArgs),
    /// Run codegen across workspace
    Codegen,
    /// Run end-to-end tests
    E2e(e2e::E2eArgs),
    /// Manage preview environment
    Preview(preview::PreviewArgs),
    /// Deploy to environment
    Deploy(deploy::DeployArgs),
    /// Manage infrastructure (Terraform, Helm)
    Infra(infra::InfraArgs),
    /// Cut a release
    Release(release::ReleaseArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = Context::from_cli(&cli.global);

    match cli.command {
        Commands::Check(args)   => check::run(&ctx, args),
        Commands::Codegen       => codegen::run(&ctx),
        Commands::E2e(args)     => e2e::run(&ctx, args),
        Commands::Preview(args) => preview::run(&ctx, args),
        Commands::Deploy(args)  => deploy::run(&ctx, args),
        Commands::Infra(args)   => infra::run(&ctx, args),
        Commands::Release(args) => release::run(&ctx, args),
    }
}
```

---

### 4. Ergonomic command execution with `xshell`

Use `xshell` for command execution; it gives you clean output, shell-like interpolation, and error propagation without spawning a real shell.[^12_3]

```rust
// xtask/src/util.rs

use anyhow::Result;
use xshell::{Shell, cmd};
use crate::context::Context;

/// Creates a Shell rooted at the workspace root.
pub fn shell(ctx: &Context) -> Result<Shell> {
    let sh = Shell::new()?;
    sh.change_dir(&ctx.workspace.root);
    Ok(sh)
}

/// Wraps cmd! to honor dry_run.
#[macro_export]
macro_rules! run {
    ($ctx:expr, $sh:expr, $($t:tt)*) => {{
        let cmd = xshell::cmd!($sh, $($t)*);
        if $ctx.dry_run {
            eprintln!("[dry-run] {}", cmd);
            Ok::<(), anyhow::Error>(())
        } else {
            cmd.run().map_err(Into::into)
        }
    }};
}
```

Usage in any task:

```rust
// xtask/src/check.rs
use anyhow::Result;
use clap::Args;
use crate::{context::Context, run, util::shell};

#[derive(Args)]
pub struct CheckArgs {
    #[arg(long)]
    pub fix: bool,
}

pub fn run(ctx: &Context, args: CheckArgs) -> Result<()> {
    let sh = shell(ctx)?;

    run!(ctx, sh, "cargo fmt --all -- --check")?;
    run!(ctx, sh, "cargo clippy --workspace --all-targets -- -D warnings")?;

    if args.fix {
        run!(ctx, sh, "cargo fix --workspace --allow-dirty")?;
    }

    Ok(())
}
```

---

### 5. Complex build orchestration: sequencing and parallelism

For simple sequential workflows, chain calls. For parallel tasks use `std::thread` or `tokio` with structured concurrency.[^12_2][^12_3]

```rust
// xtask/src/release.rs
use anyhow::Result;
use clap::Args;
use rayon::prelude::*;
use crate::{context::Context, util::shell, run};

#[derive(Args)]
pub struct ReleaseArgs {
    /// Crates to release; defaults to all publishable
    #[arg(long, num_args = 0..)]
    pub crates: Vec<String>,
    /// Skip tests
    #[arg(long)]
    pub skip_tests: bool,
}

pub fn run(ctx: &Context, args: ReleaseArgs) -> Result<()> {
    let sh = shell(ctx)?;

    // Step 1: Sequential pre-flight
    run!(ctx, sh, "cargo check --workspace")?;
    run!(ctx, sh, "cargo fmt --all -- --check")?;
    run!(ctx, sh, "cargo clippy --workspace -- -D warnings")?;

    if !args.skip_tests {
        run!(ctx, sh, "cargo test --workspace")?;
    }

    // Step 2: Parallel crate publishes
    let crates = resolve_publishable_crates(ctx, &args.crates)?;

    // Use rayon for parallel iteration if independent
    crates.par_iter().try_for_each(|krate| -> Result<()> {
        let sh = shell(ctx)?;  // each thread needs its own Shell
        run!(ctx, sh, "cargo publish -p {krate}")?;
        Ok(())
    })?;

    Ok(())
}

fn resolve_publishable_crates(ctx: &Context, overrides: &[String]) -> Result<Vec<String>> {
    if !overrides.is_empty() {
        return Ok(overrides.to_vec());
    }
    // Parse workspace Cargo.toml to find publishable crates (version != "0.0.0")
    let cargo_toml = std::fs::read_to_string(ctx.workspace.root.join("Cargo.toml"))?;
    let manifest: toml::Value = cargo_toml.parse()?;

    let members = manifest["workspace"]["members"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let mut publishable = vec![];
    for member in members {
        if let Some(path) = member.as_str() {
            let crate_toml_path = ctx.workspace.root.join(path).join("Cargo.toml");
            let crate_toml = std::fs::read_to_string(&crate_toml_path)?;
            let crate_manifest: toml::Value = crate_toml.parse()?;
            let version = crate_manifest["package"]["version"]
                .as_str()
                .unwrap_or("0.0.0");
            let publish = crate_manifest["package"]["publish"]
                .as_bool()
                .unwrap_or(true);
            if version != "0.0.0" && publish {
                let name = crate_manifest["package"]["name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                publishable.push(name);
            }
        }
    }
    Ok(publishable)
}
```

---

### 6. Workspace metadata via `cargo_metadata`

For any task that needs to know about crates dynamically — which crates exist, their dependencies, features, paths — use the `cargo_metadata` crate instead of parsing `Cargo.toml` manually.[^12_1][^12_2]

```rust
// xtask/src/meta.rs
use anyhow::Result;
use cargo_metadata::{MetadataCommand, Package};
use crate::context::Context;

pub fn workspace_packages(ctx: &Context) -> Result<Vec<Package>> {
    let metadata = MetadataCommand::new()
        .manifest_path(ctx.workspace.root.join("Cargo.toml"))
        .exec()?;

    // Only return workspace members, not transitive deps
    let workspace_members: std::collections::HashSet<_> =
        metadata.workspace_members.iter().cloned().collect();

    Ok(metadata
        .packages
        .into_iter()
        .filter(|p| workspace_members.contains(&p.id))
        .collect())
}

pub fn publishable_packages(ctx: &Context) -> Result<Vec<Package>> {
    Ok(workspace_packages(ctx)?
        .into_iter()
        .filter(|p| p.version.to_string() != "0.0.0")
        .filter(|p| p.publish.as_deref() != Some(&[]))
        .collect())
}
```

This gives you strongly-typed access to all workspace metadata without fragile TOML parsing.[^12_1]

---

### 7. Config file for shared defaults

For large teams, drive shared defaults from a checked-in config file instead of hardcoded values.[^12_1][^12_2]

```toml
# xtask/xtask.toml
[deploy]
registry = "us-central1-docker.pkg.dev/my-project/my-repo"
default_env = "dev"
helm_timeout = "5m"

[e2e]
default_retries = 3
default_timeout_secs = 300

[release]
publishable_crates = ["sdk-core", "sdk-contracts", "sdk-observability"]
```

```rust
// xtask/src/config.rs
use anyhow::Result;
use serde::Deserialize;
use crate::context::Context;

#[derive(Debug, Deserialize)]
pub struct XtaskConfig {
    pub deploy: DeployConfig,
    pub e2e: E2eConfig,
    pub release: ReleaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct DeployConfig {
    pub registry: String,
    pub default_env: String,
    pub helm_timeout: String,
}

#[derive(Debug, Deserialize)]
pub struct E2eConfig {
    pub default_retries: u32,
    pub default_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseConfig {
    pub publishable_crates: Vec<String>,
}

pub fn load(ctx: &Context) -> Result<XtaskConfig> {
    let path = ctx.workspace.root.join("xtask").join("xtask.toml");
    let contents = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&contents)?)
}
```

---

### 8. Structured task output

For a large team, machine-readable output matters as much as human-readable output, especially for CI.[^12_2][^12_3]

```rust
// xtask/src/report.rs

use crate::context::Context;

pub struct TaskReport {
    pub task: String,
    pub steps: Vec<StepResult>,
}

#[derive(Debug)]
pub struct StepResult {
    pub name: String,
    pub status: StepStatus,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub enum StepStatus {
    Passed,
    Failed(String),
    Skipped,
}

impl TaskReport {
    pub fn print(&self, ctx: &Context) {
        if ctx.ci {
            self.print_ci_friendly();
        } else {
            self.print_human();
        }
    }

    fn print_human(&self) {
        println!("\n=== {} ===", self.task);
        for step in &self.steps {
            let icon = match &step.status {
                StepStatus::Passed       => "✅",
                StepStatus::Failed(_)    => "❌",
                StepStatus::Skipped      => "⏭ ",
            };
            println!("  {} {} ({}ms)", icon, step.name, step.duration_ms);
            if let StepStatus::Failed(msg) = &step.status {
                println!("    error: {}", msg);
            }
        }
    }

    fn print_ci_friendly(&self) {
        // GitHub Actions / GitLab CI friendly groups
        println!("::group::{}", self.task);
        for step in &self.steps {
            match &step.status {
                StepStatus::Failed(msg) => {
                    println!("::error title={}::{}", step.name, msg);
                }
                _ => println!("[{}] {} ({}ms)", self.status_str(&step.status), step.name, step.duration_ms),
            }
        }
        println!("::endgroup::");
    }

    fn status_str(&self, s: &StepStatus) -> &'static str {
        match s {
            StepStatus::Passed   => "PASS",
            StepStatus::Failed(_) => "FAIL",
            StepStatus::Skipped  => "SKIP",
        }
    }
}
```

---

### 9. Testable `xtask` logic

Because `xtask` is a real Rust crate, you can test orchestration logic, config parsing, path resolution, and task decisions.[^12_4][^12_2]

```rust
// xtask/src/meta.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn workspace_root_is_parent_of_xtask() {
        let root = super::super::workspace::root();
        assert!(root.join("Cargo.toml").exists(), "root Cargo.toml should exist");
        assert!(root.join("crates").is_dir(), "crates/ dir should exist");
    }

    #[test]
    fn config_is_valid() {
        let ctx = crate::context::Context::from_cli(
            &crate::GlobalArgs { env: None, dry_run: true, verbose: false }
        );
        let cfg = crate::config::load(&ctx)
            .expect("xtask.toml should parse without errors");
        assert!(!cfg.release.publishable_crates.is_empty());
    }
}
```

---

### Final `xtask/Cargo.toml`

```toml
[package]
name = "xtask"
version = "0.0.0"         # never published
edition = "2021"
publish = false

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
xshell = "0.2"
toml = "0.8"
serde = { version = "1", features = ["derive"] }
cargo_metadata = "0.18"
rayon = "1"
```

---

### Summary of strategies

| Problem                       | Strategy                                     |
| :---------------------------- | :------------------------------------------- |
| Path/root consistency         | `CARGO_MANIFEST_DIR` + `Workspace` struct    |
| Shared config/flags           | `Context` built once from `GlobalArgs`       |
| Typed workspace introspection | `cargo_metadata` crate                       |
| External defaults             | `xtask.toml` config file                     |
| Task composition              | `clap` subcommands, module per task          |
| Parallel steps                | `rayon::par_iter` or `tokio::spawn`          |
| Dry-run support               | `run!` macro wrapping every command          |
| CI vs. human output           | `ctx.ci` flag driving `TaskReport::print`    |
| Testability                   | Unit tests in `xtask` crate                  |
| Compile time                  | Minimize heavy deps, `cargo_metadata` lazily |

The key insight from community guidance is that `xtask` compile time should stay low — don't pull in heavyweight frameworks unless the task genuinely needs them. Keep the binary fast to compile so `cargo xtask check` isn't slower than just running `cargo check` directly.[^12_1][^12_2]
<span style="display:none">[^12_10][^12_11][^12_12][^12_13][^12_14][^12_15][^12_16][^12_5][^12_6][^12_7][^12_8][^12_9]</span>

<div align="center">⁂</div>

[^12_1]: <https://matklad.github.io/2021/08/22/large-rust-workspaces.html>

[^12_2]: <https://github.com/matklad/cargo-xtask>

[^12_3]: <https://betterprogramming.pub/running-rust-tasks-with-xtask-and-xtaskops-a2193e67dc25>

[^12_4]: <https://rust-lang.github.io/rust-analyzer/src/xtask/main.rs.html>

[^12_5]: <https://www.reddit.com/r/rust/comments/djc0wx/xtask_cargo_workflows_polyfill/>

[^12_6]: <https://crates.io/crates/tracel-xtask>

[^12_7]: <https://lib.rs/crates/xtasks>

[^12_8]: <https://stackoverflow.com/questions/39166704/no-targets-specified-in-the-manifest-when-creating-a-cargo-workspace>

[^12_9]: <https://github.com/sagiegurari/cargo-make/discussions/717>

[^12_10]: <https://docs.rs/xtasks/latest/xtasks/>

[^12_11]: <https://doc.rust-lang.org/cargo/reference/workspaces.html>

[^12_12]: <https://www.youtube.com/watch?v=70_9IIsQfjs>

[^12_13]: <https://rustprojectprimer.com/tools/tasks.html>

[^12_14]: <https://forum.dfinity.org/t/rust-crate-warning-about-workspace-resolver-what-is-it-how-to-fix-it/23883>

[^12_15]: <https://users.rust-lang.org/t/in-workspace-cargo-in-check-each-package-succeeds-but-fails-in-workspace-root-with-features/114805>

[^12_16]: <https://rust-lang.github.io/rust-analyzer/xtask/index.html>
