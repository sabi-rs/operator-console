use operator_console::worker_client::WorkerRequest;

#[test]
fn worker_request_serializes_load_dashboard() {
    let request = serde_json::to_string(&WorkerRequest::LoadDashboard).expect("serialize");

    assert!(request.contains("LoadDashboard"));
}
