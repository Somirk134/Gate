use gate_integration::ConnectionSimulationPlan;

#[test]
fn performance_simulation_matrix_declares_alpha_scales_without_running_load() {
    let plans = ConnectionSimulationPlan::alpha_matrix();
    let scales: Vec<usize> = plans.iter().map(|plan| plan.connections).collect();

    assert_eq!(scales, vec![100, 500, 1000, 5000]);
    assert!(plans.iter().all(|plan| !plan.execute_load));
}
