use crate::ast::*;
use crate::semantic::ModelProfileData;

pub trait ModelCodeGenerator: Send + Sync {
    fn generate(&self, ast: &PromptRoot, profile: Option<&ModelProfileData>) -> CompiledPrompt;
    fn model_id(&self) -> &str;
}

pub struct DefaultCodeGenerator {
    model_id_str: String,
}

impl DefaultCodeGenerator {
    pub fn new(model_id: &str) -> Self {
        Self {
            model_id_str: model_id.to_string(),
        }
    }
}

impl ModelCodeGenerator for DefaultCodeGenerator {
    fn model_id(&self) -> &str {
        &self.model_id_str
    }

    fn generate(&self, ast: &PromptRoot, _profile: Option<&ModelProfileData>) -> CompiledPrompt {
        let text = self.emit_node(&PromptNode::Root(ast.clone()));
        CompiledPrompt {
            text,
            model_id: self.model_id_str.clone(),
            mode: CompilationMode::Balanced,
            max_output_tokens: None,
            temperature: None,
            structured_output_schema: None,
        }
    }
}

impl DefaultCodeGenerator {
    fn emit_node(&self, node: &PromptNode) -> String {
        match node {
            PromptNode::Root(root) => root
                .children
                .iter()
                .map(|c| self.emit_node(c))
                .collect::<Vec<_>>()
                .join("\n\n"),
            PromptNode::Section(section) => {
                let heading = "#".repeat(section.level as usize);
                let header = format!("{} {}", heading, section.heading);
                let body = section
                    .children
                    .iter()
                    .map(|c| self.emit_node(c))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{}\n{}", header, body)
            }
            PromptNode::Instruction(instr) => instr.object.clone(),
            PromptNode::Constraint(constraint) => constraint.value.clone(),
            PromptNode::Context(ctx) => ctx.content.clone(),
            PromptNode::FormatSpec(spec) => spec.format_type.clone(),
            PromptNode::RoleSpec(role) => role.role.clone(),
            PromptNode::Example(ex) => {
                format!("Input: {}\nOutput: {}", ex.input, ex.output)
            }
            PromptNode::MetaInstruction(meta) => meta.content.clone(),
            PromptNode::Block(block) => block.content.clone(),
        }
    }
}

pub struct AnthropicCodeGenerator {
    inner: DefaultCodeGenerator,
}

impl Default for AnthropicCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AnthropicCodeGenerator {
    pub fn new() -> Self {
        Self {
            inner: DefaultCodeGenerator::new("claude-3.5-sonnet"),
        }
    }
}

impl ModelCodeGenerator for AnthropicCodeGenerator {
    fn model_id(&self) -> &str {
        "claude-3.5-sonnet"
    }

    fn generate(&self, ast: &PromptRoot, profile: Option<&ModelProfileData>) -> CompiledPrompt {
        let inner_text = self.inner.emit_node(&PromptNode::Root(ast.clone()));
        let text = format!("<instructions>\n{}\n</instructions>\n\n{}", inner_text, "");
        CompiledPrompt {
            text,
            model_id: "claude-3.5-sonnet".to_string(),
            mode: CompilationMode::Balanced,
            max_output_tokens: profile.map(|p| p.max_output_tokens),
            temperature: None,
            structured_output_schema: None,
        }
    }
}

pub struct OpenAICodeGenerator {
    inner: DefaultCodeGenerator,
}

impl Default for OpenAICodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenAICodeGenerator {
    pub fn new() -> Self {
        Self {
            inner: DefaultCodeGenerator::new("gpt-4o"),
        }
    }
}

impl ModelCodeGenerator for OpenAICodeGenerator {
    fn model_id(&self) -> &str {
        "gpt-4o"
    }

    fn generate(&self, ast: &PromptRoot, profile: Option<&ModelProfileData>) -> CompiledPrompt {
        let inner_text = self.inner.emit_node(&PromptNode::Root(ast.clone()));
        let text = format!("# Instructions\n{}\n\n# Constraints\n{}", inner_text, "");
        CompiledPrompt {
            text,
            model_id: "gpt-4o".to_string(),
            mode: CompilationMode::Balanced,
            max_output_tokens: profile.map(|p| p.max_output_tokens),
            temperature: None,
            structured_output_schema: None,
        }
    }
}

pub struct GoogleCodeGenerator {
    inner: DefaultCodeGenerator,
}

impl Default for GoogleCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleCodeGenerator {
    pub fn new() -> Self {
        Self {
            inner: DefaultCodeGenerator::new("gemini-1.5-pro"),
        }
    }
}

impl ModelCodeGenerator for GoogleCodeGenerator {
    fn model_id(&self) -> &str {
        "gemini-1.5-pro"
    }

    fn generate(&self, ast: &PromptRoot, profile: Option<&ModelProfileData>) -> CompiledPrompt {
        let inner_text = self.inner.emit_node(&PromptNode::Root(ast.clone()));
        let text = format!("{}\n\n{}", inner_text, "");
        CompiledPrompt {
            text,
            model_id: "gemini-1.5-pro".to_string(),
            mode: CompilationMode::Balanced,
            max_output_tokens: profile.map(|p| p.max_output_tokens),
            temperature: None,
            structured_output_schema: None,
        }
    }
}

pub fn create_generator(model_id: &str) -> Box<dyn ModelCodeGenerator> {
    match model_id {
        id if id.contains("claude") => Box::new(AnthropicCodeGenerator::new()),
        id if id.contains("gpt")
            || id.contains("openai")
            || id.contains("o1")
            || id.contains("o3") =>
        {
            Box::new(OpenAICodeGenerator::new())
        }
        id if id.contains("gemini") => Box::new(GoogleCodeGenerator::new()),
        _ => Box::new(DefaultCodeGenerator::new(model_id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generator() {
        let ast = PromptRoot::new(vec![PromptNode::Instruction(Instruction {
            verb: InstructionVerb::Write,
            object: "Write a poem".to_string(),
            modifiers: Vec::new(),
            confidence: 0.9,
            span: SourceSpan::new(Position::new(1, 1), Position::new(1, 12)),
        })]);
        let gen = DefaultCodeGenerator::new("test-model");
        let compiled = gen.generate(&ast, None);
        assert!(!compiled.text.is_empty());
        assert!(compiled.text.contains("Write a poem"));
    }

    #[test]
    fn test_anthropic_generator() {
        let ast = PromptRoot::new(vec![PromptNode::Instruction(Instruction {
            verb: InstructionVerb::Write,
            object: "Write a poem".to_string(),
            modifiers: Vec::new(),
            confidence: 0.9,
            span: SourceSpan::new(Position::new(1, 1), Position::new(1, 12)),
        })]);
        let gen = AnthropicCodeGenerator::new();
        let compiled = gen.generate(&ast, None);
        assert!(compiled.text.contains("<instructions>"));
        assert!(compiled.text.contains("</instructions>"));
    }
}
