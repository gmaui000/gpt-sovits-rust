use super::super::super::AppState;
use actix_web::{web, HttpResponse};
use chrono::Local;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tracing::{self, info};

#[derive(serde::Deserialize)]
pub struct TTSQuery {
    text: String,
}

#[actix_web::get("/api/tts")]
pub async fn api_tts(
    data: web::Data<Arc<Mutex<AppState>>>,
    query: web::Query<TTSQuery>,
) -> HttpResponse {
    let start_time = Local::now();
    let text = &query.text;
    let mut cursor = Cursor::new(Vec::new());
    let wav = data.get_ref().lock().unwrap().engine.synthesis(text);
    let mut writer = hound::WavWriter::new(
        &mut cursor,
        hound::WavSpec {
            channels: 1,
            sample_rate: 24000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .expect("Failed to write sample to WAV.");
    for &sample in wav.iter() {
        writer
            .write_sample(sample)
            .expect("Failed to write sample to WAV.");
    }
    writer.finalize().unwrap();

    let duration = Local::now().signed_duration_since(start_time);
    data.get_ref().lock().unwrap().track.record_query(
        text.to_owned(),
        start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        std::time::Duration::from_millis(duration.num_milliseconds() as u64),
    );
    info!(
        "req: {:?} cost: {:.2}s",
        text,
        duration.num_milliseconds() as f64 / 1000.0
    );

    HttpResponse::Ok()
        .content_type("audio/wav")
        .body(cursor.into_inner())
}
