/// Marks which Serde field names of a root struct are Markdown body sections.
///
/// Front-matter keys are every other field. Names are Serde names after `rename`,
/// in declaration order of the body fields. Types with no body still implement
/// this trait with an empty list.
pub trait Markdown {
    /// Serde field names of body sections, declaration order.
    const BODY_FIELDS: &'static [&'static str];
}

#[cfg(test)]
mod tests {
    use super::Markdown;

    struct Empty;

    impl Markdown for Empty {
        const BODY_FIELDS: &'static [&'static str] = &[];
    }

    #[test]
    fn empty_body_fields() {
        assert_eq!(Empty::BODY_FIELDS, [] as [&str; 0]);
    }
}
