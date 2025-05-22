result.changes.push(MigrationChange {
    change_type: ChangeType::Created,
    file_path: format!("assets/{}/{}", dest_subdir, relative_path.display()).into(),
    description: format!("Copied static asset from {}", path.display()).into(),
}); 