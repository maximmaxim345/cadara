mod common;
use common::minimal_test_module::MinimalTestModule;
use data::{DataSession, DataUuid, Snapshot};
use document::DocumentSession;
use project::*;

#[test]
fn test_send_sync() {
    const fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Project>();
    assert_send_sync::<ModuleRegistry>();
    assert_send_sync::<DataUuid>();
    assert_send_sync::<Snapshot<MinimalTestModule>>();
    assert_send_sync::<DocumentSession>();
    assert_send_sync::<ProjectSession>();
    assert_send_sync::<DataSession<MinimalTestModule>>();
}
