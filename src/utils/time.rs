use chrono::{DateTime, Local, TimeZone, Utc};

pub fn unix_time_to_jst(unix_time_millis: u64) -> String {
    // ミリ秒を秒に変換して DateTime<Utc> を生成
    let unix_time_secs = (unix_time_millis / 1000) as i64;
    let datetime_utc = Utc.timestamp_opt(unix_time_secs, 0).single();

    match datetime_utc {
        // Utc から JST へ変換
        Some(datetime) => {
            let datetime_jst: DateTime<Local> = datetime.with_timezone(&Local);
            // フォーマットして返す
            datetime_jst.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        None => "Invalid Timestamp".to_string(), // 無効なタイムスタンプの場合
    }
}
