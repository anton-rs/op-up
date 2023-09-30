/// Macro to create a selection prompt.
#[macro_export]
macro_rules! make_selection {
    ($name:ident, $prompt:expr, $options:expr) => {
        let $name = inquire::Select::new($prompt, $options)
            .without_help_message()
            .prompt()?
            .to_string();
    };
}
