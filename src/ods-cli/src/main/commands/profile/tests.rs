mod test_profile_commands {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_profile_list_and_show() {
        let res = run_profile_list_command(&["ods".into(), "profile".into(), "list".into()]);
        assert!(res.is_ok());

        let res = run_profile_list_command(&[
            "ods".into(),
            "profile".into(),
            "list".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_profile_show_command(&["ods".into(), "profile".into(), "show".into(), "note".into()]);
        assert!(res.is_ok());

        let res = run_profile_show_command(&[
            "ods".into(),
            "profile".into(),
            "show".into(),
            "note".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let err = run_profile_show_command(&["ods".into(), "profile".into(), "show".into()]).unwrap_err();
        assert!(err.message().contains("name"));

        let err = run_profile_show_command(&["ods".into(), "profile".into(), "show".into(), "nonexistent_xyz".into()]).unwrap_err();
        assert!(err.message().contains("unknown profile"));
    }

    #[test]
    fn test_profile_init() {
        let td = tempdir().unwrap();
        let root = td.path();

        let err = run_profile_init_command(&["ods".into(), "profile".into(), "init".into()]).unwrap_err();
        assert!(err.message().contains("name"));

        let res = run_profile_init_command(&[
            "ods".into(),
            "profile".into(),
            "init".into(),
            "custom-spec".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());

        let profile_file = root.join(".ods").join("profiles").join("custom-spec.md");
        assert!(profile_file.exists());

        // duplicate init
        let res = run_profile_init_command(&[
            "ods".into(),
            "profile".into(),
            "init".into(),
            "custom-spec".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_alias_command_and_insert_alias() {
        let td = tempdir().unwrap();
        let root = td.path();

        // help/usage
        let res = run_aliases_command(&["ods".into(), "alias".into(), "--help".into()]);
        assert!(res.is_ok());

        let err = run_aliases_command(&["ods".into(), "alias".into(), "add".into(), "Overview".into()]).unwrap_err();
        assert!(err.message().contains("Synonym"));

        // create ods.toml and add alias
        let toml_path = root.join("ods.toml");
        fs::write(&toml_path, "spec = \"0.1\"\n").unwrap();

        // list
        let res = run_aliases_command(&[
            "ods".into(),
            "alias".into(),
            "list".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());

        let res = run_aliases_command(&[
            "ods".into(),
            "alias".into(),
            "add".into(),
            "Overview".into(),
            "Summary".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());

        let content = fs::read_to_string(&toml_path).unwrap();
        assert!(content.contains("Overview"));
        assert!(content.contains("Summary"));

        let res = run_aliases_command(&[
            "ods".into(),
            "alias".into(),
            "list".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_insert_alias_into_ods_toml_and_alias_add_toml() {
        let text = "version = \"0.1\"\n";
        let out = insert_alias_into_ods_toml(text, "Overview", "Summary");
        assert!(out.contains("[aliases]"));
        assert!(out.contains("Overview = [\"Summary\"]"));

        let text_with_aliases = "[aliases]\nOverview = [\"Intro\"]\n";
        let out = insert_alias_into_ods_toml(text_with_aliases, "Overview", "Summary");
        assert!(out.contains("\"Summary\", "));

        let td = tempdir().unwrap();
        let root = td.path();
        let toml_path = root.join("ods.toml");
        fs::write(&toml_path, "version = \"0.1\"\n").unwrap();

        let res = run_alias_add_command(&[
            "ods".into(),
            "alias".into(),
            "add".into(),
            "Overview".into(),
            "Summary".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());
        let content = fs::read_to_string(&toml_path).unwrap();
        assert!(content.contains("[aliases]"));
        assert!(content.contains("Overview = [\"Summary\"]"));
    }
}
