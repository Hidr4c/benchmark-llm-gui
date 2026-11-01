use benchmark_llm_gui::domain::model::ModelBenchmark;
use benchmark_llm_gui::domain::pricing::ModelPricing;

#[test]
fn test_cloud_model_pricing_has_zero_local_electricity() {
    let cloud_m = ModelBenchmark {
        id: "google/gemini-2.5-flash".into(),
        name: "Gemini 2.5 Flash".into(),
        is_cloud: true,
        pricing_input_usd: Some(0.15),
        pricing_output_usd: Some(0.60),
        tokens_per_sec: Some(150.0),
        ..Default::default()
    };

    assert_eq!(cloud_m.estimated_hardware_tdp_watts(), 0.0);
    assert_eq!(cloud_m.estimated_kwh_per_1m(), 0.0);
    assert_eq!(cloud_m.estimated_electricity_cost_usd_per_1m(), 0.0);
    // 100 tasks of ~250k tokens
    let cost = cloud_m.estimated_cost_100_tasks();
    assert!(cost > 0.01 && cost < 0.20, "Cloud 100 tasks cost expected ~$0.069, got {cost}");
}

#[test]
fn test_micro_iot_model_power_and_electricity_cost() {
    let iot_m = ModelBenchmark {
        id: "qwen/qwen2.5-0.5b".into(),
        name: "Qwen 2.5 0.5B".into(),
        size_gb: 0.35,
        vram_rec_gb: 0.8,
        tokens_per_sec: Some(120.0),
        is_cloud: false,
        ..Default::default()
    };

    assert_eq!(iot_m.estimated_hardware_tdp_watts(), 10.0);
    let kwh = iot_m.estimated_kwh_per_1m();
    assert!(kwh > 0.01 && kwh < 0.05, "IoT kWh should be small, got {kwh}");
    let cost_1m = iot_m.estimated_electricity_cost_usd_per_1m();
    assert!(cost_1m > 0.001 && cost_1m < 0.02, "IoT cost per 1M should be ~$0.006, got {cost_1m}");
}

#[test]
fn test_discrete_gpu_rtx4090_electricity_cost() {
    let rtx4090_m = ModelBenchmark {
        id: "qwen/qwen3.8-27b".into(),
        name: "Qwen 3.8 27B".into(),
        size_gb: 15.7,
        vram_rec_gb: 24.0,
        tokens_per_sec: Some(40.0),
        is_cloud: false,
        ..Default::default()
    };

    assert_eq!(rtx4090_m.estimated_hardware_tdp_watts(), 380.0);
    let kwh = rtx4090_m.estimated_kwh_per_1m();
    assert!(kwh > 2.0 && kwh < 4.0, "RTX 4090 kWh for 1M should be ~2.64 kWh, got {kwh}");
    let cost_1m = rtx4090_m.estimated_electricity_cost_usd_per_1m();
    assert!(cost_1m > 0.40 && cost_1m < 1.00, "RTX 4090 cost per 1M should be ~$0.66, got {cost_1m}");
}

#[test]
fn test_pricing_calculator_fallback_throughput() {
    let fallback_iot = ModelPricing::fallback_throughput_tps(0.4);
    assert_eq!(fallback_iot, 120.0);

    let fallback_mid = ModelPricing::fallback_throughput_tps(14.0);
    assert_eq!(fallback_mid, 32.0);

    let fallback_large = ModelPricing::fallback_throughput_tps(35.0);
    assert_eq!(fallback_large, 16.0);
}
