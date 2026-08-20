pub(crate) enum Field<'a> {
    Literal(&'a str),
    Code(Option<char>),
}

/// Expand percent field codes in one already shell-split argument.
///
/// A trailing `%` is ignored, matching the thumbnailer implementation used
/// by GNOME. The callback decides how each code is handled.
pub(crate) fn for_each_field_code(
    argument: &str,
    mut visit: impl FnMut(Field<'_>) -> Result<(), ()>,
) -> Result<(), ()> {
    let mut rest = argument;

    while let Some(pos) = rest.find('%') {
        visit(Field::Literal(&rest[..pos]))?;
        rest = &rest[pos + 1..];

        let Some(code) = rest.chars().next() else {
            visit(Field::Code(None))?;
            break;
        };
        rest = &rest[code.len_utf8()..];
        visit(Field::Code(Some(code)))?;
    }

    visit(Field::Literal(rest))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Field, for_each_field_code};

    #[test]
    fn scans_literals_codes_and_trailing_percent() {
        let mut expanded = String::new();
        for_each_field_code("left%i-middle%%-right%", |field| {
            match field {
                Field::Literal(literal) => expanded.push_str(literal),
                Field::Code(code) => {
                    expanded.push_str(&code.map_or("<trailing>".into(), |code| format!("<{code}>")))
                }
            }
            Ok(())
        })
        .unwrap();

        assert_eq!(expanded, "left<i>-middle<%>-right<trailing>");
    }
}
