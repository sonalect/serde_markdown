//! `include_str!` of golden Markdown documents.

pub const PAGE_FENCED_YAML: &str = include_str!("../../testdata/markdown/page.fenced.yaml.md");
pub const PAGE_FENCED_YML: &str = include_str!("../../testdata/markdown/page.fenced.yml.md");
pub const PAGE_FENCED_JSON: &str = include_str!("../../testdata/markdown/page.fenced.json.md");
pub const PAGE_FENCED_TOML: &str = include_str!("../../testdata/markdown/page.fenced.toml.md");
pub const PAGE_BARE_YAML: &str = include_str!("../../testdata/markdown/page.bare.yaml.md");
pub const PAGE_BARE_JSON: &str = include_str!("../../testdata/markdown/page.bare.json.md");
pub const PAGE_BARE_TOML: &str = include_str!("../../testdata/markdown/page.bare.toml.md");
pub const PAGE_UNLABELED_YAML: &str =
    include_str!("../../testdata/markdown/page.unlabeled.yaml.md");
pub const PAGE_UNLABELED_JSON: &str =
    include_str!("../../testdata/markdown/page.unlabeled.json.md");
pub const PAGE_UNLABELED_TOML: &str =
    include_str!("../../testdata/markdown/page.unlabeled.toml.md");
pub const PAGE_LEADING_PREFIX: &str =
    include_str!("../../testdata/markdown/page.leading_prefix.md");
pub const PAGE_PUBLISHED_YAML: &str =
    include_str!("../../testdata/markdown/page.published.yaml.md");
pub const PAGE_SPLIT_FENCE: &str = include_str!("../../testdata/markdown/page.split.fence.md");
pub const PAGE_SPLIT_STARS: &str = include_str!("../../testdata/markdown/page.split.stars.md");
pub const PAGE_SPLIT_LIST: &str = include_str!("../../testdata/markdown/page.split.list.md");
pub const PAGE_FENCE_NOT_FIRST: &str =
    include_str!("../../testdata/markdown/page.fence_not_first.md");
pub const FIELDS_ONLY_FENCED_YAML: &str =
    include_str!("../../testdata/markdown/fields_only.fenced.yaml.md");
pub const FIELDS_ONLY_BARE_YAML: &str =
    include_str!("../../testdata/markdown/fields_only.bare.yaml.md");
pub const BODY_ONLY_TWO_SECTIONS: &str =
    include_str!("../../testdata/markdown/body_only.two_sections.md");
pub const NESTED_FENCED_YAML: &str = include_str!("../../testdata/markdown/nested.fenced.yaml.md");
pub const OPTIONAL_MIDDLE_NONE: &str =
    include_str!("../../testdata/markdown/optional.middle_none.md");
pub const OPTIONAL_TRAILING_NONE: &str =
    include_str!("../../testdata/markdown/optional.trailing_none.md");
pub const OPTIONAL_LEADING_NONE: &str =
    include_str!("../../testdata/markdown/optional.leading_none.md");
pub const OPTIONAL_EMPTY_STRING: &str =
    include_str!("../../testdata/markdown/optional.empty_string.md");
pub const NAMES_FENCED_YAML: &str = include_str!("../../testdata/markdown/names.fenced.yaml.md");
pub const STRUCTURED_FENCED_YAML: &str =
    include_str!("../../testdata/markdown/structured.fenced.yaml.md");
pub const WELL_KNOWN_FENCED_YAML: &str =
    include_str!("../../testdata/markdown/well_known.fenced.yaml.md");
pub const PROTO3_FENCED_YAML: &str = include_str!("../../testdata/markdown/proto3.fenced.yaml.md");
pub const WHITESPACE_UNICODE: &str = include_str!("../../testdata/markdown/whitespace.unicode.md");

/// Every golden file, for smoke tests that they stayed in the crate.
pub const ALL: &[(&str, &str)] = &[
    ("page.fenced.yaml.md", PAGE_FENCED_YAML),
    ("page.fenced.yml.md", PAGE_FENCED_YML),
    ("page.fenced.json.md", PAGE_FENCED_JSON),
    ("page.fenced.toml.md", PAGE_FENCED_TOML),
    ("page.bare.yaml.md", PAGE_BARE_YAML),
    ("page.bare.json.md", PAGE_BARE_JSON),
    ("page.bare.toml.md", PAGE_BARE_TOML),
    ("page.unlabeled.yaml.md", PAGE_UNLABELED_YAML),
    ("page.unlabeled.json.md", PAGE_UNLABELED_JSON),
    ("page.unlabeled.toml.md", PAGE_UNLABELED_TOML),
    ("page.leading_prefix.md", PAGE_LEADING_PREFIX),
    ("page.published.yaml.md", PAGE_PUBLISHED_YAML),
    ("page.split.fence.md", PAGE_SPLIT_FENCE),
    ("page.split.stars.md", PAGE_SPLIT_STARS),
    ("page.split.list.md", PAGE_SPLIT_LIST),
    ("page.fence_not_first.md", PAGE_FENCE_NOT_FIRST),
    ("fields_only.fenced.yaml.md", FIELDS_ONLY_FENCED_YAML),
    ("fields_only.bare.yaml.md", FIELDS_ONLY_BARE_YAML),
    ("body_only.two_sections.md", BODY_ONLY_TWO_SECTIONS),
    ("nested.fenced.yaml.md", NESTED_FENCED_YAML),
    ("optional.middle_none.md", OPTIONAL_MIDDLE_NONE),
    ("optional.trailing_none.md", OPTIONAL_TRAILING_NONE),
    ("optional.leading_none.md", OPTIONAL_LEADING_NONE),
    ("optional.empty_string.md", OPTIONAL_EMPTY_STRING),
    ("names.fenced.yaml.md", NAMES_FENCED_YAML),
    ("structured.fenced.yaml.md", STRUCTURED_FENCED_YAML),
    ("well_known.fenced.yaml.md", WELL_KNOWN_FENCED_YAML),
    ("proto3.fenced.yaml.md", PROTO3_FENCED_YAML),
    ("whitespace.unicode.md", WHITESPACE_UNICODE),
];
