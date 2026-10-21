//! Bounded JSON-lines transport. No sockets or model dependencies.
use serde_json::json;
use starter_engine::{Driver, MAX_BYTES};
use std::io::{self, BufRead, Read, Write};

pub fn serve(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    driver: &mut Driver,
) -> io::Result<bool> {
    let mut ok = true;
    loop {
        let mut bytes = Vec::new();
        let n = (&mut *reader)
            .take((MAX_BYTES + 1) as u64)
            .read_until(b'\n', &mut bytes)?;
        if n == 0 {
            break;
        }
        // Cap allocation before parsing. Close on oversize rather than interpret the tail.
        let oversize = bytes.len() > MAX_BYTES;
        let reply = driver.handle(&bytes);
        let accepted = reply["ok"] == true;
        ok &= accepted;
        writeln!(writer, "{reply}")?;
        writer.flush()?;
        eprintln!(
            "{}",
            json!({"event":"driver_response","ok":accepted,
            "run_id":std::env::var("HARNESS_RUN_ID").unwrap_or_else(|_|"manual".into())})
        );
        if oversize {
            break;
        }
    }
    Ok(ok)
}
