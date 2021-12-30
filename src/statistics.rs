/// Implements a simple Welch's t-test with the Welford method.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TTest {
    groups: [GroupValues; 2],
}

/// GroupValues holds the necessary values for each group sample set.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct GroupValues {
    mean: f64,
    m2: f64,
    number_samples: f64,
}

impl Default for GroupValues {
    fn default() -> Self {
        Self {
            mean: 0.0,
            m2: 0.0,
            number_samples: 0.0,
        }
    }
}

impl TTest {
    /// Create a new t-test with empty values.
    pub fn new() -> Self {
        Self {
            groups: [GroupValues::default(); 2],
        }
    }

    /// Adds a new value to one of the two sample groups (a or b).
    /// Set `is_sample_group_a` to true, if the value belongs to group a.
    pub fn push(&mut self, value: f64, is_sample_group_a: bool) {
        let index = if is_sample_group_a { 0 } else { 1 };
        let group = &mut self.groups[index];

        group.number_samples += 1.0;
        let delta = value - group.mean;
        group.mean += delta / group.number_samples;
        group.m2 += delta * (value - group.mean);

        //assert(class == 0 || class == 1);
        //ctx->n[class]++;
        /*
         estimate variance on the fly as per the Welford method.
         this gives good numerical stability, see Knuth's TAOCP vol 2
        */
        //double delta = x - ctx->mean[class];
        //ctx->mean[class] = ctx->mean[class] + delta / ctx->n[class];
        //ctx->m2[class] = ctx->m2[class] + delta * (x - ctx->mean[class]);
    }

    /// Returns the t value for the test.
    /// If there are no or only one sample available in one of the groups, `None` is returned instead.
    pub fn compute(&self) -> Option<f64> {
        let group_a = self.groups[0];
        let group_b = self.groups[1];

        if group_a.number_samples <= 1.0 || group_b.number_samples <= 1.0 {
            return None;
        }

        let var_a = group_a.m2 / (group_a.number_samples - 1.0);
        let var_b = group_b.m2 / (group_b.number_samples - 1.0);
        let num = group_a.mean - group_b.mean;
        let den = f64::sqrt(var_a / group_a.number_samples + var_b / group_b.number_samples);
        if den == 0.0 {
            None
        } else {
            Some(num / den)
        }

        //double var[2] = {0.0, 0.0};
        //var[0] = ctx->m2[0] / (ctx->n[0] - 1);
        //var[1] = ctx->m2[1] / (ctx->n[1] - 1);
        //double num = (ctx->mean[0] - ctx->mean[1]);
        //double den = sqrt(var[0] / ctx->n[0] + var[1] / ctx->n[1]);
        //double t_value = num / den;
        //return t_value;
    }

    /// Returns the number of samples for group a and b.
    pub fn get_number_of_samples(&self) -> [f64; 2] {
        [self.groups[0].number_samples, self.groups[1].number_samples]
    }

    /// Returns the mean for group a and b.
    pub fn get_mean(&self) -> [f64; 2] {
        [self.groups[0].mean, self.groups[1].mean]
    }
}
