use lazy_regex::*;

pub fn wikilinks_to_markdown(input: &str) -> String {
    // The regex captures:
    //   1. `target` – everything up to the first `|` or `]`
    //   2. optional `label` – everything after the `|` up to `]`
    //
    // Example matches:
    //   [[Page]]          → target = "Page",  label = None
    //   [[Page|My Page]]  → target = "Page",  label = Some("My Page")
    //
    // We use a non‑greedy `.+?` so that the first `]` ends the match.
    let re = regex!(r"\[\[(?P<target>.+?)(?:\|(?P<label>.+?))?\]\]");

    // We will collect the resulting string in a `String`.
    let mut result = String::with_capacity(input.len());

    // `last_end` keeps track of the byte index we have already processed.
    let mut last_end = 0;

    // Iterate over all non‑overlapping matches.
    for caps in re.captures_iter(input) {
        // `span` gives us the exact byte range of the whole match.
        let span = caps.get(0).unwrap().range();

        // Append the part of the input that came *before* this match.
        result.push_str(&input[last_end..span.start]);

        // Extract the captured groups.
        let target = caps.name("target").unwrap().as_str();
        let label = caps.name("label").map_or(target, |m| m.as_str());

        // Emit the Markdown link.
        result.push('[');
        result.push_str(label);
        result.push_str("](");
        result.push_str(target);
        result.push(')');

        // Update our cursor to the end of the match.
        last_end = span.end;
    }

    // Append the tail of the input that never matched the regex.
    result.push_str(&input[last_end..]);

    result
}
