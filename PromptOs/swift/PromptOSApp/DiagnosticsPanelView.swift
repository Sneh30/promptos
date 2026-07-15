import SwiftUI

struct DiagnosticsPanelView: View {
    @EnvironmentObject private var viewModel: CompilerViewModel
    @State private var selectedTab: Tab = .warnings

    enum Tab: String, CaseIterable {
        case warnings = "Warnings"
        case errors = "Errors"
        case suggestions = "Suggestions"
        case optimization = "Optimization"
        case risk = "Risk"
    }

    var body: some View {
        VStack(spacing: 0) {
            Picker("Tab", selection: $selectedTab) {
                ForEach(Tab.allCases, id: \.self) { tab in
                    Text(tab.rawValue).tag(tab)
                }
            }
            .pickerStyle(SegmentedPickerStyle())
            .padding(.horizontal, 12)
            .padding(.vertical, 6)

            Divider()

            ScrollView {
                if viewModel.diagnostics.isEmpty && viewModel.passResults.isEmpty {
                    VStack(spacing: 8) {
                        Image(systemName: "checkmark.circle")
                            .font(.system(size: 24))
                            .foregroundColor(.green)
                        Text("No diagnostics")
                            .foregroundColor(.secondary)
                        Text("Compile a prompt to see diagnostics")
                            .font(.caption)
                            .foregroundColor(.tertiary)
                    }
                    .padding(.top, 32)
                } else {
                    LazyVStack(alignment: .leading, spacing: 4) {
                        switch selectedTab {
                        case .warnings:
                            ForEach(warnings, id: \.id) { diag in
                                DiagnosticRow(diagnostic: diag)
                            }
                        case .errors:
                            ForEach(errors, id: \.id) { diag in
                                DiagnosticRow(diagnostic: diag)
                            }
                        case .suggestions:
                            ForEach(suggestions, id: \.id) { diag in
                                DiagnosticRow(diagnostic: diag)
                            }
                        case .optimization:
                            ForEach(optimizationResults, id: \.id) { result in
                                PassResultRow(result: result)
                            }
                        case .risk:
                            RiskAssessmentView()
                        }
                    }
                    .padding(8)
                }
            }
        }
        .background(Color(NSColor.controlBackgroundColor))
    }

    private var warnings: [DiagnosticItem] {
        viewModel.diagnostics.filter { $0.severity == "warning" }
    }

    private var errors: [DiagnosticItem] {
        viewModel.diagnostics.filter { $0.severity == "error" }
    }

    private var suggestions: [DiagnosticItem] {
        viewModel.diagnostics.filter { $0.severity == "suggestion" || $0.severity == "info" }
    }

    private var optimizationResults: [PassResultItem] {
        viewModel.passResults
    }
}

struct DiagnosticRow: View {
    let diagnostic: DiagnosticItem

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Image(systemName: iconName)
                .foregroundColor(iconColor)
                .frame(width: 16)

            VStack(alignment: .leading, spacing: 2) {
                Text(diagnostic.message)
                    .font(.system(size: 12))
                if let recommendation = diagnostic.recommendation {
                    Text(recommendation)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                if let code = diagnostic.code {
                    Text(code)
                        .font(.caption2)
                        .foregroundColor(.tertiary)
                }
            }
        }
        .padding(6)
        .background(iconColor.opacity(0.1))
        .cornerRadius(4)
    }

    private var iconName: String {
        switch diagnostic.severity {
        case "error": return "xmark.circle.fill"
        case "warning": return "exclamationmark.triangle.fill"
        case "suggestion": return "lightbulb.fill"
        default: return "info.circle.fill"
        }
    }

    private var iconColor: Color {
        switch diagnostic.severity {
        case "error": return .red
        case "warning": return .orange
        case "suggestion": return .blue
        default: return .gray
        }
    }
}

struct PassResultRow: View {
    let result: PassResultItem

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: result.applied ? "checkmark.circle.fill" : "circle")
                .foregroundColor(result.applied ? .green : .gray)

            VStack(alignment: .leading, spacing: 2) {
                Text(result.pass_name)
                    .font(.system(size: 12, weight: .medium))
                Text(result.description)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            if result.tokens_saved != 0 {
                Text("\(result.tokens_saved) tokens")
                    .font(.caption)
                    .foregroundColor(result.tokens_saved > 0 ? .green : .orange)
            }
        }
        .padding(6)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(4)
    }
}

struct RiskAssessmentView: View {
    @EnvironmentObject private var viewModel: CompilerViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            if let metrics = viewModel.metrics {
                RiskRow(label: "Hallucination Risk", value: metrics.hallucination_risk, format: .percentage)
                RiskRow(label: "Quality Score", value: metrics.quality_score, format: .score)
                RiskRow(label: "Estimated Cost", value: metrics.estimated_cost, format: .currency)
                RiskRow(label: "Estimated Latency", value: Double(metrics.estimated_latency_ms), format: .milliseconds)
            } else {
                Text("Compile a prompt to see risk assessment")
                    .foregroundColor(.secondary)
            }
        }
        .padding()
    }
}

struct RiskRow: View {
    let label: String
    let value: Double
    let format: RiskFormat

    enum RiskFormat {
        case percentage, score, currency, milliseconds
    }

    var body: some View {
        HStack {
            Text(label)
                .font(.system(size: 12))
            Spacer()
            Text(formattedValue)
                .font(.system(size: 12, weight: .bold, design: .monospaced))
                .foregroundColor(valueColor)
        }
        .padding(4)
    }

    private var formattedValue: String {
        switch format {
        case .percentage:
            return "\(String(format: "%.1f", value * 100))%"
        case .score:
            return "\(String(format: "%.1f", value))/10"
        case .currency:
            return "$\(String(format: "%.4f", value))"
        case .milliseconds:
            return "\(String(format: "%.0f", value))ms"
        }
    }

    private var valueColor: Color {
        switch format {
        case .percentage:
            return value > 0.3 ? .red : value > 0.1 ? .orange : .green
        case .score:
            return value >= 7.0 ? .green : value >= 4.0 ? .orange : .red
        case .currency:
            return .primary
        case .milliseconds:
            return value > 5000 ? .orange : .primary
        }
    }
}

struct DiagnosticItem: Identifiable {
    let id = UUID()
    let severity: String
    let message: String
    let code: String?
    let recommendation: String?
    let line: Int?
}

struct PassResultItem: Identifiable {
    let id = UUID()
    let pass_name: String
    let description: String
    let applied: Bool
    let tokens_saved: Int
}
