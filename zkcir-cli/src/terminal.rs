use indicatif::{ProgressBar, ProgressStyle};

pub enum OutputColor {
    Green,
    Blue,
    Red,
}

pub fn get_formatted_left_output(output: &str, color: OutputColor) -> String {
    let reset = "\x1b[0m";

    format!(
        "{}{:>12}{reset}",
        match color {
            OutputColor::Green => "\x1b[1;32m",
            OutputColor::Blue => "\x1b[1;36m",
            OutputColor::Red => "\x1b[1;31m",
        },
        output
    )
}

pub fn create_new_pb(length: u64, progress_message_left: &str) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{} [{{bar:40}}] {{pos}}/{{len}}{{msg}}",
                get_formatted_left_output(progress_message_left, OutputColor::Blue)
            ))
            .unwrap()
            .progress_chars("=> "),
    );
    pb
}
