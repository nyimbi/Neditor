use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct HumanizeRequest {
    pub text: String,
    pub mode: String,  // "light", "standard", "heavy"
}

#[derive(Debug, Serialize)]
pub struct HumanizeResponse {
    pub humanized: String,
    pub changes: Vec<String>,
}

// Returns a system prompt and user prompt for the AI provider to use.
// The actual LLM call happens on the frontend using existing AI provider infrastructure.
#[tauri::command]
pub(crate) fn get_humanize_prompt(request: HumanizeRequest) -> Result<HumanizeResponse, String> {
    // Apply rule-based humanization heuristics in Rust (no LLM needed for basic pass)
    let text = &request.text;
    let mut result = text.clone();
    let mut changes = Vec::new();

    // Remove common AI filler phrases
    let fillers = [
        ("It's worth noting that ", ""),
        ("It's important to note that ", ""),
        ("It is worth noting that ", ""),
        ("Notably, ", ""),
        ("Certainly, ", ""),
        ("Absolutely, ", ""),
        ("Of course, ", ""),
        ("Delve into ", "Explore "),
        ("delve into ", "explore "),
        ("In conclusion, ", ""),
        ("To summarize, ", ""),
        ("In summary, ", ""),      // remove directly; avoids producing "To summarize, " after it was already erased
        ("leveraging ", "using "),
        ("Leveraging ", "Using "),
        ("utilize the ", "use the "),  // specific pattern before the general "utilize " match
        ("Utilize the ", "Use the "),
        ("utilize ", "use "),
        ("Utilize ", "Use "),
        ("multifaceted ", "complex "),
        ("robust ", "strong "),
        ("comprehensive ", "complete "),
        ("furthermore, ", "also, "),
        ("Furthermore, ", "Also, "),
        ("Moreover, ", "Also, "),
        ("In addition to this, ", "Also, "),
        ("It should be noted that ", ""),
        ("As previously mentioned, ", "As noted, "),
    ];

    for (pattern, replacement) in &fillers {
        if result.contains(pattern) {
            result = result.replace(pattern, replacement);
            if !replacement.is_empty() {
                changes.push(format!("Replaced \"{}\" with \"{}\"", pattern.trim(), replacement.trim()));
            } else {
                changes.push(format!("Removed \"{}\"", pattern.trim()));
            }
        }
    }

    if request.mode == "heavy" {
        // Additional heavy cleaning
        result = result.replace("In order to ", "To ");
        result = result.replace("Due to the fact that ", "Because ");
        result = result.replace("At this point in time ", "Now ");
        result = result.replace("In the event that ", "If ");
        result = result.replace("For the purpose of ", "To ");
        result = result.replace("With regard to ", "About ");
        result = result.replace("In terms of ", "For ");
    }

    Ok(HumanizeResponse { humanized: result, changes })
}
