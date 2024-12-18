mod common;
use common::*;
use project::*;

#[test]
fn test_send_sync() {
    const fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<ModuleRegistry>();

    assert_send_sync::<UserId>();
    assert_send_sync::<DataId>();
    assert_send_sync::<DocumentId>();

    assert_send_sync::<Project>();
    assert_send_sync::<ProjectView>();
    assert_send_sync::<DocumentView<'_>>();
    assert_send_sync::<DataView<'_, MinimalTestModule>>();
}
