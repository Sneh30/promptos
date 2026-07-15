use crate::ast::*;
use crate::lexer::{tokenize, Token, TokenKind};

const INSTRUCTION_VERBS: &[&str] = &[
    "write",
    "generate",
    "create",
    "analyze",
    "explain",
    "summarize",
    "extract",
    "classify",
    "compare",
    "translate",
    "rewrite",
    "expand",
    "list",
    "describe",
    "define",
    "calculate",
    "implement",
    "design",
    "optimize",
    "debug",
    "convert",
    "format",
    "search",
    "find",
    "evaluate",
    "assess",
    "review",
    "identify",
    "outline",
    "propose",
];

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = tokenize(input);
        Self {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<PromptRoot, String> {
        let mut children = Vec::new();
        while self.position < self.tokens.len() - 1 {
            let token = self.current();
            match token.kind {
                TokenKind::Heading => {
                    if let Ok(section) = self.parse_section() {
                        children.push(PromptNode::Section(section));
                    }
                }
                TokenKind::Separator => {
                    self.advance();
                }
                TokenKind::EOF => break,
                _ => {
                    if let Ok(block) = self.parse_block() {
                        let node = self.classify_block(&block);
                        children.push(node);
                    } else {
                        self.advance();
                    }
                }
            }
        }
        Ok(PromptRoot::new(children))
    }

    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    #[allow(dead_code)]
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn parse_section(&mut self) -> Result<Section, String> {
        let token = self.current().clone();
        let level = token.text.chars().filter(|c| *c == '#').count() as u8;
        let heading = token.text.trim_start_matches('#').trim().to_string();
        let start = token.span.start;
        self.advance();

        let mut children = Vec::new();
        while self.position < self.tokens.len() {
            let t = self.current();
            if t.kind == TokenKind::Heading
                && t.text.chars().filter(|c| *c == '#').count() as u8 <= level
            {
                break;
            }
            if t.kind == TokenKind::EOF {
                break;
            }
            if t.kind == TokenKind::Separator {
                self.advance();
                continue;
            }
            if let Ok(block) = self.parse_block() {
                let node = self.classify_block(&block);
                children.push(node);
            } else {
                self.advance();
            }
        }

        let end = if children.is_empty() {
            self.current().span.start
        } else {
            match children.last() {
                Some(PromptNode::Block(b)) => b.span.end,
                Some(PromptNode::Instruction(i)) => i.span.end,
                Some(PromptNode::Context(c)) => c.span.end,
                Some(PromptNode::Constraint(c)) => c.span.end,
                Some(PromptNode::Section(s)) => s.span.end,
                _ => self.current().span.start,
            }
        };

        Ok(Section {
            heading,
            level,
            children,
            span: SourceSpan::new(start, end),
        })
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        let token = self.current().clone();
        let start = token.span.start;
        let text = token.text.clone();
        self.advance();
        let end = self.current().span.start;
        let mut children = Vec::new();
        if !text.is_empty() {
            children.push(self.classify_text(&text, start));
        }
        Ok(Block {
            block_type: BlockType::Paragraph,
            content: text,
            children,
            span: SourceSpan::new(start, end),
        })
    }

    fn classify_block(&self, block: &Block) -> PromptNode {
        let lower = block.content.to_lowercase();
        if INSTRUCTION_VERBS.iter().any(|v| lower.starts_with(v)) {
            PromptNode::Instruction(Instruction {
                verb: self.parse_verb(&lower),
                object: block.content.clone(),
                modifiers: Vec::new(),
                confidence: 0.8,
                span: block.span,
            })
        } else if lower.contains("must") || lower.contains("should") || lower.contains("cannot") {
            PromptNode::Constraint(Constraint {
                constraint_type: ConstraintType::Positive,
                value: block.content.clone(),
                severity: ConstraintSeverity::Required,
                span: block.span,
            })
        } else if lower.contains("format") || lower.contains("output as") {
            PromptNode::FormatSpec(FormatSpec {
                format_type: block.content.clone(),
                detail: block.content.clone(),
                span: block.span,
            })
        } else if lower.contains("you are") || lower.contains("act as") {
            PromptNode::RoleSpec(RoleSpec {
                role: block.content.clone(),
                traits: Vec::new(),
                span: block.span,
            })
        } else {
            PromptNode::Context(Context {
                content: block.content.clone(),
                context_type: ContextType::Background,
                relevance_score: 0.5,
                span: block.span,
            })
        }
    }

    fn classify_text(&self, text: &str, span_start: Position) -> PromptNode {
        let lower = text.to_lowercase();
        let end = Position::new(span_start.line, span_start.column + text.len());

        if INSTRUCTION_VERBS.iter().any(|v| lower.starts_with(v)) {
            let verb = self.parse_verb(&lower);
            return PromptNode::Instruction(Instruction {
                verb,
                object: text.to_string(),
                modifiers: Vec::new(),
                confidence: 0.8,
                span: SourceSpan::new(span_start, end),
            });
        }

        if lower.contains("must")
            || lower.contains("should")
            || lower.contains("cannot")
            || lower.contains("don't")
            || lower.contains("at least")
            || lower.contains("no more than")
            || lower.contains("required")
            || lower.contains("mandatory")
        {
            return PromptNode::Constraint(Constraint {
                constraint_type: if lower.contains("cannot") || lower.contains("don't") {
                    ConstraintType::Negative
                } else {
                    ConstraintType::Positive
                },
                value: text.to_string(),
                severity: if lower.contains("must") || lower.contains("required") {
                    ConstraintSeverity::Required
                } else if lower.contains("should") {
                    ConstraintSeverity::Preferred
                } else {
                    ConstraintSeverity::Suggested
                },
                span: SourceSpan::new(span_start, end),
            });
        }

        if lower.contains("format")
            || lower.contains("output as")
            || lower.contains("respond in")
            || lower.contains("json")
            || lower.contains("xml")
            || lower.contains("markdown")
            || lower.contains("csv")
        {
            return PromptNode::FormatSpec(FormatSpec {
                format_type: text.to_string(),
                detail: text.to_string(),
                span: SourceSpan::new(span_start, end),
            });
        }

        if lower.contains("you are")
            || lower.contains("act as")
            || lower.contains("role:")
            || lower.contains("persona")
            || lower.contains("you're a")
        {
            return PromptNode::RoleSpec(RoleSpec {
                role: text.to_string(),
                traits: Vec::new(),
                span: SourceSpan::new(span_start, end),
            });
        }

        if (lower.contains("example") || lower.contains("for instance") || lower.contains("e.g."))
            && (text.contains("->") || text.contains("=>")) {
                let parts: Vec<&str> = if text.contains("->") {
                    text.splitn(2, "->").collect()
                } else {
                    text.splitn(2, "=>").collect()
                };
                if parts.len() == 2 {
                    return PromptNode::Example(Example {
                        input: parts[0].trim().to_string(),
                        output: parts[1].trim().to_string(),
                        label: None,
                        span: SourceSpan::new(span_start, end),
                    });
                }
            }

        PromptNode::Context(Context {
            content: text.to_string(),
            context_type: ContextType::Background,
            relevance_score: 0.5,
            span: SourceSpan::new(span_start, end),
        })
    }

    fn parse_verb(&self, lower: &str) -> InstructionVerb {
        let first = lower.split_whitespace().next().unwrap_or("");
        match first {
            "write" | "generate" | "create" => InstructionVerb::Write,
            "analyze" => InstructionVerb::Analyze,
            "explain" => InstructionVerb::Explain,
            "summarize" => InstructionVerb::Summarize,
            "extract" => InstructionVerb::Extract,
            "classify" => InstructionVerb::Classify,
            "compare" => InstructionVerb::Compare,
            "translate" => InstructionVerb::Translate,
            "rewrite" => InstructionVerb::Rewrite,
            "expand" => InstructionVerb::Expand,
            "calculate" => InstructionVerb::Calculate,
            "implement" | "design" => InstructionVerb::Design,
            "optimize" => InstructionVerb::Optimize,
            "debug" => InstructionVerb::Debug,
            "convert" | "format" => InstructionVerb::Convert,
            "search" | "find" => InstructionVerb::Search,
            _ => InstructionVerb::Custom(first.to_string()),
        }
    }
}

pub fn parse(input: &str) -> Result<PromptRoot, String> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = parse("");
        assert!(result.is_ok());
        let root = result.unwrap();
        assert!(root.children.is_empty());
    }

    #[test]
    fn test_parse_heading_with_text() {
        let result = parse("# Instructions\nWrite a poem");
        assert!(result.is_ok());
        let root = result.unwrap();
        assert_eq!(root.children.len(), 1);
        if let PromptNode::Section(section) = &root.children[0] {
            assert_eq!(section.heading, "Instructions");
            assert_eq!(section.level, 1);
        } else {
            panic!("Expected section");
        }
    }

    #[test]
    fn test_parse_multiple_sections() {
        let input = "# Section 1\nContent 1\n\n# Section 2\nContent 2";
        let result = parse(input);
        assert!(result.is_ok());
        let root = result.unwrap();
        assert_eq!(root.children.len(), 2);
    }

    #[test]
    fn test_parse_instruction() {
        let result = parse("Write a function that sorts an array");
        assert!(result.is_ok());
        let root = result.unwrap();
        assert!(!root.children.is_empty());
    }

    #[test]
    fn test_parse_constraint() {
        let result = parse("The response must be in JSON format");
        assert!(result.is_ok());
    }
}
