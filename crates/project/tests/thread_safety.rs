mod common;
use common::minimal_test_module::MinimalTestModule;
use data::{DataUuid, DataView, Snapshot};
use document::DocumentView;
use project::*;

#[test]
fn test_send_sync() {
    const fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<Project>();
    assert_send_sync::<ModuleRegistry>();
    assert_send_sync::<DataUuid>();
    assert_send_sync::<Snapshot<MinimalTestModule>>();
    assert_send_sync::<DocumentView>();
    assert_send_sync::<ProjectView>();
    assert_send_sync::<DataView<MinimalTestModule>>();
}
