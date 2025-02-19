use syn::Attribute;

pub fn remove_attribute(attrs: &mut Vec<Attribute>, name: &str) -> bool {
    let mut found = false;
    attrs.retain(|attr| {
        if attr.path.is_ident(name) {
            found = true;
            false
        } else {
            true
        }
    });
    found
}

#[inline]
pub fn remove_maybe_async_attr(attrs: &mut Vec<Attribute>) -> bool {
    remove_attribute(attrs, "maybe_async")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_attribute() {
        let mut attrs = vec![
            syn::parse_quote!(#[maybe_async]),
            syn::parse_quote!(#[some_other_attr]),
        ];
        let expected = vec![syn::parse_quote!(#[some_other_attr])];

        let result = remove_maybe_async_attr(&mut attrs);
        assert!(result);
        assert_attrs_eq(&attrs, &expected);

        let result = remove_maybe_async_attr(&mut attrs);
        assert!(!result);
        assert_attrs_eq(&attrs, &expected);

        let result = remove_attribute(&mut attrs, "some_other_attr");
        assert!(result);
        assert!(attrs.is_empty());
    }

    fn assert_attrs_eq(attrs: &[Attribute], expected: &[Attribute]) {
        assert_eq!(attrs.len(), expected.len());
        for (attr, expected) in attrs.iter().zip(expected.iter()) {
            assert_eq!(attr.path.get_ident(), expected.path.get_ident());
        }
    }
}
