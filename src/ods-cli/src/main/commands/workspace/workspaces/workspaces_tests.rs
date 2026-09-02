#[cfg(test)]
mod test_workspaces_command {
    use super::*;

    #[test]
    fn test_run_workspaces_command_subcommands() {
        assert!(run_workspaces_command(&["ods".into(), "workspaces".into(), "help".into()]).is_ok());
        assert!(run_workspaces_command(&["ods".into(), "workspaces".into(), "path".into()]).is_ok());
        assert!(run_workspaces_command(&["ods".into(), "workspaces".into(), "list".into()]).is_ok());

        let err = run_workspaces_command(&["ods".into(), "workspaces".into(), "unknown".into()]);
        assert!(err.is_err());

        let td = tempfile::tempdir().unwrap();
        let sample = td.path().join("ws");
        std::fs::create_dir_all(&sample).unwrap();
        std::fs::write(sample.join("ods.toml"), "spec = \"0.1\"\n").unwrap();

        let res_list_txt = run_workspaces_command(&[
            "ods".into(),
            "workspaces".into(),
            "list".into(),
        ]);
        assert!(res_list_txt.is_ok());

        let res_list_json = run_workspaces_command(&[
            "ods".into(),
            "workspaces".into(),
            "list".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_list_json.is_ok());

        // add
        let res_add = run_workspaces_command(&[
            "ods".into(),
            "workspaces".into(),
            "add".into(),
            sample.to_str().unwrap().into(),
        ]);
        assert!(res_add.is_ok());

        // duplicate add
        let res_add_dup = run_workspaces_command(&[
            "ods".into(),
            "workspaces".into(),
            "add".into(),
            sample.to_str().unwrap().into(),
        ]);
        assert!(res_add_dup.is_ok());

        // remove
        let res_rm = run_workspaces_command(&[
            "ods".into(),
            "workspaces".into(),
            "remove".into(),
            sample.to_str().unwrap().into(),
        ]);
        assert!(res_rm.is_ok());

        // remove non-tracked
        let res_rm_non = run_workspaces_command(&[
            "ods".into(),
            "workspaces".into(),
            "remove".into(),
            sample.to_str().unwrap().into(),
        ]);
        assert!(res_rm_non.is_ok());
    }
}

