use project::Path;
#[test]
fn test_valid_paths() {
    assert!(Path::new("/part".to_string()).is_ok());
    assert!(Path::new("/assemblies and drawings/drawing".to_string()).is_ok());
    assert!(Path::new("/parts/screws\\/bolts/bolt1".to_string()).is_ok());
    assert!(Path::new("/folder1/folder2/file\\/withslash".to_string()).is_ok());
    assert!(Path::new("/folder1/folder2/file".to_string()).is_ok());
    assert!(Path::new("/filewith\\/slashes\\\\".to_string()).is_ok());
    assert!(Path::new("/😐/🫢".to_string()).is_ok());
    assert!(Path::new("/💯\\//🖤".to_string()).is_ok());
}

#[test]
fn test_invalid_paths() {
    assert!(Path::new("part".to_string()).is_err());
    assert!(Path::new("/parts/".to_string()).is_err());
    assert!(Path::new("//part".to_string()).is_err());
    assert!(Path::new("/part/".to_string()).is_err());
    assert!(Path::new("/part\\".to_string()).is_err());
    assert!(Path::new("/".to_string()).is_err());
    assert!(Path::new("\\/".to_string()).is_err());
    assert!(Path::new("/doc\\\\/".to_string()).is_err());
    assert!(Path::new("/folder \\1/doc".to_string()).is_err());
    assert!(Path::new("/\\/\\\\/".to_string()).is_err());
    assert!(Path::new("/\\😐/🫢".to_string()).is_err());
}

#[test]
fn test_get_name_escaped() {
    assert_eq!(
        Path::new("/folder/part".to_string())
            .unwrap()
            .get_name_escaped(),
        "part"
    );
    assert_eq!(
        Path::new("/a/b/part".to_string())
            .unwrap()
            .get_name_escaped(),
        "part"
    );
    assert_eq!(
        Path::new("/folder/part (1) (2)".to_string())
            .unwrap()
            .get_name_escaped(),
        "part (1) (2)"
    );
    assert_eq!(
        Path::new("/folder/part (1)\\/ (2)".to_string())
            .unwrap()
            .get_name_escaped(),
        "part (1)\\/ (2)"
    );
    assert_eq!(
        Path::new("/folder/part (1)\\\\ (2)".to_string())
            .unwrap()
            .get_name_escaped(),
        "part (1)\\\\ (2)"
    );
    assert_eq!(
        Path::new("/folder/part (1)\\/ (2)\\/ (3)".to_string())
            .unwrap()
            .get_name_escaped(),
        "part (1)\\/ (2)\\/ (3)"
    );
    assert_eq!(
        Path::new("/folder/part (1)\\/ (2)\\\\ (3)".to_string())
            .unwrap()
            .get_name_escaped(),
        "part (1)\\/ (2)\\\\ (3)"
    );
    assert_eq!(
        Path::new("/part (1)\\/ (2)\\\\ (3)".to_string())
            .unwrap()
            .get_name_escaped(),
        "part (1)\\/ (2)\\\\ (3)"
    );
    assert_eq!(
        Path::new("/\\\\/part".to_string())
            .unwrap()
            .get_name_escaped(),
        "part"
    );
    assert_eq!(
        Path::new("/11\\//2\\/2".to_string())
            .unwrap()
            .get_name_escaped(),
        "2\\/2"
    );
    assert_eq!(
        Path::new("/💯💯\\//\\\\🖤\\/⩩".to_string())
            .unwrap()
            .get_name_escaped(),
        "\\\\🖤\\/⩩"
    );
}

#[test]
fn test_get_name() {
    assert_eq!(
        Path::new("/folder/part".to_string()).unwrap().get_name(),
        "part"
    );
    assert_eq!(
        Path::new("/a/b/part".to_string()).unwrap().get_name(),
        "part"
    );
    assert_eq!(
        Path::new("/folder/part (1) (2)".to_string())
            .unwrap()
            .get_name(),
        "part (1) (2)"
    );
    assert_eq!(
        Path::new("/folder/part (1)\\/ (2)".to_string())
            .unwrap()
            .get_name(),
        "part (1)/ (2)"
    );
    assert_eq!(
        Path::new("/folder/part (1)\\\\ (2)".to_string())
            .unwrap()
            .get_name(),
        "part (1)\\ (2)"
    );
    assert_eq!(
        Path::new("/folder/part (1)\\/ (2)\\/ (3)".to_string())
            .unwrap()
            .get_name(),
        "part (1)/ (2)/ (3)"
    );
    assert_eq!(
        Path::new("/folder/part (1)\\/ (2)\\\\ (3)".to_string())
            .unwrap()
            .get_name(),
        "part (1)/ (2)\\ (3)"
    );
    assert_eq!(
        Path::new("/part (1)\\/ (2)\\\\ (3)".to_string())
            .unwrap()
            .get_name(),
        "part (1)/ (2)\\ (3)"
    );
    assert_eq!(
        Path::new("/\\\\/part".to_string()).unwrap().get_name(),
        "part"
    );
    assert_eq!(
        Path::new("/11\\//2\\/2".to_string()).unwrap().get_name(),
        "2/2"
    );
    assert_eq!(
        Path::new("/💯💯\\//\\\\🖤\\/⩩".to_string())
            .unwrap()
            .get_name(),
        "\\🖤/⩩"
    );
}

#[test]
fn test_parent_folders_unescaped() {
    let path = Path::new("/folder 1/folder 2/doc".to_string()).unwrap();
    let mut f = path.ancestors_unescaped();
    assert_eq!(f.next().unwrap(), "folder 1");
    assert_eq!(f.next().unwrap(), "folder 2");
    assert!(f.next().is_none());

    let path = Path::new("/folder \\\\1/folder 2/doc".to_string()).unwrap();
    let mut f = path.ancestors_unescaped();
    assert_eq!(f.next().unwrap(), "folder \\\\1");
    assert_eq!(f.next().unwrap(), "folder 2");
    assert!(f.next().is_none());

    let path = Path::new("/folder 1/folder \\/2/doc".to_string()).unwrap();
    let mut f = path.ancestors_unescaped();
    assert_eq!(f.next().unwrap(), "folder 1");
    assert_eq!(f.next().unwrap(), "folder \\/2");
    assert!(f.next().is_none());

    let path = Path::new("/folder 1 \\\\/folder 2 \\/\\\\/doc".to_string()).unwrap();
    let mut f = path.ancestors_unescaped();
    assert_eq!(f.next().unwrap(), "folder 1 \\\\");
    assert_eq!(f.next().unwrap(), "folder 2 \\/\\\\");
    assert!(f.next().is_none());

    let path = Path::new("/💯💯\\//漢/字\\\\🖤\\/⩩/doc".to_string()).unwrap();
    let mut f = path.ancestors_unescaped();
    assert_eq!(f.next().unwrap(), "💯💯\\/");
    assert_eq!(f.next().unwrap(), "漢");
    assert_eq!(f.next().unwrap(), "字\\\\🖤\\/⩩");
    assert!(f.next().is_none());
}

#[test]
fn test_parent_folders() {
    let path = Path::new("/folder 1/folder 2/doc".to_string()).unwrap();
    let mut f = path.ancestors();
    assert_eq!(f.next().unwrap(), "folder 1");
    assert_eq!(f.next().unwrap(), "folder 2");
    assert!(f.next().is_none());

    let path = Path::new("/folder \\\\1/folder 2/doc".to_string()).unwrap();
    let mut f = path.ancestors();
    assert_eq!(f.next().unwrap(), "folder \\1");
    assert_eq!(f.next().unwrap(), "folder 2");
    assert!(f.next().is_none());

    let path = Path::new("/folder 1/folder \\/2/doc".to_string()).unwrap();
    let mut f = path.ancestors();
    assert_eq!(f.next().unwrap(), "folder 1");
    assert_eq!(f.next().unwrap(), "folder /2");
    assert!(f.next().is_none());

    let path = Path::new("/folder 1 \\\\/folder 2 \\/\\\\/doc".to_string()).unwrap();
    let mut f = path.ancestors();
    assert_eq!(f.next().unwrap(), "folder 1 \\");
    assert_eq!(f.next().unwrap(), "folder 2 /\\");
    assert!(f.next().is_none());

    let path = Path::new("/💯💯\\//漢/字\\\\🖤\\/⩩/doc".to_string()).unwrap();
    let mut f = path.ancestors();
    assert_eq!(f.next().unwrap(), "💯💯/");
    assert_eq!(f.next().unwrap(), "漢");
    assert_eq!(f.next().unwrap(), "字\\🖤/⩩");
    assert!(f.next().is_none());
}

#[test]
fn test_serialization_deserialization() {
    let path = Path::new("/parts/漢字/screws\\/bolts/bolt1".to_string()).unwrap();
    let serialized = serde_json::to_string(&path).unwrap();
    let deserialized: Path = serde_json::from_str(&serialized).unwrap();
    assert_eq!(path, deserialized);

    let path2 = Path::new("/part".to_string()).unwrap();
    let serialized = serde_json::to_string(&path2).unwrap();
    let deserialized: Path = serde_json::from_str(&serialized).unwrap();
    assert_eq!(path2, deserialized);

    assert!(serde_json::from_str::<Path>("invalid path").is_err());
}

#[test]
fn test_display() {
    let path = Path::new("/parts/screws\\/bolts/bolt1".to_string()).unwrap();
    assert_eq!(format!("{}", path), "/parts/screws\\/bolts/bolt1");
}

#[test]
fn test_increment_name_suffix() {
    let path = Path::new("/part".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/part (2)".to_string()).unwrap()
    );

    let path = Path::new("/folder/part (2)".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/folder/part (3)".to_string()).unwrap()
    );

    let path = Path::new("/folder/part (0)".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/folder/part (1)".to_string()).unwrap()
    );

    let path = Path::new("/folder/part (-1)".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/folder/part (-1) (2)".to_string()).unwrap()
    );

    let path = Path::new("/folder/subfolder/part (52)".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/folder/subfolder/part (53)".to_string()).unwrap()
    );

    let path = Path::new("/file\\/withslash (12)".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/file\\/withslash (13)".to_string()).unwrap()
    );

    let path = Path::new("/file\\/withslash".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/file\\/withslash (2)".to_string()).unwrap()
    );

    let path = Path::new("/folder (3)/part (4) (5)".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/folder (3)/part (4) (6)".to_string()).unwrap()
    );

    let path = Path::new("/folder (2)/part".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/folder (2)/part (2)".to_string()).unwrap()
    );

    let path = Path::new("/folder (2)/part".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/folder (2)/part (2)".to_string()).unwrap()
    );

    let path = Path::new("/part(3)".to_string()).unwrap();
    assert_eq!(
        path.increment_name_suffix(),
        Path::new("/part(3) (2)".to_string()).unwrap()
    );
}
