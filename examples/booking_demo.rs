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

    println!("==> Creating bookings table...");
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS bookings (\n\
         room INT NOT NULL,\n\
         booked DATERANGE NOT NULL,\n\
         title VARCHAR(255),\n\
         PRIMARY KEY (room, booked)\n\
         )",
    )?;

    println!("==> Seeding sample bookings...");
    conn.query_drop(
        "INSERT IGNORE INTO bookings (room, booked, title) VALUES\n\
         (1, DATERANGE_MAKE('2026-01-10', '2026-01-15', '[)'), 'Sprint planning'),\n\
         (1, DATERANGE_MAKE('2026-01-16', '2026-01-20', '[)'), 'Architecture review'),\n\
         (1, DATERANGE_MAKE('2026-01-25', '2026-01-27', '[)'), 'Team retro')",
    )?;

    println!("\n==> Checking availability for 2026-01-12 to 2026-01-14 (should conflict with booking 1)");
    let conflict: Vec<(i32, Option<String>, i32)> = conn.query(
        "SELECT room, title, DATERANGE_OVERLAPS(booked, DATERANGE_MAKE('2026-01-12', '2026-01-14', '[)')) AS overlaps FROM bookings WHERE room = 1",
    )?;
    for (room, title, overlaps) in &conflict {
        println!("  room={} title='{:?}' overlaps={}", room, title, overlaps);
    }

    println!("\n==> Finding free slots in room 1 between 2026-01-01 and 2026-01-31 ...");
    let bookings: Vec<(String, String)> = conn.query(
        "SELECT DATERANGE_LOWER(booked) AS start_date, DATERANGE_UPPER(booked) AS end_date FROM bookings WHERE room = 1 ORDER BY start_date",
    )?;
    let window_start = "2026-01-01".to_string();
    let window_end = "2026-01-31".to_string();
    let mut free_start = window_start.clone();
    for (start_date, end_date) in &bookings {
        if start_date >= &free_start {
            println!(
                "  room=1 free_window=[{}, {})",
                free_start, start_date
            );
        }
        if end_date > &free_start {
            free_start = end_date.clone();
        }
    }
    if free_start < window_end {
        println!(
            "  room=1 free_window=[{}, {})",
            free_start, window_end
        );
    }

    println!("\n==> Verifying a proposed booking fits inside the room's availability...");
    let fits = conn.query_first::<bool, _>(
        "SELECT NOT EXISTS (\n\
         SELECT 1 FROM bookings\n\
         WHERE room = 1\n\
         AND DATERANGE_OVERLAPS(\n\
             booked,\n\
             DATERANGE_MAKE('2026-01-05', '2026-01-07', '[)')\n\
         )\n\
         )",
    )?;
    println!(
        "  Proposed booking [2026-01-05, 2026-01-07) fits: {}",
        fits.unwrap_or(false)
    );

    println!("\nDone.");
    Ok(())
}
