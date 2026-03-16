use std::collections::HashMap;

/// Data classification. [`Class::Yes`] and [`Class::No`] represent whether
/// tennis will or won't be played respectively.
#[derive(Eq, PartialEq, Hash)]
enum Class {
    Yes,
    No,
}

#[derive(Eq, PartialEq, Hash)]
enum Outlook {
    Sunny,
    Overcast,
    Rain,
}

#[derive(Eq, PartialEq, Hash)]
enum Temperature {
    Hot,
    Mild,
    Cold,
}

#[derive(Eq, PartialEq, Hash)]
enum Humidity {
    High,
    Normal,
}

#[derive(Eq, PartialEq, Hash)]
enum Wind {
    Strong,
    Weak,
}

#[derive(Eq, PartialEq, Hash)]
struct Feature {
    outlook: Outlook,
    temperature: Temperature,
    humidity: Humidity,
    wind: Wind,
}

impl Feature {
    fn new(outlook: Outlook, temperature: Temperature, humidity: Humidity, wind: Wind) -> Self {
        Self {
            outlook,
            temperature,
            humidity,
            wind,
        }
    }
}

#[derive(Default)]
struct NaiveBayes {
    /// Occurrences of [`Class`] within dataset.
    class_counts: HashMap<Class, usize>,
    /// Occurrences of [`Feature`] within dataset.
    feature_counts: HashMap<Feature, usize>,
    /// Training samples.
    sample_size: usize,
}

impl NaiveBayes {
    pub fn train(&mut self, class: Class, feature: Feature) {
        self.sample_size += 1;

        *self.class_counts.entry(class).or_default() += 1;
        *self.feature_counts.entry(feature).or_default() += 1;
    }

    fn prior(&self, class: Class) -> f64 {
        self.class_counts
            .get(&class)
            .map(usize::to_owned)
            .unwrap_or_default() as f64
            / self.sample_size as f64
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

    use super::*;

    static MODEL: OnceLock<NaiveBayes> = OnceLock::new();

    fn initialise_model() -> NaiveBayes {
        let mut model = NaiveBayes::default();

        let datasets = vec![
            (
                Class::No,
                Feature::new(Outlook::Sunny, Temperature::Hot, Humidity::High, Wind::Weak),
            ),
            (
                Class::No,
                Feature::new(
                    Outlook::Sunny,
                    Temperature::Hot,
                    Humidity::High,
                    Wind::Strong,
                ),
            ),
            (
                Class::Yes,
                Feature::new(
                    Outlook::Overcast,
                    Temperature::Hot,
                    Humidity::High,
                    Wind::Weak,
                ),
            ),
            (
                Class::Yes,
                Feature::new(Outlook::Rain, Temperature::Mild, Humidity::High, Wind::Weak),
            ),
        ];

        for pair in datasets {
            model.train(pair.0, pair.1)
        }

        model
    }

    #[test]
    fn nb_no_for_sunny_mild_high_strong() {
        let model = MODEL.get_or_init(initialise_model);

        let feature = Feature::new(
            Outlook::Sunny,
            Temperature::Mild,
            Humidity::High,
            Wind::Strong,
        );
    }
}
