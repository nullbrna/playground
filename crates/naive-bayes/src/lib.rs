const NUM_CLASSES: usize = 2;
const NUM_FEATURES: usize = 4;

/// Discrete values each feature can hold.
/// Outlook     [0] = sunny  (0) overcast (1) rain (2)
/// Temperature [1] = hot    (0) mild     (1) cool (2)
/// Humidity    [2] = high   (0) low      (1)
/// Wind        [3] = strong (0) weak     (1)
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
        // Assume `class` is either 0 or 1. Increment the corresponding class's
        // and the overall sample count.
        self.class_occurrences[class] += 1;
        self.sample_size += 1;

        // Assume each entry is <= `MAX_VALUE_COUNT` - 1. Increment the given
        // value for the feature under the given class.
        for feature in 0..NUM_FEATURES {
            let value = features[feature];
            self.counts[class][feature][value] += 1;
        }
    }

    fn predict(&self, features: [usize; NUM_FEATURES]) -> usize {
        let mut best_class = 0;
        let mut best_log_probability = f64::NEG_INFINITY;

        for class in 0..NUM_CLASSES {
            // Base likelihood of this class before seeing any features.
            let prior = self.class_occurrences[class] as f64 / self.sample_size as f64;
            // Avoid multiplying probabilities due to underflowing, instead sum
            // the log of each value.
            let mut log_probability = prior.ln();

            for feature in 0..NUM_FEATURES {
                let value = features[feature];
                // Laplace smoothing. If we come across a value for a feature
                // that we haven't yet seen for this class, incrementing avoids
                // a 0 ensuring an "unlikely but possible" prediction.
                let count = self.counts[class][feature][value] as f64 + 1.0;

                // Add the number of possible values per-feature to the class
                // occurrences to account for the smoothing above.
                let total = self.class_occurrences[class] as f64 + FEATURE_SET[feature] as f64;

                log_probability += (count / total).ln();
            }

            // Compare against the previous class's probabilities. Store the
            // class weighed with the highest probability from `features`.
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

    use super::*;

    static MODEL: OnceLock<NaiveBayes> = OnceLock::new();

    fn prepare_model() -> NaiveBayes {
        let mut model = NaiveBayes::default();

        // Dataset for weather conditions determining whether or not a game of
        // tennis is played. See the key commented above `FEATURE_SET` for more.
        let dataset = [
            (0, [0, 0, 0, 0]),
            (0, [0, 0, 0, 1]),
            (1, [1, 0, 0, 0]),
            (1, [2, 1, 0, 0]),
            (1, [2, 2, 1, 0]),
            (0, [2, 2, 1, 1]),
            (1, [1, 2, 1, 1]),
            (0, [0, 1, 0, 0]),
            (1, [0, 2, 1, 0]),
            (1, [2, 1, 1, 0]),
            (1, [0, 1, 1, 1]),
            (1, [1, 1, 0, 1]),
            (1, [1, 0, 1, 0]),
            (0, [2, 1, 0, 1]),
        ];

        for (class, features) in dataset {
            model.train(features, class);
        }

        model
    }

    #[test]
    fn nb_yes_for_overcast_cool_low_weak() {
        let model = MODEL.get_or_init(prepare_model);

        // NOTE: Overcast is always in the "Yes" class.
        assert_eq!(model.predict([1, 2, 1, 1]), 1);
    }

    #[test]
    fn nb_no_for_rain_mild_high_weak() {
        let model = MODEL.get_or_init(prepare_model);

        // NOTE: The model could technically predict a "Yes", although this
        // exact feature list and class is within the dataset. An aggregated
        // score from the other sets could outweigh this result.
        assert_eq!(model.predict([2, 1, 0, 1]), 0);
    }
}
