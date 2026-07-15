import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var viewModel: CompilerViewModel
    @AppStorage("theme") private var theme: String = "system"
    @AppStorage("defaultMode") private var defaultMode: String = "balanced"
    @AppStorage("telemetryEnabled") private var telemetryEnabled: Bool = false
    @AppStorage("crashReportingEnabled") private var crashReportingEnabled: Bool = false

    @State private var anthropicKey: String = ""
    @State private var openaiKey: String = ""
    @State private var googleKey: String = ""
    @State private var selectedTab: SettingsTab = .general

    enum SettingsTab: String, CaseIterable {
        case general = "General"
        case compiler = "Compiler"
        case apiKeys = "API Keys"
        case privacy = "Privacy"
        case advanced = "Advanced"
    }

    var body: some View {
        TabView(selection: $selectedTab) {
            generalTab
                .tabItem { Label("General", systemImage: "gear") }
                .tag(SettingsTab.general)

            compilerTab
                .tabItem { Label("Compiler", systemImage: "hammer") }
                .tag(SettingsTab.compiler)

            apiKeysTab
                .tabItem { Label("API Keys", systemImage: "key") }
                .tag(SettingsTab.apiKeys)

            privacyTab
                .tabItem { Label("Privacy", systemImage: "hand.raised") }
                .tag(SettingsTab.privacy)

            advancedTab
                .tabItem { Label("Advanced", systemImage: "wrench") }
                .tag(SettingsTab.advanced)
        }
        .padding(20)
        .frame(width: 520, height: 400)
        .onAppear {
            loadApiKeys()
        }
    }

    // MARK: - General Tab

    private var generalTab: some View {
        Form {
            Picker("Theme", selection: $theme) {
                Text("System").tag("system")
                Text("Light").tag("light")
                Text("Dark").tag("dark")
            }

            Picker("Default Mode", selection: $defaultMode) {
                Text("Economy").tag("economy")
                Text("Balanced").tag("balanced")
                Text("Deep Analysis").tag("deep-analysis")
            }

            Toggle("Open last window on startup", isOn: .constant(true))
        }
        .padding()
    }

    // MARK: - Compiler Tab

    private var compilerTab: some View {
        Form {
            Picker("Default target model", selection: $viewModel.selectedModel) {
                Text("Claude 3.5 Sonnet").tag("claude-3.5-sonnet")
                Text("GPT-4o").tag("gpt-4o")
                Text("Gemini 1.5 Pro").tag("gemini-1.5-pro")
            }

            Picker("Optimization aggressiveness", selection: .constant("standard")) {
                Text("Conservative").tag("conservative")
                Text("Standard").tag("standard")
                Text("Aggressive").tag("aggressive")
            }

            Toggle("Auto-apply safe optimizations", isOn: .constant(true))
        }
        .padding()
    }

    // MARK: - API Keys Tab

    private var apiKeysTab: some View {
        Form {
            SecureField("Anthropic API Key", text: $anthropicKey)
                .textFieldStyle(.roundedBorder)
                .onChange(of: anthropicKey) { _ in
                    saveApiKey("anthropic", key: anthropicKey)
                }

            SecureField("OpenAI API Key", text: $openaiKey)
                .textFieldStyle(.roundedBorder)
                .onChange(of: openaiKey) { _ in
                    saveApiKey("openai", key: openaiKey)
                }

            SecureField("Google API Key", text: $googleKey)
                .textFieldStyle(.roundedBorder)
                .onChange(of: googleKey) { _ in
                    saveApiKey("google", key: googleKey)
                }

            HStack {
                Spacer()
                Button("Validate Keys") {
                    validateKeys()
                }
            }
        }
        .padding()
    }

    // MARK: - Privacy Tab

    private var privacyTab: some View {
        Form {
            VStack(alignment: .leading, spacing: 8) {
                Toggle("Enable telemetry", isOn: $telemetryEnabled)
                Text("Anonymous usage data to improve PromptOS. No prompt content is ever collected.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            VStack(alignment: .leading, spacing: 8) {
                Toggle("Enable crash reporting", isOn: $crashReportingEnabled)
                Text("Crash reports help us fix bugs. No prompt content included in reports.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            VStack(alignment: .leading, spacing: 8) {
                Toggle("Enable analytics", isOn: .constant(false))
                Text("Aggregate compilation statistics. No prompt content collected.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
    }

    // MARK: - Advanced Tab

    private var advancedTab: some View {
        Form {
            Toggle("Enable local model", isOn: .constant(true))

            HStack {
                Text("Model file path")
                Spacer()
                Text("~/Library/Application Support/com.promptos.app/model.gguf")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Picker("Logging level", selection: .constant("info")) {
                Text("Error").tag("error")
                Text("Warning").tag("warning")
                Text("Info").tag("info")
                Text("Debug").tag("debug")
            }

            HStack {
                Text("Maximum history entries")
                Spacer()
                Text("100")
                    .foregroundColor(.secondary)
            }
        }
        .padding()
    }

    // MARK: - API Key Management

    private func loadApiKeys() {
        anthropicKey = viewModel.anthropicApiKey
        openaiKey = viewModel.openaiApiKey
        googleKey = viewModel.googleApiKey
    }

    private func saveApiKey(_ provider: String, key: String) {
        UserDefaults.standard.set(key, forKey: "apiKey_\(provider)")
        viewModel.initialize()
    }

    private func validateKeys() {
        if !anthropicKey.isEmpty {
            print("Validating Anthropic key: \(anthropicKey.prefix(8))...")
        }
        if !openaiKey.isEmpty {
            print("Validating OpenAI key: \(openaiKey.prefix(8))...")
        }
        if !googleKey.isEmpty {
            print("Validating Google key: \(googleKey.prefix(8))...")
        }
    }
}

struct SettingsView_Previews: PreviewProvider {
    static var previews: some View {
        SettingsView()
            .environmentObject(CompilerViewModel())
    }
}
