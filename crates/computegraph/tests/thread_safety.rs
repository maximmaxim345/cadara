use computegraph::*;

#[test]
const fn test_send_sync() {
    const fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<ComputeGraph>();
    assert_send_sync::<DynamicNode>();
    assert_send_sync::<InputPort<()>>();
    assert_send_sync::<OutputPort<()>>();
}
