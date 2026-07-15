import SwiftUI
import OSLog

@main
struct PromptOSApp: App {
    private static let logger = Logger(subsystem: "com.promptos.app", category: "App")
    @StateObject private var viewModel = CompilerViewModel()
    @AppStorage("theme") private var theme: String = "system"

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(viewModel)
                .preferredColorScheme(colorScheme)
                .onAppear {
                    Self.logger.info("App launched")
                    viewModel.initialize()
                }
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("New Prompt") {
                    viewModel.newPrompt()
                }
                .keyboardShortcut("n", modifiers: .command)

                Button("Open Prompt...") {
                    viewModel.openPrompt()
                }
                .keyboardShortcut("o", modifiers: .command)

                Button("Save Compiled Prompt...") {
                    viewModel.saveCompiledPrompt()
                }
                .keyboardShortcut("s", modifiers: .command)
            }

            CommandMenu("Compile") {
                Button("Compile") {
                    viewModel.compile()
                }
                .keyboardShortcut(.return, modifiers: .command)

                Button("Send to Model") {
                    viewModel.sendToModel()
                }
                .keyboardShortcut(.return, modifiers: [.command, .shift])
                .disabled(!viewModel.hasApiKey)
            }
        }

        Settings {
            SettingsView()
                .environmentObject(viewModel)
        }
    }

    private var colorScheme: ColorScheme? {
        switch theme {
        case "light": return .light
        case "dark": return .dark
        default: return nil
        }
    }
}
