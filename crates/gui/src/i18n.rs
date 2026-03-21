use std::sync::OnceLock;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use fluent_syntax::parser;
use unic_langid::LanguageIdentifier;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Language {
    #[default]
    English,
    SimplifiedChinese,
}

impl Language {
    fn id(self) -> LanguageIdentifier {
        match self {
            Self::English => "en-US".parse().expect("invalid en-US lang id"),
            Self::SimplifiedChinese => "zh-CN".parse().expect("invalid zh-CN lang id"),
        }
    }
}

pub(crate) struct Translator {
    language: Language,
}

impl Translator {
    pub(crate) fn new(language: Language) -> Self {
        Self { language }
    }

    pub(crate) fn t(&self, key: &'static str) -> String {
        translate(self.language, key, None)
    }

    pub(crate) fn t_count(&self, key: &'static str, count: usize) -> String {
        let mut args = FluentArgs::new();
        args.set("count", count as i64);
        translate(self.language, key, Some(&args))
    }
}

static EN_US_RESOURCE: OnceLock<FluentResource> = OnceLock::new();
static ZH_CN_RESOURCE: OnceLock<FluentResource> = OnceLock::new();

fn en_us_resource() -> &'static FluentResource {
    EN_US_RESOURCE.get_or_init(|| {
        let source = include_str!("../i18n/en-US.ftl");
        parser::parse_runtime(source).expect("invalid en-US.ftl syntax");
        FluentResource::try_new(source.to_owned()).expect("failed to parse en-US.ftl")
    })
}

fn zh_cn_resource() -> &'static FluentResource {
    ZH_CN_RESOURCE.get_or_init(|| {
        let source = include_str!("../i18n/zh-CN.ftl");
        parser::parse_runtime(source).expect("invalid zh-CN.ftl syntax");
        FluentResource::try_new(source.to_owned()).expect("failed to parse zh-CN.ftl")
    })
}

fn resource_for(language: Language) -> &'static FluentResource {
    match language {
        Language::English => en_us_resource(),
        Language::SimplifiedChinese => zh_cn_resource(),
    }
}

fn translate(language: Language, key: &'static str, args: Option<&FluentArgs<'_>>) -> String {
    let mut bundle = FluentBundle::new(vec![language.id()]);
    bundle
        .add_resource(resource_for(language))
        .expect("failed to add Fluent resource");

    let Some(message) = bundle.get_message(key) else {
        return key.to_string();
    };
    let Some(pattern) = message.value() else {
        return key.to_string();
    };

    let mut errors = vec![];
    bundle
        .format_pattern(pattern, args, &mut errors)
        .to_string()
}
