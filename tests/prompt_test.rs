use my_prompt::ModuleContext;
use my_prompt::prompt::{PROMPT_FORMAT, PromptModule, TRANSIENT_FORMAT, render_prompt};

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

#[test]
fn test_prompt_modules_enum() {
    let context = ModuleContext::default();

    // Test that all enum variants can render
    let modules = [
        PromptModule::Character,
        PromptModule::Claude,
        PromptModule::Direnv,
        PromptModule::Fail,
        PromptModule::Git,
        PromptModule::Hostname,
        PromptModule::Path,
        PromptModule::Time,
        PromptModule::Username,
    ];

    for module in modules {
        // Should not panic
        let _ = render_prompt(&[module], &context);
    }
}

#[test]
fn test_parallel_rendering() {
    use std::time::Instant;

    let context = ModuleContext::default();
    let start = Instant::now();

    let result = render_prompt(PROMPT_FORMAT, &context);

    let elapsed = start.elapsed();

    assert!(result.is_ok());
    // Should complete in reasonable time...
    assert!(
        elapsed.as_millis() < 100,
        "Prompt took too long: {:?}",
        elapsed
    );
}
