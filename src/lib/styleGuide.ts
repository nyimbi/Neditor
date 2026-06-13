export interface StyleGuideRule {
	id: string
	category: string
	description: string
	patterns: string[]      // plain strings to match (case-insensitive)
	severity: "error" | "warn" | "info"
	suggestion: string
}

export interface StyleGuideFinding {
	ruleId: string
	category: string
	description: string
	severity: "error" | "warn" | "info"
	suggestion: string
	line: number
	column: number
	matchedText: string
	excerpt: string
}

export const BUILTIN_STYLE_RULES: StyleGuideRule[] = [
	// Hedge words / weak qualifiers
	{
		id: "hedge-very",
		category: "Word Choice",
		description: 'Weak intensifier "very"',
		patterns: ["very "],
		severity: "info",
		suggestion: 'Replace "very [adjective]" with a stronger adjective (e.g. "very important" → "critical").',
	},
	{
		id: "hedge-really",
		category: "Word Choice",
		description: 'Weak intensifier "really"',
		patterns: ["really "],
		severity: "info",
		suggestion: 'Remove "really" or replace with a precise word.',
	},
	{
		id: "hedge-quite",
		category: "Word Choice",
		description: 'Vague qualifier "quite"',
		patterns: ["quite "],
		severity: "info",
		suggestion: 'Remove "quite" — it weakens the claim.',
	},
	{
		id: "hedge-somewhat",
		category: "Word Choice",
		description: 'Vague qualifier "somewhat"',
		patterns: ["somewhat "],
		severity: "info",
		suggestion: 'Replace "somewhat" with a precise quantifier or qualifier.',
	},
	// Filler phrases
	{
		id: "filler-in-order-to",
		category: "Conciseness",
		description: 'Verbose "in order to"',
		patterns: ["in order to"],
		severity: "info",
		suggestion: 'Replace "in order to" with "to".',
	},
	{
		id: "filler-due-to-the-fact",
		category: "Conciseness",
		description: 'Verbose "due to the fact that"',
		patterns: ["due to the fact that"],
		severity: "warn",
		suggestion: 'Replace with "because".',
	},
	{
		id: "filler-at-this-point-in-time",
		category: "Conciseness",
		description: 'Verbose "at this point in time"',
		patterns: ["at this point in time"],
		severity: "warn",
		suggestion: 'Replace with "now".',
	},
	{
		id: "filler-it-is-important-to-note",
		category: "Conciseness",
		description: 'Filler phrase "it is important to note that"',
		patterns: ["it is important to note that", "it should be noted that"],
		severity: "info",
		suggestion: 'Delete the phrase — if the point is important, state it directly.',
	},
	{
		id: "filler-needless-to-say",
		category: "Conciseness",
		description: '"Needless to say" — contradicts itself',
		patterns: ["needless to say"],
		severity: "warn",
		suggestion: 'Delete "needless to say" and just say it.',
	},
	// Passive voice indicators
	{
		id: "passive-was-done",
		category: "Voice",
		description: "Likely passive voice construction",
		patterns: ["was done", "were done", "was made", "were made", "was found", "were found", "was shown", "was observed"],
		severity: "info",
		suggestion: "Consider rewriting in active voice: who did the action?",
	},
	// Clichés / jargon
	{
		id: "jargon-leverage",
		category: "Word Choice",
		description: 'Business jargon "leverage" (as a verb)',
		patterns: ["leverage the", "leveraging the", "leverage our", "leveraging our"],
		severity: "info",
		suggestion: 'Replace "leverage" with "use", "apply", or "exploit".',
	},
	{
		id: "jargon-synergy",
		category: "Word Choice",
		description: 'Overused buzzword "synergy"',
		patterns: ["synergy", "synergistic", "synergies"],
		severity: "info",
		suggestion: 'Replace with a concrete description of what is combined and why.',
	},
	{
		id: "jargon-paradigm-shift",
		category: "Word Choice",
		description: 'Overused phrase "paradigm shift"',
		patterns: ["paradigm shift", "paradigm-shift"],
		severity: "info",
		suggestion: "Describe the actual change rather than using this cliché.",
	},
	{
		id: "jargon-move-needle",
		category: "Word Choice",
		description: 'Vague phrase "move the needle"',
		patterns: ["move the needle", "moving the needle"],
		severity: "info",
		suggestion: "Quantify the improvement instead.",
	},
	// Double negatives
	{
		id: "double-negative",
		category: "Clarity",
		description: "Double negative construction",
		patterns: ["not un", "not in", "not im"],
		severity: "warn",
		suggestion: "Rewrite without the double negative for clarity.",
	},
]

export function runStyleGuide(
	text: string,
	rules: StyleGuideRule[],
): StyleGuideFinding[] {
	const findings: StyleGuideFinding[] = []
	const lines = text.split("\n")

	for (const rule of rules) {
		for (let li = 0; li < lines.length; li++) {
			const line = lines[li]
			// Skip markdown headings, code blocks, front matter
			const trimmed = line.trim()
			if (trimmed.startsWith("#") || trimmed.startsWith("```") || trimmed.startsWith("---")) continue

			const lower = line.toLowerCase()
			for (const pattern of rule.patterns) {
				let idx = lower.indexOf(pattern.toLowerCase())
				while (idx !== -1) {
					const matchedText = line.substring(idx, idx + pattern.length)
					const excerptStart = Math.max(0, idx - 20)
					const excerptEnd = Math.min(line.length, idx + pattern.length + 20)
					findings.push({
						ruleId: rule.id,
						category: rule.category,
						description: rule.description,
						severity: rule.severity,
						suggestion: rule.suggestion,
						line: li + 1,
						column: idx + 1,
						matchedText,
						excerpt: line.substring(excerptStart, excerptEnd).trim(),
					})
					idx = lower.indexOf(pattern.toLowerCase(), idx + 1)
				}
			}
		}
	}

	// Sort: errors first, then warn, then info; then by line
	const severityOrder = { error: 0, warn: 1, info: 2 }
	findings.sort((a, b) => {
		const sd = severityOrder[a.severity] - severityOrder[b.severity]
		if (sd !== 0) return sd
		return a.line - b.line
	})

	return findings
}

export function mergeStyleGuideRules(
	builtin: StyleGuideRule[],
	custom: StyleGuideRule[],
): StyleGuideRule[] {
	const customIds = new Set(custom.map((r) => r.id))
	return [...builtin.filter((r) => !customIds.has(r.id)), ...custom]
}
