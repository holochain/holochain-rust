pub mod storage;
pub mod content;
pub mod eav;

#[cfg(test)]
mod tests {
    use cas::storage::ContentAddressableStorage;
    use cas::content::AddressableContent;
    use cas::content::tests::ExampleAddressableContent;
    use cas::content::tests::OtherExampleAddressableContent;
    use cas::storage::tests::ExampleContentAddressableStorage;

    #[test]
    fn example_content_round_trip() {
        let content = ExampleAddressableContent::from_content(&"foo".to_string());
        let other_content = OtherExampleAddressableContent::from_content(&"bar".to_string());
        let mut cas = ExampleContentAddressableStorage::new();

        assert_eq!(Ok(false), cas.contains(&content.address()));
        assert_eq!(Ok(false), cas.contains(&other_content.address()));

        // round trip some AddressableContent through the ContentAddressableStorage
        assert_eq!(Ok(()), cas.add(&content));
        assert_eq!(Ok(true), cas.contains(&content.address()));
        assert_eq!(Ok(false), cas.contains(&other_content.address()));
        assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));

        // multiple types of AddressableContent can sit in a single ContentAddressableStorage
        // the safety of this is only as good as the hashing algorithm(s) used
        assert_eq!(Ok(()), cas.add(&other_content));
        assert_eq!(Ok(true), cas.contains(&content.address()));
        assert_eq!(Ok(true), cas.contains(&other_content.address()));
        assert_eq!(Ok(Some(content.clone())), cas.fetch(&content.address()));
        assert_eq!(Ok(Some(other_content.clone())), cas.fetch(&other_content.address()));
    }
}
