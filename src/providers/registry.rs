use super::types::Provider;
use super::wellknown::WellKnownProvider;
use std::sync::LazyLock;

static PROVIDERS: LazyLock<Vec<Provider>> = LazyLock::new(|| {
    vec![Provider::WellKnown(WellKnownProvider)]
});

pub fn find_provider(url: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.match_url(url).is_some())
}

#[allow(dead_code)]
pub fn get_providers() -> &'static [Provider] {
    &PROVIDERS
}
