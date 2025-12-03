use oxc_ast::ast::*;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    Buffer, Format, QuoteProperties,
    ast_nodes::{AstNode, AstNodes},
    formatter::{Formatter, prelude::text},
    utils::string::{FormatLiteralStringToken, StringLiteralParentKind},
    write,
};

/// Determines if a numeric literal can be quoted in consistent mode.
/// Returns Some(normalized_value) if it can be quoted, None otherwise.
///
/// Prettier's rules:
/// - In TypeScript, numeric literals in type positions should never be quoted
/// - If the normalized value equals the raw source, it can be quoted (e.g., `1` → `"1"`)
/// - If raw starts with `.`, normalize and quote (e.g., `.1` → `"0.1"`)
/// - If raw ends with `.`, normalize and quote (e.g., `1.` → `"1"`)
/// - Otherwise, don't quote (e.g., `1.0`, `1e2`, `0b10` stay as numeric literals)
fn can_quote_numeric_literal(num: &NumericLiteral, source_type: SourceType) -> Option<String> {
    // In TypeScript, numeric literals in type positions should never be quoted
    if source_type.is_typescript() {
        return None;
    }

    let normalized = num.value.to_string();

    if let Some(raw) = num.raw {
        let raw_str = raw.as_str();

        // If normalized equals raw, it can be quoted
        if normalized == raw_str {
            return Some(normalized);
        }

        // Special case: raw starts with '.' (e.g., .1)
        if raw_str.starts_with('.') {
            return Some(normalized);
        }

        // Special case: raw ends with '.' (e.g., 1.)
        if raw_str.ends_with('.') {
            return Some(normalized);
        }

        // Cannot safely quote
        None
    } else {
        // No raw value available, use normalized
        Some(normalized)
    }
}

/// Returns true if a string literal property key requires quotes (cannot be unquoted).
/// A property key requires quotes if:
/// - It's not a valid identifier name and not a valid number literal, OR
/// - The raw source contains escape sequences (value differs from content between quotes)
fn string_literal_key_requires_quotes(s: &StringLiteral, source_type: SourceType) -> bool {
    let value = s.value.as_str();

    // Check if the raw source contains escape sequences
    // If raw is different from the expected quoted form, there are escape sequences
    if let Some(raw) = &s.raw {
        let raw_str = raw.as_str();
        // Raw includes quotes, so check if content between quotes matches value
        if raw_str.len() >= 2 {
            let content = &raw_str[1..raw_str.len() - 1];
            if content != value {
                // Raw has escape sequences - requires quotes
                return true;
            }
        }
    }

    // Check if it can be a valid identifier
    if is_identifier_name(value) {
        return false;
    }

    // Check if it's a number that can be unquoted (only in JS, not TS)
    if !source_type.is_typescript() {
        if let Some(first_byte) = value.bytes().next() {
            if first_byte.is_ascii_digit() {
                if let Ok(parsed) = value.parse::<f64>() {
                    // Rule out inexact floats and octal literals
                    if parsed.to_string() == value {
                        return false;
                    }
                }
            }
        }
    }

    true
}

/// Checks if any property in an ObjectExpression requires quotes.
/// Used to determine if all properties should be quoted in "consistent" mode.
pub fn object_has_property_requiring_quotes<'a>(
    properties: &oxc_allocator::Vec<'a, ObjectPropertyKind<'a>>,
    source_type: SourceType,
) -> bool {
    for prop in properties.iter() {
        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
            if let PropertyKey::StringLiteral(s) = &obj_prop.key {
                if string_literal_key_requires_quotes(s, source_type) {
                    return true;
                }
            }
        }
    }
    false
}

/// Checks if any property in a TSTypeLiteral requires quotes.
/// Used to determine if all properties should be quoted in "consistent" mode.
pub fn type_literal_has_property_requiring_quotes<'a>(
    members: &oxc_allocator::Vec<'a, TSSignature<'a>>,
    source_type: SourceType,
) -> bool {
    for member in members.iter() {
        if let TSSignature::TSPropertySignature(prop) = member {
            if let PropertyKey::StringLiteral(s) = &prop.key {
                if string_literal_key_requires_quotes(s, source_type) {
                    return true;
                }
            }
        }
    }
    false
}

/// Formats a property key, checking the context for force_quotes flag.
/// When the context's force_quotes_for_object_properties is true (consistent mode
/// with at least one property requiring quotes), ALL property keys are quoted
/// (including identifiers that are converted to quoted strings).
pub fn format_property_key<'a>(key: &AstNode<'a, PropertyKey<'a>>, f: &mut Formatter<'_, 'a>) {
    let force_quotes = f.context().force_quotes_for_object_properties();

    match key.as_ref() {
        PropertyKey::StringLiteral(s) => {
            // `"constructor"` property in the class should be kept quoted
            let kind = if force_quotes {
                StringLiteralParentKind::MemberForceQuotes
            } else if matches!(key.parent, AstNodes::PropertyDefinition(_))
                && matches!(key.as_ref(), PropertyKey::StringLiteral(string) if string.value == "constructor")
            {
                StringLiteralParentKind::Expression
            } else {
                StringLiteralParentKind::Member
            };

            FormatLiteralStringToken::new(
                f.source_text().text_for(s.as_ref()),
                /* jsx */
                false,
                kind,
            )
            .fmt(f);
        }
        PropertyKey::StaticIdentifier(ident) if force_quotes => {
            // In consistent mode with force_quotes, convert identifier to quoted string
            let quote = if f.options().quote_style.is_double() { '"' } else { '\'' };
            let quoted = format!("{quote}{}{quote}", ident.name);
            let allocated = f.context().allocator().alloc_str(&quoted);
            write!(f, [text(allocated)]);
        }
        PropertyKey::NumericLiteral(num) if force_quotes => {
            // In consistent mode, numeric literals may be quoted if they can be safely
            // represented as strings. Use the normalized value (num.value.to_string()).
            if let Some(quoted_value) = can_quote_numeric_literal(num, f.context().source_type()) {
                let quote = if f.options().quote_style.is_double() { '"' } else { '\'' };
                let quoted = format!("{quote}{quoted_value}{quote}");
                let allocated = f.context().allocator().alloc_str(&quoted);
                write!(f, [text(allocated)]);
            } else {
                // Cannot safely quote - keep as numeric literal
                write!(f, key);
            }
        }
        _ => {
            write!(f, key);
        }
    }
}

/// Checks if consistent quoting should force quotes for an object.
/// Returns true if quote_properties is Consistent AND any property requires quotes.
pub fn should_force_quotes_for_object<'a>(
    properties: &oxc_allocator::Vec<'a, ObjectPropertyKind<'a>>,
    f: &Formatter<'_, 'a>,
) -> bool {
    f.options().quote_properties == QuoteProperties::Consistent
        && object_has_property_requiring_quotes(properties, f.context().source_type())
}

/// Checks if consistent quoting should force quotes for a type literal.
/// Returns true if quote_properties is Consistent AND any property requires quotes.
pub fn should_force_quotes_for_type_literal<'a>(
    members: &oxc_allocator::Vec<'a, TSSignature<'a>>,
    f: &Formatter<'_, 'a>,
) -> bool {
    f.options().quote_properties == QuoteProperties::Consistent
        && type_literal_has_property_requiring_quotes(members, f.context().source_type())
}

/// Checks if any property in a ClassBody requires quotes.
/// Used to determine if all properties should be quoted in "consistent" mode.
pub fn class_body_has_property_requiring_quotes<'a>(
    body: &oxc_allocator::Vec<'a, ClassElement<'a>>,
    source_type: SourceType,
) -> bool {
    for element in body.iter() {
        match element {
            ClassElement::PropertyDefinition(def) => {
                if let PropertyKey::StringLiteral(s) = &def.key {
                    if string_literal_key_requires_quotes(s, source_type) {
                        return true;
                    }
                }
            }
            ClassElement::MethodDefinition(def) => {
                if let PropertyKey::StringLiteral(s) = &def.key {
                    if string_literal_key_requires_quotes(s, source_type) {
                        return true;
                    }
                }
            }
            ClassElement::AccessorProperty(def) => {
                if let PropertyKey::StringLiteral(s) = &def.key {
                    if string_literal_key_requires_quotes(s, source_type) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// Checks if consistent quoting should force quotes for a class body.
/// Returns true if quote_properties is Consistent AND any property requires quotes.
pub fn should_force_quotes_for_class_body<'a>(
    body: &oxc_allocator::Vec<'a, ClassElement<'a>>,
    f: &Formatter<'_, 'a>,
) -> bool {
    f.options().quote_properties == QuoteProperties::Consistent
        && class_body_has_property_requiring_quotes(body, f.context().source_type())
}

/// Writes a member name, checking the context for force_quotes flag.
/// Returns the width of the formatted name.
pub fn write_member_name<'a>(
    key: &AstNode<'a, PropertyKey<'a>>,
    f: &mut Formatter<'_, 'a>,
) -> usize {
    let force_quotes = f.context().force_quotes_for_object_properties();

    match key.as_ast_nodes() {
        AstNodes::StringLiteral(string) => {
            let kind = if force_quotes {
                StringLiteralParentKind::MemberForceQuotes
            } else {
                StringLiteralParentKind::Member
            };
            let format =
                FormatLiteralStringToken::new(f.source_text().text_for(string), false, kind)
                    .clean_text(f.context().source_type(), f.options());

            string.format_leading_comments(f);
            write!(f, format);
            string.format_trailing_comments(f);

            format.width()
        }
        AstNodes::IdentifierName(ident) if force_quotes => {
            // In consistent mode with force_quotes, convert identifier to quoted string
            let quote = if f.options().quote_style.is_double() { '"' } else { '\'' };
            let quoted = format!("{quote}{}{quote}", ident.name);
            let width = quoted.len();
            let allocated = f.context().allocator().alloc_str(&quoted);
            write!(f, [text(allocated)]);
            width
        }
        AstNodes::NumericLiteral(num) if force_quotes => {
            // In consistent mode, numeric literals may be quoted if they can be safely
            // represented as strings. Use the normalized value (num.value.to_string()).
            if let Some(quoted_value) = can_quote_numeric_literal(num, f.context().source_type()) {
                let quote = if f.options().quote_style.is_double() { '"' } else { '\'' };
                let quoted = format!("{quote}{quoted_value}{quote}");
                let width = quoted.len();
                let allocated = f.context().allocator().alloc_str(&quoted);
                write!(f, [text(allocated)]);
                width
            } else {
                // Cannot safely quote - keep as numeric literal
                write!(f, key);
                f.source_text().span_width(key.span())
            }
        }
        _ => {
            write!(f, key);
            f.source_text().span_width(key.span())
        }
    }
}
