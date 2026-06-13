export interface WorkflowPreset {
	id: string
	label: string
	description: string
	icon: string
	exportTarget: string
	sidebar: string
	collapsedRows: string[]
	citationStyle?: string
	editorMode?: string
}

export const WORKFLOW_PRESETS: WorkflowPreset[] = [
	{
		id: "academic",
		label: "Academic / Research",
		description: "Journal articles, theses, dissertations, lab notebooks",
		icon: "⚗",
		exportTarget: "pdf",
		sidebar: "references",
		collapsedRows: ["row-business", "row-presentation"],
		citationStyle: "apa",
	},
	{
		id: "business",
		label: "Business Writing",
		description: "Proposals, reports, board packs, RFP responses",
		icon: "📋",
		exportTarget: "docx",
		sidebar: "templates",
		collapsedRows: ["row-equations", "row-citations"],
	},
	{
		id: "lab-notebook",
		label: "Lab Notebook",
		description: "Experiments, protocols, observations, results",
		icon: "🔬",
		exportTarget: "pdf",
		sidebar: "templates",
		collapsedRows: ["row-business", "row-presentation"],
		citationStyle: "apa",
	},
	{
		id: "presentation",
		label: "Presentation",
		description: "Slides, decks, speaker notes",
		icon: "📊",
		exportTarget: "pptx",
		sidebar: "exports",
		collapsedRows: ["row-equations", "row-citations", "row-business"],
		editorMode: "presentation",
	},
	{
		id: "daily-journal",
		label: "Daily Journal",
		description: "Notes, diary entries, daily review",
		icon: "📓",
		exportTarget: "html",
		sidebar: "daily-notes",
		collapsedRows: ["row-equations", "row-citations", "row-business", "row-presentation"],
	},
	{
		id: "technical",
		label: "Technical Documentation",
		description: "API docs, READMEs, specifications, user guides",
		icon: "⚙",
		exportTarget: "html",
		sidebar: "outline",
		collapsedRows: ["row-citations", "row-business", "row-presentation"],
	},
]

export function presetById(id: string): WorkflowPreset | undefined {
	return WORKFLOW_PRESETS.find((p) => p.id === id)
}
