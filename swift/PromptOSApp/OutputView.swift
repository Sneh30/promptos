import SwiftUI

struct OutputView: View {
    @EnvironmentObject private var viewModel: CompilerViewModel

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Compiled Prompt")
                    .font(.headline)
                    .foregroundColor(.secondary)
                Spacer()

                Button(action: {
                    copyToClipboard()
                }) {
                    Label("Copy", systemImage: "doc.on.doc")
                }
                .buttonStyle(.borderless)
                .disabled(viewModel.compiledText.isEmpty)
                .help("Copy compiled prompt to clipboard")

                Button(action: {
                    viewModel.sendToModel()
                }) {
                    Label("Send", systemImage: "paperplane")
                }
                .buttonStyle(.borderless)
                .disabled(!viewModel.hasApiKey || viewModel.compiledText.isEmpty)
                .help("Send to cloud model")
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)

            if viewModel.isCompiling {
                Spacer()
                ProgressView("Compiling...")
                Spacer()
            } else if viewModel.compiledText.isEmpty {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "hammer.circle")
                        .font(.system(size: 36))
                        .foregroundColor(.secondary)
                    Text("Compiled output will appear here")
                        .foregroundColor(.secondary)
                    Text("Press Cmd+Enter to compile")
                        .font(.caption)
                        .foregroundColor(.tertiary)
                }
                Spacer()
            } else {
                ScrollView([.vertical, .horizontal]) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(viewModel.compiledText)
                            .font(.system(size: 13, design: .monospaced))
                            .textSelection(.enabled)
                            .padding(8)
                    }
                }
            }
        }
        .background(Color(NSColor.controlBackgroundColor))
    }

    private func copyToClipboard() {
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        pasteboard.setString(viewModel.compiledText, forType: .string)
    }
}

struct OutputView_Previews: PreviewProvider {
    static var previews: some View {
        OutputView()
            .environmentObject(CompilerViewModel())
    }
}
