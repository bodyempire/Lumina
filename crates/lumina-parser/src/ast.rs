use lumina_lexer::token::Span;
use serde::{Serialize, Deserialize};

// ── Top-level program ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

impl Program {
    pub fn imports(&self) -> impl Iterator<Item = &ImportDecl> {
        self.statements.iter().filter_map(|s| {
            if let Statement::Import(i) = s { Some(i) } else { None }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Entity(EntityDecl),
    ExternalEntity(ExternalEntityDecl),
    Let(LetStmt),
    Rule(RuleDecl),
    Action(Action),
    Fn(FnDecl),
    Import(ImportDecl),
    Aggregate(AggregateDecl),
}

// ── Function declaration ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnDecl {
    pub name:    String,
    pub params:  Vec<FnParam>,
    pub returns: LuminaType,
    pub body:    Expr,
    pub span:    Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnParam {
    pub name:  String,
    pub type_: LuminaType,
    pub span:  Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDecl {
    pub path: String,
    pub span: Span,
}

// ── Entity declaration ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDecl {
    pub name: String,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Field {
    Stored(StoredField),
    Derived(DerivedField),
    Ref(RefField),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefField {
    pub name: String,
    pub target_entity: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredField {
    pub name: String,
    pub ty: LuminaType,
    pub metadata: FieldMetadata,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedField {
    pub name: String,
    pub expr: Expr,
    pub metadata: FieldMetadata,
    pub span: Span,
}

// ── Field metadata (@doc / @range / @affects) ──────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMetadata {
    pub doc:     Option<String>,
    pub range:   Option<(f64, f64)>,
    pub affects: Vec<String>,
}

// ── Type system ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LuminaType {
    Text,
    Number,
    Boolean,
    Timestamp,
    Entity(String),
    List(Box<LuminaType>),
}

// ── External entity ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntityDecl {
    pub name:          String,
    pub fields:        Vec<Field>,
    pub sync_path:     String,
    pub sync_strategy: SyncStrategy,
    pub poll_interval: Option<Duration>,
    pub span:          Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStrategy {
    Realtime,
    Poll,
    Webhook,
}

// ── Let statement ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LetStmt {
    pub name:  String,
    pub value: LetValue,
    pub span:  Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LetValue {
    Expr(Expr),
    EntityInit(EntityInit),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInit {
    pub entity_name: String,
    pub fields: Vec<(String, Expr)>,
    pub span: Span,
}

// ── Rule declaration ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDecl {
    pub name:     String,
    pub trigger:  RuleTrigger,
    pub actions:  Vec<Action>,
    pub cooldown: Option<Duration>,
    pub on_clear: Option<Vec<Action>>,
    pub span:     Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleTrigger {
    When(Vec<Condition>),
    Any(FleetCondition),
    All(FleetCondition),
    Every(Duration),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub expr:         Expr,
    pub becomes:      Option<Expr>,
    pub for_duration: Option<Duration>,
    pub frequency:    Option<Frequency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetCondition {
    pub entity:       String,
    pub field:        String,
    pub becomes:      Expr,
    pub for_duration: Option<Duration>,
    pub frequency:    Option<Frequency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frequency {
    pub count:  u32,
    pub within: Duration,
    pub span:   Span,
}

// ── Duration (for temporal rules) ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    pub value: f64,
    pub unit:  TimeUnit,
}

impl Duration {
    pub fn to_seconds(&self) -> f64 {
        match self.unit {
            TimeUnit::Seconds => self.value,
            TimeUnit::Minutes => self.value * 60.0,
            TimeUnit::Hours   => self.value * 3600.0,
            TimeUnit::Days    => self.value * 86400.0,
        }
    }

    pub fn to_std_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(self.to_seconds())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

// ── Actions ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Show(Expr),
    Update { target: FieldPath, value: Expr },
    Write { target: FieldPath, value: Expr },
    Create { entity: String, fields: Vec<(String, Expr)> },
    Delete(String),
    Alert(AlertAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAction {
    pub severity: Expr,
    pub message:  Expr,
    pub source:   Option<Expr>,
    pub code:     Option<Expr>,
    pub payload:  Vec<(String, Expr)>,
    pub span:     Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPath {
    pub instance: String,
    pub field:    String,
    pub span:     Span,
}

// ── Expressions ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    Text(String),
    Bool(bool),
    Ident(String),
    FieldAccess {
        obj:   Box<Expr>,
        field: String,
        span:  Span,
    },
    Binary {
        op:    BinOp,
        left:  Box<Expr>,
        right: Box<Expr>,
        span:  Span,
    },
    Unary {
        op:      UnOp,
        operand: Box<Expr>,
        span:    Span,
    },
    If {
        cond:  Box<Expr>,
        then_: Box<Expr>,
        else_: Box<Expr>,
        span:  Span,
    },
    InterpolatedString(Vec<StringSegment>),
    Call {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    ListLiteral(Vec<Expr>),
    Index {
        list:  Box<Expr>,
        index: Box<Expr>,
        span:  Span,
    },
    Prev {
        field: String,
        span:  Span,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringSegment {
    Literal(String),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Gt, Lt, Ge, Le,
    And, Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
    Not,
}

// ── Aggregates ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateDecl {
    pub name:   String,
    pub over:   String,
    pub fields: Vec<AggregateField>,
    pub span:   Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateField {
    pub name: String,
    pub expr: AggregateExpr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregateExpr {
    Avg(String),
    Min(String),
    Max(String),
    Sum(String),
    Count(Option<String>),
    Any(String),
    All(String),
}
