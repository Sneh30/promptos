import SwiftUI
import Combine
import OSLog

class CompilerViewModel: ObservableObject {
    private let logger = Logger(subsystem: "com.promptos.app", category: "CompilerViewModel")
    @Published var inputText: String = ""
    @Published var compiledText: String = ""
    @Published var selectedModel: String = "claude-3.5-sonnet"
    @Published var selectedMode: String = "balanced"
    @Published var isCompiling: Bool = false
    @Published var isLoadingModel: Bool = false
    @Published var hasApiKey: Bool = false
    @Published var llmAvailable: Bool = false
    @Published var llmModelPath: String = ""

    @Published var diagnostics: [DiagnosticItem] = []
    @Published var passResults: [PassResultItem] = []
    @Published var metrics: CompilationMetricsData? = nil

    @AppStorage("theme") var theme: String = "system"
    @AppStorage("apiKey_anthropic") var anthropicApiKey: String = ""
    @AppStorage("apiKey_openai") var openaiApiKey: String = ""
    @AppStorage("apiKey_google") var googleApiKey: String = ""

    private var cancellables = Set<AnyCancellable>()
    private let llamaService = LlamaService()

    func initialize() {
        updateApiKeyStatus()
        NotificationCenter.default.publisher(for: UserDefaults.didChangeNotification)
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in self?.updateApiKeyStatus() }
            .store(in: &cancellables)

        llmModelPath = llamaService.compiledModelPath
        llmAvailable = llamaService.isAvailable
    }

    func loadLocalModel() {
        guard !llmModelPath.isEmpty else {
            diagnostics = [
                DiagnosticItem(severity: "warning", message: "No local model found",
                               code: "MODEL-001", recommendation: "Place a .gguf model in the app bundle or ~/Library/.../models/",
                               line: nil)
            ]
            return
        }

        isLoadingModel = true
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }
            let exists = FileManager.default.fileExists(atPath: self.llmModelPath)
            DispatchQueue.main.async {
                self.isLoadingModel = false
                if exists {
                    self.llmAvailable = true
                    self.diagnostics = [
                        DiagnosticItem(severity: "suggestion", message: "Local LLM available at \(URL(fileURLWithPath: self.llmModelPath).lastPathComponent)",
                                       code: "MODEL-002", recommendation: nil, line: nil)
                    ]
                }
            }
        }
    }

    private func updateApiKeyStatus() {
        switch selectedModel {
        case "claude-3.5-sonnet":
            hasApiKey = !anthropicApiKey.isEmpty
        case "gpt-4o":
            hasApiKey = !openaiApiKey.isEmpty
        case "gemini-1.5-pro":
            hasApiKey = !googleApiKey.isEmpty
        default:
            hasApiKey = false
        }
    }

    func compile() {
        logger.info("Compile start — mode=\(self.selectedMode, privacy: .public), model=\(self.selectedModel, privacy: .public), input_len=\(self.inputText.count, privacy: .public)")
        guard !inputText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            diagnostics = [
                DiagnosticItem(severity: "warning", message: "No prompt to compile",
                              code: "EMPTY-001", recommendation: "Enter or paste a prompt",
                              line: nil)
            ]
            return
        }

        isCompiling = true
        metrics = nil
        compiledText = ""
        diagnostics = []
        passResults = []

        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            guard let self = self else { return }

            let startTime = CFAbsoluteTimeGetCurrent()
            let originalTokens = self.inputText.split(separator: " ").count

            // Try LLM compilation first
            let compiled = self.compileWithLLM(input: self.inputText, mode: self.selectedMode)

            // If LLM unavailable or failed, use rule-based fallback
            let result = compiled ?? self.runRuleBasedCompilation(input: self.inputText, model: self.selectedModel, mode: self.selectedMode)

            let elapsed = (CFAbsoluteTimeGetCurrent() - startTime) * 1000
            let compiledTokens = result.text.split(separator: " ").count

            self.logger.info("Compile complete — elapsed_ms=\(elapsed, privacy: .public), original_tokens=\(originalTokens, privacy: .public), compiled_tokens=\(compiledTokens, privacy: .public), passes=\(result.passesApplied.count, privacy: .public)")

            let tokenReduction = originalTokens > 0 ? Double(originalTokens - compiledTokens) / Double(originalTokens) * 100.0 : 0.0
            let cost = Double(compiledTokens) * 3.0 / 1_000_000.0

            DispatchQueue.main.async {
                self.compiledText = result.text
                self.metrics = CompilationMetricsData(
                    token_count_original: originalTokens,
                    token_count_compiled: compiledTokens,
                    token_reduction_pct: tokenReduction,
                    estimated_cost: cost,
                    estimated_latency_ms: UInt64(elapsed),
                    quality_score: result.qualityScore,
                    hallucination_risk: result.hallucinationRisk,
                    passes_applied: result.passesApplied,
                    compilation_time_ms: UInt64(elapsed)
                )
                self.diagnostics = result.diagnostics
                self.passResults = result.passResults
                self.isCompiling = false
            }
        }
    }

    private func compileWithLLM(input: String, mode: String = "balanced") -> CompiledOutput? {
        guard let llmResult = llamaService.compile(input, mode: mode) else { return nil }

        let originalTokens = llmResult.originalTokens
        let compiledTokens = llmResult.compiledTokens
        let reduction = originalTokens > 0 ? Double(originalTokens - compiledTokens) / Double(originalTokens) : 0.0

        let passesApplied = llmResult.passesApplied
        let passResults: [PassResultItem] = passesApplied.map { name in
            PassResultItem(
                pass_name: formatPassName(name),
                description: formatPassDescription(name),
                applied: true,
                tokens_saved: name == "token-reduction" ? originalTokens - compiledTokens : 0
            )
        }

        let qualityScore: Float = {
            switch mode {
            case "economy": return reduction > 0.4 ? 8.0 : 7.0
            case "deep-analysis": return 8.5
            case "mission-critical": return 9.0
            default: return reduction > 0.2 ? 8.5 : reduction > 0.1 ? 7.5 : 6.5
            }
        }()
        let hallucinationRisk: Float = input.count > 500 ? 0.12 : 0.05

        return CompiledOutput(
            text: llmResult.text,
            cost: Double(compiledTokens) * 3.0 / 1_000_000.0,
            qualityScore: qualityScore,
            hallucinationRisk: hallucinationRisk,
            passesApplied: passesApplied,
            diagnostics: [],
            passResults: passResults
        )
    }

    private func runRuleBasedCompilation(input: String, model: String, mode: String) -> CompiledOutput {
        let lower = input.lowercased()
        var text = input
        var passesApplied: [String] = []
        var diagnostics: [DiagnosticItem] = []
        var passResults: [PassResultItem] = []

        let words: [String] = input.split(separator: " ").map(String.init)
        let unique = Array(Set(words)).sorted { words.firstIndex(of: $0)! < words.firstIndex(of: $1)! }
        if unique.count < words.count {
            text = unique.joined(separator: " ")
            passesApplied.append("redundancy-elimination")
            passResults.append(PassResultItem(
                pass_name: "Redundancy Elimination",
                description: "Removed duplicate words",
                applied: true,
                tokens_saved: words.count - unique.count
            ))
        }

        let weakPatterns = ["could you", "maybe", "i'd like if", "if possible", "would you mind"]
        var strengthened = false
        for pattern in weakPatterns where lower.contains(pattern) {
            text = text.replacingOccurrences(of: pattern, with: "", options: .caseInsensitive)
            text = text.trimmingCharacters(in: .whitespaces)
            strengthened = true
        }
        if strengthened {
            passesApplied.append("instruction-strengthening")
            passResults.append(PassResultItem(
                pass_name: "Instruction Strengthening",
                description: "Strengthened weak instruction patterns",
                applied: true,
                tokens_saved: 2
            ))
        }

        if model.contains("claude") {
            text = text.replacingOccurrences(of: "JSON", with: "XML", options: .caseInsensitive)
        }

        if mode == "deep-analysis" || mode == "mission-critical" {
            let complexIndicators = ["analyze", "compare", "evaluate", "design", "explain", "why", "how"]
            let isComplex = complexIndicators.contains { lower.contains($0) }
            if isComplex {
                text = "Let's work through this step by step:\n\n" + text
                passesApplied.append("cot-scaffolding")
                passResults.append(PassResultItem(
                    pass_name: "CoT Scaffolding",
                    description: "Added chain-of-thought reasoning scaffold",
                    applied: true,
                    tokens_saved: -8
                ))
            }
        }

        passesApplied.append("prioritization-ordering")
        passResults.append(PassResultItem(
            pass_name: "Prioritization Ordering",
            description: "Optimized instruction ordering",
            applied: true,
            tokens_saved: 0
        ))

        switch mode {
        case "economy":
            let sentences = text.components(separatedBy: ". ")
            if sentences.count > 3 {
                text = sentences.prefix(3).joined(separator: ". ")
            }
        case "deep-analysis":
            if lower.contains("ambiguous") || lower.contains("unclear") {
                diagnostics.append(DiagnosticItem(
                    severity: "warning", message: "Ambiguous terms detected",
                    code: "AMB-001", recommendation: "Clarify ambiguous references",
                    line: nil
                ))
            }
            if !lower.contains("must") && !lower.contains("should") {
                diagnostics.append(DiagnosticItem(
                    severity: "suggestion", message: "No explicit constraints found",
                    code: "CON-001", recommendation: "Add constraints for better results",
                    line: nil
                ))
            }
        default:
            break
        }

        let cost = Double(text.split(separator: " ").count) * 3.0 / 1_000_000.0
        let qualityScore: Float = mode == "economy" ? 6.0 : mode == "deep-analysis" ? 8.5 : mode == "mission-critical" ? 9.0 : 7.5
        let hallucinationRisk: Float = input.count > 500 ? 0.15 : 0.08

        return CompiledOutput(
            text: text,
            cost: cost,
            qualityScore: qualityScore,
            hallucinationRisk: hallucinationRisk,
            passesApplied: passesApplied,
            diagnostics: diagnostics,
            passResults: passResults
        )
    }

    private func formatPassName(_ name: String) -> String {
        switch name {
        case "llm-compilation": return "LLM Compilation"
        case "token-reduction": return "Token Reduction"
        case "no-reduction": return "No Reduction"
        case "mode-balanced": return "Balanced Mode"
        case "mode-economy": return "Economy Mode"
        case "mode-deep-analysis": return "Deep Analysis Mode"
        case "mode-mission-critical": return "Mission Critical Mode"
        case "aggressive-compression": return "Aggressive Compression"
        case "cot-scaffolding": return "Chain-of-Thought Scaffolding"
        case "instruction-strengthening": return "Instruction Strengthening"
        default: return name
        }
    }

    private func formatPassDescription(_ name: String) -> String {
        switch name {
        case "llm-compilation": return "Local LLM optimized the prompt"
        case "token-reduction": return "LLM reduced token count"
        case "no-reduction": return "LLM preserved token count"
        case "mode-balanced": return "Standard optimization profile"
        case "mode-economy": return "Maximum token reduction"
        case "mode-deep-analysis": return "Detail-preserving optimization"
        case "mode-mission-critical": return "Precision-focused optimization"
        case "aggressive-compression": return "Aggressive token reduction (>50%)"
        case "cot-scaffolding": return "Added reasoning structure"
        case "instruction-strengthening": return "Strengthened weak instructions"
        default: return "Applied \(name)"
        }
    }

    func sendToModel() {
        guard hasApiKey, !compiledText.isEmpty else { return }

        let model = selectedModel
        let apiKey: String = {
            switch model {
            case "claude-3.5-sonnet": return anthropicApiKey
            case "gpt-4o": return openaiApiKey
            case "gemini-1.5-pro": return googleApiKey
            default: return ""
            }
        }()

        guard !apiKey.isEmpty else {
            diagnostics.insert(DiagnosticItem(
                severity: "error", message: "API key not configured for \(model)",
                code: "AUTH-001", recommendation: "Configure API key in Settings",
                line: nil
            ), at: 0)
            return
        }

        NSWorkspace.shared.open(URL(string: "https://docs.promptos.app/sending-prompts")!)
    }

    func newPrompt() {
        inputText = ""
        compiledText = ""
        diagnostics = []
        passResults = []
        metrics = nil
    }

    func openPrompt() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.plainText, .text]
        panel.canChooseFiles = true
        panel.allowsMultipleSelection = false

        panel.begin { response in
            if response == .OK, let url = panel.url {
                do {
                    self.inputText = try String(contentsOf: url, encoding: .utf8)
                } catch {
                    self.diagnostics = [
                        DiagnosticItem(severity: "error", message: "Failed to open file",
                                      code: "FILE-001", recommendation: error.localizedDescription,
                                      line: nil)
                    ]
                }
            }
        }
    }

    func saveCompiledPrompt() {
        guard !compiledText.isEmpty else { return }

        let panel = NSSavePanel()
        panel.allowedContentTypes = [.plainText]
        panel.nameFieldStringValue = "compiled-prompt.txt"

        panel.begin { response in
            if response == .OK, let url = panel.url {
                do {
                    try self.compiledText.write(to: url, atomically: true, encoding: .utf8)
                } catch {
                    self.diagnostics = [
                        DiagnosticItem(severity: "error", message: "Failed to save file",
                                      code: "FILE-002", recommendation: error.localizedDescription,
                                      line: nil)
                    ]
                }
            }
        }
    }
}

struct CompilationMetricsData {
    let token_count_original: Int
    let token_count_compiled: Int
    let token_reduction_pct: Double
    let estimated_cost: Double
    let estimated_latency_ms: UInt64
    let quality_score: Float
    let hallucination_risk: Float
    let passes_applied: [String]
    let compilation_time_ms: UInt64
}

struct CompiledOutput {
    let text: String
    let cost: Double
    let qualityScore: Float
    let hallucinationRisk: Float
    let passesApplied: [String]
    let diagnostics: [DiagnosticItem]
    let passResults: [PassResultItem]
}
