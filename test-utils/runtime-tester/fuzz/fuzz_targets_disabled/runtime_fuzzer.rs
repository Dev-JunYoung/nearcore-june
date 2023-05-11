#![no_main]
use libfuzzer_sys::fuzz_target;
use runtime_tester::Scenario;
use std::fs::File;
use std::time::Duration;
use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


fn do_fuzz(scenario: &Scenario) -> Result<(), String> {
    print_file_path_and_function_name!();
    let stats = scenario.run().result.map_err(|e| e.to_string())?;
    for block_stats in stats.blocks_stats {
        if block_stats.block_production_time > Duration::from_secs(2) {
            return Err(format!(
                "block at height {} was produced in {:?}",
                block_stats.height, block_stats.block_production_time
            ));
        }
    }
    Ok(())
}

fn fuzz(scenario: Scenario) {
    print_file_path_and_function_name!();
    if let Err(err) = do_fuzz(&scenario) {
        let file = "failed_scenario.json";
        serde_json::to_writer(&File::create(file).unwrap(), &scenario).unwrap();
        panic!("Bad scenario: {}, {}", file, err);
    }
}

fuzz_target!(|scenario: Scenario| { fuzz(scenario) });
