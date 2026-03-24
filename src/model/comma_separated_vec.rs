use serde::Deserialize;

#[derive(Debug)]
pub struct CommaSeparatedVec(
    Vec<String>
);

impl CommaSeparatedVec {
    pub fn parse(s: String) -> Self {
        let vec = s
            .split(',')
            .map(|item| item.to_owned())
            .collect();
        Self(vec)
    }
}

impl AsRef<Vec<String>> for CommaSeparatedVec {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

impl<'de> Deserialize<'de> for CommaSeparatedVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let str_sequence = String::deserialize(deserializer)?;
        let parsed = Self::parse(str_sequence);
        Ok(parsed)
    }
}