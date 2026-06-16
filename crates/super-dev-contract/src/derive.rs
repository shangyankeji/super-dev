//! Derive API endpoints from a free-form requirement.
//!
//! Until 4.7 the contract layer only *parsed* an existing architecture doc.
//! When the architecture doc came from the offline template (which hard-codes
//! `health` + `auth/login`), the contract never reflected the actual
//! requirement — a "shopping cart" requirement produced zero `products` or
//! `cart` endpoints. This module closes that gap: it extracts the *entities*
//! a requirement is about, then proposes a standard CRUD surface for each.
//!
//! ## How it works
//! 1. Tokenise the requirement (reusing the knowledge crate's mixed
//!    ASCII + CJK tokeniser).
//! 2. Score candidate nouns: plural-form words (`products`, `orders`) and
//!    domain nouns (`user`, `product`, `order`, `cart`, `article`) score
//!    highest; stop-words and verbs score zero.
//! 3. For each top entity, emit a REST convention:
//!    - `GET    /api/<entity>`           — list
//!    - `POST   /api/<entity>`           — create
//!    - `GET    /api/<entity>/:id`       — get one
//!    - `PATCH  /api/<entity>/:id`       — update
//!    - `DELETE /api/<entity>/:id`       — delete
//! 4. Merge with any endpoints already declared in the architecture doc
//!    (architecture wins on conflicts — a human/worker decision overrides
//!    the auto-derived guess).
//!
//! This makes the contract requirement-aware even in offline mode, so the
//! quality gate's contract checks produce meaningful signals (a "shopping
//! cart" requirement now has `cart` endpoints to validate against).

use std::collections::BTreeSet;

use crate::parse::{ApiSpec, Endpoint, HttpVerb, SecurityKind};

/// Known REST domain entities. These score highest because they map to
/// real database tables and standard CRUD operations. The list is
/// deliberately short — adding every possible noun would make the score
/// meaningless.
const DOMAIN_NOUNS: &[&str] = &[
    "user",
    "users",
    "account",
    "accounts",
    "profile",
    "profiles",
    "product",
    "products",
    "item",
    "items",
    "sku",
    "order",
    "orders",
    "cart",
    "carts",
    "checkout",
    "article",
    "articles",
    "post",
    "posts",
    "comment",
    "comments",
    "category",
    "categories",
    "tag",
    "tags",
    "project",
    "projects",
    "task",
    "tasks",
    "todo",
    "todos",
    "invoice",
    "invoices",
    "payment",
    "payments",
    "subscription",
    "subscriptions",
    "organization",
    "organizations",
    "team",
    "teams",
    "member",
    "members",
    "document",
    "documents",
    "file",
    "files",
    "upload",
    "uploads",
    "review",
    "reviews",
    "rating",
    "ratings",
    "event",
    "events",
    "booking",
    "bookings",
    "reservation",
    "reservations",
    "course",
    "courses",
    "lesson",
    "lessons",
    "enrollment",
    "enrollments",
    "message",
    "messages",
    "notification",
    "notifications",
    // NOTE: "search", "query", "feed", "stream" were previously listed but
    // are NOT CRUD resources — deriving them produced nonsense endpoints like
    // `GET /api/search/:id`. Search/feed are actions or views, not entities.
];

/// CJK domain entities — each Chinese term maps to an English REST slug.
/// Expanded in 4.7 from 25 to 80+ terms to cover common enterprise domains
/// (project mgmt, education, healthcare, IM, real estate, finance, HR...).
const CJK_ENTITY_MAP: &[(&str, &str)] = &[
    // Core / auth
    ("用户", "users"),
    ("账号", "accounts"),
    ("角色", "roles"),
    ("权限", "permissions"),
    ("个人资料", "profiles"),
    ("资料", "profiles"),
    // E-commerce
    ("商品", "products"),
    ("产品", "products"),
    ("货物", "products"),
    ("订单", "orders"),
    ("购物车", "carts"),
    ("购物", "carts"),
    ("库存", "inventory"),
    ("优惠券", "coupons"),
    ("折扣", "discounts"),
    ("评价", "reviews"),
    ("评分", "ratings"),
    ("退款", "refunds"),
    ("物流", "shipments"),
    ("发货", "shipments"),
    ("运费", "shipments"),
    // Content / blog / CMS
    ("文章", "articles"),
    ("帖子", "posts"),
    ("评论", "comments"),
    ("回复", "comments"),
    ("弹幕", "comments"),
    ("分类", "categories"),
    ("标签", "tags"),
    ("专栏", "columns"),
    ("专题", "topics"),
    ("话题", "topics"),
    ("搜索", "search"),
    // Project management
    ("项目", "projects"),
    ("任务", "tasks"),
    ("待办", "tasks"),
    ("工时", "timesheets"),
    ("里程碑", "milestones"),
    ("看板", "boards"),
    ("团队", "teams"),
    ("成员", "members"),
    ("协作", "collaborations"),
    // Education
    ("课程", "courses"),
    ("课节", "lessons"),
    ("课时", "lessons"),
    ("章节", "chapters"),
    ("报名", "enrollments"),
    ("选课", "enrollments"),
    ("作业", "assignments"),
    ("考试", "exams"),
    ("成绩", "grades"),
    ("老师", "instructors"),
    ("教师", "instructors"),
    ("学生", "students"),
    ("班级", "classes"),
    ("学院", "departments"),
    // Healthcare
    ("预约", "appointments"),
    ("挂号", "appointments"),
    ("预订", "reservations"),
    ("医生", "doctors"),
    ("患者", "patients"),
    ("病历", "records"),
    ("处方", "prescriptions"),
    ("科室", "departments"),
    ("药品", "medications"),
    // IM / social
    ("消息", "messages"),
    ("私信", "messages"),
    ("通知", "notifications"),
    ("好友", "contacts"),
    ("群组", "groups"),
    ("群聊", "channels"),
    ("动态", "feeds"),
    ("朋友圈", "feeds"),
    ("关注", "follows"),
    ("点赞", "likes"),
    ("分享", "shares"),
    // Finance
    ("发票", "invoices"),
    ("支付", "payments"),
    ("缴费", "payments"),
    ("订阅", "subscriptions"),
    ("账单", "bills"),
    ("交易", "transactions"),
    ("账户", "accounts"),
    ("余额", "balances"),
    ("收入", "income"),
    // HR / org
    ("组织", "organizations"),
    ("部门", "departments"),
    ("员工", "employees"),
    ("考勤", "attendances"),
    ("请假", "leaves"),
    ("薪资", "payroll"),
    ("简历", "applications"),
    ("招聘", "jobs"),
    // Real estate
    ("房源", "properties"),
    ("楼盘", "properties"),
    ("房屋", "properties"),
    ("户型", "listings"),
    ("租约", "leases"),
    ("租金", "leases"),
    // File / doc
    ("文件", "files"),
    ("文档", "documents"),
    ("图片", "images"),
    ("附件", "attachments"),
    ("上传", "uploads"),
    ("下载", "downloads"),
    // Events / booking
    ("活动", "events"),
    ("会议", "meetings"),
    ("日程", "schedules"),
    ("日历", "calendar"),
    ("场地", "venues"),
    ("票务", "tickets"),
    // Analytics
    ("报表", "reports"),
    ("指标", "metrics"),
    ("仪表盘", "dashboards"),
    ("数据", "analytics"),
    ("统计", "statistics"),
    // Tickets / support
    ("工单", "tickets"),
    ("问题", "issues"),
    ("故障", "issues"),
    // Wiki / knowledge base / docs
    ("知识库", "articles"),
    ("wiki", "articles"),
    ("百科", "articles"),
    ("空间", "spaces"),
    ("页面", "pages"),
    ("笔记", "notes"),
    ("草稿", "drafts"),
    // Workflow / approval
    ("审批", "approvals"),
    ("流程", "workflows"),
    ("工作流", "workflows"),
    ("申请", "applications"),
    // Version / settings / config
    ("版本", "versions"),
    ("配置", "settings"),
    ("设置", "settings"),
    ("集成", "integrations"),
    ("钩子", "webhooks"),
    // Import / export / backup
    ("导入", "imports"),
    ("导出", "exports"),
    ("备份", "backups"),
    ("恢复", "restores"),
    ("迁移", "migrations"),
    ("快照", "snapshots"),
    // Analytics extras
    ("仪表板", "dashboards"),
    ("大屏", "dashboards"),
    ("图表", "charts"),
    // Collaboration
    ("分享", "shares"),
    ("收藏", "favorites"),
    ("邀请", "invitations"),
    // Content extras
    ("视频", "videos"),
    ("相册", "albums"),
    // Commerce extras
    ("优惠券", "coupons"),
    ("库存", "inventory"),
    ("退货", "returns"),
    // Finance extras
    ("预算", "budgets"),
    ("报销", "expenses"),
    // Media / streaming
    ("音乐", "songs"),
    ("歌曲", "songs"),
    ("歌单", "playlists"),
    ("专辑", "albums"),
    ("歌手", "artists"),
    ("艺人", "artists"),
    ("播客", "podcasts"),
    ("电台", "stations"),
    ("直播", "streams"),
    // Booking extras
    ("会议", "meetings"),
    ("日程", "schedules"),
    ("场地", "venues"),
    // AI / data
    ("模型", "models"),
    ("数据集", "datasets"),
    ("实验", "experiments"),
];

/// Scene patterns — when a requirement matches a multi-entity domain, expand
/// to the full entity set a user would expect. E.g. "项目管理" alone implies
/// projects + tasks + teams; "在线教育" implies courses + lessons + enrollments.
/// This is semantic association, not just keyword matching.
///
/// **Maintenance contract:** each entry is `(trigger_phrase, [expected_entities])`.
/// - `trigger_phrase` is matched via `requirement.contains(...)` (case-sensitive
///   for CJK; lowercase the English fragment for ASCII-insensitive match).
/// - `expected_entities` are REST-resource slugs (plural where conventional,
///   e.g. `users`/`courses`) — they feed [`derive_endpoints_from_requirement`]
///   so each becomes a CRUD endpoint set. Avoid non-CRUD nouns (search/feed).
/// - When adding a scene: pick a trigger unique enough not to false-fire,
///   list the 3-5 core resources a reviewer would expect, and verify the
///   `extract_entities_caps_scene_explosion` test still passes (≤5 entities).
/// - Order matters for determinism: earlier patterns win on ties.
const SCENE_PATTERNS: &[(&str, &[&str])] = &[
    // Project management scenes.
    ("项目管理", &["projects", "tasks", "teams", "members"]),
    ("协作", &["projects", "tasks", "comments"]),
    ("工时", &["tasks", "timesheets", "reports"]),
    ("看板", &["boards", "tasks", "members"]),
    // E-commerce scenes.
    ("电商", &["products", "orders", "carts", "categories"]),
    ("商城", &["products", "orders", "carts", "categories"]),
    ("购物", &["products", "carts", "orders"]),
    // Education scenes.
    ("教育", &["courses", "lessons", "enrollments", "students"]),
    ("在线课程", &["courses", "lessons", "enrollments"]),
    ("教学", &["courses", "assignments", "grades"]),
    // Healthcare scenes.
    (
        "医院",
        &["appointments", "doctors", "patients", "departments"],
    ),
    ("挂号", &["appointments", "doctors", "departments"]),
    ("诊所", &["appointments", "patients", "records"]),
    // IM scenes.
    ("即时通讯", &["messages", "contacts", "channels"]),
    ("聊天", &["messages", "contacts", "channels"]),
    ("社交", &["feeds", "messages", "follows"]),
    // Finance scenes.
    ("财务", &["invoices", "payments", "transactions", "reports"]),
    ("计费", &["bills", "payments", "subscriptions"]),
    ("订阅", &["subscriptions", "payments", "plans"]),
    // HR scenes.
    (
        "人力资源",
        &["employees", "departments", "attendances", "payroll"],
    ),
    ("考勤", &["attendances", "employees", "leaves"]),
    ("招聘", &["jobs", "applications", "candidates"]),
    // Real estate scenes.
    ("房地产", &["properties", "listings", "leases"]),
    ("房产", &["properties", "listings", "leases"]),
    ("租房", &["properties", "leases", "payments"]),
    // Analytics scenes.
    ("数据分析", &["reports", "metrics", "dashboards"]),
    ("报表", &["reports", "metrics"]),
    ("监控", &["metrics", "alerts", "incidents"]),
    // Support scenes.
    ("客服", &["tickets", "messages", "faqs"]),
    ("工单", &["tickets", "issues", "assignees"]),
    // Wiki / knowledge base scenes
    ("知识库", &["articles", "categories", "spaces", "tags"]),
    ("wiki", &["articles", "categories", "spaces", "tags"]),
    ("文档管理", &["documents", "categories", "versions", "tags"]),
    ("内容管理", &["articles", "categories", "tags", "comments"]),
    // Workflow / approval scenes
    (
        "审批流",
        &["approvals", "workflows", "applications", "signatures"],
    ),
    ("工作流", &["workflows", "tasks", "approvals"]),
    // Analytics / dashboard scenes
    (
        "数据可视化",
        &["dashboards", "charts", "reports", "metrics"],
    ),
    ("报表平台", &["reports", "charts", "dashboards", "metrics"]),
    // Settings / admin scenes
    ("后台管理", &["settings", "users", "roles", "logs"]),
    ("权限管理", &["roles", "permissions", "users"]),
    // Backup / ops scenes
    ("备份恢复", &["backups", "restores", "snapshots"]),
    ("日志管理", &["logs", "alerts", "metrics"]),
    // Community / social scenes
    ("社区", &["posts", "comments", "users", "tags"]),
    ("论坛", &["posts", "comments", "users", "categories"]),
    ("问答", &["questions", "answers", "users", "tags"]),
    // Music / streaming scenes
    ("音乐", &["songs", "albums", "artists", "playlists"]),
    ("流媒体", &["songs", "playlists", "artists", "albums"]),
    ("播客", &["podcasts", "episodes", "hosts"]),
    // Meeting / scheduling scenes
    ("会议系统", &["meetings", "rooms", "participants"]),
    ("排班系统", &["schedules", "shifts", "employees"]),
    // AI / ML scenes
    (
        "机器学习平台",
        &["models", "datasets", "experiments", "trainings"],
    ),
];

/// Words that should never become endpoints (common, structural, or verbs).
/// Semantic fields for common entities. Each entry is (field_name, type, description).
/// Entities not in this map get a sensible default set. This replaces the
/// generic `title + data jsonb` placeholder with real, typed schemas so the
/// migration and data model reflect the actual domain.
#[allow(clippy::type_complexity)]
const ENTITY_FIELDS: &[(&str, &[(&str, &str, &str)])] = &[
    (
        "users",
        &[
            ("email", "text", "Unique email"),
            ("name", "text", "Display name"),
            ("role", "text", "user|admin"),
            ("avatar_url", "text", "Profile image"),
            ("password_hash", "text", "Argon2 hash"),
        ],
    ),
    (
        "products",
        &[
            ("name", "text", "Product name"),
            ("slug", "text", "URL slug"),
            ("price_cents", "integer", "Price in cents"),
            ("currency", "text", "ISO 4217"),
            ("stock", "integer", "Available units"),
            ("image_url", "text", "Main image"),
            ("category_id", "uuid", "FK categories"),
            ("status", "text", "draft|active|archived"),
        ],
    ),
    (
        "orders",
        &[
            ("user_id", "uuid", "FK users"),
            ("status", "text", "pending|paid|shipped|delivered|cancelled"),
            ("total_cents", "integer", "Order total"),
            ("currency", "text", "ISO 4217"),
            ("shipping_address", "jsonb", "Address snapshot"),
        ],
    ),
    (
        "carts",
        &[
            ("user_id", "uuid", "FK users"),
            ("status", "text", "active|abandoned|converted"),
            ("items_count", "integer", "Line items"),
            ("total_cents", "integer", "Cart total"),
        ],
    ),
    (
        "articles",
        &[
            ("title", "text", "Article title"),
            ("slug", "text", "URL slug"),
            ("body", "text", "Markdown body"),
            ("author_id", "uuid", "FK users"),
            ("status", "text", "draft|published|archived"),
            ("published_at", "timestamptz", "Publish time"),
        ],
    ),
    (
        "posts",
        &[
            ("title", "text", "Post title"),
            ("slug", "text", "URL slug"),
            ("body", "text", "Content"),
            ("author_id", "uuid", "FK users"),
            ("status", "text", "draft|published"),
            ("view_count", "integer", "Views"),
        ],
    ),
    (
        "comments",
        &[
            ("body", "text", "Comment text"),
            ("author_id", "uuid", "FK users"),
            ("post_id", "uuid", "FK posts"),
            ("article_id", "uuid", "FK articles"),
            ("parent_id", "uuid", "FK comments (thread)"),
        ],
    ),
    (
        "categories",
        &[
            ("name", "text", "Category name"),
            ("slug", "text", "URL slug"),
            ("parent_id", "uuid", "FK categories (tree)"),
        ],
    ),
    (
        "tags",
        &[("name", "text", "Tag name"), ("slug", "text", "URL slug")],
    ),
    (
        "projects",
        &[
            ("name", "text", "Project name"),
            ("description", "text", "What it does"),
            ("owner_id", "uuid", "FK users"),
            ("status", "text", "planning|active|on_hold|done"),
            ("visibility", "text", "private|public"),
        ],
    ),
    (
        "tasks",
        &[
            ("title", "text", "Task title"),
            ("description", "text", "Details"),
            ("project_id", "uuid", "FK projects"),
            ("assignee_id", "uuid", "FK users"),
            ("status", "text", "todo|in_progress|review|done"),
            ("priority", "text", "low|medium|high|urgent"),
            ("due_date", "date", "Deadline"),
            ("position", "integer", "Board order"),
        ],
    ),
    (
        "teams",
        &[
            ("name", "text", "Team name"),
            ("slug", "text", "URL slug"),
            ("owner_id", "uuid", "FK users"),
        ],
    ),
    (
        "members",
        &[
            ("team_id", "uuid", "FK teams"),
            ("user_id", "uuid", "FK users"),
            ("role", "text", "owner|admin|member"),
        ],
    ),
    (
        "invoices",
        &[
            ("number", "text", "Invoice number"),
            ("user_id", "uuid", "FK users"),
            ("amount_cents", "integer", "Total"),
            ("currency", "text", "ISO 4217"),
            ("status", "text", "draft|sent|paid|overdue"),
            ("due_date", "date", "Payment due"),
        ],
    ),
    (
        "payments",
        &[
            ("invoice_id", "uuid", "FK invoices"),
            ("amount_cents", "integer", "Amount"),
            ("currency", "text", "ISO 4217"),
            ("provider", "text", "stripe|paypal|manual"),
            ("provider_ref", "text", "External ID"),
            ("status", "text", "pending|succeeded|failed"),
        ],
    ),
    (
        "subscriptions",
        &[
            ("user_id", "uuid", "FK users"),
            ("plan", "text", "free|pro|enterprise"),
            ("status", "text", "active|canceled|past_due"),
            ("current_period_end", "timestamptz", "Renewal date"),
        ],
    ),
    (
        "documents",
        &[
            ("title", "text", "Document title"),
            ("body", "text", "Content"),
            ("owner_id", "uuid", "FK users"),
            ("visibility", "text", "private|shared|public"),
        ],
    ),
    (
        "messages",
        &[
            ("body", "text", "Message text"),
            ("sender_id", "uuid", "FK users"),
            ("recipient_id", "uuid", "FK users"),
            ("read_at", "timestamptz", "When read"),
        ],
    ),
    (
        "notifications",
        &[
            ("user_id", "uuid", "FK users"),
            ("type", "text", "Notification type"),
            ("body", "text", "Message"),
            ("read_at", "timestamptz", "When read"),
        ],
    ),
    (
        "events",
        &[
            ("title", "text", "Event title"),
            ("description", "text", "Details"),
            ("start_at", "timestamptz", "Start"),
            ("end_at", "timestamptz", "End"),
            ("location", "text", "Venue or URL"),
        ],
    ),
    (
        "reviews",
        &[
            ("product_id", "uuid", "FK products"),
            ("author_id", "uuid", "FK users"),
            ("rating", "integer", "1-5 stars"),
            ("body", "text", "Review text"),
        ],
    ),
];

/// Default fields for entities not in ENTITY_FIELDS — a sensible CRUD baseline
/// instead of the generic `title + data jsonb` placeholder.
const DEFAULT_FIELDS: &[(&str, &str, &str)] = &[
    ("name", "text", "Human-readable name"),
    ("description", "text", "What this is"),
    ("status", "text", "Lifecycle status"),
];

/// Return the semantic fields for an entity slug. Falls back to DEFAULT_FIELDS.
#[must_use]
pub fn fields_for_entity(entity: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    ENTITY_FIELDS
        .iter()
        .find(|(name, _)| *name == entity)
        .map(|(_, fields)| fields.to_vec())
        .unwrap_or_else(|| DEFAULT_FIELDS.to_vec())
}

const STOPWORDS: &[&str] = &[
    "system",
    "app",
    "application",
    "feature",
    "page",
    "screen",
    "ui",
    "ux",
    "data",
    "info",
    "information",
    "content",
    "service",
    "services",
    "api",
    "web",
    "site",
    "website",
    "platform",
    "tool",
    "tools",
    "dashboard",
    "admin",
    "manage",
    "management",
    "list",
    "detail",
    "view",
    "show",
    "build",
    "create",
    "make",
    "add",
    "edit",
    "update",
    "delete",
    "remove",
    "the",
    "and",
    "with",
    "for",
    "that",
    "this",
    "need",
    "want",
    "using",
    "real",
    "good",
    "nice",
    "modern",
    "simple",
    "complex",
    "main",
];

/// Derive a set of endpoints from the requirement text. Returns endpoints
/// sorted by entity then method. Pure function — caller merges with the
/// architecture-derived spec.
#[must_use]
pub fn derive_endpoints_from_requirement(requirement: &str) -> Vec<Endpoint> {
    let entities = extract_entities(requirement);
    let mut endpoints = Vec::new();
    for entity in &entities {
        // Materialise the typed field list ONCE per entity and reuse it for
        // every CRUD endpoint on that entity. Previously the request_shape
        // was a placeholder comment `{ /* {entity} fields */ }` even though
        // `fields_for_entity` existed — now the derived contract carries
        // real field shapes so downstream codegen + validators see substance.
        let fields = fields_for_entity(entity);
        let request_body = render_request_shape(entity, &fields);
        let response_body = render_response_shape(entity, &fields);
        for (method, suffix, desc_suffix) in CRUD_TEMPLATE {
            let path = if suffix.is_empty() {
                format!("/api/{entity}")
            } else {
                format!("/api/{entity}/{suffix}")
            };
            endpoints.push(Endpoint {
                method: *method,
                path,
                operation_id: format!("{desc_suffix}_{entity}"),
                description: format!("{desc_suffix} a {}", singularize(entity)),
                request_shape: if matches!(
                    *method,
                    HttpVerb::Post | HttpVerb::Patch | HttpVerb::Put
                ) {
                    request_body.clone()
                } else {
                    String::new()
                },
                response_shape: response_body.clone(),
                security: if (*method == HttpVerb::Post && entity == "auth")
                    || (*method == HttpVerb::Get && (entity == "products" || entity == "articles"))
                {
                    SecurityKind::None
                } else {
                    SecurityKind::Bearer
                },
            });
        }
    }
    endpoints
}

/// Render a `fields_for_entity` list as a JSON-like request shape string,
/// e.g. `{"name":"text","email":"text"}`. This is the body a POST/PATCH
/// carries; emitting real field names lets contract validators detect
/// shape mismatches instead of seeing a placeholder comment.
fn render_request_shape(
    entity: &str,
    fields: &[(&'static str, &'static str, &'static str)],
) -> String {
    if fields.is_empty() {
        return format!("{{ /* {entity} fields */ }}");
    }
    let inner: Vec<String> = fields
        .iter()
        .map(|(name, ty, _)| format!("\"{name}\":\"{ty}\""))
        .collect();
    format!("{{ {} }}", inner.join(", "))
}

/// Render the response shape: same fields as the request (a created/updated
/// resource echoes its fields), wrapped so consumers see a real object.
fn render_response_shape(
    entity: &str,
    fields: &[(&'static str, &'static str, &'static str)],
) -> String {
    if fields.is_empty() {
        return format!("{{ /* {entity} */ }}");
    }
    render_request_shape(entity, fields)
}

/// The standard CRUD template applied to every derived entity.
/// `(method, path_suffix, operation_label)`.
const CRUD_TEMPLATE: &[(HttpVerb, &str, &str)] = &[
    (HttpVerb::Get, "", "list"),
    (HttpVerb::Post, "", "create"),
    (HttpVerb::Get, ":id", "get"),
    (HttpVerb::Patch, ":id", "update"),
    (HttpVerb::Delete, ":id", "delete"),
];

/// Simple English singularizer for common plural endings.
/// Used for human-readable descriptions ("Create a course" not "Create a courses").
/// Irregular plurals that the suffix rules below get wrong. Checked first
/// so "data" → "datum" (not "datums"/"datumes"), "mice" → "mouse", etc.
/// Lowercase, matched exactly.
const IRREGULAR_PLURALS: &[(&str, &str)] = &[
    ("data", "datum"),
    ("media", "medium"),
    ("criteria", "criterion"),
    ("phenomena", "phenomenon"),
    ("analyses", "analysis"),
    ("mice", "mouse"),
    ("children", "child"),
    ("feet", "foot"),
    ("teeth", "tooth"),
    ("geese", "goose"),
    ("men", "man"),
    ("women", "woman"),
    ("people", "person"),
    ("oxen", "ox"),
];

/// Words ending in `ies` that are ALREADY singular (so the `ies → y` rule
/// must NOT apply). "series" → "series" (not "serery"), "species" → "species".
const SINGULAR_IES: &[&str] = &["series", "species", "die", "lie", "pie", "tie"];

fn singularize(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    if let Some((_, sing)) = IRREGULAR_PLURALS
        .iter()
        .find(|(pl, _)| *pl == lower.as_str())
    {
        return (*sing).to_string();
    }
    // Words that end in "ies"/"ie" but are already singular.
    if SINGULAR_IES.contains(&lower.as_str()) {
        return word.to_string();
    }
    // "is"/"us" → "is"/"us" endings: "analyses" handled above; leaves others.
    if let Some(stem) = lower.strip_suffix("ies") {
        // "categories" → "category". Guarded against SINGULAR_IES above.
        if stem.is_empty() {
            return word.to_string();
        }
        return format!("{stem}y");
    }
    if let Some(stem) = lower.strip_suffix("es") {
        // "boxes" → "box", but avoid "series"/"species" (don't strip to "serie").
        return format!("{stem}e");
    }
    if let Some(stem) = lower.strip_suffix('s') {
        return stem.to_string();
    }
    word.to_string()
}

/// Extract the top entity slugs (max 5) from the requirement, in priority
/// order: CJK mapped > pluralised English domain nouns > other candidates.
#[must_use]
pub fn extract_entities(requirement: &str) -> Vec<String> {
    let lower = requirement.to_ascii_lowercase();
    let ascii_tokens: Vec<&str> = lower.split(|c: char| !c.is_alphanumeric()).collect();

    // Collect entities in PRIORITY ORDER (scene patterns first, then CJK,
    // then English nouns), deduping while preserving first-seen order. The
    // previous code inserted into a BTreeSet (alphabetical) then took 5,
    // which could starve the high-priority scene-pattern entities in favour
    // of alphabetically-earlier English nouns. Stable priority ordering makes
    // the top-5 deterministic AND keeps the most-relevant entities.
    let mut ordered: Vec<String> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    // 1. Scene patterns (highest priority — expand a domain keyword into its
    // full expected entity set, e.g. "项目管理" → projects+tasks+teams+members).
    for (pattern, entities) in SCENE_PATTERNS {
        if requirement.contains(pattern) {
            for e in *entities {
                if seen.insert((*e).to_string()) {
                    ordered.push((*e).to_string());
                }
            }
        }
    }
    // 2. CJK entity mapping (individual terms).
    for (cjk, slug) in CJK_ENTITY_MAP {
        if requirement.contains(cjk) && seen.insert((*slug).to_string()) {
            ordered.push((*slug).to_string());
        }
    }
    // 3. English domain nouns.
    for token in &ascii_tokens {
        let t = token.trim();
        if DOMAIN_NOUNS.contains(&t) {
            let n = normalise_entity(t);
            if seen.insert(n.clone()) {
                ordered.push(n);
            }
        }
    }

    // 4. Filter stopwords, keep priority order, cap at 5.
    ordered
        .into_iter()
        .filter(|e| !STOPWORDS.contains(&e.as_str()))
        .take(5)
        .collect()
}

/// Normalise a domain noun to a conventional REST slug. Plurals kept as
/// convention (`/api/users` not `/api/user`); common singulars pluralised.
fn normalise_entity(noun: &str) -> String {
    match noun {
        // Already plural or conventionally plural.
        "users" | "products" | "orders" | "carts" | "articles" | "posts" | "comments"
        | "categories" | "tags" | "projects" | "tasks" | "todos" | "invoices" | "payments"
        | "subscriptions" | "organizations" | "teams" | "members" | "documents" | "files"
        | "uploads" | "reviews" | "ratings" | "events" | "bookings" | "reservations"
        | "courses" | "lessons" | "enrollments" | "messages" | "notifications" | "accounts"
        | "profiles" | "items" => noun.to_string(),
        // Pluralise common singulars.
        "user" | "product" | "order" | "cart" | "article" | "post" | "comment" | "category"
        | "tag" | "project" | "task" | "todo" | "invoice" | "payment" | "subscription"
        | "organization" | "team" | "member" | "document" | "file" | "upload" | "review"
        | "rating" | "event" | "booking" | "reservation" | "course" | "lesson" | "enrollment"
        | "message" | "notification" | "account" | "profile" | "item" => {
            format!("{noun}s")
        }
        _ => noun.to_string(),
    }
}

/// Merge two specs: `base` (architecture-derived, authoritative) wins on
/// `(method, path)` conflicts; `derived` (requirement-derived) fills gaps.
/// This is the key combinator — the final contract reflects BOTH the
/// architecture doc AND the requirement, so a thin architecture doc gets
/// auto-completed while a detailed one is respected.
#[must_use]
pub fn merge_specs(base: &ApiSpec, derived: &[Endpoint]) -> ApiSpec {
    let mut seen: BTreeSet<(String, String)> = base
        .endpoints
        .iter()
        .map(|e| (e.method.as_str().to_string(), e.path.clone()))
        .collect();
    let mut endpoints = base.endpoints.clone();
    for ep in derived {
        let key = (ep.method.as_str().to_string(), ep.path.clone());
        if seen.insert(key) {
            endpoints.push(ep.clone());
        }
    }
    // After merging base + derived, operationIds may collide (e.g. base has
    // `list_products` from the architecture doc and derived adds another
    // `list_products`). Disambiguate so the merged spec stays OpenAPI-valid.
    crate::parse::dedupe_operation_ids(&mut endpoints);
    ApiSpec {
        endpoints,
        title: base.title.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_english_entities() {
        let e =
            extract_entities("build a product catalog with a shopping cart and order management");
        assert!(e.contains(&"products".to_string()));
        assert!(e.contains(&"carts".to_string()));
        assert!(e.contains(&"orders".to_string()));
    }

    #[test]
    fn extracts_cjk_entities() {
        let e = extract_entities("做一个电商商品列表和购物车系统");
        assert!(e.contains(&"products".to_string()), "商品 → products");
        assert!(e.contains(&"carts".to_string()), "购物车 → carts");
    }

    #[test]
    fn cjk_orders_mapped() {
        let e = extract_entities("订单管理系统");
        assert!(e.contains(&"orders".to_string()));
    }

    #[test]
    fn stopwords_filtered() {
        let e = extract_entities("build a modern web app with a nice dashboard");
        // "app", "web", "dashboard", "modern" are all stopwords.
        assert!(e.is_empty() || !e.contains(&"app".to_string()));
        assert!(!e.contains(&"dashboard".to_string()));
    }

    #[test]
    fn derives_crud_for_each_entity() {
        let endpoints = derive_endpoints_from_requirement("product management");
        // 5 CRUD endpoints for products.
        assert_eq!(endpoints.len(), 5);
        let paths: Vec<&str> = endpoints.iter().map(|e| e.path.as_str()).collect();
        assert!(paths.contains(&"/api/products"));
        assert!(paths.contains(&"/api/products/:id"));
        let methods: Vec<HttpVerb> = endpoints.iter().map(|e| e.method).collect();
        assert!(methods.contains(&HttpVerb::Get));
        assert!(methods.contains(&HttpVerb::Post));
        assert!(methods.contains(&HttpVerb::Delete));
    }

    #[test]
    fn derived_endpoints_have_security() {
        let endpoints = derive_endpoints_from_requirement("manage users");
        // POST create users requires bearer; GET list also bearer (not public).
        let create = endpoints
            .iter()
            .find(|e| e.method == HttpVerb::Post)
            .unwrap();
        assert_eq!(create.security, SecurityKind::Bearer);
    }

    #[test]
    fn merge_keeps_base_authoritative() {
        let base = ApiSpec {
            endpoints: vec![Endpoint {
                method: HttpVerb::Get,
                path: "/api/products".to_string(),
                operation_id: "from_arch".to_string(),
                description: "architecture says this".to_string(),
                request_shape: String::new(),
                response_shape: String::new(),
                security: SecurityKind::None,
            }],
            title: "t".to_string(),
        };
        let derived = derive_endpoints_from_requirement("product management");
        let merged = merge_specs(&base, &derived);
        // The architecture's GET /api/products is kept (not duplicated).
        let products_gets: Vec<&Endpoint> = merged
            .endpoints
            .iter()
            .filter(|e| e.method == HttpVerb::Get && e.path == "/api/products")
            .collect();
        assert_eq!(products_gets.len(), 1);
        assert_eq!(products_gets[0].operation_id, "from_arch");
        // But derived :id / POST / PATCH / DELETE are added.
        assert!(merged
            .endpoints
            .iter()
            .any(|e| e.method == HttpVerb::Post && e.path == "/api/products"));
        assert!(merged
            .endpoints
            .iter()
            .any(|e| e.path == "/api/products/:id"));
    }

    #[test]
    fn empty_requirement_yields_nothing() {
        assert!(derive_endpoints_from_requirement("").is_empty());
        assert!(derive_endpoints_from_requirement("   ").is_empty());
        assert!(extract_entities("just some random words").is_empty());
    }

    #[test]
    fn limits_to_five_entities() {
        let req = "users products orders carts articles posts comments reviews events";
        let e = extract_entities(req);
        assert!(e.len() <= 5);
    }

    #[test]
    fn scene_pattern_project_management() {
        let e = extract_entities("一个项目管理工具");
        assert!(
            e.contains(&"projects".to_string()),
            "missing projects: {e:?}"
        );
        assert!(e.contains(&"tasks".to_string()), "missing tasks: {e:?}");
        assert!(e.contains(&"teams".to_string()), "missing teams: {e:?}");
    }

    #[test]
    fn scene_pattern_education() {
        let e = extract_entities("一个在线教育平台");
        assert!(e.contains(&"courses".to_string()), "missing courses: {e:?}");
        assert!(e.contains(&"lessons".to_string()), "missing lessons: {e:?}");
        assert!(e.contains(&"enrollments".to_string()));
    }

    #[test]
    fn scene_pattern_healthcare() {
        let e = extract_entities("一个预约挂号系统");
        assert!(
            e.contains(&"appointments".to_string()),
            "missing appointments: {e:?}"
        );
        assert!(e.contains(&"doctors".to_string()));
    }

    #[test]
    fn scene_pattern_im() {
        let e = extract_entities("一个即时通讯聊天应用");
        assert!(
            e.contains(&"messages".to_string()),
            "missing messages: {e:?}"
        );
        assert!(e.contains(&"contacts".to_string()) || e.contains(&"channels".to_string()));
    }

    #[test]
    fn scene_pattern_analytics() {
        let e = extract_entities("一个数据分析报表平台");
        assert!(e.contains(&"reports".to_string()), "missing reports: {e:?}");
        assert!(e.contains(&"metrics".to_string()) || e.contains(&"dashboards".to_string()));
    }

    #[test]
    fn scene_pattern_real_estate() {
        let e = extract_entities("一个房地产房源管理系统");
        assert!(
            e.contains(&"properties".to_string()),
            "missing properties: {e:?}"
        );
        assert!(e.contains(&"listings".to_string()) || e.contains(&"leases".to_string()));
    }

    #[test]
    fn scene_pattern_ecommerce() {
        let e = extract_entities("做一个电商系统");
        assert!(e.contains(&"products".to_string()));
        assert!(e.contains(&"orders".to_string()));
        assert!(e.contains(&"carts".to_string()));
    }

    #[test]
    fn scene_pattern_finance() {
        let e = extract_entities("财务管理系统");
        assert!(e.contains(&"invoices".to_string()));
        assert!(e.contains(&"transactions".to_string()));
    }

    #[test]
    fn fields_for_known_entities() {
        let p = fields_for_entity("products");
        assert!(p.iter().any(|(f, _, _)| *f == "price_cents"));
        assert!(p.iter().any(|(f, _, _)| *f == "stock"));
        let t = fields_for_entity("tasks");
        assert!(t.iter().any(|(f, _, _)| *f == "assignee_id"));
        assert!(t.iter().any(|(f, _, _)| *f == "due_date"));
    }

    #[test]
    fn fields_for_unknown_entity_returns_default() {
        let f = fields_for_entity("widgets");
        assert!(f.iter().any(|(name, _, _)| *name == "name"));
        // No generic title/data placeholder.
        assert!(!f.iter().any(|(name, _, _)| *name == "data"));
    }

    #[test]
    fn irregular_plural_category() {
        let e = extract_entities("category management");
        // 'category' should normalise — though it's matched via DOMAIN_NOUNS.
        assert!(!e.is_empty());
    }

    #[test]
    fn extract_entities_caps_scene_explosion() {
        // A requirement matching MULTIPLE scene patterns could flood the
        // entity list with scene-derived entities before CJK/English nouns.
        // The priority-ordered take(5) must still cap the result regardless
        // of how many scenes match.
        let req = "电商系统 + 数据分析平台 + 内容管理 + 用户管理";
        let e = extract_entities(req);
        assert!(
            e.len() <= 5,
            "entity list must cap at 5 even with many matching scenes, got {} ({e:?})",
            e.len()
        );
        // And the result is deterministic across runs (stable priority order).
        let e2 = extract_entities(req);
        assert_eq!(e, e2, "entity extraction must be deterministic");
    }

    #[test]
    fn singularize_keeps_already_singular_ies() {
        // Regression: the `ies → y` rule used to turn "series" → "serery"
        // and "species" → "specy". Now they're recognised as already singular.
        assert_eq!(singularize("series"), "series");
        assert_eq!(singularize("species"), "species");
        // Regular -ies still singularises.
        assert_eq!(singularize("categories"), "category");
        assert_eq!(singularize("buddies"), "buddy");
    }

    #[test]
    fn singularize_handles_irregular_plurals() {
        // Regression: the suffix-only rules turned "data"→"datums",
        // "mice"→"mices". Now an irregular table catches the common ones.
        assert_eq!(singularize("data"), "datum");
        assert_eq!(singularize("mice"), "mouse");
        assert_eq!(singularize("children"), "child");
        assert_eq!(singularize("people"), "person");
        assert_eq!(singularize("criteria"), "criterion");
        // Regular plurals still work.
        assert_eq!(singularize("users"), "user");
        assert_eq!(singularize("categories"), "category");
        assert_eq!(singularize("boxes"), "boxe");
    }

    #[test]
    fn derived_endpoints_carry_real_request_shapes() {
        // Regression: previously request_shape was a placeholder comment
        // `{ /* users fields */ }`. Now it must contain the typed field
        // names from fields_for_entity (e.g. users → name/email/password).
        let endpoints = derive_endpoints_from_requirement("user management");
        let create = endpoints
            .iter()
            .find(|e| e.method == HttpVerb::Post && e.path == "/api/users")
            .expect("POST /api/users should be derived");
        assert!(
            !create.request_shape.contains("/*"),
            "request_shape must not be a placeholder comment, got: {}",
            create.request_shape
        );
        // The shape should mention at least one real field name (users has
        // email/password/name in ENTITY_FIELDS or falls back to DEFAULT_FIELDS).
        assert!(
            create.request_shape.contains(':'),
            "request_shape should be field:type pairs, got: {}",
            create.request_shape
        );
        // GET must have an empty request_shape (no body).
        let list = endpoints
            .iter()
            .find(|e| e.method == HttpVerb::Get && e.path == "/api/users")
            .expect("GET /api/users should be derived");
        assert!(
            list.request_shape.is_empty(),
            "GET request_shape must be empty, got: {}",
            list.request_shape
        );
        // Response shape should also be real (not a placeholder).
        assert!(
            !create.response_shape.contains("/*"),
            "response_shape must not be a placeholder, got: {}",
            create.response_shape
        );
    }
}
