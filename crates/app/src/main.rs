//! Thin process boundary. Stdout is the result; stderr is structured telemetry.
mod driver;
use serde_json::json;
use starter_domain::{Name, greet};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["capabilities"] {
        println!(
            "{}",
            starter_engine::capabilities(&starter_engine::Session::default(), false)
        );
        return ExitCode::SUCCESS;
    }
    if args == ["driver"] || (args.len() == 2 && args[0] == "script") {
        return run_driver(&args);
    }
    let run_id = std::env::var("HARNESS_RUN_ID").unwrap_or_else(|_| "manual".into());
    let result = match args.as_slice() {
        [command, raw] if command == "greet" => Name::try_from(raw.as_str())
            .map(|name| json!({"message": greet(&name)}))
            .map_err(|error| ("invalid_name", error.to_string())),
        _ => Err(("usage", "usage: starter-app greet <name>".to_owned())),
    };
    match result {
        Ok(value) => {
            println!("{value}");
            eprintln!("{}", json!({"event":"greet_completed", "run_id":run_id}));
            ExitCode::SUCCESS
        }
        Err((code, message)) => {
            println!("{}", json!({"error":{"code":code,"message":message}}));
            // Do not log the supplied name: diagnostics need no personal data.
            eprintln!(
                "{}",
                json!({"event":"request_rejected","code":code,"run_id":run_id})
            );
            ExitCode::from(2)
        }
    }
}

fn run_driver(args: &[String]) -> ExitCode {
    let result = (|| -> Result<bool, Box<dyn std::error::Error>> {
        let root = std::env::var_os("APP_DATA_DIR").map(std::path::PathBuf::from);
        if root.as_ref().is_some_and(|p| p.as_os_str().is_empty()) {
            return Err("APP_DATA_DIR must not be empty".into());
        }
        let mut engine = starter_engine::Driver::open(root.as_deref())
            .map_err(|e| format!("{}: {}", e.code, e.message))?;
        let mut writer = std::io::stdout().lock();
        if let Some(path) = args.get(1) {
            let file = std::fs::File::open(path)?;
            Ok(driver::serve(
                &mut std::io::BufReader::new(file),
                &mut writer,
                &mut engine,
            )?)
        } else {
            Ok(driver::serve(
                &mut std::io::stdin().lock(),
                &mut writer,
                &mut engine,
            )?)
        }
    })();
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(2),
        Err(error) => {
            println!(
                "{}",
                json!({"schema_version":1,"id":null,"ok":false,
                "error":{"code":"driver_io","message":error.to_string()}})
            );
            ExitCode::from(2)
        }
    }
}
