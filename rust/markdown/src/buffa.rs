//! Buffa codegen attributes for `(markdown.body) = true`.
//!
//! `buffa_build::Config` has no callback that receives a compiled
//! [`FileDescriptorSet`]. Walk a set the caller already has (from
//! `buf build --as-file-descriptor-set` or `protoc --descriptor_set_out`)
//! and loop [`MarkdownBodyAttributes::message_attributes`] /
//! [`MarkdownBodyAttributes::field_attributes`] into
//! `Config::message_attribute` / `Config::field_attribute`.
//!
//! Attribute paths use proto identifiers (`body_note`), not proto3 JSON
//! names (`bodyNote`). Buffa still emits `#[serde(rename = json_name)]` on
//! generated fields, so `#[derive(Markdown)]` fills `BODY_FIELDS` with those
//! JSON names. This helper does not write JSON names into `BODY_FIELDS`.

use buffa::ExtensionSet;
use buffa_descriptor::generated::descriptor::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto,
};

/// Protobuf [`FileDescriptorSet`] walked by [`annotate_markdown_body`].
pub use buffa_descriptor::generated::descriptor::FileDescriptorSet;

const OPTIONS_PROTO: &str = "markdown/options.proto";
const OPTIONS_PROTO_SUFFIX: &str = "/markdown/options.proto";
const WKT_PACKAGE: &str = "google.protobuf";
const WKT_PACKAGE_PREFIX: &str = "google.protobuf.";
const BODY_NUMBER: u32 = 20_260_917;
const BODY: buffa::Extension<buffa::extension::codecs::Bool> =
    buffa::Extension::new(BODY_NUMBER, "google.protobuf.FieldOptions");
const MESSAGE_ATTR: &str = "#[derive(::serde_markdown::Markdown)]";
const FIELD_ATTR: &str = "#[markdown(body)]";

/// Result of walking a [`FileDescriptorSet`].
pub struct MarkdownBodyAttributes {
    message_attributes: Vec<(String, String)>,
    field_attributes: Vec<(String, String)>,
}

impl MarkdownBodyAttributes {
    /// `(proto path, attribute)` pairs for buffa `Config::message_attribute`.
    ///
    /// Each attribute is `#[derive(::serde_markdown::Markdown)]`.
    pub fn message_attributes(&self) -> &[(String, String)] {
        &self.message_attributes
    }

    /// `(proto path, attribute)` pairs for buffa `Config::field_attribute`.
    ///
    /// Each attribute is `#[markdown(body)]`. The last path segment is the
    /// proto field identifier (`body_note`), not the proto3 JSON name
    /// (`bodyNote`). Buffa matches that proto field path.
    pub fn field_attributes(&self) -> &[(String, String)] {
        &self.field_attributes
    }
}

/// Walk `fds` and collect buffa `message_attribute` / `field_attribute` pairs.
///
/// Files whose package is `google.protobuf` or starts with `google.protobuf.`
/// are skipped. A remaining file is annotated when it imports
/// `markdown/options.proto` (`dependency` or `option_dependency`, exact name
/// or a path ending with `/markdown/options.proto`) or when any field in the
/// file has `(markdown.body) = true` (field number `20260917` on
/// `google.protobuf.FieldOptions`).
///
/// Every message in an annotated file, including nested messages, gets
/// `#[derive(::serde_markdown::Markdown)]` at a path such as
/// `.markdown.testdata.Page` or `.markdown.testdata.NestedFields.Meta`.
/// An empty package yields `.MessageName`.
///
/// Each field with the body option gets `#[markdown(body)]` at
/// `.pkg.Msg.field` using the proto identifier as the last segment, not
/// `json_name`.
pub fn annotate_markdown_body(fds: &FileDescriptorSet) -> MarkdownBodyAttributes {
    let mut out = MarkdownBodyAttributes {
        message_attributes: Vec::new(),
        field_attributes: Vec::new(),
    };
    for file in &fds.file {
        if skip_well_known_file(file) || !file_needs_annotation(file) {
            continue;
        }
        let package = file.package.as_deref().unwrap_or("");
        for msg in &file.message_type {
            walk_message(package, "", msg, &mut out);
        }
    }
    out
}

fn skip_well_known_file(file: &FileDescriptorProto) -> bool {
    let package = file.package.as_deref().unwrap_or("");
    package == WKT_PACKAGE || package.starts_with(WKT_PACKAGE_PREFIX)
}

fn file_needs_annotation(file: &FileDescriptorProto) -> bool {
    file_imports_options(file) || file_has_body_option(file)
}

fn file_imports_options(file: &FileDescriptorProto) -> bool {
    file.dependency
        .iter()
        .chain(file.option_dependency.iter())
        .any(|dep| is_markdown_options_import(dep))
}

fn is_markdown_options_import(dep: &str) -> bool {
    dep == OPTIONS_PROTO || dep.ends_with(OPTIONS_PROTO_SUFFIX)
}

fn file_has_body_option(file: &FileDescriptorProto) -> bool {
    file.message_type.iter().any(message_has_body_option)
}

fn message_has_body_option(msg: &DescriptorProto) -> bool {
    msg.field.iter().any(field_has_body) || msg.nested_type.iter().any(message_has_body_option)
}

fn field_has_body(field: &FieldDescriptorProto) -> bool {
    field
        .options
        .as_option()
        .is_some_and(|opts| opts.extension(&BODY) == Some(true))
}

fn walk_message(
    package: &str,
    parent_type: &str,
    msg: &DescriptorProto,
    out: &mut MarkdownBodyAttributes,
) {
    let Some(name) = msg.name.as_deref().filter(|n| !n.is_empty()) else {
        return;
    };
    let type_path = if parent_type.is_empty() {
        name.to_owned()
    } else {
        let mut path = String::with_capacity(parent_type.len() + 1 + name.len());
        path.push_str(parent_type);
        path.push('.');
        path.push_str(name);
        path
    };
    out.message_attributes
        .push((qualified_path(package, &type_path), MESSAGE_ATTR.to_owned()));
    for field in &msg.field {
        if !field_has_body(field) {
            continue;
        }
        let Some(field_name) = field.name.as_deref().filter(|n| !n.is_empty()) else {
            continue;
        };
        let mut field_path = qualified_path(package, &type_path);
        field_path.push('.');
        field_path.push_str(field_name);
        out.field_attributes
            .push((field_path, FIELD_ATTR.to_owned()));
    }
    for nested in &msg.nested_type {
        walk_message(package, &type_path, nested, out);
    }
}

fn qualified_path(package: &str, type_path: &str) -> String {
    let mut path = String::from(".");
    if !package.is_empty() {
        path.push_str(package);
        path.push('.');
    }
    path.push_str(type_path);
    path
}

#[cfg(test)]
mod tests {
    use buffa::ExtensionSet;
    use buffa_descriptor::generated::descriptor::FieldOptions;

    use super::*;

    fn field(name: &str, json_name: Option<&str>, body: bool) -> FieldDescriptorProto {
        let mut field = FieldDescriptorProto {
            name: Some(name.into()),
            json_name: json_name.map(str::to_owned),
            ..Default::default()
        };
        if body {
            let mut opts = FieldOptions::default();
            opts.set_extension(&BODY, true);
            field.options = opts.into();
        }
        field
    }

    fn message(
        name: &str,
        fields: Vec<FieldDescriptorProto>,
        nested: Vec<DescriptorProto>,
    ) -> DescriptorProto {
        DescriptorProto {
            name: Some(name.into()),
            field: fields,
            nested_type: nested,
            ..Default::default()
        }
    }

    fn testdata_fds() -> FileDescriptorSet {
        FileDescriptorSet {
            file: vec![
                FileDescriptorProto {
                    name: Some("markdown/testdata/names.proto".into()),
                    package: Some("markdown.testdata".into()),
                    dependency: vec![OPTIONS_PROTO.into()],
                    message_type: vec![
                        message(
                            "JsonNames",
                            vec![
                                field("published_at", Some("publishedAt"), false),
                                field("body_note", Some("bodyNote"), true),
                            ],
                            vec![],
                        ),
                        message(
                            "NestedFields",
                            vec![
                                field("meta", None, false),
                                field("draft", None, false),
                                field("body", None, true),
                            ],
                            vec![message("Meta", vec![field("author", None, false)], vec![])],
                        ),
                    ],
                    ..Default::default()
                },
                FileDescriptorProto {
                    name: Some("google/protobuf/timestamp.proto".into()),
                    package: Some("google.protobuf".into()),
                    message_type: vec![message(
                        "Timestamp",
                        vec![field("seconds", None, true)],
                        vec![],
                    )],
                    ..Default::default()
                },
                FileDescriptorProto {
                    name: Some("plain.proto".into()),
                    package: Some("other.pkg".into()),
                    message_type: vec![message("Plain", vec![field("x", None, false)], vec![])],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    fn contains_pair(pairs: &[(String, String)], path: &str, attr: &str) -> bool {
        pairs.iter().any(|(p, a)| p == path && a == attr)
    }

    #[test]
    fn annotate_marks_owned_messages_and_proto_field_names() {
        let attrs = annotate_markdown_body(&testdata_fds());
        let messages = attrs.message_attributes();
        let fields = attrs.field_attributes();

        assert!(contains_pair(
            messages,
            ".markdown.testdata.JsonNames",
            MESSAGE_ATTR
        ));
        assert!(contains_pair(
            messages,
            ".markdown.testdata.NestedFields",
            MESSAGE_ATTR
        ));
        assert!(contains_pair(
            messages,
            ".markdown.testdata.NestedFields.Meta",
            MESSAGE_ATTR
        ));
        assert!(
            !messages.iter().any(|(p, _)| p.contains("View")),
            "views are not Markdown: {messages:?}"
        );
        assert!(
            !messages.iter().any(|(p, _)| p.contains("google.protobuf")),
            "well-known files must not be annotated: {messages:?}"
        );
        assert!(
            !messages.iter().any(|(p, _)| p.contains("Plain")),
            "files without options import or body option stay unmarked: {messages:?}"
        );

        assert!(contains_pair(
            fields,
            ".markdown.testdata.JsonNames.body_note",
            FIELD_ATTR
        ));
        assert!(contains_pair(
            fields,
            ".markdown.testdata.NestedFields.body",
            FIELD_ATTR
        ));
        assert!(
            !fields.iter().any(|(p, _)| p.ends_with(".published_at")
                || p.ends_with(".publishedAt")
                || p.ends_with(".bodyNote")),
            "front-matter and json_name must not appear as body field paths: {fields:?}"
        );
        assert!(
            !fields.iter().any(|(p, _)| p.contains("View")),
            "views are not Markdown: {fields:?}"
        );
        assert!(
            !fields.iter().any(|(p, _)| p.contains("google.protobuf")),
            "well-known body options must not be annotated: {fields:?}"
        );
    }

    #[test]
    fn option_dependency_and_empty_package_and_body_without_import() {
        let fds = FileDescriptorSet {
            file: vec![
                FileDescriptorProto {
                    name: Some("via_option_dep.proto".into()),
                    package: Some("opt.dep".into()),
                    option_dependency: vec!["vendor/markdown/options.proto".into()],
                    message_type: vec![message("Doc", vec![field("n", None, false)], vec![])],
                    ..Default::default()
                },
                FileDescriptorProto {
                    name: Some("no_package.proto".into()),
                    package: None,
                    message_type: vec![message("Root", vec![field("body", None, true)], vec![])],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let attrs = annotate_markdown_body(&fds);
        assert!(contains_pair(
            attrs.message_attributes(),
            ".opt.dep.Doc",
            MESSAGE_ATTR
        ));
        assert!(contains_pair(
            attrs.message_attributes(),
            ".Root",
            MESSAGE_ATTR
        ));
        assert!(contains_pair(
            attrs.field_attributes(),
            ".Root.body",
            FIELD_ATTR
        ));
    }
}
