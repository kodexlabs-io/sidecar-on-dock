use sidecar_on_dock::sidecar::normalise_quotes;

#[test]
fn normalise_right_single_quote() {
    assert_eq!(normalise_quotes("Dominic\u{2019}s iPad"), "Dominic's iPad");
}

#[test]
fn normalise_left_single_quote() {
    assert_eq!(normalise_quotes("\u{2018}hello\u{2019}"), "'hello'");
}

#[test]
fn normalise_modifier_letter_apostrophe() {
    assert_eq!(normalise_quotes("test\u{02BC}s"), "test's");
}

#[test]
fn normalise_plain_apostrophe_unchanged() {
    assert_eq!(normalise_quotes("Dominic's iPad"), "Dominic's iPad");
}

#[test]
fn normalise_no_quotes() {
    assert_eq!(normalise_quotes("My iPad"), "My iPad");
}

#[test]
fn normalise_empty() {
    assert_eq!(normalise_quotes(""), "");
}
