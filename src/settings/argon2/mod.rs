use argon2::{Algorithm, Params, Version};
use serde::Deserialize;


#[derive(Deserialize, Clone)]
pub struct ArgonSettings {
    pub algorithm: ArgonAlgorithm,
    pub version: ArgonVersion,
    pub parameters: Option<ArgonParameterSettings>
}

impl ArgonSettings {
    /// Get Argon2 params from the parameter settings, or default when not specified
    pub fn parameters(&self) -> Params {
        if let Some(settings) = self.parameters.clone() {
            settings.into()
        } else {
            Params::default()
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArgonAlgorithm {
    Argon2d,
    Argon2i,
    Argon2id
}

impl Into<Algorithm> for ArgonAlgorithm {
    fn into(self) -> Algorithm {
        match self {
            Self::Argon2d => Algorithm::Argon2d,
            Self::Argon2i => Algorithm::Argon2i,
            Self::Argon2id => Algorithm::Argon2id
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ArgonVersion {
    V0x13,
    V0x10
}

impl Into<Version> for ArgonVersion {
    fn into(self) -> Version {
        match self {
            Self::V0x10 => Version::V0x10,
            Self::V0x13 => Version::V0x13
        }
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ArgonParameterSettings {
    pub memory: u32,
    pub time: u32,
    pub parallel: u32,
    pub output_len: Option<usize>
}

impl Into<Params> for ArgonParameterSettings {
    fn into(self) -> Params {
        Params::new(
            self.memory,
            self.time,
            self.parallel,
            self.output_len
        )
        .expect("Couldn't construct the Argon2 parameters.")
    }
}