use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ReadabilityStats {
    pub word_count: usize,
    pub sentence_count: usize,
    pub paragraph_count: usize,
    pub syllable_count: usize,
    pub avg_words_per_sentence: f64,
    pub flesch_reading_ease: f64,
    pub flesch_kincaid_grade: f64,
    pub gunning_fog: f64,
    pub reading_time_minutes: f64,
    pub long_sentence_count: usize,
    pub complex_word_count: usize,
}

pub fn count_syllables(word: &str) -> usize {
    let word = word.to_ascii_lowercase();
    let word = word.trim_matches(|c: char| !c.is_alphabetic());
    if word.is_empty() {
        return 0;
    }

    let mut count: usize = 0;
    let mut prev_vowel = false;
    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        let is_vowel = matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
        // Silent 'e' at end: subtract if count > 1 and last two aren't "le"
        if i == len - 1 && ch == 'e' && count > 1 {
            let is_le = len >= 2 && chars[len - 2] == 'l';
            if !is_le {
                count = count.saturating_sub(1);
            }
        }
    }

    count.max(1)
}

fn is_complex_word(word: &str) -> bool {
    count_syllables(word) >= 3
}

fn strip_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_code = false;
    let mut in_fence = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Skip fenced code blocks
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        // Skip YAML front matter (lines between --- delimiters)
        if trimmed == "---" {
            continue;
        }

        // Strip headings, blockquotes, list markers
        let stripped = trimmed
            .trim_start_matches('#')
            .trim_start_matches('>')
            .trim_start_matches("- ")
            .trim_start_matches("* ")
            .trim_start_matches("+ ")
            .trim();

        // Remove inline code backticks
        let mut cleaned = String::new();
        let mut chars = stripped.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '`' {
                in_code = !in_code;
            } else if !in_code {
                cleaned.push(ch);
            }
        }
        in_code = false;

        // Remove bold/italic markers
        let cleaned = cleaned
            .replace("**", "")
            .replace("__", "")
            .replace("*", "")
            .replace("_", " ");

        // Remove markdown links [text](url) → text
        let mut result = String::new();
        let mut chars = cleaned.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '[' {
                let mut link_text = String::new();
                for c in chars.by_ref() {
                    if c == ']' {
                        break;
                    }
                    link_text.push(c);
                }
                // Skip (url) part
                if chars.peek() == Some(&'(') {
                    chars.next();
                    for c in chars.by_ref() {
                        if c == ')' {
                            break;
                        }
                    }
                }
                result.push_str(&link_text);
            } else {
                result.push(ch);
            }
        }

        if !result.trim().is_empty() {
            out.push_str(result.trim());
            out.push(' ');
        }
    }
    out
}

pub fn analyze_text(text: &str) -> ReadabilityStats {
    let plain = strip_markdown(text);

    // Count paragraphs (blank-line-separated blocks in original)
    let paragraph_count = text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .count()
        .max(1);

    // Count words
    let words: Vec<&str> = plain
        .split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_alphabetic()))
        .collect();
    let word_count = words.len();

    if word_count == 0 {
        return ReadabilityStats {
            word_count: 0,
            sentence_count: 0,
            paragraph_count,
            syllable_count: 0,
            avg_words_per_sentence: 0.0,
            flesch_reading_ease: 100.0,
            flesch_kincaid_grade: 0.0,
            gunning_fog: 0.0,
            reading_time_minutes: 0.0,
            long_sentence_count: 0,
            complex_word_count: 0,
        };
    }

    // Count syllables and complex words
    let syllable_count: usize = words.iter().map(|w| count_syllables(w)).sum();
    let complex_word_count = words.iter().filter(|w| is_complex_word(w)).count();

    // Count sentences (split on . ! ? followed by space or end)
    let sentence_count = plain
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| s.split_whitespace().count() >= 2)
        .count()
        .max(1);

    // Count long sentences (> 30 words)
    let long_sentence_count = plain
        .split(|c| c == '.' || c == '!' || c == '?')
        .filter(|s| s.split_whitespace().count() > 30)
        .count();

    let words_f = word_count as f64;
    let sentences_f = sentence_count as f64;
    let syllables_f = syllable_count as f64;

    let avg_words_per_sentence = words_f / sentences_f;

    // Flesch Reading Ease: 206.835 - 1.015*(W/S) - 84.6*(Syl/W)
    let flesch_reading_ease =
        (206.835 - 1.015 * avg_words_per_sentence - 84.6 * (syllables_f / words_f))
            .clamp(0.0, 100.0);

    // Flesch-Kincaid Grade Level: 0.39*(W/S) + 11.8*(Syl/W) - 15.59
    let flesch_kincaid_grade =
        (0.39 * avg_words_per_sentence + 11.8 * (syllables_f / words_f) - 15.59).max(0.0);

    // Gunning Fog: 0.4 * ((W/S) + 100*(complex/W))
    let gunning_fog =
        0.4 * (avg_words_per_sentence + 100.0 * (complex_word_count as f64 / words_f));

    // Reading time: 250 wpm average
    let reading_time_minutes = words_f / 250.0;

    ReadabilityStats {
        word_count,
        sentence_count,
        paragraph_count,
        syllable_count,
        avg_words_per_sentence,
        flesch_reading_ease,
        flesch_kincaid_grade,
        gunning_fog,
        reading_time_minutes,
        long_sentence_count,
        complex_word_count,
    }
}

#[tauri::command]
pub(crate) fn analyze_readability(text: String) -> ReadabilityStats {
    analyze_text(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syllable_counts() {
        assert_eq!(count_syllables("the"), 1);
        assert_eq!(count_syllables("because"), 2);
        assert_eq!(count_syllables("simple"), 2);
        assert_eq!(count_syllables("I"), 1);
        assert_eq!(count_syllables("education"), 4);
    }

    #[test]
    fn test_simple_text() {
        // "The cat sat on the mat." — very simple, grade ~1
        let stats = analyze_text("The cat sat on the mat. The dog ran fast.");
        assert!(stats.word_count > 0);
        assert!(
            stats.flesch_reading_ease > 70.0,
            "simple text should score high ease"
        );
        assert!(
            stats.flesch_kincaid_grade < 5.0,
            "simple text should be low grade"
        );
    }

    #[test]
    fn test_empty_text() {
        let stats = analyze_text("");
        assert_eq!(stats.word_count, 0);
        assert_eq!(stats.reading_time_minutes, 0.0);
    }
}
