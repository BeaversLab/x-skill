use crate::types::{ProviderMatch, RemoteSkill};

pub enum Provider {
    WellKnown(super::wellknown::WellKnownProvider),
}

impl Provider {
    pub fn id(&self) -> &str {
        match self {
            Self::WellKnown(p) => p.id(),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::WellKnown(p) => p.display_name(),
        }
    }

    pub fn match_url(&self, url: &str) -> Option<ProviderMatch> {
        match self {
            Self::WellKnown(p) => p.match_url(url),
        }
    }

    #[allow(dead_code)]
    pub fn to_raw_url(&self, url: &str) -> String {
        match self {
            Self::WellKnown(p) => p.to_raw_url(url),
        }
    }

    pub fn source_identifier(&self, url: &str) -> String {
        match self {
            Self::WellKnown(p) => p.source_identifier(url),
        }
    }

    #[allow(dead_code)]
    pub async fn fetch_skill(&self, url: &str) -> anyhow::Result<Option<RemoteSkill>> {
        match self {
            Self::WellKnown(p) => p.fetch_skill(url).await,
        }
    }

    pub async fn fetch_all_skills(&self, url: &str) -> anyhow::Result<Vec<RemoteSkill>> {
        match self {
            Self::WellKnown(p) => p.fetch_all_skills(url).await,
        }
    }
}
