#[derive(Debug, Clone, Copy)]
pub struct TotalCmpF32(pub f32);

impl From<f32> for TotalCmpF32 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl core::ops::Deref for TotalCmpF32 {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for TotalCmpF32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialOrd for TotalCmpF32 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalCmpF32 {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialEq for TotalCmpF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for TotalCmpF32 {}

impl core::fmt::Display for TotalCmpF32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
