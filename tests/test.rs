#[cfg(test)]
mod test {
    use backup_config::prelude::*;

    #[test]
    fn test_config() {
        let conf = Conf::builder()
            .file("./test_data/simple.toml")
            .file("./tests/test_data/simple.toml")
            .load()
            .unwrap();
        assert_eq!(conf.db_url, "sqlite://data.db?mode=rwc");
    }

    #[test]
    fn print_template_toml() {
        use confique::toml::{template, FormatOptions};
        let template_out = template::<Conf>(FormatOptions::default());
        println!("template_out: \n```\n{}\n```", template_out);
    }
}
