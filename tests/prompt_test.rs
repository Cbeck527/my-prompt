use my_prompt::ModuleContext;
use my_prompt::prompt::{PROMPT_FORMAT, TRANSIENT_FORMAT, render_prompt};

#[test]
fn test_render_prompt_full() {
    let context = ModuleContext {
        exit_code: Some(0),
        no_color: true,
        ..ModuleContext::default()
    };

    let result = render_prompt(PROMPT_FORMAT, &context);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(!output.is_empty());
}

#[test]
fn test_render_transient_prompt() {
    let context = ModuleContext {
        exit_code: Some(0),
        no_color: true,
        ..ModuleContext::default()
    };

    let result = render_prompt(TRANSIENT_FORMAT, &context);
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(!output.is_empty());
}
