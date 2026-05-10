use crate::token::{Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        LexError { message: message.into(), span }
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lexer error at {}: {}", self.span, self.message)
    }
}

impl std::error::Error for LexError {}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        while !lexer.is_at_end() {
            if let Some(token) = lexer.skip_whitespace_and_comments()? {
                tokens.push(token);
                continue;
            }
            if let Some(token) = lexer.next_token()? {
                tokens.push(token);
            }
        }
        tokens.push(Token::eof(lexer.current_span()));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn current_char(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn peek2_char(&self) -> Option<char> {
        self.source.get(self.pos + 2).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        let c = self.source[self.pos];
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn advance_if(&mut self, expected: char) -> bool {
        if self.current_char() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn current_span(&self) -> Span {
        Span::new(self.line, self.column, self.pos)
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<Option<Token>, LexError> {
        loop {
            match self.current_char() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') => match self.peek_char() {
                    Some('/') => {
                        while !self.is_at_end() && self.current_char() != Some('\n') {
                            self.advance();
                        }
                    }
                    Some('*') => {
                        self.advance();
                        self.advance();
                        let mut depth = 1;
                        while !self.is_at_end() && depth > 0 {
                            if self.current_char() == Some('/') && self.peek_char() == Some('*') {
                                depth += 1;
                                self.advance();
                                self.advance();
                            } else if self.current_char() == Some('*') && self.peek_char() == Some('/') {
                                depth -= 1;
                                self.advance();
                                self.advance();
                            } else {
                                self.advance();
                            }
                        }
                        if depth > 0 {
                            return Err(LexError::new("unterminated block comment", self.current_span()));
                        }
                    }
                    _ => return Ok(None),
                },
                _ => return Ok(None),
            }
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        let span = self.current_span();
        let c = match self.current_char() {
            Some(c) => c,
            None => return Ok(None),
        };

        if c == '"' && self.peek_char() == Some('"') && self.peek2_char() == Some('"') {
            return self.read_multiline_string(span).map(Some);
        }

        if c == '"' {
            return self.read_string(span).map(Some);
        }

        if c == '\'' {
            return self.read_char_or_symbol(span).map(Some);
        }

        if c.is_ascii_digit() {
            return self.read_number(span).map(Some);
        }

        if c == '.' && self.peek_char().map_or(false, |p| p.is_ascii_digit()) {
            return self.read_number(span).map(Some);
        }

        if c == '`' {
            return self.read_backtick_identifier(span).map(Some);
        }

        if Self::is_identifier_start(c) {
            return Ok(Some(self.read_identifier_or_keyword(span)));
        }

        if Self::is_operator_char(c) {
            return Ok(Some(self.read_operator_or_delimiter(span)?));
        }

        if let Some(tk) = self.read_delimiter(c, span) {
            return Ok(Some(tk));
        }

        Err(LexError::new(format!("unexpected character '{}'", c), span))
    }

    fn read_string(&mut self, start_span: Span) -> Result<Token, LexError> {
        self.advance(); // opening "
        let mut s = String::new();
        while !self.is_at_end() {
            match self.current_char() {
                Some('"') => {
                    self.advance();
                    return Ok(Token::new(TokenKind::StringLiteral(s), start_span));
                }
                Some('\\') => {
                    self.advance();
                    let escaped = self.read_escape()?;
                    s.push(escaped);
                }
                Some(c) => {
                    self.advance();
                    s.push(c);
                }
                None => {
                    return Err(LexError::new("unterminated string", start_span));
                }
            }
        }
        Err(LexError::new("unterminated string", start_span))
    }

    fn read_multiline_string(&mut self, start_span: Span) -> Result<Token, LexError> {
        self.advance(); // first "
        self.advance(); // second "
        self.advance(); // third "
        let mut s = String::new();
        while !self.is_at_end() {
            if self.current_char() == Some('"')
                && self.peek_char() == Some('"')
                && self.peek2_char() == Some('"')
            {
                self.advance();
                self.advance();
                self.advance();
                return Ok(Token::new(TokenKind::StringLiteral(s), start_span));
            }
            s.push(self.advance().unwrap());
        }
        Err(LexError::new("unterminated multi-line string", start_span))
    }

    fn read_escape(&mut self) -> Result<char, LexError> {
        let span = self.current_span();
        match self.advance() {
            Some('n') => Ok('\n'),
            Some('t') => Ok('\t'),
            Some('r') => Ok('\r'),
            Some('\\') => Ok('\\'),
            Some('\'') => Ok('\''),
            Some('"') => Ok('"'),
            Some('0') => Ok('\0'),
            Some('u') => {
                let mut hex = String::new();
                for _ in 0..4 {
                    match self.advance() {
                        Some(c) if c.is_ascii_hexdigit() => hex.push(c),
                        _ => return Err(LexError::new("invalid unicode escape", span)),
                    }
                }
                let code = u32::from_str_radix(&hex, 16).map_err(|_| LexError::new("invalid unicode escape", span))?;
                char::from_u32(code).ok_or_else(|| LexError::new("invalid unicode escape", span))
            }
            _ => Err(LexError::new("invalid escape sequence", span)),
        }
    }

    fn read_char_or_symbol(&mut self, start_span: Span) -> Result<Token, LexError> {
        self.advance(); // opening '
        if self.is_at_end() {
            return Err(LexError::new("unterminated character literal", start_span));
        }

        if Self::is_identifier_start(self.current_char().unwrap())
            && self.current_char() != Some('\\')
        {
            let mut sym = String::new();
            while !self.is_at_end() && Self::is_identifier_continue(self.current_char().unwrap()) {
                sym.push(self.advance().unwrap());
            }
            if sym.len() == 1 && self.current_char() == Some('\'') {
                self.advance();
                return Ok(Token::new(TokenKind::CharLiteral(sym.chars().next().unwrap()), start_span));
            }
            return Ok(Token::new(TokenKind::SymbolLiteral(sym), start_span));
        }

        let c = if self.current_char() == Some('\\') {
            self.advance();
            self.read_escape()?
        } else {
            self.advance().unwrap()
        };

        if self.advance_if('\'') {
            Ok(Token::new(TokenKind::CharLiteral(c), start_span))
        } else {
            Err(LexError::new("unterminated character literal", start_span))
        }
    }

    fn read_number(&mut self, start_span: Span) -> Result<Token, LexError> {
        let mut num = String::new();

        if self.current_char() == Some('0') {
            self.advance();
            match self.current_char() {
                Some('x') | Some('X') => {
                    self.advance();
                    while !self.is_at_end() && self.current_char().map_or(false, |c| c.is_ascii_hexdigit()) {
                        num.push(self.advance().unwrap());
                    }
                    let value = i64::from_str_radix(&num, 16)
                        .map_err(|_| LexError::new("invalid hex literal", start_span))?;
                    return self.read_number_suffix(value, false, start_span);
                }
                Some('b') | Some('B') => {
                    self.advance();
                    while !self.is_at_end() && self.current_char().map_or(false, |c| c == '0' || c == '1') {
                        num.push(self.advance().unwrap());
                    }
                    let value = i64::from_str_radix(&num, 2)
                        .map_err(|_| LexError::new("invalid binary literal", start_span))?;
                    return self.read_number_suffix(value, false, start_span);
                }
                Some('o') | Some('O') => {
                    self.advance();
                    while !self.is_at_end() && self.current_char().map_or(false, |c| ('0'..='7').contains(&c)) {
                        num.push(self.advance().unwrap());
                    }
                    let value = i64::from_str_radix(&num, 8)
                        .map_err(|_| LexError::new("invalid octal literal", start_span))?;
                    return self.read_number_suffix(value, false, start_span);
                }
                _ => {
                    num.push('0');
                }
            }
        }

        while !self.is_at_end() && self.current_char().map_or(false, |c| c.is_ascii_digit()) {
            num.push(self.advance().unwrap());
        }

        if self.current_char() == Some('.') && self.peek_char().map_or(true, |c| c.is_ascii_digit()) {
            if self.peek_char().is_some() {
                num.push(self.advance().unwrap()); // .
                while !self.is_at_end() && self.current_char().map_or(false, |c| c.is_ascii_digit()) {
                    num.push(self.advance().unwrap());
                }
            }
        }

        if self.current_char() == Some('e') || self.current_char() == Some('E') {
            num.push(self.advance().unwrap());
            if self.current_char() == Some('+') || self.current_char() == Some('-') {
                num.push(self.advance().unwrap());
            }
            while !self.is_at_end() && self.current_char().map_or(false, |c| c.is_ascii_digit()) {
                num.push(self.advance().unwrap());
            }
        }

        let is_float = num.contains('.') || num.contains('e') || num.contains('E');

        if is_float {
            let value = num.parse::<f64>()
                .map_err(|_| LexError::new("invalid float literal", start_span))?;
            if self.current_char() == Some('f') || self.current_char() == Some('F') {
                self.advance();
                Ok(Token::new(TokenKind::FloatLiteral(value as f64), start_span))
            } else {
                if self.current_char() == Some('d') || self.current_char() == Some('D') {
                    self.advance();
                }
                Ok(Token::new(TokenKind::DoubleLiteral(value), start_span))
            }
        } else {
            let value = num.parse::<i64>()
                .map_err(|_| LexError::new("invalid integer literal", start_span))?;
            self.read_number_suffix(value, false, start_span)
        }
    }

    fn read_number_suffix(&mut self, value: i64, _is_float: bool, span: Span) -> Result<Token, LexError> {
        match self.current_char() {
            Some('l') | Some('L') => {
                self.advance();
                Ok(Token::new(TokenKind::LongLiteral(value), span))
            }
            Some('f') | Some('F') => {
                self.advance();
                Ok(Token::new(TokenKind::FloatLiteral(value as f64), span))
            }
            Some('d') | Some('D') => {
                self.advance();
                Ok(Token::new(TokenKind::DoubleLiteral(value as f64), span))
            }
            _ => Ok(Token::new(TokenKind::IntLiteral(value), span)),
        }
    }

    fn read_identifier_or_keyword(&mut self, start_span: Span) -> Token {
        let mut ident = String::new();
        while !self.is_at_end() && self.current_char().map_or(false, |c| Self::is_identifier_continue(c)) {
            ident.push(self.advance().unwrap());
        }
        let kind = Self::lookup_keyword(&ident).unwrap_or_else(|| {
            if ident == "true" {
                TokenKind::BoolLiteral(true)
            } else if ident == "false" {
                TokenKind::BoolLiteral(false)
            } else if ident == "null" {
                TokenKind::NullLiteral
            } else {
                TokenKind::Identifier(ident)
            }
        });
        Token::new(kind, start_span)
    }

    fn lookup_keyword(ident: &str) -> Option<TokenKind> {
        Some(match ident {
            "abstract" => TokenKind::Abstract,
            "case" => TokenKind::Case,
            "catch" => TokenKind::Catch,
            "class" => TokenKind::Class,
            "def" => TokenKind::Def,
            "do" => TokenKind::Do,
            "else" => TokenKind::Else,
            "extends" => TokenKind::Extends,
            "final" => TokenKind::Final,
            "finally" => TokenKind::Finally,
            "for" => TokenKind::For,
            "forSome" => TokenKind::ForSome,
            "if" => TokenKind::If,
            "implicit" => TokenKind::Implicit,
            "import" => TokenKind::Import,
            "lazy" => TokenKind::Lazy,
            "match" => TokenKind::Match,
            "new" => TokenKind::New,
            "object" => TokenKind::Object,
            "override" => TokenKind::Override,
            "package" => TokenKind::Package,
            "private" => TokenKind::Private,
            "protected" => TokenKind::Protected,
            "return" => TokenKind::Return,
            "sealed" => TokenKind::Sealed,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "throw" => TokenKind::Throw,
            "trait" => TokenKind::Trait,
            "try" => TokenKind::Try,
            "type" => TokenKind::Type,
            "val" => TokenKind::Val,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            "with" => TokenKind::With,
            "yield" => TokenKind::Yield,
            _ => return None,
        })
    }

    fn read_backtick_identifier(&mut self, start_span: Span) -> Result<Token, LexError> {
        self.advance(); // opening `
        let mut ident = String::new();
        while !self.is_at_end() {
            match self.current_char() {
                Some('`') => {
                    self.advance();
                    return Ok(Token::new(TokenKind::Identifier(ident), start_span));
                }
                Some(c) => {
                    self.advance();
                    ident.push(c);
                }
                None => return Err(LexError::new("unterminated backtick identifier", start_span)),
            }
        }
        Err(LexError::new("unterminated backtick identifier", start_span))
    }

    fn read_operator_or_delimiter(&mut self, start_span: Span) -> Result<Token, LexError> {
        let c = self.current_char().unwrap();

        match c {
            '=' => {
                self.advance();
                if self.advance_if('>') {
                    return Ok(Token::new(TokenKind::Arrow, start_span));
                }
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::EqualsEquals, start_span));
                }
                Ok(Token::new(TokenKind::Equals, start_span))
            }
            '!' => {
                self.advance();
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::BangEquals, start_span));
                }
                Ok(Token::new(TokenKind::Exclaim, start_span))
            }
            '<' => {
                self.advance();
                if self.advance_if('-') {
                    return Ok(Token::new(TokenKind::LeftArrow, start_span));
                }
                if self.current_char() == Some('<') {
                    self.advance();
                    return Ok(Token::new(TokenKind::LeftShift, start_span));
                }
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::LessEquals, start_span));
                }
                Ok(Token::new(TokenKind::LessThan, start_span))
            }
            '>' => {
                self.advance();
                if self.current_char() == Some('>') {
                    self.advance();
                    if self.current_char() == Some('>') {
                        self.advance();
                        return Ok(Token::new(TokenKind::UnsignedRightShift, start_span));
                    }
                    return Ok(Token::new(TokenKind::RightShift, start_span));
                }
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::GreaterEquals, start_span));
                }
                Ok(Token::new(TokenKind::GreaterThan, start_span))
            }
            '&' => {
                self.advance();
                if self.advance_if('&') {
                    return Ok(Token::new(TokenKind::AmpersandAmpersand, start_span));
                }
                Ok(Token::new(TokenKind::Ampersand, start_span))
            }
            '|' => {
                self.advance();
                if self.advance_if('|') {
                    return Ok(Token::new(TokenKind::PipePipe, start_span));
                }
                Ok(Token::new(TokenKind::Pipe, start_span))
            }
            '+' => {
                self.advance();
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::PlusEquals, start_span));
                }
                Ok(Token::new(TokenKind::Plus, start_span))
            }
            '-' => {
                self.advance();
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::MinusEquals, start_span));
                }
                Ok(Token::new(TokenKind::Minus, start_span))
            }
            '*' => {
                self.advance();
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::StarEquals, start_span));
                }
                Ok(Token::new(TokenKind::Star, start_span))
            }
            '/' => {
                self.advance();
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::SlashEquals, start_span));
                }
                Ok(Token::new(TokenKind::Slash, start_span))
            }
            '%' => {
                self.advance();
                Ok(Token::new(TokenKind::Percent, start_span))
            }
            '^' => {
                self.advance();
                Ok(Token::new(TokenKind::Caret, start_span))
            }
            '~' => {
                self.advance();
                Ok(Token::new(TokenKind::Tilde, start_span))
            }
            ':' => {
                self.advance();
                if self.advance_if('=') {
                    return Ok(Token::new(TokenKind::ColonEquals, start_span));
                }
                Ok(Token::new(TokenKind::Colon, start_span))
            }
            _ => {
                let mut op = String::new();
                while !self.is_at_end() && self.current_char().map_or(false, |c| Self::is_operator_char(c)) {
                    op.push(self.advance().unwrap());
                }
                Ok(Token::new(TokenKind::Operator(op), start_span))
            }
        }
    }

    fn read_delimiter(&mut self, c: char, span: Span) -> Option<Token> {
        match c {
            '(' => {
                if self.peek_char() == Some(')') {
                    self.advance();
                    self.advance();
                    Some(Token::new(TokenKind::UnitLiteral, span))
                } else {
                    self.advance();
                    Some(Token::new(TokenKind::LeftParen, span))
                }
            }
            ')' => { self.advance(); Some(Token::new(TokenKind::RightParen, span)) }
            '{' => { self.advance(); Some(Token::new(TokenKind::LeftBrace, span)) }
            '}' => { self.advance(); Some(Token::new(TokenKind::RightBrace, span)) }
            '[' => { self.advance(); Some(Token::new(TokenKind::LeftBracket, span)) }
            ']' => { self.advance(); Some(Token::new(TokenKind::RightBracket, span)) }
            ',' => { self.advance(); Some(Token::new(TokenKind::Comma, span)) }
            '.' => { self.advance(); Some(Token::new(TokenKind::Dot, span)) }
            ';' => { self.advance(); Some(Token::new(TokenKind::Semicolon, span)) }
            ':' => { self.advance(); Some(Token::new(TokenKind::Colon, span)) }
            '_' => {
                self.advance();
                Some(Token::new(TokenKind::Underscore, span))
            }
            '@' => { self.advance(); Some(Token::new(TokenKind::At, span)) }
            _ => None,
        }
    }

    fn is_identifier_start(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    fn is_identifier_continue(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    fn is_operator_char(c: char) -> bool {
        matches!(c,
            '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '~' | '!'
            | '<' | '>' | '=' | '?' | '#' | '@'
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(source: &str) -> Vec<TokenKind> {
        Lexer::tokenize(source).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_integers() {
        assert_eq!(kind("42"), vec![TokenKind::IntLiteral(42), TokenKind::Eof]);
        assert_eq!(kind("0"), vec![TokenKind::IntLiteral(0), TokenKind::Eof]);
        assert_eq!(kind("0xFF"), vec![TokenKind::IntLiteral(255), TokenKind::Eof]);
        assert_eq!(kind("0b1010"), vec![TokenKind::IntLiteral(10), TokenKind::Eof]);
    }

    #[test]
    fn test_longs() {
        assert_eq!(kind("42L"), vec![TokenKind::LongLiteral(42), TokenKind::Eof]);
    }

    #[test]
    fn test_doubles() {
        assert!(matches!(kind("3.14").first(), Some(TokenKind::DoubleLiteral(_))));
        assert!(matches!(kind("1e10").first(), Some(TokenKind::DoubleLiteral(_))));
    }

    #[test]
    fn test_floats() {
        assert!(matches!(kind("3.14f").first(), Some(TokenKind::FloatLiteral(_))));
    }

    #[test]
    fn test_strings() {
        assert_eq!(kind("\"hello\""), vec![TokenKind::StringLiteral("hello".into()), TokenKind::Eof]);
        assert_eq!(kind("\"hello\\nworld\""), vec![TokenKind::StringLiteral("hello\nworld".into()), TokenKind::Eof]);
    }

    #[test]
    fn test_multiline_strings() {
        assert_eq!(
            kind("\"\"\"hello\nworld\"\"\""),
            vec![TokenKind::StringLiteral("hello\nworld".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn test_chars() {
        assert_eq!(kind("'a'"), vec![TokenKind::CharLiteral('a'), TokenKind::Eof]);
        assert_eq!(kind("'\\n'"), vec![TokenKind::CharLiteral('\n'), TokenKind::Eof]);
    }

    #[test]
    fn test_booleans_and_null() {
        assert_eq!(kind("true"), vec![TokenKind::BoolLiteral(true), TokenKind::Eof]);
        assert_eq!(kind("false"), vec![TokenKind::BoolLiteral(false), TokenKind::Eof]);
        assert_eq!(kind("null"), vec![TokenKind::NullLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_unit() {
        assert_eq!(kind("()"), vec![TokenKind::UnitLiteral, TokenKind::Eof]);
    }

    #[test]
    fn test_keywords() {
        assert_eq!(kind("val"), vec![TokenKind::Val, TokenKind::Eof]);
        assert_eq!(kind("def"), vec![TokenKind::Def, TokenKind::Eof]);
        assert_eq!(kind("class"), vec![TokenKind::Class, TokenKind::Eof]);
        assert_eq!(kind("match"), vec![TokenKind::Match, TokenKind::Eof]);
    }

    #[test]
    fn test_operators() {
        assert_eq!(kind("+"), vec![TokenKind::Plus, TokenKind::Eof]);
        assert_eq!(kind("=>"), vec![TokenKind::Arrow, TokenKind::Eof]);
        assert_eq!(kind("<-"), vec![TokenKind::LeftArrow, TokenKind::Eof]);
        assert_eq!(kind("=="), vec![TokenKind::EqualsEquals, TokenKind::Eof]);
        assert_eq!(kind("!="), vec![TokenKind::BangEquals, TokenKind::Eof]);
        assert_eq!(kind("&&"), vec![TokenKind::AmpersandAmpersand, TokenKind::Eof]);
        assert_eq!(kind("||"), vec![TokenKind::PipePipe, TokenKind::Eof]);
        assert_eq!(kind("<<"), vec![TokenKind::LeftShift, TokenKind::Eof]);
        assert_eq!(kind(">>"), vec![TokenKind::RightShift, TokenKind::Eof]);
    }

    #[test]
    fn test_identifiers() {
        assert_eq!(kind("foo"), vec![TokenKind::Identifier("foo".into()), TokenKind::Eof]);
        assert_eq!(kind("_bar"), vec![TokenKind::Identifier("_bar".into()), TokenKind::Eof]);
        assert_eq!(kind("`back tick`"), vec![TokenKind::Identifier("back tick".into()), TokenKind::Eof]);
    }

    #[test]
    fn test_comments() {
        assert_eq!(kind("// comment\n42"), vec![TokenKind::IntLiteral(42), TokenKind::Eof]);
        assert_eq!(kind("/* block */42"), vec![TokenKind::IntLiteral(42), TokenKind::Eof]);
        assert_eq!(kind("/* outer /* inner */ */42"), vec![TokenKind::IntLiteral(42), TokenKind::Eof]);
    }

    #[test]
    fn test_span_tracking() {
        let tokens = Lexer::tokenize("val\n  x").unwrap();
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[1].span.column, 3);
    }
}
