use mysql::prelude::Queryable;
use std::env;

fn main() -> mysql::Result<()> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("Error: DATABASE_URL is not set.");
        eprintln!("Example: DATABASE_URL='mysql://root@127.0.0.1:3307/test_ranger'");
        std::process::exit(1);
    });

    let pool = mysql::Pool::new(database_url.as_str())?;
    let mut conn = pool.get_conn()?;

    println!("==> Creating availability windows table...");
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS availability_windows (\n\
         room INT NOT NULL,\n\
         windows DATEMULTIRANGE NOT NULL,\n\
         source VARCHAR(255),\n\
         PRIMARY KEY (room, source)\n\
         )",
    )?;

    println!("==> Seeding sample windows...");
    conn.query_drop(
        "INSERT IGNORE INTO availability_windows (room, windows, source) VALUES\n\
         (1, DATEMULTIRANGE_MAKE(\n\
              '[2026-01-05, 2026-01-10), [2026-01-15, 2026-01-20), [2026-01-25, 2026-01-27)'\n\
          ), 'team'),\n\
         (1, DATEMULTIRANGE_MAKE('[2026-01-12, 2026-01-14)'), 'solo'),\n\
         (2, DATEMULTIRANGE_MAKE('[2026-01-09, 2026-01-12)'), 'team')",
    )?;

    println!("\n==> Checking room 1 overlap with a proposed booking [2026-01-12, 2026-01-14)...");
    let overlaps: Vec<(i32, Option<String>, i32)> = conn.query(
        "SELECT room, source, DATEMULTIRANGE_OVERLAPS(windows, DATEMULTIRANGE_MAKE('[2026-01-12, 2026-01-14)')) AS overlaps\n\
         FROM availability_windows WHERE room = 1",
    )?;
    for (room, source, overlaps) in &overlaps {
        println!(
            "  room={} source='{:?}' overlaps={}",
            room, source, overlaps
        );
    }

    println!("\n==> Merging team + solo windows into one schedule...");
    let merged: Vec<(String,)> = conn.query(
        "SELECT DATEMULTIRANGE_DECODE(\n\
             DATEMULTIRANGE_MERGE(windows, DATEMULTIRANGE_MAKE('[2026-01-12, 2026-01-14)'))\n\
         ) AS merged\n\
         FROM availability_windows\n\
         WHERE room = 1 AND source = 'team'",
    )?;
    for (merged,) in &merged {
        println!("  merged={}", merged);
    }

    println!("\n==> Removing booked time from the team's windows...");
    let difference: Vec<(String,)> = conn.query(
        "SELECT DATEMULTIRANGE_DECODE(\n\
             DATEMULTIRANGE_DIFFERENCE(\n\
                 windows,\n\
                 DATEMULTIRANGE_MAKE('[2026-01-16, 2026-01-18)')\n\
             )\n\
         ) AS remaining\n\
         FROM availability_windows\n\
         WHERE room = 1 AND source = 'team'",
    )?;
    for (remaining,) in &difference {
        println!("  remaining={}", remaining);
    }

    println!("\n==> Intersecting room 1 and room 2 schedules...");
    let intersection: Vec<(String,)> = conn.query(
        "SELECT DATEMULTIRANGE_DECODE(\n\
             DATEMULTIRANGE_INTERSECT(a.windows, b.windows)\n\
         ) AS intersection\n\
         FROM availability_windows AS a\n\
         JOIN availability_windows AS b ON b.room = 2 AND b.source = 'team'\n\
         WHERE a.room = 1 AND a.source = 'team'",
    )?;
    for (intersection,) in &intersection {
        println!("  intersection={}", intersection);
    }

    println!("\nDone.");
    Ok(())
}
