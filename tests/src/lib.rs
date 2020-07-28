#[cfg(test)]
mod tests {
    use servirtium::servirtium_playback_test;
    use servirtium::{servirtium_record_test, ServirtiumConfiguration};

    fn configure_servirtium(config: &mut ServirtiumConfiguration) {
        config.set_domain_name("test");
    }

    #[servirtium_playback_test("test1.md", configure_servirtium)]
    fn simple_playback_test() {
        println!("this test does nothing...");
    }

    #[servirtium_record_test("test2.md", configure_servirtium)]
    fn simple_record_test() {
        println!("this test does nothing...");
    }
}
