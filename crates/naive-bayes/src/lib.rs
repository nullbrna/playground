const NUM_CLASSES: usize = 2;
const NUM_FEATURES: usize = 4;
/// Discrete values each feature can hold.
const FEATURE_SET: [usize; NUM_FEATURES] = [3, 3, 2, 2];
/// Maximum number of values any feature can take.
/// NOTE: Specified for a fixed-size array.
const MAX_VALUE_COUNT: usize = 3;

#[derive(Default)]
struct NaiveBayes {
    /// Count of samples per-class (0 = No, 1 = Yes).
    class_occurrences: [usize; NUM_CLASSES],
    /// Amount of a given value for a feature, per-class.
    counts: [[[usize; MAX_VALUE_COUNT]; NUM_FEATURES]; NUM_CLASSES],
    /// Total training samples.
    sample_size: usize,
}

#[allow(unused)]
impl NaiveBayes {
    fn train(&mut self, features: [usize; NUM_FEATURES], class: usize) {
        // Increment the corresponding class's and the overall sample count.
        // NOTE: Assume the class is either: 0 or 1.
        self.class_occurrences[class] += 1;
        self.sample_size += 1;

        // Increment the given value for the feature under the given class.
        // NOTE: Assume each entry is less or equal: max value count - 1.
        for feature in 0..NUM_FEATURES {
            let value = features[feature];
            self.counts[class][feature][value] += 1;
        }
    }

    fn predict(&self, features: [usize; NUM_FEATURES]) -> usize {
        let mut best_class = 0;
        let mut best_log_probability = f64::NEG_INFINITY;

        for class in 0..NUM_CLASSES {
            let base_likelihood = self.class_occurrences[class] as f64 / self.sample_size as f64;
            // NOTE: Avoid multiplying probabilities due to underflowing,
            // instead sum the log of each value.
            let mut log_probability = base_likelihood.ln();

            for feature in 0..NUM_FEATURES {
                let value = features[feature];
                // Laplace smoothing. If we come across a value for a feature
                // that we haven't yet seen for this class, incrementing avoids
                // a 0 ensuring an unlikely but possible prediction.
                let count = self.counts[class][feature][value] as f64 + 1.0;
                let total = self.class_occurrences[class] as f64 + FEATURE_SET[feature] as f64;

                log_probability += (count / total).ln();
            }

            if log_probability > best_log_probability {
                best_log_probability = log_probability;
                best_class = class;
            }
        }

        best_class
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use crate::NaiveBayes;

    static MODEL: OnceLock<NaiveBayes> = OnceLock::new();

    fn prepare_model() -> NaiveBayes {
        let mut model = NaiveBayes::default();

        let dataset = [
            ([0, 0, 0, 0], 0),
            ([0, 0, 0, 1], 0),
            ([1, 0, 0, 0], 1),
            ([2, 1, 0, 0], 1),
            ([2, 2, 1, 0], 1),
            ([2, 2, 1, 1], 0),
            ([1, 2, 1, 1], 1),
            ([0, 1, 0, 0], 0),
            ([0, 2, 1, 0], 1),
            ([2, 1, 1, 0], 1),
            ([0, 1, 1, 1], 1),
            ([1, 1, 0, 1], 1),
            ([1, 0, 1, 0], 1),
            ([2, 1, 0, 1], 0),
        ];

        for (features, class) in dataset {
            model.train(features, class);
        }

        model
    }

    #[test]
    fn should_return_yes_class_for_positive_aggregate() {
        let model = MODEL.get_or_init(prepare_model);

        assert_eq!(model.predict([1, 2, 1, 1]), 1);
    }

    #[test]
    fn should_return_no_class_for_negative_aggregate() {
        let model = MODEL.get_or_init(prepare_model);

        assert_eq!(model.predict([2, 1, 0, 1]), 0);
    }
}
