use anyhow::{Context, Result};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

pub struct AudioUtils;

impl AudioUtils {
    /// Read a WAV file and decode it to a vector of 16-bit signed PCM samples
    pub fn decode_path_to_data(audio_path: &str, sr_to: u32) -> Result<Vec<i16>> {
        // Open the WAV file with hound
        let mut reader = WavReader::open(audio_path).context("Failed to open WAV file")?;
        let spec = reader.spec();

        // Ensure we are working with 16-bit signed PCM data
        if spec.sample_format != SampleFormat::Int || spec.bits_per_sample != 16 {
            println!(
                "Unsupported sample format or bit depth. Only 16-bit signed PCM is supported."
            );
        }

        // Read all the samples from the WAV file
        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();

        // Resample if needed
        if spec.sample_rate != sr_to {
            let resampled_data =
                Self::resample_pcm16(&samples, spec.sample_rate as f64, sr_to as f64)?;
            Ok(resampled_data)
        } else {
            Ok(samples)
        }
    }

    pub fn resample_pcm16(audio_data: &[i16], from_hz: f64, to_hz: f64) -> Result<Vec<i16>> {
        if from_hz == to_hz {
            return Ok(audio_data.to_vec());
        }

        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        // Create a rubato resampler with the required interpolation method
        let mut resampler =
            SincFixedIn::<f32>::new(to_hz / from_hz, 2.0, params, audio_data.len(), 1).unwrap();

        // Convert audio_data from i16 to f32
        let input_f32: Vec<f32> = audio_data
            .iter()
            .map(|&sample| (sample as f32) / (i16::MAX as f32))
            .collect();
        let input_frames: Vec<Vec<f32>> = vec![input_f32];

        // Resample the data
        let output_frames = resampler.process(&input_frames, None)?;

        // Convert the resampled data from f32 back to i16
        let resampled_data: Vec<i16> = output_frames
            .iter()
            .flat_map(|frame| {
                frame.iter().map(|&sample| {
                    // Clamp the value to avoid overflow/underflow
                    let clamped_sample = sample.clamp(-1.0, 1.0);
                    (clamped_sample * i16::MAX as f32) as i16
                })
            })
            .collect();

        Ok(resampled_data)
    }

    #[allow(dead_code)]
    /// Save the PCM data to a new WAV file
    pub fn decode_data_to_path(
        audio_data: &[i16],
        out_file_path: &str,
        sample_rate: u32,
        append: bool,
    ) -> Result<()> {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let path = std::path::Path::new(out_file_path);
        let mut writer: WavWriter<_>;
        if path.exists() && append {
            writer = WavWriter::append(out_file_path).context("Failed to append WAV file")?;
        } else {
            writer = WavWriter::create(out_file_path, spec).context("Failed to create WAV file")?;
        }

        // Write the audio samples to the output file
        for &sample in audio_data {
            writer
                .write_sample(sample)
                .context("Failed to write sample to WAV")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_and_resample() {
        let input_path = "../assets/tts_16_2.wav";
        let output_path = "../assets/tts_16_2_out.wav";
        let sr_to = 32000;

        let pcm_data = AudioUtils::decode_path_to_data(input_path, sr_to).unwrap();
        println!("Decoded PCM data: {} samples", pcm_data.len());

        AudioUtils::decode_data_to_path(&pcm_data, output_path, sr_to, true).unwrap();
        println!("Wrote output file: {}", output_path);
    }
}
