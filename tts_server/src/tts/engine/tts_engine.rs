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

    #[test]
    fn test_synthesis1() {
        println!("test_synthesis1");
        let engine = TTSEngine::default();
        let wav = engine.synthesis("乘客朋友，您好，您现在即将体验和参观的是无人之境项目，无人之境示范体验区是国家智能网联汽车上海试点示范区的重要组成部分，可支撑无人化高级别自动驾驶技术测试验证。目前已实现无人驾驶小巴，robot taxi，无人清扫等多业态无人驾驶应用场景。同时也欢迎您乘坐体验酷哇科技无人驾驶小巴，我们具备完善的功能配置，可完成十余项自动驾驶场景展示。包括路径规划，智能避障，站点停泊，临时起停，自动返场，自主泊车等，360度全景智能交互。在感知，控制，底盘，供电等各个环节，执行冗余式安全策略，切实保障乘客安全，后续将以预约形式逐步开放给社会公众。本车由上海汽车博物馆站，开往一维诶爱智行港终点站，下一站，房车中国上海基地站，车辆离站，请系好安全带。");
        let mut writer = WavWriter::create(
            "tts1.wav",
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

    #[test]
    fn test_synthesis2() {
        println!("test_synthesis2");
        let engine = TTSEngine::default();
        let wav = engine.synthesis("robot taxi, 一维诶爱.");
        let mut writer = WavWriter::create(
            "tts2.wav",
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
