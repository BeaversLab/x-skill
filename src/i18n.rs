use std::collections::HashMap;
use std::sync::OnceLock;

static EN: OnceLock<HashMap<String, String>> = OnceLock::new();
static ZH: OnceLock<HashMap<String, String>> = OnceLock::new();
static LANG: OnceLock<String> = OnceLock::new();

const EN_YAML: &str = include_str!("../locales/en.yaml");
const ZH_YAML: &str = include_str!("../locales/zh.yaml");

fn en_map() -> &'static HashMap<String, String> {
    EN.get_or_init(|| serde_yml::from_str(EN_YAML).unwrap_or_default())
}

fn zh_map() -> &'static HashMap<String, String> {
    ZH.get_or_init(|| serde_yml::from_str(ZH_YAML).unwrap_or_default())
}

pub fn lang() -> &'static str {
    LANG.get_or_init(|| std::env::var("LANGUAGE").unwrap_or_else(|_| "en".into()))
}

pub fn is_zh() -> bool {
    lang() == "zh"
}

/// Look up a translation key, falling back to English then the raw key.
pub fn get(key: &str) -> &'static str {
    let map = if is_zh() { zh_map() } else { en_map() };
    if let Some(val) = map.get(key) {
        return val.as_str();
    }
    // Fallback to English if the key is missing in current language
    if let Some(val) = en_map().get(key) {
        return val.as_str();
    }
    // Final fallback: return the key itself (leak to get 'static lifetime)
    Box::leak(key.to_string().into_boxed_str())
}

/// Look up a translation key and return an owned String.
///
/// - `t!("key")` — simple lookup
/// - `t!("key", "name" => value, "count" => 5)` — lookup with `{name}` placeholder replacement
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::get($key).to_string()
    };
    ($key:expr, $($name:expr => $val:expr),+ $(,)?) => {{
        let mut __s = $crate::i18n::get($key).to_string();
        $(
            __s = __s.replace(
                &format!("{{{}}}", $name),
                &format!("{}", $val),
            );
        )+
        __s
    }};
}
