use project::ModuleRegistry;
use project::*;

pub mod minimal_test_module;
pub mod test_module;

pub use minimal_test_module::MinimalTestModule;
pub use test_module::TestModule;

/// [`Project`] setup using [`setup_project`].
#[allow(dead_code)]
#[derive(Debug)]
pub struct ProjectSetup {
    /// [`ModuleRegistry`] with [`MinimalTestModule`] and [`TestModule`].
    pub reg: ModuleRegistry,
    /// [`Project`] with 2 documents and 2 data
    pub project: Project,
    // Document `/doc1` with `doc1_minimal_data`.
    pub doc1: DocumentId,
    /// Data of [`MinimalTestModule`] with all 4 data sections set to `10`.
    pub doc1_minimal_data: DataId,
    // Document `/doc2` with `doc2_test_data`.
    pub doc2: DocumentId,
    /// Default [`TestModule`] data.
    pub doc2_test_data: DataId,
    /// Default [`MinimalTestModule`], not contained in any document.
    pub orphan_minimal_data: DataId,
}

impl ProjectSetup {
    #[allow(dead_code)]
    pub fn view(&self) -> ProjectView {
        self.project.create_view(&self.reg).unwrap()
    }
}

/// Setup a simple [`ProjectSetup`] for testing.
#[allow(dead_code)]
pub fn setup_project() -> ProjectSetup {
    let mut reg = ModuleRegistry::new();
    reg.register::<TestModule>();
    reg.register::<MinimalTestModule>();
    let mut project = Project::new();

    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();

    // Create Documents
    let doc1 = *view.create_document(&mut cb, "/doc1".try_into().unwrap());
    let doc2 = *view.create_document(&mut cb, "/doc2".try_into().unwrap());

    // Apply
    project.apply_changes(cb, &reg).unwrap();
    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();

    // Create Data
    let doc1_minimal_data = *view
        .open_document(doc1)
        .unwrap()
        .create_data::<MinimalTestModule>(&mut cb);
    let doc2_test_data = *view
        .open_document(doc2)
        .unwrap()
        .create_data::<TestModule>(&mut cb);
    let orphan_minimal_data = view.create_data::<MinimalTestModule>(&mut cb);

    // Apply
    project.apply_changes(cb, &reg).unwrap();
    let mut cb = ChangeBuilder::from(&project);
    let view = project.create_view(&reg).unwrap();

    let minimal_view = view
        .open_data_by_id::<MinimalTestModule>(doc1_minimal_data)
        .unwrap();
    minimal_view.apply_persistent(10, &mut cb);
    minimal_view.apply_persistent_user(10, &mut cb);
    minimal_view.apply_session(10, &mut cb);
    minimal_view.apply_shared(10, &mut cb);

    // Apply
    project.apply_changes(cb, &reg).unwrap();

    ProjectSetup {
        reg,
        project,
        doc1,
        doc1_minimal_data,
        doc2,
        doc2_test_data,
        orphan_minimal_data,
    }
}

// This is a test of setup_project, we do not mark it as a test here,
// since otherwise it will be run every time another test includes this module
#[allow(dead_code)]
pub fn test_setup_project() {
    let p = setup_project();
    assert!(p
        .view()
        .open_data_by_id::<TestModule>(p.doc2_test_data)
        .is_some());
    let view = p.view();
    let min = view
        .open_data_by_id::<MinimalTestModule>(p.doc1_minimal_data)
        .unwrap();

    assert_eq!(min.persistent.num, 10);
    assert_eq!(min.persistent_user.num, 10);
    assert_eq!(min.session_data.num, 10);
    assert_eq!(min.shared_data.num, 10);
}
