mod test_profile_commands {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_profile_list_and_show() {
        let td = tempdir().unwrap();
        let root = td.path();
        ods_core::init_workspace(root, ods_core::InitOptions::default()).unwrap();

        let root_s = root.to_str().unwrap().to_string();
        let res = run_profile_list_command(&[
            "ods".into(),
            "profile".into(),
            "list".into(),
            root_s.clone(),
        ]);
        assert!(res.is_ok());

        let res = run_profile_list_command(&[
            "ods".into(),
            "profile".into(),
            "list".into(),
            root_s.clone(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_profile_show_command(&[
            "ods".into(),
            "profile".into(),
            "show".into(),
            "note".into(),
            root_s.clone(),
        ]);
        assert!(res.is_ok());

        let res = run_profile_show_command(&[
            "ods".into(),
            "profile".into(),
            "show".into(),
            "note".into(),
            root_s.clone(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let err =
            run_profile_show_command(&["ods".into(), "profile".into(), "show".into()]).unwrap_err();
        assert!(err.message().contains("name"));

        let err = run_profile_show_command(&[
            "ods".into(),
            "profile".into(),
            "show".into(),
            "nonexistent_xyz".into(),
        ])
        .unwrap_err();
        assert!(err.message().contains("profile not found"));
    }

    #[test]
    fn test_profile_init() {
        let td = tempdir().unwrap();
        let root = td.path();

        let err =
            run_profile_init_command(&["ods".into(), "profile".into(), "init".into()]).unwrap_err();
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
        let profile_text = fs::read_to_string(&profile_file).unwrap();
        assert!(profile_text.contains("custom_profile:"));
        assert!(profile_text.contains("required_keys:"));
        assert!(!profile_text.contains("optional_keys:"));
        assert!(!profile_text.contains("forbidden_keys:"));

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
}
