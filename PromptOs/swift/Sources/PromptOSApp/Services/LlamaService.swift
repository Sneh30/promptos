import Foundation
import OSLog

struct LlamaCompilationResult {
    let text: String
    let originalTokens: Int
    let compiledTokens: Int
    let passesApplied: [String]
    let inferenceTimeMs: UInt64
}

class LlamaService {
    private let logger = Logger(subsystem: "com.promptos.app", category: "LlamaService")
    private let bridge: NativeBridge?
    private let modelsDir: String
    private let llamaCompletionPath: String
    private let bundledModelPath: String
    private let modelFilename = "qwen2.5-0.5b-instruct-q4_k_m.gguf"

    init() {
        logger.info("LlamaService initializing")
        self.bridge = NativeBridge()

        let home = FileManager.default.homeDirectoryForCurrentUser.path
        self.modelsDir = "\(home)/Library/Application Support/com.promptos.app/models"
        try? FileManager.default.createDirectory(atPath: modelsDir, withIntermediateDirectories: true)

        let bundle = Bundle.main
        let bundled = bundle.bundleURL
            .appendingPathComponent("Contents/Resources/\(modelFilename)").path
        if FileManager.default.fileExists(atPath: bundled) {
            self.bundledModelPath = bundled
        } else {
            self.bundledModelPath = "\(modelsDir)/\(modelFilename)"
        }

        let bundledBinary = bundle.bundleURL
            .appendingPathComponent("Contents/MacOS/llama-completion").path
        if FileManager.default.fileExists(atPath: bundledBinary) {
            self.llamaCompletionPath = bundledBinary
        } else {
            self.llamaCompletionPath = "/opt/homebrew/bin/llama-completion"
        }
    }

    var isAvailable: Bool {
        bridge?.isModelLoaded ?? false || FileManager.default.fileExists(atPath: bundledModelPath)
    }

    var isModelLoaded: Bool {
        bridge?.isModelLoaded ?? false
    }

    var compiledBinaryPath: String { llamaCompletionPath }
    var compiledModelPath: String { bundledModelPath }

    func compile(_ input: String, mode: String = "balanced") -> LlamaCompilationResult? {
        let originalTokens = input.split(separator: " ").count
        logger.info("Compile — mode=\(mode, privacy: .public), original_tokens=\(originalTokens, privacy: .public), input_len=\(input.count, privacy: .public)")

        if let b = bridge, b.isModelLoaded {
            logger.debug("Compile — using native bridge")
            let start = CFAbsoluteTimeGetCurrent()
            if let result = b.compile(input) {
                let elapsed = UInt64((CFAbsoluteTimeGetCurrent() - start) * 1000)
                let compiledTokens = result.split(separator: " ").count
                logger.info("Compile — bridge result, compiled_tokens=\(compiledTokens, privacy: .public), elapsed_ms=\(elapsed, privacy: .public)")
                let passes = buildPasses(mode, originalTokens, compiledTokens)
                return LlamaCompilationResult(
                    text: result,
                    originalTokens: originalTokens,
                    compiledTokens: compiledTokens,
                    passesApplied: passes,
                    inferenceTimeMs: elapsed
                )
            }
        }

        logger.warning("Compile — bridge unavailable, falling back to subprocess")
        return runCompletion(input, mode: mode, originalTokens: originalTokens)
    }

    private func runCompletion(_ input: String, mode: String, originalTokens: Int) -> LlamaCompilationResult? {
        guard FileManager.default.fileExists(atPath: bundledModelPath) else {
            logger.error("Completion — model not found at \(self.bundledModelPath, privacy: .public)")
            return nil
        }
        guard FileManager.default.fileExists(atPath: llamaCompletionPath) else {
            logger.error("Completion — llama-completion binary not found at \(self.llamaCompletionPath, privacy: .public)")
            return nil
        }
        logger.debug("Completion — starting subprocess with mode=\(mode, privacy: .public)")

        let systemPrompt = modePrompt(mode)
        let maxTokens = mode == "economy" ? 512 : 1024

        let fullPrompt = """
        \(systemPrompt)
        Original: \(input)
        Optimized:
        """

        let tmpDir = FileManager.default.temporaryDirectory
        let promptFile = tmpDir.appendingPathComponent("promptos_prompt_\(ProcessInfo().processIdentifier).txt")
        do {
            try fullPrompt.write(to: promptFile, atomically: true, encoding: .utf8)
        } catch {
            print("[LlamaService] Failed to write prompt file: \(error)")
            return nil
        }
        defer { try? FileManager.default.removeItem(at: promptFile) }

        let process = Process()
        process.executableURL = URL(fileURLWithPath: llamaCompletionPath)
        process.arguments = [
            "-m", bundledModelPath,
            "-f", promptFile.path,
            "-n", String(maxTokens),
            "--temp", "0.1",
            "--top-p", "0.9",
            "--repeat-penalty", "1.1",
            "--no-display-prompt",
            "-ngl", "99"
        ]

        let start = CFAbsoluteTimeGetCurrent()
        let outputPipe = Pipe()
        let errorPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = errorPipe

        do {
            try process.run()
            process.waitUntilExit()
        } catch {
            print("[LlamaService] Failed to run llama-completion: \(error)")
            return nil
        }

        let elapsed = UInt64((CFAbsoluteTimeGetCurrent() - start) * 1000)

        guard process.terminationStatus == 0 else {
            let errData = errorPipe.fileHandleForReading.readDataToEndOfFile()
            let errStr = String(data: errData, encoding: .utf8) ?? "unknown error"
            print("[LlamaService] llama-completion failed: \(errStr)")
            return nil
        }

        let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
        let rawOutput = String(data: outputData, encoding: .utf8) ?? ""

        let compiled = extractCompletionOutput(rawOutput)
        guard !compiled.isEmpty else { return nil }

        let compiledTokens = compiled.split(separator: " ").count
        let passes = buildPasses(mode, originalTokens, compiledTokens)

        return LlamaCompilationResult(
            text: compiled,
            originalTokens: originalTokens,
            compiledTokens: compiledTokens,
            passesApplied: passes,
            inferenceTimeMs: elapsed
        )
    }

    private func modePrompt(_ mode: String) -> String {
        switch mode {
        case "economy":
            return """
            Rewrite this prompt with maximum conciseness. Rules:
            - Cut all filler, pleasantries, and meta-commentary
            - Replace every weak phrase with a direct imperative
            - Use the fewest possible words while preserving ALL intent and constraints
            - Aim for 50%+ token reduction
            - Output ONLY the optimized prompt
            """
        case "deep-analysis":
            return """
            Rewrite this prompt to be clear and detailed. Rules:
            - Remove filler words but preserve ALL nuance and context
            - Add reasoning structure: break complex requests into numbered steps
            - Strengthen weak instructions while keeping detail
            - Preserve every specific requirement and constraint
            - Output ONLY the optimized prompt
            """
        case "mission-critical":
            return """
            Rewrite this prompt for maximum precision and reliability. Rules:
            - Remove ambiguity and vague language
            - Replace weak phrases with explicit, testable instructions
            - Add verification criteria and output format specifications
            - Preserve EVERY constraint, requirement, and edge case
            - Prioritize clarity over brevity
            - Output ONLY the optimized prompt
            """
        default: // balanced
            return """
            Rewrite this prompt to be concise and direct. Remove filler words, replace weak phrases with imperatives, preserve all requirements.
            """
        }
    }

    private func buildPasses(_ mode: String, _ original: Int, _ compiled: Int) -> [String] {
        var passes = ["llm-compilation"]
        if compiled < original {
            passes.append("token-reduction")
        }
        passes.append("mode-\(mode)")
        if mode == "economy" && compiled < original / 2 {
            passes.append("aggressive-compression")
        }
        if mode == "deep-analysis" {
            passes.append("cot-scaffolding")
        }
        if mode == "mission-critical" {
            passes.append("instruction-strengthening")
        }
        return passes
    }

    private func extractCompletionOutput(_ text: String) -> String {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        if let range = trimmed.range(of: "> EOF by user") {
            return String(trimmed[..<range.lowerBound]).trimmingCharacters(in: .whitespacesAndNewlines)
        }
        return trimmed
    }
}
