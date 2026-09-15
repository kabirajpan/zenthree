//! Native Drosophila (Fruit Fly) Biological Connectome Neural Decision Kernel
//!
//! Replicates the PyTorch ConnectomeRNN with strict biological constraints:
//! 1. Anatomical Connectome: Sparse recurrent connectivity matrix.
//! 2. Dale's Principle: Strict row-wise sign preservation (excitatory vs inhibitory).
//! 3. Continuous-time leaky membrane dynamics:
//!    tau * dr/dt = -r + ReLU(tanh(r @ W + external + bias))

use std::convert::TryInto;

/// Default embedded weights trained on 500 Central Complex neurons
pub const EMBEDDED_WEIGHTS: &[u8] = include_bytes!("../weights/fly_brain_500.bin");

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Synapse {
    pub pre: u16,
    pub post: u16,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct ConnectomeWeights {
    pub n_neurons: usize,
    pub input_idx: Vec<usize>,
    pub output_idx: Vec<usize>,
    pub bias: Vec<f32>,
    pub readout_weight: Vec<f32>, // 2 x len(output_idx)
    pub readout_bias: [f32; 2],
    pub synapses: Vec<Synapse>,
}

impl ConnectomeWeights {
    /// Deserializes weights from binary "FLYB" stream
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 16 {
            return Err("Binary payload too short".to_string());
        }

        if &bytes[0..4] != b"FLYB" {
            return Err("Invalid magic header: expected FLYB".to_string());
        }

        let mut offset = 4;
        let read_u32 = |offset: &mut usize| -> Result<u32, String> {
            if *offset + 4 > bytes.len() {
                return Err("Unexpected EOF while reading u32".to_string());
            }
            let val = u32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            Ok(val)
        };

        let read_f32 = |offset: &mut usize| -> Result<f32, String> {
            if *offset + 4 > bytes.len() {
                return Err("Unexpected EOF while reading f32".to_string());
            }
            let val = f32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            Ok(val)
        };

        let read_u16 = |offset: &mut usize| -> Result<u16, String> {
            if *offset + 2 > bytes.len() {
                return Err("Unexpected EOF while reading u16".to_string());
            }
            let val = u16::from_le_bytes(bytes[*offset..*offset + 2].try_into().unwrap());
            *offset += 2;
            Ok(val)
        };

        let version = read_u32(&mut offset)?;
        if version != 1 {
            return Err(format!("Unsupported format version: {}", version));
        }

        let n_neurons = read_u32(&mut offset)? as usize;
        let n_inputs = read_u32(&mut offset)? as usize;
        let n_outputs = read_u32(&mut offset)? as usize;

        let mut input_idx = Vec::with_capacity(n_inputs);
        for _ in 0..n_inputs {
            input_idx.push(read_u32(&mut offset)? as usize);
        }

        let mut output_idx = Vec::with_capacity(n_outputs);
        for _ in 0..n_outputs {
            output_idx.push(read_u32(&mut offset)? as usize);
        }

        let mut bias = Vec::with_capacity(n_neurons);
        for _ in 0..n_neurons {
            bias.push(read_f32(&mut offset)?);
        }

        let readout_len = 2 * n_outputs;
        let mut readout_weight = Vec::with_capacity(readout_len);
        for _ in 0..readout_len {
            readout_weight.push(read_f32(&mut offset)?);
        }

        let readout_bias = [read_f32(&mut offset)?, read_f32(&mut offset)?];

        let num_synapses = read_u32(&mut offset)? as usize;
        let mut synapses = Vec::with_capacity(num_synapses);
        for _ in 0..num_synapses {
            let pre = read_u16(&mut offset)?;
            let post = read_u16(&mut offset)?;
            let weight = read_f32(&mut offset)?;
            synapses.push(Synapse { pre, post, weight });
        }

        Ok(Self {
            n_neurons,
            input_idx,
            output_idx,
            bias,
            readout_weight,
            readout_bias,
            synapses,
        })
    }
}

/// In silico biological circuit ablation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AblationMode {
    /// Targeted hub silencing: Lesions internal neurons with highest synaptic degree
    TargetedHubs,
    /// Random lesion: Distributed random silencing across internal neurons
    Random,
    /// Silences GABAergic / inhibitory interneurons
    InhibitoryOnly,
    /// Silences Cholinergic / excitatory projection neurons
    ExcitatoryOnly,
}

impl AblationMode {
    pub fn label(&self) -> &'static str {
        match self {
            AblationMode::TargetedHubs => "Targeted Hubs",
            AblationMode::Random => "Random Lesion",
            AblationMode::InhibitoryOnly => "Inhibitory (GABA)",
            AblationMode::ExcitatoryOnly => "Excitatory (ACh)",
        }
    }
}

/// Output decision report containing neuro-deliberation metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DecisionResult {
    pub choice: usize, // 0 = Option A (Left), 1 = Option B (Right)
    pub choice_label: String,
    pub logits: [f32; 2],
    pub probabilities: [f32; 2],
    pub confidence: f32,
    pub reaction_time_dt: usize,
    pub final_firing_rates: Vec<f32>,
    pub ablated_neurons: usize,
}

/// Continuous-rate recurrent neural network constrained to biological connectome
#[derive(Debug, Clone)]
pub struct ConnectomeRNN {
    pub weights: ConnectomeWeights,
    pub tau: f32,
    pub dt: f32,
}

impl ConnectomeRNN {
    /// Loads the embedded 500-neuron Drosophila connectome brain
    pub fn load_default() -> Self {
        let weights = ConnectomeWeights::from_bytes(EMBEDDED_WEIGHTS)
            .expect("Embedded connectome weights must be valid");
        Self {
            weights,
            tau: 5.0,
            dt: 1.0,
        }
    }

    pub fn new(weights: ConnectomeWeights, tau: f32, dt: f32) -> Self {
        Self { weights, tau, dt }
    }

    /// Forward pass through the recurrent connectome over T time steps.
    ///
    /// * `input_stream`: Matrix of size [T][n_inputs], where each row represents input at step t.
    /// * `ablation_mask`: Optional slice of length `n_neurons` (1.0 = alive, 0.0 = silenced).
    pub fn forward(&self, input_stream: &[Vec<f32>], ablation_mask: Option<&[f32]>) -> DecisionResult {
        let t_steps = input_stream.len();
        let n = self.weights.n_neurons;
        let alpha = self.dt / self.tau;

        let mut r = vec![0.0f32; n];
        let mut trace = Vec::with_capacity(t_steps);

        for t in 0..t_steps {
            let mut external = vec![0.0f32; n];
            let step_input = &input_stream[t];
            for (ch, &idx) in self.weights.input_idx.iter().enumerate() {
                if ch < step_input.len() {
                    external[idx] = step_input[ch];
                }
            }

            // Recurrent synaptic drive: r @ W where W is [pre, post]
            // Equivalent to accumulating: rec_input[syn.post] += r[syn.pre] * syn.weight
            let mut rec_input = vec![0.0f32; n];
            for syn in &self.weights.synapses {
                rec_input[syn.post as usize] += r[syn.pre as usize] * syn.weight;
            }

            // Continuous leaky integration with biological non-linearity:
            // total = rec_input + external + bias
            // activation = ReLU(tanh(total))
            // r = (1 - alpha) * r + alpha * activation
            for i in 0..n {
                let total = rec_input[i] + external[i] + self.weights.bias[i];
                let activation = total.tanh().max(0.0);
                r[i] = (1.0 - alpha) * r[i] + alpha * activation;
            }

            // In silico lesion / silencing if ablation mask is provided
            if let Some(mask) = ablation_mask {
                for i in 0..n {
                    if i < mask.len() {
                        r[i] *= mask[i];
                    }
                }
            }

            trace.push(r.clone());
        }

        // Decision readout from output neurons at the final step
        let n_out = self.weights.output_idx.len();
        let mut logits = self.weights.readout_bias;

        for choice in 0..2 {
            let row_offset = choice * n_out;
            let mut sum = 0.0f32;
            for (k, &out_id) in self.weights.output_idx.iter().enumerate() {
                sum += self.weights.readout_weight[row_offset + k] * r[out_id];
            }
            logits[choice] += sum;
        }

        // Softmax
        let max_logit = logits[0].max(logits[1]);
        let exp0 = (logits[0] - max_logit).exp();
        let exp1 = (logits[1] - max_logit).exp();
        let sum_exp = exp0 + exp1;
        let probabilities = [exp0 / sum_exp, exp1 / sum_exp];

        let choice = if logits[0] >= logits[1] { 0 } else { 1 };
        let choice_label = if choice == 0 { "Option A (Left)" } else { "Option B (Right)" }.to_string();
        let confidence = probabilities[choice];

        // Biological reaction time: time step when readout activity first crossed threshold (0.22)
        let half_out = n_out / 2;
        let mut reaction_time = t_steps;
        for (t, step_r) in trace.iter().enumerate() {
            let mut sum_a = 0.0f32;
            for &id in &self.weights.output_idx[..half_out] {
                sum_a += step_r[id];
            }
            let acc_a = sum_a / (half_out as f32).max(1.0);

            let mut sum_b = 0.0f32;
            for &id in &self.weights.output_idx[half_out..] {
                sum_b += step_r[id];
            }
            let acc_b = sum_b / ((n_out - half_out) as f32).max(1.0);

            if acc_a > 0.22 || acc_b > 0.22 {
                reaction_time = t + 1;
                break;
            }
        }

        let ablated_neurons = if let Some(mask) = ablation_mask {
            mask.iter().filter(|&&m| m <= 0.0).count()
        } else {
            0
        };

        DecisionResult {
            choice,
            choice_label,
            logits,
            probabilities,
            confidence,
            reaction_time_dt: reaction_time,
            final_firing_rates: r,
            ablated_neurons,
        }
    }

    /// Generates an in silico biological ablation mask (1.0 = functional, 0.0 = silenced)
    /// based on circuit topology and chosen biological ablation mode.
    pub fn generate_ablation_mask(&self, mode: AblationMode, fraction: f32) -> Vec<f32> {
        let n = self.weights.n_neurons;
        let mut mask = vec![1.0f32; n];
        let frac = fraction.clamp(0.0, 1.0);
        if frac <= 0.0 {
            return mask;
        }

        let is_protected = |idx: usize| -> bool {
            self.weights.input_idx.contains(&idx) || self.weights.output_idx.contains(&idx)
        };

        let candidate_pool: Vec<usize> = (0..n).filter(|&i| !is_protected(i)).collect();
        if candidate_pool.is_empty() {
            return mask;
        }

        match mode {
            AblationMode::TargetedHubs => {
                let mut degrees = vec![0usize; n];
                for syn in &self.weights.synapses {
                    degrees[syn.pre as usize] += 1;
                    degrees[syn.post as usize] += 1;
                }
                let mut sorted = candidate_pool;
                sorted.sort_by(|&a, &b| degrees[b].cmp(&degrees[a]));
                let k = ((frac * sorted.len() as f32).round() as usize).min(sorted.len());
                for &id in &sorted[..k] {
                    mask[id] = 0.0;
                }
            }
            AblationMode::Random => {
                let mut shuffled = candidate_pool;
                let mut seed: u64 = 777 + (frac * 1000.0) as u64;
                for i in (1..shuffled.len()).rev() {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let j = (seed as usize) % (i + 1);
                    shuffled.swap(i, j);
                }
                let k = ((frac * shuffled.len() as f32).round() as usize).min(shuffled.len());
                for &id in &shuffled[..k] {
                    mask[id] = 0.0;
                }
            }
            AblationMode::InhibitoryOnly => {
                let mut is_inhib = vec![false; n];
                for syn in &self.weights.synapses {
                    if syn.weight < 0.0 {
                        is_inhib[syn.pre as usize] = true;
                    }
                }
                let mut inhib_pool: Vec<usize> = candidate_pool.into_iter().filter(|&id| is_inhib[id]).collect();
                let mut seed: u64 = 888 + (frac * 1000.0) as u64;
                for i in (1..inhib_pool.len()).rev() {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let j = (seed as usize) % (i + 1);
                    inhib_pool.swap(i, j);
                }
                let k = ((frac * inhib_pool.len() as f32).round() as usize).min(inhib_pool.len());
                for &id in &inhib_pool[..k] {
                    mask[id] = 0.0;
                }
            }
            AblationMode::ExcitatoryOnly => {
                let mut is_excit = vec![false; n];
                for syn in &self.weights.synapses {
                    if syn.weight > 0.0 {
                        is_excit[syn.pre as usize] = true;
                    }
                }
                let mut excit_pool: Vec<usize> = candidate_pool.into_iter().filter(|&id| is_excit[id]).collect();
                let mut seed: u64 = 999 + (frac * 1000.0) as u64;
                for i in (1..excit_pool.len()).rev() {
                    seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    let j = (seed as usize) % (i + 1);
                    excit_pool.swap(i, j);
                }
                let k = ((frac * excit_pool.len() as f32).round() as usize).min(excit_pool.len());
                for &id in &excit_pool[..k] {
                    mask[id] = 0.0;
                }
            }
        }

        mask
    }

    /// Evaluates evidence between two competing candidates (Option A vs Option B)
    /// across T deliberation steps with an optional in silico biological ablation mask.
    pub fn deliberate_with_mask(
        &self,
        evidence_a: f32,
        evidence_b: f32,
        t_steps: usize,
        ablation_mask: Option<&[f32]>,
    ) -> DecisionResult {
        let n_in = self.weights.input_idx.len();
        let half_in = n_in / 2;

        let mut stream = Vec::with_capacity(t_steps);
        for _ in 0..t_steps {
            let mut step = vec![0.0f32; n_in];
            for i in 0..half_in {
                step[i] = evidence_a;
            }
            for i in half_in..n_in {
                step[i] = evidence_b;
            }
            stream.push(step);
        }

        self.forward(&stream, ablation_mask)
    }

    /// Evaluates evidence between two competing candidates (Option A vs Option B)
    /// across T deliberation steps with an intact, unablated biological connectome.
    pub fn deliberate(&self, evidence_a: f32, evidence_b: f32, t_steps: usize) -> DecisionResult {
        self.deliberate_with_mask(evidence_a, evidence_b, t_steps, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_weights_deserialization() {
        let rnn = ConnectomeRNN::load_default();
        assert_eq!(rnn.weights.n_neurons, 500);
        assert_eq!(rnn.weights.input_idx.len(), 20);
        assert_eq!(rnn.weights.output_idx.len(), 20);
        assert_eq!(rnn.weights.bias.len(), 500);
        assert_eq!(rnn.weights.readout_weight.len(), 40);
        assert_eq!(rnn.weights.synapses.len(), 5399);
    }

    #[test]
    fn test_pytorch_numerical_parity() {
        let rnn = ConnectomeRNN::load_default();

        // Exact test vector used in PyTorch validation:
        // 30 time steps, 20 input channels, first 10 channels = 0.8, last 10 channels = 0.2
        let mut input_stream = Vec::with_capacity(30);
        for _ in 0..30 {
            let mut step = vec![0.2f32; 20];
            for i in 0..10 {
                step[i] = 0.8;
            }
            input_stream.push(step);
        }

        let res = rnn.forward(&input_stream, None);

        // Expected ground truth from PyTorch model(x):
        // Logits: [0.22891545, -0.16634345]
        // Final r[:5]: [0.7008897, 0.7028472, 0.6632055, 0.66426176, 0.6607892]
        let expected_logits = [0.22891545f32, -0.16634345f32];
        assert!((res.logits[0] - expected_logits[0]).abs() < 1e-4, "Logit 0 diff: {}", (res.logits[0] - expected_logits[0]).abs());
        assert!((res.logits[1] - expected_logits[1]).abs() < 1e-4, "Logit 1 diff: {}", (res.logits[1] - expected_logits[1]).abs());
        assert_eq!(res.choice, 0, "Choice must be Option A (Left)");

        let expected_r5 = [0.7008897f32, 0.7028472f32, 0.6632055f32, 0.66426176f32, 0.6607892f32];
        for i in 0..5 {
            let diff = (res.final_firing_rates[i] - expected_r5[i]).abs();
            assert!(diff < 1e-4, "Neuron {} firing rate diff: {}", i, diff);
        }
    }

    #[test]
    fn test_deliberate_decision_options() {
        let rnn = ConnectomeRNN::load_default();
        
        // Option A wins
        let res_a = rnn.deliberate(0.85, 0.15, 30);
        assert_eq!(res_a.choice, 0);
        assert!(res_a.confidence > 0.55);

        // Option B wins
        let res_b = rnn.deliberate(0.15, 0.85, 30);
        assert_eq!(res_b.choice, 1);
        assert!(res_b.confidence > 0.55);
    }

    #[test]
    fn test_ablation_mask_and_lesion() {
        let rnn = ConnectomeRNN::load_default();

        // 20% targeted hub ablation
        let mask = rnn.generate_ablation_mask(AblationMode::TargetedHubs, 0.20);
        assert_eq!(mask.len(), 500);
        let silenced = mask.iter().filter(|&&m| m == 0.0).count();
        assert!(silenced >= 80 && silenced <= 100, "Silenced count: {}", silenced);

        // Run deliberation with ablation mask
        let res_ablated = rnn.deliberate_with_mask(0.85, 0.15, 30, Some(&mask));
        assert_eq!(res_ablated.ablated_neurons, silenced);
    }
}
