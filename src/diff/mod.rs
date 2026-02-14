use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};

pub fn extract_content(html: &str, css_selector: Option<&str>) -> Result<String> {
    let document = Html::parse_document(html);

    let selector = match css_selector {
        Some(sel) => {
            Selector::parse(sel).map_err(|e| anyhow::anyhow!("Invalid selector: {:?}", e))?
        }
        None => {
            Selector::parse("body").map_err(|e| anyhow::anyhow!("Invalid selector: {:?}", e))?
        }
    };

    let mut content_parts = Vec::new();

    for element in document.select(&selector) {
        let text = extract_text_from_element(element);
        if !text.is_empty() {
            content_parts.push(text);
        }
    }

    Ok(content_parts.join("\n"))
}

fn extract_text_from_element(element: scraper::ElementRef<'_>) -> String {
    let mut text_parts = Vec::new();

    for node in element.descendants() {
        if let Some(text) = node.value().as_text() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                text_parts.push(trimmed.to_string());
            }
        }
    }

    text_parts.join(" ")
}

pub fn filter_noise(content: &str) -> String {
    let lines: Vec<String> = content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    let timestamp_patterns = [
        r"\d{1,2}:\d{2}(?::\d{2})?(?:\s*[AP]M)?",
        r"\d{4}-\d{2}-\d{2}",
        r"\d{2}/\d{2}/\d{4}",
        r"\d{1,2}/\d{1,2}/\d{2,4}",
        r"(?:Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday)",
        r"(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s+\d{1,2}",
        r"Last updated?:?\s*.*",
        r"Updated?:?\s*\d+\s*(?:seconds?|minutes?|hours?|days?)\s*ago",
        r"Published?:?\s*.*",
    ];

    let combined_pattern = timestamp_patterns.join("|");
    let re = Regex::new(&combined_pattern).expect("Invalid regex pattern - this is a bug");

    let ad_patterns = [
        r"advertisement",
        r"sponsored",
        r"ad\s*choice",
        r"cookie\s*policy",
        r"accept\s*cookies",
        r"subscribe\s*now",
        r"sign\s*up",
        r"newsletter",
        r"follow\s*us",
        r"share\s*this",
        r"advertisement\s*close",
        r"×\s*close",
        r"skip\s*to\s*content",
        r"skip\s*ad",
    ];

    let ad_pattern = ad_patterns.join("|");
    let ad_re =
        Regex::new(&format!("(?i){}", ad_pattern)).expect("Invalid regex pattern - this is a bug");

    let filtered: Vec<String> = lines
        .into_iter()
        .filter(|line| {
            if ad_re.is_match(line) {
                return false;
            }

            if re.is_match(line) && line.len() < 100 {
                return false;
            }

            if line.len() < 3 {
                return false;
            }

            true
        })
        .collect();

    filtered.join("\n")
}

pub fn compute_diff(old_content: &str, new_content: &str) -> String {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();

    for line in new_lines.iter() {
        if !old_lines.contains(line) {
            added.push(line.to_string());
        }
    }

    for line in old_lines.iter() {
        if !new_lines.contains(line) {
            removed.push(line.to_string());
        }
    }

    let mut result = String::new();

    if !removed.is_empty() {
        result.push_str("Removed:\n");
        for line in removed.iter().take(10) {
            result.push_str(&format!("- {}\n", line));
        }
    }

    if !added.is_empty() {
        result.push_str("Added:\n");
        for line in added.iter().take(10) {
            result.push_str(&format!("+ {}\n", line));
        }
    }

    if result.is_empty() {
        result = "Content structure changed (check the site for details)".to_string();
    }

    result
}
