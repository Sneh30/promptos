use crate::ast::{Position, SourceSpan};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Instruction,
    Context,
    Constraint,
    FormatSpec,
    RoleSpec,
    Example,
    MetaInstruction,
    Separator,
    Text,
    Heading,
    Newline,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
    pub text: String,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.position).copied();
        if let Some(c) = ch {
            self.position += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn current_position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    fn read_line(&mut self) -> String {
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            s.push(ch);
            self.advance();
        }
        s
    }

    fn read_heading(&mut self) -> Token {
        let start = self.current_position();
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch == '#' {
                text.push(ch);
                self.advance();
            } else if ch == ' ' {
                text.push(ch);
                self.advance();
                text.push_str(&self.read_line());
                break;
            } else {
                break;
            }
        }
        let end = self.current_position();
        Token {
            kind: TokenKind::Heading,
            span: SourceSpan::new(start, end),
            text: text.trim().to_string(),
        }
    }

    fn read_code_block(&mut self) -> Token {
        let start = self.current_position();
        let mut text = String::new();
        for _ in 0..3 {
            if let Some(ch) = self.advance() {
                text.push(ch);
            }
        }
        text.push('\n');
        loop {
            if self.peek() == Some('`')
                && self.peek_next() == Some('`')
                && self.input.get(self.position + 2) == Some(&'`')
            {
                for _ in 0..3 {
                    if let Some(ch) = self.advance() {
                        text.push(ch);
                    }
                }
                break;
            }
            match self.advance() {
                Some(ch) => text.push(ch),
                None => break,
            }
        }
        let end = self.current_position();
        Token {
            kind: TokenKind::Text,
            span: SourceSpan::new(start, end),
            text,
        }
    }

    fn read_blockquote(&mut self) -> Token {
        let start = self.current_position();
        self.advance();
        self.skip_whitespace();
        let content = self.read_line();
        let end = self.current_position();
        Token {
            kind: TokenKind::Context,
            span: SourceSpan::new(start, end),
            text: content.trim().to_string(),
        }
    }

    fn read_separator(&mut self) -> Token {
        let start = self.current_position();
        let mut text = String::new();
        for _ in 0..3 {
            if let Some(ch) = self.advance() {
                text.push(ch);
            }
        }
        while self.peek() == Some('-') {
            self.advance();
            text.push('-');
        }
        let end = self.current_position();
        Token {
            kind: TokenKind::Separator,
            span: SourceSpan::new(start, end),
            text,
        }
    }

    fn read_list_item(&mut self) -> String {
        let mut s = String::new();
        if self.peek() == Some('-') || self.peek() == Some('*') {
            self.advance();
            s.push_str("- ");
            self.skip_whitespace();
            s.push_str(&self.read_line());
        } else if self.peek().map_or(false, |c| c.is_ascii_digit()) {
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                if let Some(ch) = self.advance() {
                    s.push(ch);
                }
            }
            if self.peek() == Some('.') {
                self.advance();
                s.push_str(". ");
                self.skip_whitespace();
                s.push_str(&self.read_line());
            }
        }
        s
    }

    fn tokenize_line(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let start = self.current_position();

        match self.peek()? {
            '\n' => {
                self.advance();
                Some(Token {
                    kind: TokenKind::Newline,
                    span: SourceSpan::new(start, self.current_position()),
                    text: "\n".to_string(),
                })
            }
            '#' => Some(self.read_heading()),
            '`' if self.peek_next() == Some('`')
                && self.input.get(self.position + 2) == Some(&'`') =>
            {
                Some(self.read_code_block())
            }
            '>' => Some(self.read_blockquote()),
            '-' if self.peek_next() == Some('-')
                && self.input.get(self.position + 2) == Some(&'-') =>
            {
                Some(self.read_separator())
            }
            '-' | '*' | '+' => {
                let text = self.read_list_item();
                if !text.is_empty() {
                    let end = self.current_position();
                    Some(Token {
                        kind: TokenKind::Text,
                        span: SourceSpan::new(start, end),
                        text,
                    })
                } else {
                    let line = self.read_line();
                    let end = self.current_position();
                    Some(Token {
                        kind: TokenKind::Text,
                        span: SourceSpan::new(start, end),
                        text: line,
                    })
                }
            }
            _ => {
                let line = self.read_line();
                let end = self.current_position();
                Some(Token {
                    kind: TokenKind::Text,
                    span: SourceSpan::new(start, end),
                    text: line.trim().to_string(),
                })
            }
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.input.len() {
            return None;
        }
        self.tokenize_line()
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(input);
    while let Some(token) = lexer.next() {
        if token.kind != TokenKind::Newline {
            tokens.push(token);
        }
    }
    tokens.push(Token {
        kind: TokenKind::EOF,
        span: SourceSpan::new(
            Position::new(lexer.line, lexer.column),
            Position::new(lexer.line, lexer.column),
        ),
        text: String::new(),
    });
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::EOF);
    }

    #[test]
    fn test_heading() {
        let tokens = tokenize("# Hello World");
        assert_eq!(tokens[0].kind, TokenKind::Heading);
        assert_eq!(tokens[0].text, "# Hello World");
    }

    #[test]
    fn test_multiple_headings() {
        let tokens = tokenize("# Title\n## Subtitle");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Heading);
        assert_eq!(tokens[1].kind, TokenKind::Heading);
    }

    #[test]
    fn test_code_block() {
        let tokens = tokenize("```rust\nfn main() {}\n```");
        assert!(tokens.iter().any(|t| t.text.contains("fn main()")));
    }

    #[test]
    fn test_blockquote() {
        let tokens = tokenize("> quoted text");
        assert!(tokens.iter().any(|t| t.text.contains("quoted text")));
    }

    #[test]
    fn test_separator() {
        let tokens = tokenize("---");
        assert_eq!(tokens[0].kind, TokenKind::Separator);
    }

    #[test]
    fn test_plain_text() {
        let tokens = tokenize("Hello world");
        assert_eq!(tokens[0].kind, TokenKind::Text);
        assert_eq!(tokens[0].text, "Hello world");
    }
}
