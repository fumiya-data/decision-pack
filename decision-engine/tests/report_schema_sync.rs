use std::fs;
use std::path::Path;

#[test]
fn decision_engine_and_reporting_schema_files_match() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let engine_schema =
        fs::read_to_string(manifest_dir.join("schemas/simulation_report_v0.1.schema.json"))
            .expect("decision-engine schema should be readable");
    let reporting_schema = fs::read_to_string(
        manifest_dir.join("../reporting/schemas/simulation_report_v0.1.schema.json"),
    )
    .expect("reporting schema should be readable");

    assert_eq!(
        normalize_newlines(&engine_schema),
        normalize_newlines(&reporting_schema),
        "simulation_report_v0.1 schema files must stay byte-equivalent apart from newlines"
    );
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").trim_end().to_string()
}
