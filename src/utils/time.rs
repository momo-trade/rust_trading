use chrono::{DateTime, Local, TimeZone, Utc};

pub fn unix_time_to_jst(unix_time_millis: u64) -> String {
    // Convert milliseconds to seconds and generate DateTime<Utc>
    let unix_time_secs = (unix_time_millis / 1000) as i64;
    let datetime_utc = Utc.timestamp_opt(unix_time_secs, 0).single();

    match datetime_utc {
        // Convert from Utc to JST
        Some(datetime) => {
            let datetime_jst: DateTime<Local> = datetime.with_timezone(&Local);
            // Format the result as a string
            datetime_jst.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        None => "Invalid Timestamp".to_string(), // Handle invalid timestamps
    }
}

pub fn calculate_time_range(hours_back: u64) -> (u64, u64) {
    let now = Utc::now();
    let end_time = now.timestamp_millis() as u64;
    let start_time = end_time - hours_back * 60 * 60 * 1000;
    (start_time, end_time)
}
