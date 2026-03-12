use blocks_moc::MocManifest;

pub fn canonical_yaml(manifest: &MocManifest) -> Result<String, String> {
    serde_yaml::to_string(manifest)
        .map_err(|error| format!("failed to emit canonical moc yaml: {error}"))
}

pub fn check_parity(emitted_yaml: &str, against_source: &str) -> Result<(), String> {
    let emitted_manifest = MocManifest::from_yaml_str(emitted_yaml)
        .map_err(|error| format!("failed to parse emitted moc yaml: {error}"))?;
    let against_manifest = MocManifest::from_yaml_str(against_source)
        .map_err(|error| format!("failed to parse check-against moc yaml: {error}"))?;

    let emitted_canonical = canonical_yaml(&emitted_manifest)?;
    let against_canonical = canonical_yaml(&against_manifest)?;
    if emitted_canonical == against_canonical {
        Ok(())
    } else {
        Err(
            "emitted moc does not match the check-against manifest after canonical normalization"
                .to_string(),
        )
    }
}
