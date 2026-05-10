use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Span {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Span { line, column, offset }
    }

    pub fn zero() -> Self {
        Span { line: 0, column: 0, offset: 0 }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLiteral(i64),
    LongLiteral(i64),
    DoubleLiteral(f64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    CharLiteral(char),
    NullLiteral,
    UnitLiteral,
    SymbolLiteral(String),

    // Identifiers
    Identifier(String),
    Operator(String),

    // Keywords
    Abstract,
    Case,
    Catch,
    Class,
    Def,
    Do,
    Else,
    Extends,
    False,
    Final,
    Finally,
    For,
    ForSome,
    If,
    Implicit,
    Import,
    Lazy,
    Match,
    New,
    Null,
    Object,
    Override,
    Package,
    Private,
    Protected,
    Return,
    Sealed,
    Super,
    This,
    Throw,
    Trait,
    True,
    Try,
    Type,
    Val,
    Var,
    While,
    With,
    Yield,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Semicolon,
    Colon,
    Equals,
    Underscore,
    Arrow,
    LeftArrow,
    At,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    Exclaim,
    LessThan,
    GreaterThan,
    BangEquals,
    EqualsEquals,
    LessEquals,
    GreaterEquals,
    AmpersandAmpersand,
    PipePipe,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    ColonEquals,

    // String interpolation
    InterpolationStart(String, String), // prefix, first part
    InterpolationPart(String),
    InterpolationEnd(String),

    // Special
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }

    pub fn eof(span: Span) -> Self {
        Token::new(TokenKind::Eof, span)
    }

    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::IntLiteral(v) => write!(f, "{}", v),
            TokenKind::LongLiteral(v) => write!(f, "{}L", v),
            TokenKind::DoubleLiteral(v) => write!(f, "{}", v),
            TokenKind::FloatLiteral(v) => write!(f, "{}f", v),
            TokenKind::BoolLiteral(b) => write!(f, "{}", b),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::CharLiteral(c) => write!(f, "'{}'", c),
            TokenKind::NullLiteral => write!(f, "null"),
            TokenKind::UnitLiteral => write!(f, "()"),
            TokenKind::SymbolLiteral(s) => write!(f, "'{}", s),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::Operator(s) => write!(f, "{}", s),
            TokenKind::Abstract => write!(f, "abstract"),
            TokenKind::Case => write!(f, "case"),
            TokenKind::Catch => write!(f, "catch"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Def => write!(f, "def"),
            TokenKind::Do => write!(f, "do"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Extends => write!(f, "extends"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Final => write!(f, "final"),
            TokenKind::Finally => write!(f, "finally"),
            TokenKind::For => write!(f, "for"),
            TokenKind::ForSome => write!(f, "forSome"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Implicit => write!(f, "implicit"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Lazy => write!(f, "lazy"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::New => write!(f, "new"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Object => write!(f, "object"),
            TokenKind::Override => write!(f, "override"),
            TokenKind::Package => write!(f, "package"),
            TokenKind::Private => write!(f, "private"),
            TokenKind::Protected => write!(f, "protected"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Sealed => write!(f, "sealed"),
            TokenKind::Super => write!(f, "super"),
            TokenKind::This => write!(f, "this"),
            TokenKind::Throw => write!(f, "throw"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::True => write!(f, "true"),
            TokenKind::Try => write!(f, "try"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Val => write!(f, "val"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::While => write!(f, "while"),
            TokenKind::With => write!(f, "with"),
            TokenKind::Yield => write!(f, "yield"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::Arrow => write!(f, "=>"),
            TokenKind::LeftArrow => write!(f, "<-"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::Tilde => write!(f, "~"),
            TokenKind::Exclaim => write!(f, "!"),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::BangEquals => write!(f, "!="),
            TokenKind::EqualsEquals => write!(f, "=="),
            TokenKind::LessEquals => write!(f, "<="),
            TokenKind::GreaterEquals => write!(f, ">="),
            TokenKind::AmpersandAmpersand => write!(f, "&&"),
            TokenKind::PipePipe => write!(f, "||"),
            TokenKind::LeftShift => write!(f, "<<"),
            TokenKind::RightShift => write!(f, ">>"),
            TokenKind::UnsignedRightShift => write!(f, ">>>"),
            TokenKind::PlusEquals => write!(f, "+="),
            TokenKind::MinusEquals => write!(f, "-="),
            TokenKind::StarEquals => write!(f, "*="),
            TokenKind::SlashEquals => write!(f, "/="),
            TokenKind::ColonEquals => write!(f, ":="),
            TokenKind::InterpolationStart(prefix, _) => write!(f, "{}\"...\"", prefix),
            TokenKind::InterpolationPart(s) => write!(f, "...{}...", s),
            TokenKind::InterpolationEnd(s) => write!(f, "...{}\"", s),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}
