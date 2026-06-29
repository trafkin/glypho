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

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;
    use proptest::prelude::*;
    use rstest::rstest;

    // ==================== Unit Tests ====================

    #[rstest]
    #[case("[[Page]]", "[Page](Page)")]
    #[case("[[Page|My Page]]", "[My Page](Page)")]
    #[case("[[target|label]]", "[label](target)")]
    #[case("[[path/to/file]]", "[path/to/file](path/to/file)")]
    #[case("[[path/to/file|Custom Label]]", "[Custom Label](path/to/file)")]
    fn test_basic_wikilinks(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_no_wikilinks() {
        let input = "This is plain text without any wikilinks.";
        assert_eq!(wikilinks_to_markdown(input), input);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(wikilinks_to_markdown(""), "");
    }

    #[test]
    fn test_multiple_wikilinks() {
        let input = "Check out [[Page1]] and [[Page2|Second Page]] for more info.";
        let expected = "Check out [Page1](Page1) and [Second Page](Page2) for more info.";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_wikilinks_with_surrounding_text() {
        let input = "Before [[Link]] after";
        let expected = "Before [Link](Link) after";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_consecutive_wikilinks() {
        let input = "[[First]][[Second]][[Third]]";
        let expected = "[First](First)[Second](Second)[Third](Third)";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_wikilink_with_spaces() {
        let input = "[[My Page Name]]";
        let expected = "[My Page Name](My Page Name)";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_wikilink_with_special_chars() {
        let input = "[[file-name_123]]";
        let expected = "[file-name_123](file-name_123)";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_wikilink_in_markdown_context() {
        let input = "# Heading\n\nSee [[OtherPage]] for details.\n\n- Item with [[Link]]";
        let expected =
            "# Heading\n\nSee [OtherPage](OtherPage) for details.\n\n- Item with [Link](Link)";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_nested_brackets_not_wikilink() {
        // Single brackets should not be converted
        let input = "[not a wikilink]";
        assert_eq!(wikilinks_to_markdown(input), input);
    }

    #[test]
    fn test_existing_markdown_links_preserved() {
        let input = "[existing link](https://example.com)";
        assert_eq!(wikilinks_to_markdown(input), input);
    }

    #[test]
    fn test_mixed_markdown_and_wikilinks() {
        let input = "See [regular link](url) and [[WikiLink]]";
        let expected = "See [regular link](url) and [WikiLink](WikiLink)";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_wikilink_at_start_of_line() {
        let input = "[[StartLink]] is at the beginning";
        let expected = "[StartLink](StartLink) is at the beginning";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_wikilink_at_end_of_line() {
        let input = "This ends with [[EndLink]]";
        let expected = "This ends with [EndLink](EndLink)";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_multiline_content() {
        let input = "Line 1 with [[Link1]]\nLine 2 with [[Link2|Label2]]\nLine 3";
        let expected = "Line 1 with [Link1](Link1)\nLine 2 with [Label2](Link2)\nLine 3";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    // ==================== Snapshot Tests ====================

    #[test]
    fn test_complex_document_snapshot() {
        let input = r#"# My Document

This is a document with various [[WikiLinks]].

## Section 1

See [[Page1|First Page]] and [[Page2]] for more information.

### Subsection

- Item with [[Item1]]
- Another [[Item2|Custom Label]]

## Code Block (should not affect wikilinks outside)

```rust
// This is code
let x = 1;
```

Back to [[Normal]] text with [[Multiple|Links]] in [[One|Line]].
"#;
        assert_snapshot!(wikilinks_to_markdown(input));
    }

    #[test]
    fn test_edge_cases_snapshot() {
        let input = r#"[[A]]
[[B|C]]
[[path/to/file.md]]
[[path/to/file.md|File Link]]
[[Page With Spaces]]
[[Page-With-Dashes]]
[[Page_With_Underscores]]
[[123NumericStart]]
"#;
        assert_snapshot!(wikilinks_to_markdown(input));
    }

    // ==================== Property-Based Tests ====================

    proptest! {
        #[test]
        fn test_output_never_contains_double_brackets(target in "[a-zA-Z0-9_-]{1,20}") {
            let input = format!("[[{}]]", target);
            let result = wikilinks_to_markdown(&input);
            prop_assert!(!result.contains("[["));
            prop_assert!(!result.contains("]]"));
        }

        #[test]
        fn test_output_contains_markdown_link_format(target in "[a-zA-Z0-9_-]{1,20}") {
            let input = format!("[[{}]]", target);
            let result = wikilinks_to_markdown(&input);
            let expected = format!("[{}]({})", target, target);
            prop_assert!(result.contains(&expected));
        }

        #[test]
        fn test_labeled_link_preserves_label(
            target in "[a-zA-Z0-9_-]{1,20}",
            label in "[a-zA-Z0-9 ]{1,20}"
        ) {
            let input = format!("[[{}|{}]]", target, label);
            let result = wikilinks_to_markdown(&input);
            let expected = format!("[{}]({})", label, target);
            prop_assert!(result.contains(&expected));
        }

        #[test]
        fn test_plain_text_unchanged(text in "[a-zA-Z0-9 ,.!?]{0,100}") {
            // Text without [[ should remain unchanged
            let text_no_brackets = text.replace(['[', ']'], "");
            let result = wikilinks_to_markdown(&text_no_brackets);
            prop_assert_eq!(result, text_no_brackets);
        }

        #[test]
        fn test_output_length_reasonable(target in "[a-zA-Z0-9]{1,20}") {
            let input = format!("[[{}]]", target);
            let result = wikilinks_to_markdown(&input);
            // Output should be roughly: [target](target)
            // Which is: 1 + target.len() + 2 + target.len() + 1 = 4 + 2*target.len()
            let expected_len = 4 + 2 * target.len();
            prop_assert_eq!(result.len(), expected_len);
        }
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_brackets() {
        // Empty wikilinks should be handled gracefully
        let input = "[[]]";
        // The regex requires at least one character, so this should remain unchanged
        assert_eq!(wikilinks_to_markdown(input), "[[]]");
    }

    #[test]
    fn test_pipe_only() {
        let input = "[[|]]";
        // The regex matches "|" as the target (non-empty), so it becomes [|](|)
        assert_eq!(wikilinks_to_markdown(input), "[|](|)");
    }

    #[test]
    fn test_unicode_content() {
        let input = "[[日本語]]";
        let expected = "[日本語](日本語)";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_unicode_with_label() {
        let input = "[[日本語|Japanese]]";
        let expected = "[Japanese](日本語)";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_triple_brackets() {
        // The regex matches [[Page] as target (with the leading bracket), result is [[Page]([Page)]
        let input = "[[[Page]]]";
        let expected = "[[Page]([Page)]";
        assert_eq!(wikilinks_to_markdown(input), expected);
    }

    #[test]
    fn test_unclosed_wikilink() {
        let input = "[[Unclosed";
        assert_eq!(wikilinks_to_markdown(input), "[[Unclosed");
    }

    #[test]
    fn test_partially_closed_wikilink() {
        let input = "[[Partial]";
        assert_eq!(wikilinks_to_markdown(input), "[[Partial]");
    }
}
