import SwiftUI

struct ContentView: View {
    @EnvironmentObject private var viewModel: CompilerViewModel
    @State private var leftPanelWidth: CGFloat = 350
    @State private var bottomPanelHeight: CGFloat = 200

    var body: some View {
        VStack(spacing: 0) {
            ToolbarView()

            HSplitView {
                InputEditorView(text: $viewModel.inputText)
                    .frame(minWidth: 250, idealWidth: leftPanelWidth)
                    .layoutPriority(1)

                OutputView()
                    .frame(minWidth: 250)
                    .layoutPriority(1)
            }
            .frame(maxHeight: .infinity)

            VSplitView {
                DiagnosticsPanelView()
                    .frame(minHeight: 100, idealHeight: bottomPanelHeight, maxHeight: .infinity)
            }
            .frame(height: bottomPanelHeight)

            StatusBarView()
        }
        .frame(minWidth: 800, minHeight: 600)
    }
}

struct StatusBarView: View {
    @EnvironmentObject private var viewModel: CompilerViewModel

    var body: some View {
        HStack {
            if viewModel.isCompiling {
                ProgressView()
                    .scaleEffect(0.7)
                    .frame(width: 16, height: 16)
                Text("Compiling...")
                    .font(.caption)
            } else if let metrics = viewModel.metrics {
                Text("Tokens: \(metrics.token_count_original) → \(metrics.token_count_compiled)")
                    .font(.caption)
                Divider()
                Text("Cost: $\(String(format: "%.4f", metrics.estimated_cost))")
                    .font(.caption)
                Divider()
                Text("Quality: \(String(format: "%.1f", metrics.quality_score))/10")
                    .font(.caption)
            } else {
                Text("Ready to compile")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            if viewModel.isLoadingModel {
                ProgressView()
                    .scaleEffect(0.5)
                Text("Loading model...")
                    .font(.caption)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 4)
        .background(Color(NSColor.controlBackgroundColor))
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
            .environmentObject(CompilerViewModel())
    }
}
