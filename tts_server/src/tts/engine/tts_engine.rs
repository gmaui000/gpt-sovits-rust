use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use sovits::bert_utils::ChBertUtils;

#[derive(Default)]
pub struct TTSEngine {
    engine: ChBertUtils,
}

impl TTSEngine {
    pub fn synthesis(&self, text: &str) -> Vec<i16> {
        // 32K 16bit 1channel
        let audio = self.engine.infer(text);
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        let mut resampler =
            SincFixedIn::<f32>::new(24000_f64 / 32000_f64, 2.0, params, audio.len(), 1).unwrap();
        let converted_data: Vec<Vec<f32>> = vec![audio
            .iter()
            .map(|&x| (x as f32 / i16::MAX as f32))
            .collect::<Vec<f32>>()];
        let res_audio = resampler.process(&converted_data, None).unwrap();
        res_audio
            .into_iter()
            .flat_map(|x| x.into_iter().map(|a| (a * i16::MAX as f32) as i16))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{SampleFormat, WavSpec, WavWriter};

    #[test]
    fn test_synthesis() {
        println!("test_synthesis");
        let engine = TTSEngine::default();
        let wav = engine.synthesis("今天天气不错,有50%的概率会下雨！");
        let mut writer = WavWriter::create(
            "tts.wav",
            WavSpec {
                channels: 1,
                sample_rate: 24000,
                bits_per_sample: 16,
                sample_format: SampleFormat::Int,
            },
        )
        .expect("Failed to write sample to WAV.");
        for &sample in wav.iter() {
            writer
                .write_sample(sample)
                .expect("Failed to write sample to WAV.");
        }
    }
}
