import SwiftUI
import AppKit

struct InputEditorView: View {
    @Binding var text: String
    @EnvironmentObject private var viewModel: CompilerViewModel

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("Source Prompt")
                    .font(.headline)
                    .foregroundColor(.secondary)
                Spacer()
                Text("\(tokenCount) tokens")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)

            TextEditor(text: $text)
                .font(.system(size: 13, design: .monospaced))
                .lineSpacing(1.2)
                .disableAutocorrection(true)
                .accessibilityLabel("Prompt input editor")
                .accessibilityHint("Enter or paste your prompt here")

            HStack {
                Button("Clear") {
                    text = ""
                }
                .buttonStyle(.borderless)
                .foregroundColor(.secondary)
                .help("Clear the input editor")

                Spacer()

                Text("\(text.count) characters")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 4)
        }
        .background(Color(NSColor.controlBackgroundColor))
    }

    private var tokenCount: Int {
        text.split(separator: " ").count
    }
}

struct InputEditorView_Previews: PreviewProvider {
    static var previews: some View {
        InputEditorView(text: .constant("Write a poem about Swift programming."))
            .environmentObject(CompilerViewModel())
    }
}
