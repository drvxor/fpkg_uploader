use dialoguer::Confirm;

pub fn yn_prompt(prompt: &str) -> bool {
    Confirm::new()
        .with_prompt(prompt)
        .default(true)
        .interact()
        .unwrap_or(false)
}