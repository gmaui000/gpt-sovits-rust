pub mod base;
pub mod error;
pub mod tts;

use tts::engine::tts_engine::TTSEngine;
use base::record::QueryTracker;

// 定义全局状态
pub struct AppState {
    pub engine: TTSEngine,
    pub track: QueryTracker,
}

