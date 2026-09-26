pub(crate) fn map_name(name: &str) -> String {
    let converted: String = name
        .chars()
        .map(|c| match c {
            ',' | '/' | '\\' => '-',
            other => other,
        })
        .collect();

    converted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_conversion() {
        let name = "BuggySoftware";

        let new_name = map_name(name);

        assert_eq!(new_name, name);
    }

    #[test]
    fn test_conversion() {
        let name = "A file name, which is correct / Something else \\ more";

        let converted_name = map_name(name);

        assert_eq!(
            converted_name,
            "A file name- which is correct - Something else - more"
        )
    }
}
