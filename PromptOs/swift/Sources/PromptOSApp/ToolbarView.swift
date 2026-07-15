import SwiftUI

struct ToolbarView: View {
    @EnvironmentObject private var viewModel: CompilerViewModel

    var body: some View {
        HStack(spacing: 12) {
            Picker("Model", selection: $viewModel.selectedModel) {
                Text("Claude 3.5 Sonnet").tag("claude-3.5-sonnet")
                Text("GPT-4o").tag("gpt-4o")
                Text("Gemini 1.5 Pro").tag("gemini-1.5-pro")
            }
            .pickerStyle(MenuPickerStyle())
            .frame(width: 180)
            .help("Select the target model for compilation")

            Picker("Mode", selection: $viewModel.selectedMode) {
                Text("Economy").tag("economy")
                Text("Balanced").tag("balanced")
                Text("Deep Analysis").tag("deep-analysis")
                Text("Mission Critical").tag("mission-critical")
            }
            .pickerStyle(SegmentedPickerStyle())
            .help("Select compilation mode")

            Spacer()

            Button(action: {
                viewModel.compile()
            }) {
                Label("Compile", systemImage: "hammer")
            }
            .keyboardShortcut(.return, modifiers: .command)
            .help("Compile the current prompt (Cmd+Enter)")

            Button(action: {
                viewModel.sendToModel()
            }) {
                Label("Send", systemImage: "paperplane")
            }
            .keyboardShortcut(.return, modifiers: [.command, .shift])
            .disabled(!viewModel.hasApiKey || viewModel.compiledText.isEmpty)
            .help("Send compiled prompt to model (Cmd+Shift+Enter)")
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color(NSColor.windowBackgroundColor))
    }
}

struct ToolbarView_Previews: PreviewProvider {
    static var previews: some View {
        ToolbarView()
            .environmentObject(CompilerViewModel())
    }
}
