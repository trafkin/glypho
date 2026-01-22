use clap::Parser;
use clap_stdin::FileOrStdin;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(default_value = "-")]
    pub input: Option<FileOrStdin>,
    #[arg(short, long)]
    pub port: Option<u16>,
    #[arg(short, long, default_value_t = false)]
    pub no_browser: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // ==================== Default Value Tests ====================

    #[test]
    fn test_args_default_values() {
        let args = Args::try_parse_from(["glypho"]).unwrap();

        // Port should be None by default
        assert!(args.port.is_none());

        // no_browser should be false by default
        assert!(!args.no_browser);
    }

    // ==================== Port Flag Tests ====================

    #[rstest]
    #[case(&["glypho", "-p", "8080"], Some(8080))]
    #[case(&["glypho", "--port", "3000"], Some(3000))]
    #[case(&["glypho", "-p", "0"], Some(0))]
    #[case(&["glypho", "--port", "65535"], Some(65535))]
    fn test_port_flag(#[case] args: &[&str], #[case] expected_port: Option<u16>) {
        let parsed = Args::try_parse_from(args).unwrap();
        assert_eq!(parsed.port, expected_port);
    }

    #[test]
    fn test_port_flag_invalid() {
        // Port out of range should fail
        let result = Args::try_parse_from(["glypho", "-p", "99999"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_port_flag_non_numeric() {
        let result = Args::try_parse_from(["glypho", "-p", "abc"]);
        assert!(result.is_err());
    }

    // ==================== No Browser Flag Tests ====================

    #[rstest]
    #[case(&["glypho", "-n"], true)]
    #[case(&["glypho", "--no-browser"], true)]
    #[case(&["glypho"], false)]
    fn test_no_browser_flag(#[case] args: &[&str], #[case] expected: bool) {
        let parsed = Args::try_parse_from(args).unwrap();
        assert_eq!(parsed.no_browser, expected);
    }

    // ==================== Combined Flags Tests ====================

    #[test]
    fn test_multiple_flags() {
        let args = Args::try_parse_from(["glypho", "-p", "8080", "-n"]).unwrap();

        assert_eq!(args.port, Some(8080));
        assert!(args.no_browser);
    }

    #[test]
    fn test_long_flags_combined() {
        let args = Args::try_parse_from(["glypho", "--port", "3000", "--no-browser"]).unwrap();

        assert_eq!(args.port, Some(3000));
        assert!(args.no_browser);
    }

    #[test]
    fn test_flags_order_independent() {
        let args1 = Args::try_parse_from(["glypho", "-n", "-p", "8080"]).unwrap();
        let args2 = Args::try_parse_from(["glypho", "-p", "8080", "-n"]).unwrap();

        assert_eq!(args1.port, args2.port);
        assert_eq!(args1.no_browser, args2.no_browser);
    }

    // ==================== Input File Tests ====================

    #[test]
    fn test_input_with_file_path() {
        use assert_fs::prelude::*;
        let temp_file = assert_fs::NamedTempFile::new("test.md").unwrap();
        temp_file.write_str("# Test").unwrap();

        let args = Args::try_parse_from(["glypho", temp_file.path().to_str().unwrap()]).unwrap();

        assert!(args.input.is_some());
    }

    #[test]
    fn test_input_with_flags_and_file() {
        use assert_fs::prelude::*;
        let temp_file = assert_fs::NamedTempFile::new("test.md").unwrap();
        temp_file.write_str("# Test").unwrap();

        let args = Args::try_parse_from([
            "glypho",
            "-p",
            "8080",
            "-n",
            temp_file.path().to_str().unwrap(),
        ])
        .unwrap();

        assert_eq!(args.port, Some(8080));
        assert!(args.no_browser);
        assert!(args.input.is_some());
    }

    // ==================== Debug Trait Test ====================

    #[test]
    fn test_args_debug() {
        let args = Args::try_parse_from(["glypho", "-p", "8080"]).unwrap();
        let debug_str = format!("{:?}", args);

        assert!(debug_str.contains("Args"));
        assert!(debug_str.contains("port"));
        assert!(debug_str.contains("8080"));
    }

    // ==================== Help and Version Tests ====================

    #[test]
    fn test_help_flag() {
        let result = Args::try_parse_from(["glypho", "--help"]);
        // Help should cause an error (early exit)
        assert!(result.is_err());
    }

    #[test]
    fn test_version_flag() {
        let result = Args::try_parse_from(["glypho", "--version"]);
        // Version should cause an error (early exit)
        assert!(result.is_err());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_port_minimum_value() {
        let args = Args::try_parse_from(["glypho", "-p", "1"]).unwrap();
        assert_eq!(args.port, Some(1));
    }

    #[test]
    fn test_port_maximum_value() {
        let args = Args::try_parse_from(["glypho", "-p", "65535"]).unwrap();
        assert_eq!(args.port, Some(65535));
    }

    #[test]
    fn test_negative_port_rejected() {
        let result = Args::try_parse_from(["glypho", "-p", "-1"]);
        assert!(result.is_err());
    }
}
