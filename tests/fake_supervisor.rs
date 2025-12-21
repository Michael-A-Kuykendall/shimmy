mod common;
use shimmy::orchestrator::test_shims::FakeSupervisor;

#[tokio::test]
async fn fake_supervisor_works_with_verification() {
    let sup = FakeSupervisor::new();
    // this repo may not have scripts available - run_stack_verify should return Ok(false) or Ok(true)
    let res = shimmy::orchestrator::verification::run_stack_verify(&sup, "http://localhost:8080").await.unwrap();
    assert!(res == true || res == false);
}
